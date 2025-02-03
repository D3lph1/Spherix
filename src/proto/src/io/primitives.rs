use std::io::{Read, Write};
use std::mem;

use crate::io::error::Error;
use crate::io::io::{Readable, Writable};
use crate::io::VarInt;

macro_rules! read_write_impl {
    ($t:ty)=>{
        impl Readable for $t {
            fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
                let mut bytes = [0u8; mem::size_of::<$t>()];
                buf.read_exact(&mut bytes)?;

                Ok(<$t>::from_be_bytes(bytes))
            }
        }

        impl Writable for $t {
            fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
                let bytes = self.to_be_bytes();

                Ok(buf.write(&bytes)?)
            }
        }
    }
}

// Type aliases for Rust's primitives corresponds to type names in Minecraft
// Protocol documentation.

/// Do not implement traits for signed byte. It is required to cast
/// UnsignedByte to Byte in order to obtain signed byte type
pub type Byte = i8;

impl Readable for Byte {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        Ok(UnsignedByte::read(buf)? as Byte)
    }
}

impl Writable for Byte {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        (*self as UnsignedByte).write(buf)
    }
}

pub type UnsignedByte = u8;
read_write_impl!(UnsignedByte);

pub type Short = i16;
read_write_impl!(Short);

pub type UnsignedShort = u16;
read_write_impl!(UnsignedShort);

pub type Int = i32;
read_write_impl!(Int);

pub type Long = i64;
read_write_impl!(Long);

read_write_impl!(u64);

pub type Float = f32;
read_write_impl!(Float);

pub type Double = f64;
read_write_impl!(Double);

impl Readable for bool {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let mut byte = [0u8; 1];
        buf.read(&mut byte)?;

        Ok(if byte[0] == 0x00 { false } else { true })
    }
}

impl Writable for bool {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let byte = vec![if *self { 0x01 } else { 0x00 }; 1];

        return Ok(buf.write(&byte)?);
    }
}

impl Readable for String {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let len = VarInt::read(buf)?;

        let mut bytes = vec![0; len.0 as usize];
        buf.read(&mut bytes)?;

        return Ok(String::from_utf8(bytes)?);
    }
}

impl Writable for String {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let len = VarInt(self.len() as i32);
        let written = len.write(buf)?;

        return Ok(written + buf.write(self.as_bytes())?);
    }
}

#[cfg(test)]
mod tests {
    use crate::io::io::tests::ser_write_read_type_assert;

    #[test]
    fn byte() {
        for val in [
            27i8,
            -54i8,
            127i8,
            -128i8,
            0
        ] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn short() {
        for val in [
            13414i16,
            -20712i16,
            29821i16,
            0
        ] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn unsigned_short() {
        for val in [
            20552i16,
            31792i16,
            14801i16,
            0
        ] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn i32() {
        for val in [415, -97, 0, 238] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn i64() {
        for val in [
            5185022206092528496i64,
            -2601411373777554811i64,
            3665597292681087282i64,
            0
        ] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn float() {
        for val in [0.12f32, 1.4f32, -5.1f32, 0f32] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn double() {
        for val in [0.12, 1.4, -5.1, 0.0] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn bool() {
        for val in [false, true] {
            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn string() {
        for val in [
            "Yada Yada".to_owned(),
            "Mountain Out of a Molehill".to_owned(),
            "You Can't Teach an Old Dog New Tricks".to_owned(),
            "".to_owned()
        ] {
            ser_write_read_type_assert(&val);
        }
    }
}
