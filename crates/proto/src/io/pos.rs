use std::fmt::{Debug, Formatter};
use std::io::{Read, Write};

use owo_colors::OwoColorize;

use crate::io::error::Error;
use crate::io::io::{Readable, Writable};
use crate::io::Long;

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct Position {
    // [-33554432, 33554431]
    x: i32,
    // [-2048, 2047]
    y: i32,
    // [-33554432, 33554431]
    z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x.red(), self.y.green(), self.z.blue())
    }
}

impl Readable for Position {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let val = Long::read(buf)?;

        let x = val >> 38;
        let y = val << 52 >> 52;
        let z = val << 26 >> 38;

        Ok(Position::new(x as i32, y as i32, z as i32))
    }
}

impl Writable for Position {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let val = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);

        Ok(buf.write(&val.to_be_bytes())?)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Default, PartialEq))]
pub struct Angle(pub u8);

impl Readable for Angle {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let mut byte = [0 as u8; 1];
        buf.read_exact(&mut byte)?;

        return Ok(Angle(byte[0]));
    }
}

impl Writable for Angle {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let byte = [self.0];

        Ok(buf.write(&byte)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::io::tests::{ser_write_read_type, ser_write_read_type_assert};
    use crate::io::{Angle, Position};

    #[test]
    fn position() {
        const X: i32 = 510;
        const Y: i32 = 67;
        const Z: i32 = 200;

        let position = Position::new(X, Y, Z);

        ser_write_read_type(&position, &|pos: _| {
            assert_eq!(X, pos.x);
            assert_eq!(Y, pos.y);
            assert_eq!(Z, pos.z);
        })
    }

    #[test]
    fn angle() {
        const ANGLE: u8 = 241;

        let angle = Angle(ANGLE);

        ser_write_read_type_assert(&angle)
    }
}
