use std::path::PathBuf;

use anyhow::anyhow;
use reqwest::Client;
use serde::Deserialize;

pub(crate) struct MojangSessionApi {
    base_url: PathBuf
}

impl MojangSessionApi {
    pub(crate) fn new(base_url: PathBuf) -> Self {
        Self {
            base_url,
        }
    }

    pub(crate) async fn send_has_joined_request(&self, req: HasJoinedRequest) -> anyhow::Result<HasJoinedResponse> {
        let query = vec![
            ("username", req.username),
            ("serverId", req.server_id_hash),
            ("ip", req.ip),
        ];

        let client = Client::new();
        let response = client.get(
            self.base_url
                .join("session/minecraft/hasJoined")
                .to_str()
                .unwrap()
        )
            .query(&query)
            .send()
            .await
            .unwrap();

        if response.status() == 200 {
            let content = response.text().await.unwrap();
            let resp: HasJoinedResponse = serde_json::from_str(&content)?;

            Ok(resp)
        } else {
            Err(anyhow!("not 200"))
        }
    }
}

pub(crate) struct HasJoinedRequest {
    pub(crate) username: String,
    pub(crate) server_id_hash: String,
    pub(crate) ip: String
}

#[derive(Deserialize, Debug)]
pub(crate) struct HasJoinedResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) properties: Vec<HasJoinedProperties>
}

#[derive(Deserialize, Debug)]
pub(crate) struct HasJoinedProperties {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) signature: String
}
