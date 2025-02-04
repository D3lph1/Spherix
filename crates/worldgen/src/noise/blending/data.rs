use crate::noise::blending::level::{ConstLevelHeightAccessor, LevelHeightAccessor};
use crate::noise::density::cache::{from_block_pos, section_to_block_coord};

pub struct BlendingData {
    area_with_old_generation: ConstLevelHeightAccessor,
    densities: Vec<Option<Vec<f64>>>,
    heights: Vec<f64>,
}

impl BlendingData {
    const QUARTS_PER_SECTION: i32 = from_block_pos(16);
    const CELL_HORIZONTAL_MAX_INDEX_INSIDE: i32 = Self::QUARTS_PER_SECTION - 1;
    const CELL_HORIZONTAL_MAX_INDEX_OUTSIDE: i32 = Self::QUARTS_PER_SECTION;
    const CELL_COLUMN_INSIDE_COUNT: i32 = 2 * Self::CELL_HORIZONTAL_MAX_INDEX_INSIDE + 1;
    const CELL_COLUMN_OUTSIDE_COUNT: i32 = 2 * Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE + 1;
    const CELL_COLUMN_COUNT: i32 = Self::CELL_COLUMN_INSIDE_COUNT + Self::CELL_COLUMN_OUTSIDE_COUNT;

    pub fn new(x: i32, z: i32, heights: Vec<f64>) -> Self {
        let i = section_to_block_coord(x);
        let j = section_to_block_coord(z) - i;

        Self {
            area_with_old_generation: ConstLevelHeightAccessor::new(i, j),
            densities: vec![None; Self::CELL_COLUMN_COUNT as usize],
            heights,
        }
    }

    fn density_from(&self, densities: &Option<Vec<f64>>, y: i32) -> f64 {
        if densities.is_none() {
            f64::MAX
        } else {
            let i = self.cell_y_index(y);
            let densities = densities.as_ref().unwrap();

            if i >= 0 && i < densities.len() as i32 {
                densities[i as usize] * 0.1
            } else {
                f64::MAX
            }
        }
    }

    pub fn density(&self, x: i32, y: i32, z: i32) -> f64 {
        if y == self.min_y() {
            0.1
        } else if x != Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE && z != Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            if x != 0 && z != 0 {
                f64::MAX
            } else {
                self.density_from(
                    &self.densities[Self::inside_index(x, z)],
                    y,
                )
            }
        } else {
            self.density_from(
                &self.densities[Self::outside_index(x, z)],
                y,
            )
        }
    }

    pub fn densities_iter(&self, x: i32, z: i32, y_min: i32, y_max: i32) -> DensitiesIter {
        let i = self.column_min_y();
        let j = 0.max(y_min);
        let k = self.cell_count_per_column().min(y_max - i);

        DensitiesIter {
            x,
            z,
            i,
            j,
            k,
            l: 0,
            k1: 0,
            densities: &self.densities,
        }
    }

    pub fn height(&self, x: i32, z: i32) -> f64 {
        if x != Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE && z != Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            if x != 0 && z != 0 {
                f64::MAX
            } else {
                self.heights[Self::inside_index(x, z)]
            }
        } else {
            return self.heights[Self::outside_index(x, z)];
        }
    }

    pub fn heights_iter(&self, x: i32, z: i32) -> HeightsIter {
        HeightsIter {
            x,
            z,
            i: 0,
            heights: &self.heights,
        }
    }

    fn cell_count_per_column(&self) -> i32 {
        self.area_with_old_generation.section_count() * 2
    }

    #[inline]
    fn min_y(&self) -> i32 {
        self.area_with_old_generation.min_section() * 2
    }

    #[inline]
    fn column_min_y(&self) -> i32 {
        self.min_y() + 1
    }

    #[inline]
    fn cell_y_index(&self, y: i32) -> i32 {
        y - self.column_min_y()
    }

    fn inside_index(x: i32, z: i32) -> usize {
        (Self::CELL_HORIZONTAL_MAX_INDEX_INSIDE - x + z) as usize
    }

    fn outside_index(x: i32, z: i32) -> usize {
        (Self::CELL_COLUMN_INSIDE_COUNT + x + Self::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - z) as usize
    }
}

pub struct DensityEntry {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
    pub(crate) val: f64,
}

pub struct DensitiesIter<'a> {
    x: i32,
    z: i32,
    i: i32,
    j: i32,
    k: i32,
    l: usize,
    k1: i32,
    densities: &'a Vec<Option<Vec<f64>>>,
}

impl<'a> Iterator for DensitiesIter<'a> {
    type Item = DensityEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.l < self.densities.len() {
            let l = self.l;
            self.l += 1;

            let densities = &self.densities[l];

            if densities.is_some() {
                let i1 = self.x + x(l as i32);
                let j1 = self.z + z(l as i32);

                while !(self.k1 < self.j || self.k1 >= self.k) {
                    let k1 = self.k1;
                    self.k1 += 1;

                    return Some(DensityEntry {
                        x: i1,
                        y: k1 + self.i,
                        z: j1,
                        val: densities.as_ref().unwrap()[k1 as usize] * 0.1,
                    });
                }
            }
        }

        None
    }
}

pub struct HeightEntry {
    pub x: i32,
    pub z: i32,
    pub val: f64,
}

pub struct HeightsIter<'a> {
    x: i32,
    z: i32,
    i: usize,
    heights: &'a Vec<f64>,
}

impl Iterator for HeightsIter<'_> {
    type Item = HeightEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        self.i += 1;
        let d0 = self.heights[i];
        if d0 != f64::MAX {
            return Some(HeightEntry {
                x: self.x + x(i as i32),
                z: self.z + z(i as i32),
                val: d0,
            })
        }

        None
    }
}

pub(crate) fn x(val: i32) -> i32 {
    if val < BlendingData::CELL_COLUMN_INSIDE_COUNT {
        zero_if_negative(BlendingData::CELL_HORIZONTAL_MAX_INDEX_INSIDE - val)
    } else {
        let i = val - BlendingData::CELL_COLUMN_INSIDE_COUNT;

        BlendingData::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - zero_if_negative(BlendingData::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - i)
    }
}

pub(crate) fn z(val: i32) -> i32 {
    if val < BlendingData::CELL_COLUMN_INSIDE_COUNT {
        zero_if_negative(val - BlendingData::CELL_COLUMN_INSIDE_COUNT)
    } else {
        let i = val - BlendingData::CELL_COLUMN_INSIDE_COUNT;

        BlendingData::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - zero_if_negative(i - BlendingData::CELL_HORIZONTAL_MAX_INDEX_OUTSIDE)
    }
}

#[inline]
fn zero_if_negative(val: i32) -> i32 {
    val & !(val >> 31)
}
