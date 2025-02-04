use crate::entities::component_with_inner;
use bevy_ecs::prelude::Component;
use spherix_math::vector::Vector3f;
use spherix_world::chunk::pos::ChunkPos;

component_with_inner!(Position(Vector3f), PartialEq);

impl From<Position> for ChunkPos {
    fn from(value: Position) -> Self {
        value.0.into()
    }
}

#[derive(Clone, Debug, PartialEq, Component)]
pub struct Rotation {
    pub yaw: Angle,
    pub pitch: Angle
}

impl Rotation {
    pub fn new(yaw: Angle, pitch: Angle) -> Self {
        Self {
            yaw,
            pitch
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Angle(pub f32);

impl Angle {
    const F32_FACTOR: f32 = 256.0 / 360.0;

    pub fn new(degrees: f32) -> Self {
        Self(degrees)
    }

    #[inline]
    pub fn degrees(&self) -> f32 {
        self.0
    }

    #[inline]
    pub fn degrees_normalized(&self) -> f32 {
        Self::normalize(self.0)
    }

    fn normalize(degrees: f32) -> f32 {
        let mut normalized_angle = degrees % 360.0;
        if normalized_angle < 0.0 {
            normalized_angle += 360.0;
        }
        normalized_angle
    }
}

impl From<Angle> for spherix_proto::io::Angle {
    fn from(value: Angle) -> Self {
        Self((Angle::normalize(value.0) * Angle::F32_FACTOR) as u8)
    }
}

impl From<spherix_proto::io::Angle> for Angle {
    fn from(value: spherix_proto::io::Angle) -> Self {
        Self(value.0 as f32 / Angle::F32_FACTOR)
    }
}
