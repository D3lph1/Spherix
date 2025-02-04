use crate::rng::Rng;
use crate::surface::rule::{ClayBands, CLAY_BANDS_MAX_SIZE};
use spherix_util::array::vec_to_array;
use spherix_world::block::block::Block;
use spherix_world::block::state::BlockState;
use spherix_world::chunk::palette::BlockGlobalPalette;
use std::sync::Arc;

/// The goal of this function is to generate an array of block states representing the
/// colored layers of terracotta that make up the Badlands biome. These layers are not
/// uniform; they have variations and bands of different colors, and this code is
/// designed to create that kind of varied pattern.
pub fn generate_bands<R>(rng: &mut R, palette: Arc<BlockGlobalPalette>) -> Arc<ClayBands>
where
    R: Rng
{
    let mut block_states = vec![
        palette.get_default_obj_by_index(&Block::TERRACOTTA).unwrap();
        CLAY_BANDS_MAX_SIZE
    ];

    let orange_terracotta = palette.get_default_obj_by_index(&Block::ORANGE_TERRACOTTA).unwrap();

    // Randomly place ORANGE_TERRACOTTA blocks in the array.
    let mut i = 0;
    while i < block_states.len() {
        i += rng.next_u32(5) as usize + 1; // Randomly advance index
        if i < block_states.len() {
            block_states[i] = orange_terracotta.clone();
        }
        i += 1;
    }

    // Make colored bands
    make_bands(
        rng,
        &mut block_states,
        1,
        palette.get_default_obj_by_index(&Block::YELLOW_TERRACOTTA).unwrap()
    );
    make_bands(
        rng,
        &mut block_states,
        2,
        palette.get_default_obj_by_index(&Block::BROWN_TERRACOTTA).unwrap()
    );
    make_bands(
        rng,
        &mut block_states,
        1,
        palette.get_default_obj_by_index(&Block::RED_TERRACOTTA).unwrap()
    );

    let white_terracotta = palette.get_default_obj_by_index(&Block::WHITE_TERRACOTTA).unwrap();
    let light_gray_terracotta = palette.get_default_obj_by_index(&Block::LIGHT_GRAY_TERRACOTTA).unwrap();

    let white_terracotta_bands_count = rng.next_u32_inclusive(9, 15);
    let mut current_band_index = 0;
    let mut array_index = 0;

    // Add white and light gray bands
    while current_band_index < white_terracotta_bands_count && array_index < block_states.len() {
        block_states[array_index] = white_terracotta.clone();
        
        // Randomly add light gray block below the white blocks
        if array_index > 1 && rng.next_bool() {
            block_states[array_index - 1] = light_gray_terracotta.clone();
        }

        // Randomly add light gray block above the white blocks
        if array_index + 1 < block_states.len() && rng.next_bool() {
            block_states[array_index + 1] = light_gray_terracotta.clone();
        }

        array_index += rng.next_u32(16) as usize + 4; // Randomly advance index
        current_band_index += 1;
    }

    Arc::new(vec_to_array(block_states))
}

pub fn make_bands<R>(rng: &mut R, block_states: &mut Vec<Arc<BlockState>>, band_height: usize, block_state: Arc<BlockState>)
where
    R: Rng
{
    let band_count = rng.next_u32_inclusive(6, 15);

    for _ in 0..band_count {
        let offset = band_height + rng.next_u32(3) as usize;
        let start_index = rng.next_u32(block_states.len() as u32) as usize;

        // Places the band blocks into the array
        let mut band_array_index = 0;
        while start_index + band_array_index < block_states.len() && band_array_index < offset {
            block_states[start_index + band_array_index] = block_state.clone();
            band_array_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rng::XoroShiro;
    use crate::surface::bands::{generate_bands, make_bands};
    use crate::surface::rule::CLAY_BANDS_MAX_SIZE;
    use spherix_world::block::block::Block;
    use spherix_world::block::state::BlockState;
    use spherix_world::block::variant::VariantVec;
    use spherix_world::chunk::palette::global::GlobalId;
    use spherix_world::chunk::palette::BlockGlobalPalette;
    use std::sync::Arc;

    #[test]
    fn generate_bands_test() {
        let mut rng = XoroShiro::new(0xC956D750);

        let mut palette = BlockGlobalPalette::new(3);
        palette.insert(GlobalId(0), BlockState::new(Block::TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(1), BlockState::new(Block::ORANGE_TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(2), BlockState::new(Block::YELLOW_TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(3), BlockState::new(Block::BROWN_TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(4), BlockState::new(Block::RED_TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(5), BlockState::new(Block::WHITE_TERRACOTTA, true, VariantVec::empty()));
        palette.insert(GlobalId(6), BlockState::new(Block::LIGHT_GRAY_TERRACOTTA, true, VariantVec::empty()));
        let palette = Arc::new(palette);

        let bands = generate_bands(&mut rng, palette.clone());

        for (i, block_state) in bands.iter().enumerate() {
            if i == 5
                || i == 12
                || i == 21
                || i == 30
                || i == 34
                || i == 44
                || i == 46
                || i == 48
                || i == 61
                || i == 66
                || i == 69
                || i == 84
                || i == 90
                || i == 110
                || i == 114
                || i == 127
                || i == 145
                || i == 162
                || i == 165
                || i == 182
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::ORANGE_TERRACOTTA).as_ref().unwrap(), block_state)
            } else if i == 6
                || i == 7
                || i == 40
                || i == 62
                || i == 63
                || i == 72
                || i == 79
                || i == 99
                || i == 120
                || i == 123
                || i == 124
                || (137..=139).contains(&i)
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::YELLOW_TERRACOTTA).as_ref().unwrap(), block_state)
            } else if i == 25
                || i == 26
                || (73..=75).contains(&i)
                || (102..=104).contains(&i)
                || i == 121
                || i == 122
                || i == 132
                || (147..=150).contains(&i)
                || i == 172
                || i == 173
                || (176..=179).contains(&i)
                || i == 190
                || i == 191
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::BROWN_TERRACOTTA).as_ref().unwrap(), block_state)
            } else if i == 8
                || i == 9
                || (13..=15).contains(&i)
                || (49..=51).contains(&i)
                || i == 54
                || i == 55
                || i == 58
                || i == 59
                || i == 93
                || i == 94
                || i == 97
                || i == 98
                || (154..=156).contains(&i)
                || i == 171
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::RED_TERRACOTTA).as_ref().unwrap(), block_state)
            } else if i == 0
                || i == 19
                || i == 38
                || i == 57
                || i == 65
                || i == 77
                || i == 95
                || i == 101
                || i == 105
                || i == 119
                || i == 134
                || i == 143
                || i == 151
                || i == 169
                || i == 186
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::WHITE_TERRACOTTA).as_ref().unwrap(), block_state)
            } else if i == 18
                || i == 39
                || i == 56
                || i == 64
                || i == 78
                || i == 96
                || i == 100
                || i == 106
                || i == 118
                || i == 133
                || i == 135
                || i == 144
                || i == 152
                || i == 185
                || i == 187
            {
                assert_eq!(palette.get_default_obj_by_index(&Block::LIGHT_GRAY_TERRACOTTA).as_ref().unwrap(), block_state)
            } else {
                assert_eq!(palette.get_default_obj_by_index(&Block::TERRACOTTA).as_ref().unwrap(), block_state)
            }
        }
    }

    #[test]
    fn make_bands_test_1() {
        let terracotta = Arc::new(BlockState::new(
            Block::TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let brown_terracotta = Arc::new(BlockState::new(
            Block::BROWN_TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let mut rng = XoroShiro::new(0x5CF261BD);
        let mut bands = vec![terracotta.clone(); CLAY_BANDS_MAX_SIZE];

        make_bands(&mut rng, &mut bands, 2, brown_terracotta.clone());

        for (i, block_state) in bands.iter().enumerate() {
            if (18..=19).contains(&i)
                || (96..=98).contains(&i)
                || (101..=103).contains(&i)
                || (141..=144).contains(&i)
                || (178..=180).contains(&i)
            {
                assert_eq!(&brown_terracotta, block_state);
            } else {
                assert_eq!(&terracotta, block_state);
            }
        }
    }

    #[test]
    fn make_bands_test_2() {
        let terracotta = Arc::new(BlockState::new(
            Block::TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let yellow_terracotta = Arc::new(BlockState::new(
            Block::YELLOW_TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let brown_terracotta = Arc::new(BlockState::new(
            Block::BROWN_TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let red_terracotta = Arc::new(BlockState::new(
            Block::RED_TERRACOTTA,
            true,
            VariantVec::empty()
        ));

        let mut rng = XoroShiro::new(0x53FD56F5);
        let mut bands = vec![terracotta.clone(); CLAY_BANDS_MAX_SIZE];

        make_bands(&mut rng, &mut bands, 1, yellow_terracotta.clone());
        make_bands(&mut rng, &mut bands, 2, brown_terracotta.clone());
        make_bands(&mut rng, &mut bands, 1, red_terracotta.clone());

        for (i, block_state) in bands.iter().enumerate() {
            if i == 16
                || i == 17
                || i == 35
                || i == 39
                || i == 40
                || (66..=68).contains(&i)
                || i == 98
                || (104..=106).contains(&i)
                || i == 118
                || (140..=143).contains(&i)
                || i == 157
                || i == 169
                || i == 175
            {
                assert_eq!(&yellow_terracotta, block_state);
            } else if i == 23
                || i == 24
                || (52..=56).contains(&i)
                || i == 109
                || (137..=139).contains(&i)
                || i == 154
                || i == 155
            {
                assert_eq!(&brown_terracotta, block_state);
            } else if i == 6
                || i == 50
                || i == 51
                || (110..=112).contains(&i)
                || i == 119
                || i == 120
                || i == 146
                || (148..=150).contains(&i)
                || i == 159
                || i == 160
            {
                assert_eq!(&red_terracotta, block_state);
            } else {
                assert_eq!(&terracotta, block_state);
            }
        }
    }
}
