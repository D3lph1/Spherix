use std::io::{BufRead, Read, Write};

use flate2::bufread::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};

pub enum Compression {
    Gzip,
    Zlib
}

impl Compression {
    const TYPE_GZIP: u8 = 1;
    const TYPE_ZLIB: u8 = 2;

    pub fn decode<R: BufRead>(self, buf_read: &mut R) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();

        match self {
            Compression::Gzip => {
                let mut decoder = GzDecoder::new(buf_read);
                decoder.read_to_end(&mut buf).unwrap();
            },
            Compression::Zlib => {
                let mut decoder = ZlibDecoder::new(buf_read);
                decoder.read_to_end(&mut buf).unwrap();
            }
        };

        buf
    }

    pub fn encode<W: Write>(&self, data: &Vec<u8>, buf: &mut W, level: flate2::Compression) -> anyhow::Result<usize> {
        Ok(self.wrap_encoder(buf, level).write(data)?)
    }

    pub fn wrap_encoder<'a, W: Write + 'a>(&self, buf: W, level: flate2::Compression) -> Box<dyn Write + 'a> {
        match self {
            Compression::Gzip => Box::new(GzEncoder::new(buf, level)),
            Compression::Zlib => Box::new(ZlibEncoder::new(buf, level)),
        }
    }
}

impl From<u8> for Compression {
    fn from(value: u8) -> Self {
        match value {
            Self::TYPE_GZIP => Self::Gzip,
            Self::TYPE_ZLIB => Self::Zlib,
            x => panic!("{}", format!("Unexpected compression type {} given", x))
        }
    }
}
