use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::BufReader;
use std::io::{Read, Write};
use std::ops::{Add, Mul, Sub};

use spherix_util::hex::decode;

use crate::io::error::Error;
use crate::io::io::{Readable, Writable};

macro_rules! var_impl {
    (
        $t:ident, $prim:ty, $max_len:literal
    ) => {
        impl $t {
            pub const MAX_LENGTH: usize = $max_len;

            const SEGMENT_BITS: $prim = 0x7F;
            const CONTINUE_BIT: $prim = 0x80;

            #[allow(unused)]
            pub(crate) fn from_hex(hex: &String) -> Result<Self, Error> {
                Self::read(&mut BufReader::new(decode(hex).unwrap().as_slice()))
            }
        }

        impl Display for $t {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<$prim> for $t {
            fn from(value: $prim) -> Self {
                $t(value)
            }
        }

        impl PartialEq<$prim> for $t {
            fn eq(&self, other: &$prim) -> bool {
                self.0 == *other
            }
        }

        impl PartialOrd<$prim> for $t {
            fn partial_cmp(&self, other: &$prim) -> Option<Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl Add<$t> for $t {
            type Output = $t;

            fn add(self, rhs: $t) -> Self::Output {
                $t(self.0 + rhs.0)
            }
        }

        impl Sub<$t> for $t {
            type Output = $t;

            fn sub(self, rhs: $t) -> Self::Output {
                $t(self.0 - rhs.0)
            }
        }

        impl Mul<$t> for $t {
            type Output = $t;

            fn mul(self, rhs: $t) -> Self::Output {
                $t(self.0 * rhs.0)
            }
        }

        var_ops_impl!($t, $prim);
        var_ops_impl!($t, i16 as $prim);
        var_ops_impl!($t, isize as $prim);
        var_ops_impl!($t, usize as $prim);
    }
}

macro_rules! var_ops_impl {
    ($t:ident, $prim:ty $(as $cast:ty)*) => {
        impl Add<$prim> for $t {
            type Output = $t;

            fn add(self, rhs: $prim) -> Self::Output {
                $t(self.0 + rhs $(as $cast)*)
            }
        }

        impl Sub<$prim> for $t {
            type Output = $t;

            fn sub(self, rhs: $prim) -> Self::Output {
                $t(self.0 - rhs $(as $cast)*)
            }
        }

        impl Mul<$prim> for $t {
            type Output = $t;

            fn mul(self, rhs: $prim) -> Self::Output {
                $t(self.0 * rhs $(as $cast)*)
            }
        }
    };
}

/// Compact i32 implementation small values of which occupies less space in memory.
///
/// [`Read more`] about it.
///
/// [`Read more`]: https://wiki.vg/Protocol#VarInt_and_VarLong
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(test, derive(Default))]
pub struct VarInt(pub i32);

var_impl!(VarInt, i32, 5);
var_ops_impl!(VarInt, i64 as i32);

impl Readable for VarInt {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        let mut current_byte: u8;

        loop {
            let mut tmp = [0];
            let res = buf.read_exact(&mut tmp);
            if res.is_err() {
                return Err(Error::Eof);
            }

            current_byte = tmp[0];

            value |= (i32::from(current_byte) & VarInt::SEGMENT_BITS) << position;

            if (i32::from(current_byte) & VarInt::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                // VarInt is too big
                return Err(Error::TooBig);
            }
        }

        return Ok(VarInt(value));
    }
}

impl Writable for VarInt {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut value = self.0 as u32;
        let mut written = 0;

        loop {
            if (value & !VarInt::SEGMENT_BITS as u32) == 0 {
                let tmp = [value as u8];
                let res = buf.write(tmp.as_slice())?;

                written += res;

                return Ok(written);
            }

            let tmp = [((value & VarInt::SEGMENT_BITS as u32) | VarInt::CONTINUE_BIT as u32) as u8];
            let res = buf.write(tmp.as_slice())?;

            written += res;

            value = value >> 7;
        }
    }
}


/// Compact i64 implementation small values of which occupies less space in memory.
///
/// [`Read more`] about it.
///
/// [`Read more`]: https://wiki.vg/Protocol#VarInt_and_VarLong
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct VarLong(pub i64);

var_impl!(VarLong, i64, 10);
var_ops_impl!(VarLong, i32 as i64);

impl Readable for VarLong {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let mut value: i64 = 0;
        let mut position: i64 = 0;
        let mut current_byte: u8;

        loop {
            let mut tmp = [0];
            let res = buf.read_exact(&mut tmp);
            if res.is_err() {
                return Err(Error::Eof);
            }

            current_byte = tmp[0];

            value |= (i64::from(current_byte) & VarLong::SEGMENT_BITS) << position;

            if (i64::from(current_byte) & VarLong::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 64 {
                // VarLong is too big
                return Err(Error::TooBig);
            }
        }

        return Ok(VarLong(value));
    }
}

impl Writable for VarLong {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut value = self.0 as u64;
        let mut written = 0;

        loop {
            if (value & !VarLong::SEGMENT_BITS as u64) == 0 {
                let tmp = [value as u8];
                let res = buf.write(tmp.as_slice())?;

                written += res;

                return Ok(written);
            }

            let tmp = [((value & VarLong::SEGMENT_BITS as u64) | VarLong::CONTINUE_BIT as u64) as u8];
            let res = buf.write(tmp.as_slice())?;

            written += res;

            value = value >> 7;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::io::Cursor;

    use crate::io::io::tests::ser_write_read_type_assert;
    use crate::io::io::{Readable, Writable};
    use crate::io::{VarInt, VarLong};

    #[test]
    fn var_int() {
        for (input, out) in [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08])
        ] {
            let mut buf = Vec::new();
            VarInt(input).write(&mut buf).unwrap();

            assert_eq!(out, buf);

            let mut cursor = Cursor::new(buf);
            assert_eq!(input, VarInt::read(&mut cursor).unwrap().0);
        }
    }

    #[test]
    fn var_int_symmetric() {
        for hex in ["AD33", "7D"] {
            let val = VarInt::from_hex(&String::from(hex)).unwrap();

            ser_write_read_type_assert(&val);
        }
    }

    #[test]
    fn var_long() {
        for (input, out) in [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (9223372036854775807, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-9223372036854775808, vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01])
        ] {
            let mut buf = Vec::new();
            VarLong(input).write(&mut buf).unwrap();

            assert_eq!(out, buf);

            let mut cursor = Cursor::new(buf);
            assert_eq!(input, VarLong::read(&mut cursor).unwrap().0);
        }
    }

    #[test]
    pub(crate) fn var_long_symmetric() {
        for hex in ["84FCD0A9EF7E", "C8EBAFD1D425"] {
            let val = VarLong::from_hex(&String::from(hex)).unwrap();

            ser_write_read_type_assert(&val);
        }
    }
}
