use crate::biome::climate::rtree::point::{max_inline, PointExt};
use num_traits::{Bounded, One, Zero};
use rstar::{Envelope, Point, RTreeObject};
use std::ops::{Add, Mul, Sub};

/// Custom implementation of Axis-Aligned Bounding Box for RTree. It is based on
/// default [`rstar::AABB`] from rstar crate with modification required for
/// operations with large integers.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct AABB<P>
where
    P: Point,
{
    lower: P,
    upper: P,
}

impl<P> AABB<P>
where
    P: Point
{
    /// See [`rstar::AABB::lower()`].
    pub fn lower(&self) -> P {
        self.lower.clone()
    }

    /// See [`rstar::AABB::upper()`].
    pub fn upper(&self) -> P {
        self.upper.clone()
    }

    /// See [`rstar::AABB::from_corners()`].
    pub fn from_corners(p1: P, p2: P) -> Self {
        AABB {
            lower: p1.min_point(&p2),
            upper: p1.max_point(&p2),
        }
    }

    /// See [`rstar::AABB::min_point()`].
    pub fn min_point(&self, point: &P) -> P {
        self.upper.min_point(&self.lower.max_point(point))
    }

    /// Calculates and returns 100 as a [`P::Scalar`] type by adding units and
    /// multiplying obtained numbers. We have to use this approach because
    /// the trait of [`P::Scalar`] ([`rstar::RTreeNum`]) does not extend
    /// convenient trait [`num_traits::NumCast`].
    #[inline]
    fn hundred() -> P::Scalar {
        let one = P::Scalar::one();
        let mut ten = one;

        // In release builds the optimizer replaces it by a single const
        for _ in 0..9 {
            ten = ten.add(one);
        }

        // In order to not increase performance overhead in debug builds with 99 iterations
        // loop above, we just multiply 10 on 10 to obtain 100.
        ten.mul(ten)
    }
}

impl<P> Envelope for AABB<P>
where
    P: Point,
{
    type Point = P;

    /// See [`rstar::AABB::new_empty()`].
    fn new_empty() -> Self {
        let max = P::Scalar::max_value();
        let min = P::Scalar::min_value();
        AABB {
            lower: P::from_value(max),
            upper: P::from_value(min),
        }
    }

    /// See [`rstar::AABB::contains_point()`].
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.lower.all_component_wise(point, |x, y| x <= y)
            && self.upper.all_component_wise(point, |x, y| x >= y)
    }

    /// See [`rstar::AABB::contains_envelope()`].
    fn contains_envelope(&self, other: &Self) -> bool {
        self.lower.all_component_wise(&other.lower, |l, r| l <= r)
            && self.upper.all_component_wise(&other.upper, |l, r| l >= r)
    }

    /// See [`rstar::AABB::merge()`].
    fn merge(&mut self, other: &Self) {
        self.lower = self.lower.min_point(&other.lower);
        self.upper = self.upper.max_point(&other.upper);
    }

    /// See [`rstar::AABB::merged()`].
    fn merged(&self, other: &Self) -> Self {
        AABB {
            lower: self.lower.min_point(&other.lower),
            upper: self.upper.max_point(&other.upper),
        }
    }

    /// See [`rstar::AABB::intersects()`].
    fn intersects(&self, other: &Self) -> bool {
        self.lower.all_component_wise(&other.upper, |l, r| l <= r)
            && self.upper.all_component_wise(&other.lower, |l, r| l >= r)
    }

    /// See [`rstar::AABB::intersection_area()`].
    fn intersection_area(&self, other: &Self) -> <Self::Point as Point>::Scalar {
        AABB {
            lower: self.lower.max_point(&other.lower),
            upper: self.upper.min_point(&other.upper),
        }
            .area()
    }

    /// Method based on [`rstar::AABB::area()`].
    /// This code calculates the area (or volume in general) of an AABB (Axis-Aligned
    /// Bounding Box). It iterates over the dimensions of the box and multiplies the
    /// length of each dimension together to get the total area.
    /// 
    /// The original implementation did not have the division by 100 in the calculation
    /// of the diagonal. This change was added to prevent potential overflow issues
    /// when dealing with very large numbers.
    /// While this change may slightly reduce the accuracy of the area calculation, it
    /// does not affect the core logic of the RTree building process. The RTree uses
    /// these AABB’s for spatial indexing and the slight reduction in accuracy in the
    /// area calculation should not significantly impact the overall performance of
    /// the search operations.
    fn area(&self) -> <Self::Point as Point>::Scalar {
        let zero = P::Scalar::zero();
        let one = P::Scalar::one();

        // The original implementation did not have this division by 100.
        // This change was introduced to prevent overflow when dealing with large numbers.
        // It does not affect the logic of the RTree building process.
        let diag = self.upper.sub(&self.lower).div(Self::hundred());
        diag.fold(one, |acc, cur| max_inline(cur, zero) * acc)
    }
    
    /// This function calculates the squared distance between a point and the AABB.
    /// It operates by considering the distance from the point to each axis-aligned
    /// plane of the bounding box.
    /// 
    /// For each dimension, the code calculates the distance from the point to the
    /// closest plane, squares that distance, and adds it to the sum. The final
    /// squared distance is returned as the sum of the squared distances across all
    /// dimensions.
    /// 
    /// Such metric is proposed by the original Vanilla Minecraft implementation.
    fn distance_2(&self, point: &Self::Point) -> <Self::Point as Point>::Scalar {
        let mut sum = P::Scalar::zero();

        for i in 0..Self::Point::DIMENSIONS {
            let comp = point.nth(i);
            let min = self.lower.nth(i);
            let max = self.upper.nth(i);
            let d1 = comp.sub(max);
            let d2 = min.sub(comp);

            let zero = P::Scalar::zero();
            let to_sum = if d1 > zero { d1 } else { max_inline(d2, zero) };

            sum = sum.add(to_sum * to_sum);
        }

        sum
    }

    /// See [`rstar::AABB::min_max_dist_2()`].
    fn min_max_dist_2(&self, point: &Self::Point) -> <Self::Point as Point>::Scalar {
        let l = self.lower.sub(point);
        let u = self.upper.sub(point);
        let mut max_diff = (Zero::zero(), Zero::zero(), 0); // diff, min, index
        let mut result = P::new();

        for i in 0..P::DIMENSIONS {
            let mut min = l.nth(i);
            let mut max = u.nth(i);
            max = max * max;
            min = min * min;
            if max < min {
                core::mem::swap(&mut min, &mut max);
            }

            let diff = max - min;
            *result.nth_mut(i) = max;

            if diff >= max_diff.0 {
                max_diff = (diff, min, i);
            }
        }

        *result.nth_mut(max_diff.2) = max_diff.1;
        result.fold(Zero::zero(), |acc, curr| acc + curr)
    }

    /// See [`rstar::AABB::center()`].
    fn center(&self) -> Self::Point {
        let one = <Self::Point as Point>::Scalar::one();
        let two = one + one;
        self.lower.component_wise(&self.upper, |x, y| (x + y) / two)
    }

    /// See [`rstar::AABB::perimeter_value()`].
    fn perimeter_value(&self) -> <Self::Point as Point>::Scalar {
        let diag = self.upper.sub(&self.lower);
        let zero = P::Scalar::zero();
        max_inline(diag.fold(zero, |acc, value| acc + value), zero)
    }

    /// See [`rstar::AABB::sort_envelopes()`].
    fn sort_envelopes<T: RTreeObject<Envelope=Self>>(axis: usize, envelopes: &mut [T]) {
        envelopes.sort_by(|l, r| {
            l.envelope()
                .lower
                .nth(axis)
                .partial_cmp(&r.envelope().lower.nth(axis))
                .unwrap()
        });
    }

    /// See [`rstar::AABB::partition_envelopes()`].
    fn partition_envelopes<T: RTreeObject<Envelope=Self>>(axis: usize, envelopes: &mut [T], selection_size: usize) {
        envelopes.select_nth_unstable_by(selection_size, |l, r| {
            l.envelope()
                .lower
                .nth(axis)
                .partial_cmp(&r.envelope().lower.nth(axis))
                .unwrap()
        });
    }
}
