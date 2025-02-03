pub const fn smallest_encompassing_log2(x: u32) -> u32 {
    let x = if is_power_of_two(x) { x } else { smallest_encompassing_power_of_two(x) };

    BIT_POS_LOOKUP[((x.wrapping_mul(DE_BRUIJN_SEQUENCE) >> DE_BRUJIN_BITS_SHIFT) & 31) as usize]
}

const DE_BRUIJN_SEQUENCE: u32 = 125613361;
const DE_BRUJIN_BITS_SHIFT: u32 = u32::BITS - u32::BITS.ilog2();

const BIT_POS_LOOKUP: [u32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
    31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9
];

pub const fn smallest_encompassing_power_of_two(x: u32) -> u32 {
    let mut x = x - 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;

    x + 1
}

#[inline]
pub const fn is_power_of_two(x: u32) -> bool {
    x != 0 && x & (x - 1) == 0
}

#[cfg(test)]
mod tests {
    use crate::math::smallest_encompassing_log2;

    #[test]
    fn test_smallest_encompassing_log2() {
        assert_eq!(0, smallest_encompassing_log2(1));
        assert_eq!(1, smallest_encompassing_log2(2));
        assert_eq!(2, smallest_encompassing_log2(3));
        assert_eq!(2, smallest_encompassing_log2(4));
        assert_eq!(4, smallest_encompassing_log2(16));
        assert_eq!(5, smallest_encompassing_log2(17));
        assert_eq!(9, smallest_encompassing_log2(512));
        assert_eq!(10, smallest_encompassing_log2(513));
    }
}
