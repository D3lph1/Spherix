use crate::biome::climate::rtree::aabb::AABB;
use crate::biome::climate::rtree::point::PointExt;
use rstar::{Envelope, Point, PointDistance, RTreeObject};

/// Just like the original [`rstar::primitives::Rectangle`] but uses [`custom implementation`]
/// of AABB.
/// 
/// [`custom implementation`]: AABB
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rectangle<P>
where
    P: Point,
{
    aabb: AABB<P>,
}

impl<P> Rectangle<P>
where
    P: Point,
{
    /// Creates a new rectangle defined by two corners.
    pub fn from_corners(corner_1: P, corner_2: P) -> Self {
        AABB::from_corners(corner_1, corner_2).into()
    }

    pub fn from_aabb(aabb: AABB<P>) -> Self {
        Self { aabb }
    }

    pub fn lower(&self) -> P {
        self.aabb.lower()
    }

    pub fn upper(&self) -> P {
        self.aabb.upper()
    }
}

impl<P> From<AABB<P>> for Rectangle<P>
where
    P: Point,
{
    fn from(aabb: AABB<P>) -> Self {
        Self::from_aabb(aabb)
    }
}

impl<P> RTreeObject for Rectangle<P>
where
    P: Point,
{
    type Envelope = AABB<P>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb.clone()
    }
}

impl<P> Rectangle<P>
where
    P: Point,
{
    pub fn nearest_point(&self, query_point: &P) -> P {
        self.aabb.min_point(query_point)
    }
}

impl<P> PointDistance for Rectangle<P>
where
    P: Point,
{
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        self.nearest_point(point).sub(point).length_2()
    }

    fn contains_point(&self, point: &<Self::Envelope as Envelope>::Point) -> bool {
        self.aabb.contains_point(point)
    }

    fn distance_2_if_less_or_equal(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
        max_distance_2: <<Self::Envelope as Envelope>::Point as Point>::Scalar,
    ) -> Option<<<Self::Envelope as Envelope>::Point as Point>::Scalar> {
        let distance_2 = self.distance_2(point);
        if distance_2 <= max_distance_2 {
            Some(distance_2)
        } else {
            None
        }
    }
}
