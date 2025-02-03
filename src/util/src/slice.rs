#[inline]
pub fn slice_copy<T: Copy>(src: &[T], src_pos: usize, dest: &mut [T], dest_pos: usize, len: usize) {
    for i in 0..len {
        let src_i = src_pos + i;
        let dest_i = dest_pos + i;

        dest[dest_i] = src[src_i]
    }
}
