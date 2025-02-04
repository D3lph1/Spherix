use crate::vector::Vector3;
use std::ops::{Add, BitAnd, Deref, DerefMut, Mul, Sub};

pub trait VectorPlain: Clone {
    fn new_from(&self, x: i32, y: i32) -> Self;

    fn x(&self) -> i32;

    fn z(&self) -> i32;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Vector2(nalgebra::Vector2<i32>);

impl Vector2 {
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self(nalgebra::Vector2::new(x, z))
    }

    #[inline]
    pub const fn origin() -> Self {
        Self::new(0, 0)
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.y
    }
}

impl VectorPlain for Vector2 {
    fn new_from(&self, x: i32, y: i32) -> Self {
        Self(nalgebra::Vector2::new(x, y))
    }

    #[inline]
    fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    fn z(&self) -> i32 {
        self.y
    }
}

impl From<Vector2> for i64 {
    fn from(value: Vector2) -> Self {
        value.x as i64 & 4294967295 | (value.y as i64 & 4294967295) << 32
    }
}

impl Deref for Vector2 {
    type Target = nalgebra::Vector2<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vector2(self.0.add(rhs.0))
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2(self.0.sub(rhs.0))
    }
}

impl BitAnd<u32> for Vector2 {
    type Output = Vector2u;

    fn bitand(self, rhs: u32) -> Self::Output {
        Vector2u::new((self.x() & rhs as i32) as u32, (self.z() & rhs as i32) as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2u(nalgebra::Vector2<u32>);

impl Vector2u {
    #[inline]
    pub const fn new(x: u32, y: u32) -> Self {
        Self(nalgebra::Vector2::new(x, y))
    }

    #[inline]
    pub fn origin() -> Self {
        Self::new(0, 0)
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> u32 {
        self.0.y
    }
}

impl From<Vector3> for Vector2u {
    fn from(value: Vector3) -> Self {
        Vector2u::new(value.x as u32, value.y as u32)
    }
}

impl BitAnd<u32> for Vector2u {
    type Output = Vector2u;

    fn bitand(self, rhs: u32) -> Self::Output {
        Vector2u::new(self.x() & rhs, self.z() & rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2f(pub nalgebra::Vector2<f64>);

impl Vector2f {
    pub fn new(x: f64, z: f64) -> Self {
        Self(nalgebra::Vector2::new(x, z))
    }

    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.y
    }
}

impl Deref for Vector2f {
    type Target = nalgebra::Vector2<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector2f {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Vector2f {
    type Output = Vector2f;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Vector2f {
    type Output = Vector2f;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Vector2f {
    type Output = Vector2f;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(nalgebra::Vector2::new(self.x * rhs, self.z() * rhs))
    }
}

impl Mul<f64> for &Vector2f {
    type Output = Vector2f;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector2f(nalgebra::Vector2::new(self.x() * rhs, self.z() * rhs))
    }
}

impl From<Vec<f64>> for Vector2f {
    fn from(value: Vec<f64>) -> Self {
        if value.len() != 2 {
            panic!("Expected vector with 2 elements, but it has length {}", value.len())
        }

        Self::new(value[0], value[1])
    }
}

impl From<Vector2> for Vector2f {
    fn from(value: Vector2) -> Self {
        Self::new(value.x() as f64, value.z() as f64)
    }
}

impl From<&Vector2> for Vector2f {
    fn from(value: &Vector2) -> Self {
        Self::new(value.x() as f64, value.z() as f64)
    }
}
