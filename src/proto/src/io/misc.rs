use std::io::{Read, Write};

use nbt::{from_reader, Blob, Value};
use uuid::Uuid;

use crate::io::error::Error;
use crate::io::io::{Readable, Writable};

const UUID_LENGTH: usize = 128 / 8;

impl Readable for Uuid {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let mut bytes = [0u8; UUID_LENGTH];
        buf.read(&mut bytes)?;

        Ok(Uuid::from_slice(&bytes).unwrap())
    }
}

impl Writable for Uuid {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        Ok(buf.write(self.as_bytes())?)
    }
}

impl Readable for Value {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let val = from_reader::<_, Value>(buf);

        if val.is_err() {
            return Err(Error::Other);
        }

        Ok(val.unwrap())
    }
}

impl Writable for Value {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        // We have to use intermediate buffer in order to count amount of written bytes
        let mut tmp = Vec::<u8>::new();
        if self.to_writer(&mut tmp).is_err() {
            return Err(Error::Other);
        }

        buf.write(&tmp)?;

        Ok(tmp.len())
    }
}

impl Readable for Blob {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        Ok(Blob::from_reader(buf).map_err(|_| Error::Other)?)
    }
}

impl Writable for Blob {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        // We have to use intermediate buffer in order to count amount of written bytes
        let mut tmp = Vec::<u8>::new();
        if self.to_writer(&mut tmp).is_err() {
            return Err(Error::Other);
        }

        buf.write(&tmp)?;

        Ok(tmp.len())
    }
}

impl<T: Readable> Readable for Box<T> {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        Ok(Box::new(T::read(buf)?))
    }
}

impl<T: Writable> Writable for Box<T> {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        Ok(T::write(self, buf)?)
    }
}

impl<T: Writable> Writable for Option<T> {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        match self {
            None => false.write(buf),
            Some(x) => {
                let written = true.write(buf)?;
                Ok(written + x.write(buf)?)
            }
        }
    }
}

impl<T: Readable> Readable for Option<T> {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        if bool::read(buf)? {
            Ok(Some(T::read(buf)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::io::io::tests::ser_write_read_type_assert;

    #[test]
    fn uuid() {
        for val in [
            Uuid::parse_str("5d0b1e90-4071-42f0-8512-a9dc4c9e7af2").unwrap(),
            Uuid::parse_str("e5c0ece3-ed7c-44de-8816-b9e3eb50afe2").unwrap(),
            Uuid::parse_str("09626361-6fb9-4e76-b313-8a1202ab715f").unwrap(),
            Uuid::parse_str("8b023f1e-8307-4b2c-b432-deda6bf889b6").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        ] {
            ser_write_read_type_assert(&val);
        }

        for _ in 0..100 {
            ser_write_read_type_assert(&Uuid::new_v4());
        }
    }

    // #[test]
    // fn nbt() {
    //     for val in [
    //         nbt::Value::Int(10293874),
    //         nbt::Value::Long(829082394819),
    //         nbt::Value::Short(2341),
    //         nbt::Value::Compound(HashMap::from([
    //             ("first".to_owned(), nbt::Value::List(vec![nbt::Value::Byte(1), nbt::Value::String("example".to_owned())])),
    //             ("second".to_owned(), nbt::Value::Double(0.418)),
    //             ("third".to_owned(), nbt::Value::Compound(HashMap::from([
    //                 ("nested_1".to_owned(), nbt::Value::Int(-94)),
    //                 ("nested_2".to_owned(), nbt::Value::LongArray(vec![0, 0, -359, 171]))
    //             ])))
    //         ]))
    //     ] {
    //         ser_write_read_type_assert(&val);
    //     }
    // }

    #[test]
    fn option() {
        for val in [
            Some(1), Some(-4), Some(5), Some(241252937), Some(-432513009), None
        ] {
            ser_write_read_type_assert(&val);
        }
    }
}
