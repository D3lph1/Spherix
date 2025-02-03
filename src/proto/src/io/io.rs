use std::io::{Read, Write};

use crate::io::Error;

/// Reads itself from the passed instance of [`Read`].
///
/// Primarily, this trait is implemented by all types (including primitives) that
/// could be read via network from the client.
pub trait Readable {
    /// Reads data from underlying source R wrapped with [`Read`].
    /// It is fully synchronous, and it is assumed that reading will be done from the
    /// buffer in memory W, not from the Tokio stream directly.
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error>
        where Self: Sized;
}

/// Writes itself to the passed instance of [`Write`].
///
/// Primarily, this trait is implemented by all types (including primitives) that
/// could be transferred through network to the client.
pub trait Writable {
    /// Writes data to underlying sink W wrapped with [`Write`].
    /// It is fully synchronous, and it is assumed that writing will be done to the
    /// buffer in memory W, not to the Tokio stream directly.
    ///
    /// For success, it returns number of written bytes.
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error>;
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::Debug;
    use std::io::{BufReader, BufWriter};
    use std::sync::{Arc, Mutex};

    use crate::io::io::{Readable, Writable};

    pub(crate) fn ser_write_read_type_assert<RW>(rw: &RW)
        where RW: Readable + Writable + PartialEq<RW> + Debug
    {
        ser_write_read_type(rw, &|x| {
            assert_eq!(*rw, x)
        })
    }

    pub(crate) fn ser_write_read_type<RW>(ser: &RW, read_fn: &dyn Fn(RW))
        where RW: Readable + Writable
    {
        ser_write_read_buf(ser, &|mut reader| {
            let pos = RW::read(&mut reader).unwrap();

            read_fn(pos);
        });
    }

    fn ser_write_read_buf<RW>(ser: &RW, read_fn: &dyn Fn(BufReader<&[u8]>))
        where RW: Readable + Writable
    {
        let mutex = Arc::new(Mutex::new(ser));

        write_read_buf(&|writer| {
            mutex.lock().unwrap().write(writer).unwrap();
        }, read_fn);
    }

    fn write_read_buf(write_fn: &dyn Fn(&mut BufWriter<Vec<u8>>), read_fn: &dyn Fn(BufReader<&[u8]>)) {
        let vec = Vec::new();
        let mut writer = BufWriter::new(vec);

        write_fn(&mut writer);

        let bytes = writer.buffer();
        let reader = BufReader::new(bytes);

        read_fn(reader);
    }
}

