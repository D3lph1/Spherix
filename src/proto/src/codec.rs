use std::io::{Cursor, Read};

use aes::Aes128;
use cfb8::cipher::{AsyncStreamCipher, NewCipher};
use cfb8::Cfb8;
use flate2::bufread::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;

use crate::io::{Error, VarInt};
use crate::io::{Readable, Writable};

pub struct CompressionContext {
    threshold: usize,
    compression: Compression,
    buf: Vec<u8>
}

impl CompressionContext {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            compression: Compression::default(),
            buf: Vec::new()
        }
    }

    #[allow(unused)]
    pub fn with_compression(threshold: usize, compression: Compression) -> Self {
        Self {
            threshold,
            compression,
            buf: Vec::new()
        }
    }
}

/// 16-byte shared secret key as described [`here`].
///
/// [`here`]: https://wiki.vg/Protocol_Encryption#Symmetric_Encryption
type CipherKey = [u8; 16];

pub struct CipherContext {
    cipher: Cfb8<Aes128>
}

impl CipherContext {
    pub fn new(key: CipherKey) -> Self {
        Self {
            cipher: Cfb8::new_from_slices(&key, &key).unwrap()
        }
    }

    pub fn encrypt(&mut self, buf: &mut Vec<u8>) {
        self.cipher.encrypt(buf)
    }

    pub fn decrypt(&mut self, buf: &mut [u8]) {
        self.cipher.decrypt(buf)
    }
}

pub struct ReadableCodec {
    // Buffer of received bytes. It accumulates input data in order to form
    // a packet later.
    buf: Vec<u8>,
    compression: Option<CompressionContext>,
    cipher: Option<CipherContext>
}

impl ReadableCodec {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            // Initially, all codec features such as compression and encryption are
            // disabled. They could be enabled upon request at Login state.
            compression: None,
            cipher: None
        }
    }

    pub fn enable_compression(&mut self, context: CompressionContext) {
        self.compression = Some(context)
    }

    pub fn enable_encryption(&mut self, context: CipherContext) {
        self.cipher = Some(context)
    }

    pub fn append(&mut self, buf: &[u8]) {
        let start = self.buf.len();
        self.buf.extend_from_slice(buf);

        if let Some(cipher) = &mut self.cipher {
            cipher.decrypt(&mut self.buf[start..])
        }
    }

    pub fn next<R: Readable>(&mut self) -> Result<Option<R>, Error> {
        let mut cursor = Cursor::new(&self.buf[..]);

        let packet = if let Ok(length) = VarInt::read(&mut cursor) {
            let length_field_pos = cursor.position() as usize;

            if self.buf.len() - length_field_pos >= length.0 as usize {
                cursor = Cursor::new(&self.buf[length_field_pos..length_field_pos + length.0 as usize]);

                if let Some(ctx) = &mut self.compression {
                    let data_length = VarInt::read(&mut cursor)?;
                    if data_length != 0 {
                        let mut decoder = ZlibDecoder::new(&cursor.get_ref()[cursor.position() as usize..]);
                        decoder.read_to_end(&mut ctx.buf)?;
                        cursor = Cursor::new(&mut ctx.buf);
                    }
                }

                let packet = R::read(&mut cursor)?;

                self.buf = self.buf.split_off(length.0 as usize + length_field_pos);

                if let Some(ctx) = &mut self.compression {
                    ctx.buf.clear();
                }

                Some(packet)
            } else {
                None
            }
        } else {
            None
        };

        Ok(packet)
    }

    pub fn buf(&self) -> &Vec<u8> {
        &self.buf
    }
}

pub struct WritableCodec {
    // Buffer to store temporary data about to send. This storage uses only
    // during write() method invocation and not present between method calls
    buf: Vec<u8>,
    compression: Option<CompressionContext>,
    cipher: Option<CipherContext>
}

impl WritableCodec {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            // Initially, all codec features such as compression and encryption are
            // disabled. They could be enabled upon request at Login state.
            compression: None,
            cipher: None
        }
    }

    pub fn enable_compression(&mut self, context: CompressionContext) {
        self.compression = Some(context)
    }

    pub fn enable_encryption(&mut self, context: CipherContext) {
        self.cipher = Some(context)
    }

    pub fn write(&mut self, packet: &impl Writable, buf: &mut Vec<u8>) -> Result<(), Error> {
        packet.write(&mut self.buf)?;

        if let Some(ctx) = &mut self.compression {
            let (data_length, data) = if self.buf.len() >= ctx.threshold {
                let mut encoder = ZlibEncoder::new(self.buf.as_slice(), ctx.compression);
                encoder.read_to_end(&mut ctx.buf)?;

                (self.buf.len(), ctx.buf.as_slice())
            } else {
                (0, self.buf.as_slice())
            };

            let mut data_length_buf = Vec::new();
            let written = VarInt(data_length as i32).write(&mut data_length_buf)?;

            let packet_length = data.len() + written;

            VarInt(packet_length as i32).write(buf)?;
            buf.extend_from_slice(&data_length_buf[..written]);
            buf.extend_from_slice(data);

            ctx.buf.clear();
        } else {
            let length = self.buf.len();
            VarInt(length as i32).write(buf)?;
            buf.extend_from_slice(&self.buf);
        }

        if let Some(ctx) = &mut self.cipher {
            ctx.encrypt(buf);
        }

        self.buf.clear();

        Ok(())
    }
}
