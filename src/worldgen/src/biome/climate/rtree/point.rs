use num_traits::Zero;
use rstar::{Point, RTreeNum};

impl<T> PointExt for T where T: Point {}

pub trait PointExt: Point {
    fn new() -> Self {
        Self::from_value(Zero::zero())
    }

    fn component_wise(
        &self,
        other: &Self,
        mut f: impl FnMut(Self::Scalar, Self::Scalar) -> Self::Scalar,
    ) -> Self {
        Self::generate(|i| f(self.nth(i), other.nth(i)))
    }
    
    fn all_component_wise(
        &self,
        other: &Self,
        mut f: impl FnMut(Self::Scalar, Self::Scalar) -> bool,
    ) -> bool {
        for i in 0..Self::DIMENSIONS {
            if !f(self.nth(i), other.nth(i)) {
                return false;
            }
        }
        true
    }
    
    fn fold<T>(&self, start_value: T, mut f: impl FnMut(T, Self::Scalar) -> T) -> T {
        let mut accumulated = start_value;
        for i in 0..Self::DIMENSIONS {
            accumulated = f(accumulated, self.nth(i));
        }
        accumulated
    }

    fn from_value(value: Self::Scalar) -> Self {
        Self::generate(|_| value)
    }

    fn min_point(&self, other: &Self) -> Self {
        self.component_wise(other, min_inline)
    }

    fn max_point(&self, other: &Self) -> Self {
        self.component_wise(other, max_inline)
    }

    fn length_2(&self) -> Self::Scalar {
        self.fold(Zero::zero(), |acc, cur| cur * cur + acc)
    }

    fn sub(&self, other: &Self) -> Self {
        self.component_wise(other, |l, r| l - r)
    }

    fn add(&self, other: &Self) -> Self {
        self.component_wise(other, |l, r| l + r)
    }

    fn mul(&self, scalar: Self::Scalar) -> Self {
        self.map(|coordinate| coordinate * scalar)
    }

    fn div(&self, scalar: Self::Scalar) -> Self {
        self.map(|coordinate| coordinate / scalar)
    }
    
    fn map(&self, mut f: impl FnMut(Self::Scalar) -> Self::Scalar) -> Self {
        Self::generate(|i| f(self.nth(i)))
    }

    fn distance_2(&self, other: &Self) -> Self::Scalar {
        self.sub(other).length_2()
    }
}

#[inline]
pub fn min_inline<S>(a: S, b: S) -> S
where
    S: RTreeNum,
{
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn max_inline<S>(a: S, b: S) -> S
where
    S: RTreeNum,
{
    if a > b {
        a
    } else {
        b
    }
}
