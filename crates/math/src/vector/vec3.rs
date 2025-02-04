use spherix_util::math::smallest_encompassing_power_of_two;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, BitAnd, Deref, DerefMut, Div, Mul, Shl, Shr, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vector3(nalgebra::Vector3<i32>);

impl Vector3 {
    const PACKED_X_LENGTH: i64 = 1 + smallest_encompassing_power_of_two(30000000).ilog2() as i64;
    const PACKED_Y_LENGTH: i64 = 64 - Self::PACKED_X_LENGTH - Self::PACKED_Z_LENGTH;
    const PACKED_Z_LENGTH: i64 = Self::PACKED_X_LENGTH;

    const PACKED_X_MASK: i64 = (1 << Self::PACKED_X_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1 << Self::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1 << Self::PACKED_Z_LENGTH) - 1;

    const Y_OFFSET: i64 = 0;
    const Z_OFFSET: i64 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: i64 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(nalgebra::Vector3::new(x, y, z))
    }

    #[inline]
    pub const fn origin() -> Self {
        Self::new(0, 0, 0)
    }
    
    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.0.z
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    pub fn seed(&self) -> i64 {
        let mut i = self.x.wrapping_mul(3129871) as i64
            ^ (self.z as i64).wrapping_mul(116129781)
            ^ self.y as i64;
        i = i.wrapping_mul(i).wrapping_mul(42317861) + i * 11;

        i >> 16
    }
}

impl From<Vector3> for i64 {
    fn from(value: Vector3) -> Self {
        i64::from(&value)
    }
}

impl From<&Vector3> for i64 {
    fn from(value: &Vector3) -> Self {
        let mut val = (value.x as i64 & Vector3::PACKED_X_MASK) << Vector3::X_OFFSET;
        val |= (value.y as i64 & Vector3::PACKED_Y_MASK) << Vector3::Y_OFFSET;

        val | (value.z as i64 & Vector3::PACKED_Z_MASK) << Vector3::Z_OFFSET
    }
}

/// Macro to eliminate duplicate code for Vector3 and &Vector3 types
macro_rules! vector3_defs {
    ($($ty:ty),*) => {
        $(
            impl Add<i32> for $ty {
                type Output = Vector3;
            
                fn add(self, rhs: i32) -> Self::Output {
                    Vector3::new(self.x + rhs, self.y + rhs, self.z + rhs)
                }
            }
        
            impl Sub<i32> for $ty {
                type Output = Vector3;
            
                fn sub(self, rhs: i32) -> Self::Output {
                    Vector3::new(self.x - rhs, self.y - rhs, self.z - rhs)
                }
            }
            
            impl Div<f64> for $ty {
                type Output = Vector3f;
            
                fn div(self, rhs: f64) -> Self::Output {
                    Vector3f::new(self.x as f64 / rhs, self.y as f64 / rhs, self.z as f64 / rhs)
                }
            }
            
            impl Shr<i32> for $ty {
                type Output = Vector3;
            
                fn shr(self, rhs: i32) -> Self::Output {
                    Vector3::new(self.x >> rhs, self.y >> rhs, self.z >> rhs)
                }
            }
            
            impl Shl<i32> for $ty {
                type Output = Vector3;
            
                fn shl(self, rhs: i32) -> Self::Output {
                    Vector3::new(self.x << rhs, self.y << rhs, self.z << rhs)
                }
            }
        
            impl BitAnd<i32> for $ty {
                type Output = Vector3;
                
                fn bitand(self, rhs: i32) -> Self::Output {
                    Vector3::new(self.x & rhs, self.y & rhs, self.z & rhs)
                }
            }
        )*
    };
}

impl Deref for Vector3 {
    type Target = nalgebra::Vector3<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

vector3_defs!(Vector3, &Vector3);

#[derive(Clone, Copy)]
pub struct Vector3u(nalgebra::Vector3<u32>);

impl Debug for Vector3u {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector3u {{{}, {}, {}}}", self.x, self.z, self.y)
    }
}

impl Vector3u {
    #[inline]
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self(nalgebra::Vector3::new(x, y, z))
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> u32 {
        self.0.z
    }

    #[inline]
    pub fn y(&self) -> u32 {
        self.0.y
    }
}

impl Deref for Vector3u {
    type Target = nalgebra::Vector3<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector3u {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vector3> for Vector3u {
    fn from(value: Vector3) -> Self {
        Vector3u::new(value.x as u32, value.y as u32, value.z as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3f(pub nalgebra::Vector3<f64>);

impl Vector3f {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(nalgebra::Vector3::new(x, y, z))
    }
}

impl Deref for Vector3f {
    type Target = nalgebra::Vector3<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector3f {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Vector3f {
    type Output = Vector3f;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(nalgebra::Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs))
    }
}

impl Mul<f64> for &Vector3f {
    type Output = Vector3f;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector3f(nalgebra::Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs))
    }
}

impl From<Vec<f64>> for Vector3f {
    fn from(value: Vec<f64>) -> Self {
        if value.len() != 3 {
            panic!("Expected vector with 3 elements, but it has length {}", value.len())
        }

        Self::new(value[0], value[1], value[2])
    }
}

impl From<Vector3> for Vector3f {
    fn from(value: Vector3) -> Self {
        Self::new(value.x as f64, value.y as f64, value.z as f64)
    }
}

impl From<&Vector3> for Vector3f {
    fn from(value: &Vector3) -> Self {
        Self::new(value.x as f64, value.y as f64, value.z as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::vector::Vector3;

    #[test]
    fn vector3_to_i64() {
        assert_eq!(2377419014901760, <Vector3 as Into<i64>>::into(Vector3::new(8648, 0, -551)));
        assert_eq!(54975581577185, <Vector3 as Into<i64>>::into(Vector3::new(200, -31, 45)));
        assert_eq!(-5094862015115200, <Vector3 as Into<i64>>::into(Vector3::new(-18536, 64, -2419)));
    }

    #[test]
    fn vector3_seed() {
        assert_eq!(-59795694467688, Vector3::new(-100, 63, 18).seed());
        assert_eq!(7227615167957, Vector3::new(52928, -25, -94158127).seed());
    }
}
