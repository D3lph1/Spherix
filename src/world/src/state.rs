use crate::chunk::palette::BlockGlobalPalette;
use std::sync::OnceLock;

pub const BLOCK_PALETTE: OnceLock<BlockGlobalPalette> = OnceLock::new();

pub const BIOME_PALETTE: OnceLock<BlockGlobalPalette> = OnceLock::new();
