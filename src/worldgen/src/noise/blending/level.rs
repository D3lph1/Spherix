use crate::noise::density::cache::block_to_section_coord;

pub trait LevelHeightAccessor {
    fn height(&self) -> i32;

    fn min_build_height(&self) -> i32;

    fn max_build_height(&self) -> i32 {
        self.min_build_height() + self.height()
    }

    fn section_count(&self) -> i32 {
        self.max_section() - self.min_section()
    }

    fn min_section(&self) -> i32 {
        block_to_section_coord(self.min_build_height())
    }

    fn max_section(&self) -> i32 {
        block_to_section_coord(self.max_build_height() - 1) + 1
    }

    fn is_outside_build_height(&self, y: i32) -> bool {
        y < self.min_build_height() || y >= self.max_build_height()
    }

    fn section_index(&self, y: i32) -> i32 {
        self.section_index_from_section_y(block_to_section_coord(y))
    }

    fn section_index_from_section_y(&self, y: i32) -> i32 {
        y - self.min_section()
    }

    fn section_y_from_section_index(&self, idx: i32) -> i32 {
        self.min_section() + idx
    }
}

pub struct ConstLevelHeightAccessor {
    height: i32,
    min_build_height: i32
}

impl ConstLevelHeightAccessor {
    pub fn new(height: i32, min_build_height: i32) -> Self {
        Self {
            height,
            min_build_height,
        }
    }
}

impl LevelHeightAccessor for ConstLevelHeightAccessor {
    fn height(&self) -> i32 {
        self.height
    }

    fn min_build_height(&self) -> i32 {
        self.min_build_height
    }
}
