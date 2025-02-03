use crate::noise::density::density::DensityFunctionContext;
use crate::noise::settings::NoiseSettings;
use spherix_math::vector::Vector3;
use spherix_world::block::block::Block;
use spherix_world::block::state::BlockState;
use spherix_world::chunk::palette::BlockGlobalPalette;
use std::sync::Arc;

pub struct FluidStatus {
    level: i32,
    ty: Arc<BlockState>,
    default: Arc<BlockState>,
}

impl FluidStatus {
    #[inline]
    pub fn at(&self, level: i32) -> Arc<BlockState> {
        if level < self.level {
            self.ty.clone()
        } else {
            self.default.clone()
        }
    }
}

pub struct FluidPicker {
    palette: Arc<BlockGlobalPalette>,
    lava_fluid: FluidStatus,
    default_fluid: FluidStatus,
}

impl FluidPicker {
    pub fn create(noise_settings: &NoiseSettings, palette: &Arc<BlockGlobalPalette>) -> Self {
        let air = palette.get_default_obj_by_index(&Block::AIR).unwrap();

        Self {
            palette: palette.clone(),
            lava_fluid: FluidStatus {
                level: -54,
                ty: palette.get_default_obj_by_index(&Block::LAVA).unwrap(),
                default: air.clone(),
            },
            default_fluid: FluidStatus {
                level: noise_settings.sea_level,
                ty: noise_settings.default_fluid.clone(),
                default: air,
            },
        }
    }

    fn pick(&self, pos: &Vector3) -> &FluidStatus {
        if pos.y < self.default_fluid.level.min(-54) {
            &self.lava_fluid
        } else {
            &self.default_fluid
        }
    }
}

pub trait Aquifer {
    fn compute(&self, ctx: &DensityFunctionContext, noise_value: f64) -> Option<Arc<BlockState>>;
}

pub struct DisabledAquifer {
    picker: FluidPicker
}

impl DisabledAquifer {
    #[inline]
    pub fn new(picker: FluidPicker) -> Self {
        Self {
            picker,
        }
    }
}

impl Aquifer for DisabledAquifer {
    fn compute(&self, ctx: &DensityFunctionContext, noise_value: f64) -> Option<Arc<BlockState>> {
        if noise_value > 0.0 {
            None
        } else {
            Some(self.picker.pick(&ctx.pos()).at(ctx.pos().y))
        }
    }
}
