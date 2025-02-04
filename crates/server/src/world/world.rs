use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy_ecs::prelude::Resource;

use spherix_world::dimension::DimensionKind;

use crate::world::dimension::Dimension;
use spherix_world::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};

#[allow(dead_code)]
#[derive(Resource)]
pub struct World {
    dir: PathBuf,
    palette: Arc<BlockGlobalPalette>,
    dimensions: HashMap<DimensionKind, Dimension>
}

impl World {
    pub fn new(dir: PathBuf, palette: Arc<BlockGlobalPalette>, biomes_palette: Arc<BiomeGlobalPalette>) -> Self {
        let dimensions = Self::create_dimensions(dir.clone(), palette.clone(), biomes_palette);

        Self {
            dir,
            palette,
            dimensions
        }
    }

    fn create_dimensions(dir: PathBuf, palette: Arc<BlockGlobalPalette>, biomes_palette: Arc<BiomeGlobalPalette>) -> HashMap<DimensionKind, Dimension> {
        HashMap::from([
            (DimensionKind::Overworld, Dimension::new(DimensionKind::Overworld, dir.clone(), palette.clone(), biomes_palette.clone())),
            (DimensionKind::TheNether, Dimension::new(DimensionKind::TheNether, dir.clone(), palette.clone(), biomes_palette.clone())),
            (DimensionKind::TheEnd, Dimension::new(DimensionKind::TheEnd, dir, palette, biomes_palette))
        ])
    }

    pub fn dimension(&self, kind: DimensionKind) -> &Dimension {
        self.dimensions.get(&kind).unwrap()
    }

    pub fn dimension_mut(&mut self, kind: DimensionKind) -> &mut Dimension {
        self.dimensions.get_mut(&kind).unwrap()
    }

    pub fn dimensions(&self) -> &HashMap<DimensionKind, Dimension> {
        &self.dimensions
    }
}
