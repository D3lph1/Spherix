use crate::noise::density::cache::quart_pos_to_block;
use crate::noise::density::density::DensityFunctions;
use crate::noise::json::Resolver;
use crate::noise::router::NoiseRouter;
use anyhow::anyhow;
use serde_json::Value;
use spherix_world::block::block::BLOCKS;
use spherix_world::block::state::BlockState;
use spherix_world::chunk::palette::BlockGlobalPalette;
use std::sync::Arc;

#[derive(Clone)]
pub struct NoiseSettings {
    pub router: NoiseRouter,
    pub use_legacy_random_source: bool,
    pub default_block: Arc<BlockState>,
    pub default_fluid: Arc<BlockState>,
    pub sea_level: i32,
    pub noise_height: u32,
    pub noise_min_y: i32,
    pub noise_size_horizontal: i32,
    pub noise_size_vertical: i32,
}

impl NoiseSettings {
    pub fn from_json(json: &Value, resolver: &mut Resolver<DensityFunctions>, palette: Arc<BlockGlobalPalette>) -> anyhow::Result<Self> {
        let Value::Object(map) = json else {
            return Err(anyhow!("Expected object, found {:?}", json))
        };

        let noise_router_json = if map.contains_key("noise_router") {
            map.get("noise_router").unwrap()
        } else {
            return Err(anyhow!("No \"noise_router\" key"))
        };

        // TODO: Mb write generic function for deserialization default_block / default_fluid,
        // which will support any block / fluid???

        let default_block_name = if map.contains_key("default_block") {
            let default_block = map.get("default_block").unwrap();

            let Value::Object(default_block) = default_block else {
                return Err(anyhow!("Expected object, found {:?}", json))
            };

            if default_block.contains_key("Name") {
                let default_block_name = default_block.get("Name").unwrap();

                let Value::String(default_block_name) = default_block_name else {
                    return Err(anyhow!("Expected string, found {:?}", json))
                };

                default_block_name
            } else {
                return Err(anyhow!("No \"default_block.Name\" key"))
            }
        } else {
            return Err(anyhow!("No \"default_block\" key"))
        };
        
        let default_block = BLOCKS.get(default_block_name.as_str()).unwrap();

        let default_fluid_name = if map.contains_key("default_fluid") {
            let default_fluid = map.get("default_fluid").unwrap();

            let Value::Object(default_fluid) = default_fluid else {
                return Err(anyhow!("Expected object, found {:?}", json))
            };

            if default_fluid.contains_key("Name") {
                let default_fluid_name = default_fluid.get("Name").unwrap();

                let Value::String(default_fluid_name) = default_fluid_name else {
                    return Err(anyhow!("Expected string, found {:?}", json))
                };

                // TODO: fluid properties deserialization???

                default_fluid_name
            } else {
                return Err(anyhow!("No \"default_block.Name\" key"))
            }
        } else {
            return Err(anyhow!("No \"default_fluid\" key"))
        };

        let default_fluid = BLOCKS.get(default_fluid_name.as_str()).unwrap();

        let sea_level = if map.contains_key("sea_level") {
            let sea_level = map.get("sea_level").unwrap();

            let Value::Number(sea_level) = sea_level else {
                return Err(anyhow!("Expected number, found {:?}", json))
            };

            sea_level.as_i64().unwrap() as i32
        } else {
            return Err(anyhow!("No \"sea_level\" key"))
        };

        let noise = if map.contains_key("noise") {
            let noise = map.get("noise").unwrap();

            let Value::Object(noise) = noise else {
                return Err(anyhow!("Expected object, found {:?}", json))
            };

            noise
        } else {
            return Err(anyhow!("No \"noise\" key"))
        };

        let noise_height = if noise.contains_key("height") {
            let noise_height = noise.get("height").unwrap();

            let Value::Number(noise_height) = noise_height else {
                return Err(anyhow!("Expected number, found {:?}", json))
            };

            noise_height.as_u64().unwrap() as u32
        } else {
            return Err(anyhow!("No \"noise.height\" key"))
        };

        let noise_min_y = if noise.contains_key("min_y") {
            let noise_min_y = noise.get("min_y").unwrap();

            let Value::Number(noise_min_y) = noise_min_y else {
                return Err(anyhow!("Expected number, found {:?}", json))
            };

            noise_min_y.as_i64().unwrap() as i32
        } else {
            return Err(anyhow!("No \"noise.min_y\" key"))
        };

        let noise_size_horizontal = if noise.contains_key("size_horizontal") {
            let noise_size_horizontal = noise.get("size_horizontal").unwrap();

            let Value::Number(noise_size_horizontal) = noise_size_horizontal else {
                return Err(anyhow!("Expected number, found {:?}", json))
            };

            noise_size_horizontal.as_i64().unwrap() as i32
        } else {
            return Err(anyhow!("No \"noise.size_horizontal\" key"))
        };

        let noise_size_vertical = if noise.contains_key("size_vertical") {
            let noise_size_vertical = noise.get("size_vertical").unwrap();

            let Value::Number(noise_size_vertical) = noise_size_vertical else {
                return Err(anyhow!("Expected number, found {:?}", json))
            };

            noise_size_vertical.as_i64().unwrap() as i32
        } else {
            return Err(anyhow!("No \"noise.size_vertical\" key"))
        };

        Ok(Self {
            router: NoiseRouter::from_json(noise_router_json, resolver)?,
            use_legacy_random_source: false, // TODO: deserialize this field!
            default_block: palette.get_default_obj_by_index(&default_block).unwrap(),
            default_fluid: palette.get_default_obj_by_index(&default_fluid).unwrap(),
            sea_level,
            noise_height,
            noise_min_y,
            noise_size_horizontal,
            noise_size_vertical
        })
    }

    pub fn cell_height(&self) -> u32 {
        quart_pos_to_block(self.noise_size_vertical) as u32
    }

    pub fn cell_width(&self) -> u32 {
        quart_pos_to_block(self.noise_size_horizontal) as u32
    }
}
