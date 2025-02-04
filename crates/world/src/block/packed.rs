/// Structure that stores values of a fixed bit-length in a packed manner.
/// Each value occupies exactly [`bits_per_val`] bits, and multiple values are packed
/// into a single 64-bit entry in the entries vector. If there is not enough space
/// left in an entry for a full value, the remaining bits are padded with zeros.
/// So values cannot span across multiple entries, ensuring each value is entirely
/// contained within one entry.
#[derive(Clone)]
pub struct PackedArray {
    entries: Vec<u64>,
    pub bits_per_val: usize,
    len: usize
}

impl PackedArray {
    const BITS: usize = u64::BITS as usize;

    #[inline]
    pub fn new(entries: Vec<u64>, bits_per_val: usize, len: usize) -> Self {
        Self {
            entries,
            bits_per_val,
            len
        }
    }

    #[inline]
    pub fn empty(bits_per_val: usize) -> Self {
        Self {
            entries: Vec::new(),
            bits_per_val,
            len: 0
        }
    }

    pub fn zeros(bits_per_val: usize, len: usize) -> Self {
        let val_per_entry = Self::BITS / bits_per_val;
        let entries_len = (len as f64 / val_per_entry as f64).ceil() as usize;

        let mut entries = Vec::with_capacity(entries_len);

        for _ in 0..entries_len {
            entries.push(0);
        }

        Self {
            entries,
            bits_per_val,
            len,
        }
    }

    pub fn with_capacity_for(bits_per_val: usize, len: usize) -> Self {
        let val_per_entry = Self::BITS / bits_per_val;
        let entries_len = (len as f64 / val_per_entry as f64).ceil() as usize;

        Self {
            entries: Vec::with_capacity(entries_len),
            bits_per_val,
            len: 0
        }
    }

    pub fn get(&self, idx: usize) -> u16 {
        let (entry_idx, val_idx) = self.explode(idx);
        self.do_get(entry_idx, val_idx)
    }

    #[inline]
    fn do_get(&self, entry_idx: usize, val_idx: usize) -> u16 {
        let mask = (1 << self.bits_per_val) - 1;
        let x = self.entries[entry_idx];

        let val = x & mask << val_idx;

        (val >> val_idx) as u16
    }

    pub fn set(&mut self, idx: usize, val: u16) {
        let (entry_idx, val_idx) = self.explode(idx);

        self.do_set(idx, entry_idx, val_idx, val)
    }

    #[inline]
    fn do_set(&mut self, idx: usize, entry_idx: usize, val_idx: usize, val: u16) {
        self.ensure_len(entry_idx);

        let x = &mut self.entries[entry_idx];

        let mask = (1 << self.bits_per_val) - 1;
        *x &= !(mask << val_idx);
        *x |= (val as u64) << val_idx;

        if idx + 1 > self.len {
            self.len = idx + 1;
        }
    }

    #[inline]
    pub fn push(&mut self, val: u16) {
        self.set(self.max_idx(), val)
    }

    /// Slightly optimized version of sequential [`Self::get()`] and [`Self::set()`] calls.
    /// It contains only one call of [`Self::explode()`].
    pub fn get_and_set(&mut self, idx: usize, val: u16) -> u16 {
        let (entry_idx, val_idx) = self.explode(idx);

        let old_val = self.do_get(entry_idx, val_idx);
        self.do_set(idx, entry_idx, val_idx, val);

        old_val
    }

    #[inline]
    pub fn entry_of(&self, idx: usize) -> u64 {
        let (entry_idx, _) = self.explode(idx);

        self.entries[entry_idx]
    }

    pub fn shrink(&mut self) {
        let mut shrink_i = None;
        for i in 0..self.entries.len() {
            if self.entries[i] == 0 {
                if shrink_i.is_none() {
                    shrink_i = Some(i);
                }
            } else {
                shrink_i = None;
            }
        }

        if shrink_i.is_some() {
            for i in (shrink_i.unwrap()..self.entries.len()).rev() {
                self.entries.remove(i);
            }
        }
    }

    pub fn resize(&self, bits_per_val: usize) -> Self {
        let mut resized = Self::with_capacity_for(bits_per_val, self.len);
        for idx in 0..self.len {
            resized.set(idx, self.get(idx))
        }

        resized
    }

    #[inline]
    fn ensure_len(&mut self, entry_idx: usize) {
        if self.entries.len() <= entry_idx {
            for _ in 0..(entry_idx - self.entries.len()) + 1 {
                self.entries.push(0);
            }
        }
    }

    #[inline]
    fn explode(&self, idx: usize) -> (usize, usize) {
        (
            idx / (Self::BITS / self.bits_per_val),
            (idx % (Self::BITS / self.bits_per_val)) * self.bits_per_val
        )
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn max_idx(&self) -> usize {
        if self.len() == 0 {
            0
        } else {
            self.len() - 1
        }
    }

    #[inline]
    pub fn entries(&self) -> &Vec<u64> {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use crate::block::packed::PackedArray;

    #[test]
    fn explode() {
        assert_eq!((0, 12), PackedArray::empty(3).explode(4));
        assert_eq!((1, 0), PackedArray::empty(9).explode(7));
        assert_eq!((1, 27), PackedArray::empty(9).explode(10));
        assert_eq!((2, 15), PackedArray::empty(15).explode(9));
    }

    #[test]
    fn ensure_len () {
        let mut a = PackedArray::empty(20);
        assert_eq!(0, a.entries.len());

        a.set(1, 10);
        assert_eq!(1, a.entries.len());

        a.set(3, 10);
        assert_eq!(2, a.entries.len());

        a.set(4, 10);
        assert_eq!(2, a.entries.len());
    }

    #[test]
    fn set() {
        let mut a = PackedArray::empty(4);
        a.set(0, 2);
        assert_eq!(2, a.entries[0]);

        a.set(1, 1);
        assert_eq!(18, a.entries[0]);
        assert_eq!("0b10010", format!("{:#b}", a.entries[0]));

        let mut a = PackedArray::empty(20);

        a.set(3, 10);
        assert_eq!(10, a.entries[1]);
        assert_eq!("0b1010", format!("{:#b}", a.entries[1]));

        a.set(4, 3);
        assert_eq!("0b1100000000000000001010", format!("{:#b}", a.entries[1]));
        //           |-|-------- 20 -------|
        assert_eq!(5, a.len());
        assert_eq!(4, a.max_idx());
    }

    #[test]
    fn get() {
        let mut a = PackedArray::empty(4);
        a.set(0, 2);
        assert_eq!(2, a.get(0));

        let mut a = PackedArray::empty(20);
        a.set(3, 10);
        a.set(4, 3);
        assert_eq!(10, a.get(3));
        assert_eq!(3, a.get(4));
    }

    #[test]
    fn shrink() {
        let mut a = PackedArray::empty(20);
        a.set(3, 10);
        a.set(4, 10);

        a.set(3, 0);
        a.set(4, 0);

        a.shrink();

        assert_eq!(0, a.entries.len());

        let mut a = PackedArray::empty(20);
        a.set(0, 10);
        a.set(3, 10);
        a.set(4, 10);

        a.set(3, 0);
        a.set(4, 0);

        a.shrink();

        assert_eq!(1, a.entries.len());
        assert_eq!(10, a.entries[0]);
    }
}
