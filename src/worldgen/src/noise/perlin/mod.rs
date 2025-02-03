pub mod noise;
pub mod octave;
pub mod double;
pub mod inner;
pub mod simplex;
pub mod grid;

pub use double::DoubleMultiOctavePerlinNoise;
pub use grid::{GridNoise, LegacyMultiOctaveGridNoise};
pub use noise::{LegacyNoise, Noise};
pub use octave::MultiOctaveNoise;
pub use simplex::{MultiOctaveNoise as SimplexMultiOctaveNoise, SimplexNoise};


pub type DefaultNoise = DoubleMultiOctavePerlinNoise<MultiOctaveNoise<GridNoise>>;
