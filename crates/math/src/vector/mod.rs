pub mod iterator;
pub mod vec2;
pub mod vec3;

pub use vec2::{Vector2, Vector2f, VectorPlain};
pub use vec3::{Vector3, Vector3f, Vector3u};

pub use iterator::{OrderedSquareIter, RadialIter, UnorderedSquareIter};
