use std::fmt::Debug;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use crate::io::error::Error;
use crate::io::io::{Readable, Writable};
use crate::io::var::VarInt;

macro_rules! length_prefixed_sequence_writable {
    ($t: tt<$param: tt>) => {
        impl<T: Writable> Writable for $t < $param > {
            fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
                let mut written = VarInt(self.len() as i32).write(buf)?;
                for x in self.iter() {
                    written += x.write(buf)?;
                }

                Ok(written)
            }
        }
    };
    ($t: tt) => {
        impl Writable for $t {
            fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
                let mut written = VarInt(self.0.len() as i32).write(buf)?;
                for x in &self.0 {
                    written += x.write(buf)?;
                }

                Ok(written)
            }
        }
    };
}

pub(crate) use length_prefixed_sequence_writable;

impl<T: Readable + Debug, const C: usize> Readable for [T; C] {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let mut vec = Vec::new();
        for _ in 0..C {
            vec.push(T::read(buf)?);
        }

        Ok(vec.try_into().unwrap())
    }
}

impl<T: Writable, const C: usize> Writable for [T; C] {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut written = 0;
        for i in 0..self.len() {
            written += self[i].write(buf)?;
        }

        Ok(written)
    }
}

impl<T: Readable> Readable for Box<[T]> {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let len = VarInt::read(buf)?;

        let mut slice = Vec::new();
        for _ in 0..len.0 {
            slice.push(T::read(buf)?);
        }

        Ok(slice.into_boxed_slice())
    }
}

length_prefixed_sequence_writable!(Box<[T]>);

impl<T: Readable> Readable for Vec<T> {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let len = VarInt::read(buf)?;

        let mut vec = Vec::new();
        for _ in 0..len.0 {
            vec.push(T::read(buf)?);
        }

        Ok(vec)
    }
}

length_prefixed_sequence_writable!(Vec<T>);

/// Byte array with unspecified size. Must be read till the end of a packet.
/// This type is intended for exceptional cases (for example, for PluginMessage
/// packet). In most cases you must use Box<[T]>.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct ByteArray(Box<[u8]>);

impl Deref for ByteArray {
    type Target = Box<[u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Readable for ByteArray {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let mut bytes = Vec::new();
        buf.read(&mut bytes)?;

        Ok(Self(bytes.into_boxed_slice()))
    }
}

impl Writable for ByteArray {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        self.0.write(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::io::tests::ser_write_read_type_assert;

    #[test]
    fn array() {
        for array in [
            [0x01, 0x2F, 0x74],
            [0xA9, 0x5B, -0x23],
            [-0xC1, -0x60, -0xD2],
        ] {
            ser_write_read_type_assert(&array);
        }

        let array: [i32; 0] = [];

        ser_write_read_type_assert(&array);
    }

    #[test]
    fn boxed_slice() {
        for slice in [
            vec![0x33, -0x03, -0x3A, 0xA9, 0x12].into_boxed_slice(),
            vec![0x12, 0xC6, 0x8B, 0x83, 64].into_boxed_slice(),
            vec![0x55, 0x62, 0x28, -0xFF, 0xDB].into_boxed_slice(),
            vec![].into_boxed_slice(),
        ] {
            ser_write_read_type_assert(&slice);
        }
    }

    #[test]
    fn vec() {
        for vec in [
            vec![
                "Like Father Like Son".to_owned(),
                "Right Off the Bat".to_owned(),
                "Foaming At The Mouth".to_owned(),
            ],
            vec![
                "Short End of the Stick".to_owned(),
                "Wake Up Call".to_owned(),
                "Back To the Drawing Board".to_owned(),
                "Happy as a Clam".to_owned()
            ],
            vec![
                "".to_owned(),
                "".to_owned(),
            ],
            vec![]
        ] {
            ser_write_read_type_assert(&vec);
        }
    }

    #[test]
    fn byte_array() {
        for array in [
            vec![0xDBF3, 0x4D54, 0xDF82, 0x5E2E, 0x40FB].into_boxed_slice(),
            vec![0x7aa5, 0xac51, 0x0eb0, -0x715d, 0x88b1].into_boxed_slice(),
            vec![-0x6f4b, -0x4382].into_boxed_slice(),
            vec![].into_boxed_slice(),
        ] {
            ser_write_read_type_assert(&array);
        }
    }
}
