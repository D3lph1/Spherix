use anyhow::anyhow;
use rand::random;
use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, PublicKeyParts, RsaPrivateKey};
use rsa_der::public_key_to_der;
use sha1::{Digest, Sha1};

use spherix_proto::codec::{CipherContext, CompressionContext};
use spherix_proto::io::VarInt;
use spherix_proto::packet::clientbound::{EncryptionRequest, LoginMapping as ClientboundLoginMapping, LoginSuccess, LoginSuccessProperty, SetCompression};
use spherix_proto::packet::serverbound::{LoginMapping as ServerboundLoginMapping, LoginStart};
use spherix_util::sha1::notchian_digest;

use crate::join::JoinContext;
use crate::mojang::{HasJoinedRequest, MojangSessionApi};
use crate::preamble::Preamble;

/// Server ID appears to be empty
const SERVER_ID: &str = "";

/// 4-byte verify token as described [`here`].
///
/// [`here`]: https://wiki.vg/Protocol#Encryption_Request
type VerifyToken = [u8; 4];

const RSA_BIT_SIZE: usize = 1024;

struct Auth {
    verify_token: VerifyToken,
}

impl Auth {
    fn new() -> Self {
        Self {
            verify_token: Self::verify_token()
        }
    }

    fn verify_token() -> VerifyToken {
        random::<VerifyToken>()
    }
}

pub async fn handle_login<'a>(mut preamble: Preamble<'a>, config: &spherix_config::Config) -> anyhow::Result<JoinContext<'a>> {
    let p = preamble.reader.read::<ServerboundLoginMapping>().await?;

    let ServerboundLoginMapping::LoginStart(p) = p else {
        return Err(anyhow!("expected LoginStart packet, but {} given", p.name()));
    };

    let properties = if config.auth.enabled {
        auth(&mut preamble, &p, config).await?
    } else {
        Vec::new()
    };

    let compression = &config.network.compression;
    if compression.enabled {
        enable_compression(&mut preamble, compression.threshold).await?;
    }

    login_success(&mut preamble, &p, properties).await?;

    Ok(preamble.to_join_context(p.name, p.player_uuid))
}

async fn auth(preamble: &mut Preamble<'_>, p: &LoginStart, config: &spherix_config::Config) -> anyhow::Result<Vec<LoginSuccessProperty>> {
    let auth = Auth::new();

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, RSA_BIT_SIZE)?;

    let der = public_key_to_der(
        &private_key.n().to_bytes_be(), &private_key.e().to_bytes_be(),
    );

    preamble.writer.write(ClientboundLoginMapping::EncryptionRequest(EncryptionRequest {
        server_id: SERVER_ID.to_string(),
        public_key: der.clone().into_boxed_slice(),
        verify_token: Box::new(auth.verify_token),
    })).await?;

    let er = preamble.reader.read::<ServerboundLoginMapping>().await?;

    let ServerboundLoginMapping::EncryptionResponse(er) = er else {
        return Err(anyhow!("expected EncryptionResponse packet, but {} given", er.name()));
    };

    let shared = private_key.decrypt(Pkcs1v15Encrypt, &er.shared_secret_key)?;

    let verify_token = private_key.decrypt(Pkcs1v15Encrypt, &er.verify_token)?;

    if auth.verify_token != verify_token.as_slice() {
        return Err(anyhow!("invalid verify token"));
    }

    preamble.reader.codec().enable_encryption(CipherContext::new(shared.clone().try_into().unwrap()));
    preamble.writer.codec().enable_encryption(CipherContext::new(shared.clone().try_into().unwrap()));

    let server_id_hash = calculate_server_id_hah(
        SERVER_ID.to_owned(),
        shared,
        der,
    );

    let mojang = MojangSessionApi::new(config.auth.session_host.0.clone());
    let resp = mojang.send_has_joined_request(HasJoinedRequest {
        username: p.name.to_owned(),
        server_id_hash,
        ip: format!("127.0.0.1:{}", config.network.port),
    })
        .await?;

    let mut properties = Vec::new();
    for property in resp.properties {
        properties.push(LoginSuccessProperty {
            name: property.name,
            value: property.value,
            signature: Some(property.signature),
        });
    }

    Ok(properties)
}

async fn enable_compression(preamble: &mut Preamble<'_>, threshold: usize) -> anyhow::Result<()> {
    preamble.writer.write(ClientboundLoginMapping::SetCompression(SetCompression {
        threshold: VarInt(threshold as i32)
    })).await?;

    preamble.reader.codec().enable_compression(CompressionContext::new(threshold));
    preamble.writer.codec().enable_compression(CompressionContext::new(threshold));

    Ok(())
}

async fn login_success(preamble: &mut Preamble<'_>, p: &LoginStart, properties: Vec<LoginSuccessProperty>) -> anyhow::Result<()> {
    preamble.writer.write(ClientboundLoginMapping::LoginSuccess(LoginSuccess {
        uuid: p.player_uuid,
        username: p.name.clone(),
        properties: properties.into_boxed_slice(),
    })).await?;

    Ok(())
}

/// As described here: https://wiki.vg/Protocol_Encryption#Client
fn calculate_server_id_hah(
    server_id: String,
    shared: Vec<u8>,
    public_key: Vec<u8>,
) -> String {
    let mut hasher = Sha1::default();

    hasher.update(&server_id);
    hasher.update(&shared);
    hasher.update(&public_key);

    let hash = hasher.finalize();

    notchian_digest(hash[..].to_vec().try_into().unwrap())
}
