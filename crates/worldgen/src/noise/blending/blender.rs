use crate::noise::blending::data::BlendingData;
use crate::noise::density::cache::{from_block_pos, length, length_2, quart_pos_from_block, quart_pos_from_section};
use crate::noise::math::{lerp, positive_mod};
use spherix_math::vector::Vector3;
use spherix_world::chunk::pos::ChunkPos;
use std::collections::HashMap;

pub struct Blender {
    height_and_biome_blending_data: HashMap<i64, BlendingData>,
    density_blending_data: HashMap<i64, BlendingData>,
}

impl Blender {
    const HEIGHT_BLENDING_RANGE_CELLS: i32 = quart_pos_from_section(7) - 1;

    pub fn new(
        height_and_biome_blending_data: HashMap<i64, BlendingData>,
        density_blending_data: HashMap<i64, BlendingData>,
    ) -> Self {
        Self {
            height_and_biome_blending_data,
            density_blending_data,
        }
    }

    pub fn blend_density(&self, pos: Vector3, min: f64) -> f64 {
        let i = from_block_pos(pos.x);
        let j = pos.y / 8;
        let k = from_block_pos(pos.z);
        let d0 = self.blending_data_value(i, j, k, |data, x, y, z| {
            data.density(x, y, z)
        });

        if d0 != f64::MAX {
            return d0;
        } else {
            let mut d = 0.0;
            let mut d1 = 0.0;
            let mut d2 = f64::INFINITY;

            self.density_blending_data.iter().for_each(|(k, v)| {
                v.densities_iter(
                    quart_pos_from_section(ChunkPos::extract_x(*k)),
                    quart_pos_from_section(ChunkPos::extract_z(*k)),
                    j - 1,
                    j + 1,
                ).for_each(|entry| {
                    let d3 = length(
                        (i - entry.x) as f64,
                        ((j - entry.y) * 2) as f64,
                        (k - entry.z as i64) as f64,
                    );
                    if d3 <= 2.0 {
                        if d3 < d2 {
                            d2 = d3;
                        }

                        let d4 = 1.0 / (d3 * d3 * d3 * d3);
                        d1 += entry.val * d4;
                        d += d4;
                    }
                })
            });

            if d2 == f64::INFINITY {
                min
            } else {
                let d1 = d1 / d;
                let d2 = (d2 / 3.0).clamp(0.0, 1.0);
                lerp(d2, d1, min)
            }
        }
    }

    fn blending_data_value(&self, x: i32, y: i32, z: i32, cell_value_getter: CellValueGetter) -> f64 {
        let i = quart_pos_from_section(x);
        let j = quart_pos_from_section(z);
        let f = x & 3 == 0;
        let f1 = z & 3 == 0;
        let mut d0 = self.calc_blending_data_value(i, j, x, y, z, cell_value_getter);
        if d0 == f64::MAX {
            if f && f1 {
                d0 = self.calc_blending_data_value(i - 1, j - 1, x, y, z, cell_value_getter);
            }

            if d0 == f64::MAX {
                if f {
                    d0 = self.calc_blending_data_value(i - 1, j, x, y, z, cell_value_getter);
                }

                if d0 == f64::MAX && f1 {
                    d0 = self.calc_blending_data_value(i, j - 1, x, y, z, cell_value_getter);
                }
            }
        }

        d0
    }

    fn calc_blending_data_value(
        &self,
        x: i32,
        z: i32,
        x1: i32,
        y: i32,
        z1: i32,
        cell_value_getter: CellValueGetter,
    ) -> f64 {
        let blending_data = self.height_and_biome_blending_data.get(
            &ChunkPos::new(x, z).into()
        );

        if blending_data.is_some() {
            cell_value_getter(
                blending_data.unwrap(),
                x1 - quart_pos_from_section(x),
                y,
                z1 - quart_pos_from_section(z),
            )
        } else {
            f64::MAX
        }
    }

    pub fn blend_offset_and_factor(&self, x: i32, z: i32) -> BlendingOutput {
        let i = quart_pos_from_block(x);
        let j = quart_pos_from_block(z);
        let d0 = self.blending_data_value(i, 0, j, |data, x, _, z| {
            data.height(x, z)
        });
        if d0 != f64::MAX {
            BlendingOutput {
                alpha: 0.0,
                offset: Self::height_to_offset(d0),
            }
        } else {
            let mut d = 0.0;
            let mut d1 = 0.0;
            let mut d2 = f64::INFINITY;
            self.height_and_biome_blending_data.iter().for_each(|(k, v)| {
                v.heights_iter(
                    quart_pos_from_section(ChunkPos::extract_x(*k)),
                    quart_pos_from_section(ChunkPos::extract_z(*k)),
                ).for_each(|entry| {
                    let d3 = length_2((i - entry.x) as f64, (j - entry.z) as f64);
                    if d3 <= Self::HEIGHT_BLENDING_RANGE_CELLS as f64 {
                        if d3 < d2 {
                            d2 = d3;
                        }

                        let d4 = 1.0 / (d3 * d3 * d3 * d3);
                        d1 += entry.val * d4;
                        d += d4;
                    }
                });
            });

            if d2 == f64::INFINITY {
                BlendingOutput {
                    alpha: 1.0,
                    offset: 0.0
                }
            } else {
                let d1 = d1 / d;
                let mut d2 = (d2 / (Self::HEIGHT_BLENDING_RANGE_CELLS + 1) as f64).clamp(0.0, 1.0);
                d2 = 3.0 * d2 * d2 - 2.0 * d2 * d2 * d2;

                BlendingOutput {
                    alpha: d2,
                    offset: Self::height_to_offset(d1)
                }
            }
        }
    }

    #[inline]
    fn height_to_offset(height: f64) -> f64 {
        let d0 = 1.0;
        let d1 = height + 0.5;
        let d2 = positive_mod(d1, 8.0);

        1.0 * (32.0 * (d1 - 128.0) - 3.0 * (d1 - 120.0) * d2 + 3.0 * d2 * d2) / (128.0 * (32.0 - 3.0 * d2))
    }
}

type CellValueGetter = fn(data: &BlendingData, x: i32, y: i32, z: i32) -> f64;

#[derive(Clone)]
pub struct BlendingOutput {
    pub alpha: f64,
    pub offset: f64
}

#[cfg(test)]
mod tests {
    use crate::noise::blending::blender::Blender;
    use crate::noise::blending::data::BlendingData;
    use spherix_math::vector::Vector3;
    use spherix_util::assert_f64_eq;
    use std::collections::HashMap;

    #[test]
    fn blender_blend_density() {
        let blender = Blender::new(
            HashMap::from([
                (0, BlendingData::new(0, 0, vec![0.74]))
            ]),
            HashMap::new()
        );

        assert_f64_eq!(0.1, blender.blend_density(Vector3::new(2, 5, 1), 0.0), 10);
    }
}
