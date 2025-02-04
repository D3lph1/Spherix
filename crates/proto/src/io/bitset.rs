use std::fmt::{Debug, Formatter};
use std::io::{Read, Write};
use std::mem;

use crate::io::array::length_prefixed_sequence_writable;
use crate::io::error::Error;
use crate::io::io::{Readable, Writable};
use crate::io::primitives::{Byte, Long};
use crate::io::var::VarInt;

macro_rules! bitset_impl {
    ($type_name:ident $(<$gen_param:ident>)* $($impl_gen:ident: $impl_rest:path)*, $el:ty) => {
        impl $(<const $impl_gen: $impl_rest>)* $type_name$(<$gen_param>)* {
            pub fn get(&self, pos: usize) -> Option<bool> {
                let offset = pos >> Self::word_bit_places();
                if offset >= self.words() {
                    return None;
                }

                Some((self.0[offset] & (1 << (pos % Self::word_bits()))) != 0)
            }

            #[inline]
            const fn word_bytes() -> usize {
                mem::size_of::<$el>()
            }

            #[inline]
            const fn word_bits() -> usize {
                Self::word_bytes() * u8::BITS as usize
            }

            #[inline]
            const fn word_bit_places() -> usize {
                Self::word_bits().ilog2() as usize
            }

            #[inline]
            pub fn words(&self) -> usize {
                self.0.len()
            }

            #[inline]
            pub fn bytes(&self) -> usize {
                Self::word_bytes() * self.words()
            }

            #[inline]
            pub fn bits(&self) -> usize {
                Self::word_bits() * self.words()
            }

            /// Returns the number of bits set to true in this BitSet
            pub fn cardinality(&self) -> usize {
                let mut k = 0;

                for i in 0..self.words() {
                    k += self.0[i].count_ones();
                }

                k as usize
            }

            /// Returns the "logical size" of this BitSet: the index of the highest set
            /// bit in the BitSet plus one.
            pub fn length(&self) -> usize {
                let mut idx = 0;

                for i in 0..self.bits() {
                    if self.get(i).unwrap() {
                        idx = i;
                    }
                }

                if idx == 0 {
                    return 0;
                }

                idx + 1
            }
        }

        impl $(<const $impl_gen: $impl_rest>)* Debug for $type_name$(<$gen_param>)* {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let mut s = String::with_capacity(self.words() * Self::word_bits());

                let mut first = true;

                for i in 0..self.words() {
                    let mut bit = 1;
                    let word = self.0[i];
                    if word == 0 {
                        continue;
                    }

                    for j in 0..Self::word_bits() {
                        if (word & bit) != 0 {
                            if !first {
                                s.push_str(", ");
                            }

                            s.push_str(&(Self::word_bits() * i + j).to_string());
                            first = false;
                        }

                        bit <<= 1;
                    }
                }

                write!(f, "[{}]", s)
            }
        }
    };
}

/// BitSet structure as described [`here`]
///
/// [`here`]: https://wiki.vg/Protocol#BitSet
#[derive(Clone, PartialEq, Default)]
pub struct BitSet(Vec<Long>);

impl From<BitSet> for Vec<Long> {
    fn from(value: BitSet) -> Self {
        value.0
    }
}

impl From<Vec<Long>> for BitSet {
    fn from(value: Vec<Long>) -> Self {
        BitSet(value)
    }
}

bitset_impl!(BitSet, Long);

/// Implementation of Java's java.util.BitSet class
impl BitSet {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn set(&mut self, pos: usize) {
        let offset = pos >> 6;
        self.ensure(offset);

        self.0[offset] |= 1 << (pos % 64);
    }

    pub fn clear(&mut self, pos: usize) {
        let offset = pos >> 6;
        self.ensure(offset);

        self.0[offset] &= !(1 << (pos % 64));
    }

    #[inline]
    fn ensure(&mut self, last_elt: usize) {
        if last_elt >= self.0.len() {
            self.0.extend(Some(0));
        }
    }
}

impl Readable for BitSet {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let mut vec = Vec::new();

        let len = VarInt::read(buf)?;

        for _ in 0..len.0 {
            vec.push(Long::read(buf)?);
        }

        Ok(Self(vec))
    }
}

length_prefixed_sequence_writable!(BitSet);

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FixedBitSet<const C: usize>([Byte; C]);

pub const fn bitset_bits_to_bytes(bits: usize) -> usize {
    const SIZE: usize = mem::size_of::<i8>() * u8::BITS as usize;

    if bits % SIZE == 0 {
        return bits / SIZE
    }

    bits / SIZE + 1
}

impl<const C: usize> From<[Byte; C]> for FixedBitSet<C> {
    fn from(value: [Byte; C]) -> Self {
        Self(value)
    }
}

#[cfg(test)]
impl <const C: usize> Default for FixedBitSet<C> {
    fn default() -> Self {
        Self([(); C].map(|_| Byte::default()))
    }
}

bitset_impl!(FixedBitSet<C> C: usize, Byte);

impl <const C: usize> FixedBitSet<C> {
    #[inline]
    pub fn new() -> Self {
        Self([0; C])
    }

    pub fn set(&mut self, pos: usize) {
        let offset = pos >> 3;
        if !self.is_enough(offset) {
            return;
        }

        self.0[offset] |= (1 << (pos % u8::BITS as usize)) as Byte;
    }

    pub fn clear(&mut self, pos: usize) {
        let offset = pos >> 3;
        if !self.is_enough(offset) {
            return;
        }

        self.0[offset] &= !(1 << (pos % u8::BITS as usize)) as Byte;
    }

    #[inline]
    fn is_enough(&self, last_elt: usize) -> bool {
        return last_elt < self.0.len();
    }
}

impl <const C: usize> Readable for FixedBitSet<C> {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let mut vec = Vec::with_capacity(C);
        for _ in 0..C {
            vec.push(Byte::read(buf)?);
        }

        Ok(FixedBitSet::<C>(vec.try_into().map_err(|e| Error::Other)?))
    }
}

impl <const C: usize> Writable for FixedBitSet<C> {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        Ok(self.0.write(buf)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::io::tests::ser_write_read_type_assert;
    use crate::io::{bitset_bits_to_bytes, BitSet, FixedBitSet};

    #[test]
    fn test_bitset_bits_to_bytes() {
        assert_eq!(1, bitset_bits_to_bytes(1));
        assert_eq!(1, bitset_bits_to_bytes(2));
        assert_eq!(1, bitset_bits_to_bytes(7));
        assert_eq!(1, bitset_bits_to_bytes(8));
        assert_eq!(2, bitset_bits_to_bytes(9));
        assert_eq!(2, bitset_bits_to_bytes(15));
        assert_eq!(2, bitset_bits_to_bytes(16));
        assert_eq!(3, bitset_bits_to_bytes(17));
        assert_eq!(3, bitset_bits_to_bytes(20));
        assert_eq!(3, bitset_bits_to_bytes(24));
        assert_eq!(4, bitset_bits_to_bytes(25));
    }

    #[test]
    fn bitset_1() {
        let mut bitset = BitSet::from(
            vec![
                0b0011001010001001100,
                0b0001011000000000100
            ]
        );

        assert_eq!("[2, 3, 6, 10, 12, 15, 16, 66, 76, 77, 79]", format!("{bitset:?}"));
        assert_eq!(11, bitset.cardinality());
        assert_eq!(80, bitset.length());

        ser_write_read_type_assert(&bitset);

        bitset.set(4);
        bitset.set(20);
        bitset.set(23);
        bitset.clear(77);

        assert_eq!("[2, 3, 4, 6, 10, 12, 15, 16, 20, 23, 66, 76, 79]", format!("{bitset:?}"));
        assert_eq!(13, bitset.cardinality());
        assert_eq!(80, bitset.length());

        ser_write_read_type_assert(&bitset);
    }

    #[test]
    fn bitset_2() {
        let mut bitset = BitSet::new();

        bitset.set(0);
        bitset.set(1);
        bitset.set(4);
        bitset.set(8);

        bitset.set(31);
        bitset.set(34);

        bitset.set(60);
        bitset.set(61);
        bitset.set(63);
        bitset.set(64);
        bitset.set(75);
        bitset.set(80);
        bitset.set(81);

        assert!(bitset.get(0).unwrap());
        assert!(bitset.get(1).unwrap());
        assert!(!bitset.get(2).unwrap());
        assert!(!bitset.get(3).unwrap());
        assert!(bitset.get(4).unwrap());
        assert!(!bitset.get(5).unwrap());
        assert!(!bitset.get(6).unwrap());
        assert!(!bitset.get(7).unwrap());
        assert!(bitset.get(8).unwrap());
        assert!(!bitset.get(9).unwrap());
        assert!(!bitset.get(10).unwrap());

        assert!(!bitset.get(30).unwrap());
        assert!(bitset.get(31).unwrap());
        assert!(!bitset.get(32).unwrap());
        assert!(!bitset.get(33).unwrap());
        assert!(bitset.get(34).unwrap());
        assert!(!bitset.get(35).unwrap());

        assert!(!bitset.get(59).unwrap());
        assert!(bitset.get(60).unwrap());
        assert!(bitset.get(61).unwrap());
        assert!(!bitset.get(62).unwrap());
        assert!(bitset.get(63).unwrap());
        assert!(bitset.get(64).unwrap());
        assert!(!bitset.get(65).unwrap());
        assert!(!bitset.get(66).unwrap());
        assert!(!bitset.get(73).unwrap());
        assert!(!bitset.get(74).unwrap());
        assert!(bitset.get(75).unwrap());
        assert!(!bitset.get(76).unwrap());
        assert!(!bitset.get(79).unwrap());
        assert!(bitset.get(80).unwrap());
        assert!(bitset.get(81).unwrap());
        assert!(!bitset.get(82).unwrap());
        assert!(!bitset.get(83).unwrap());

        assert_eq!("[0, 1, 4, 8, 31, 34, 60, 61, 63, 64, 75, 80, 81]", format!("{bitset:?}"));
        assert_eq!(13, bitset.cardinality());
        assert_eq!(82, bitset.length());

        ser_write_read_type_assert(&bitset);

        bitset.clear(34);
        bitset.clear(63);
        bitset.clear(64);

        assert_eq!("[0, 1, 4, 8, 31, 60, 61, 75, 80, 81]", format!("{bitset:?}"));
        assert_eq!(10, bitset.cardinality());
        assert_eq!(82, bitset.length());

        ser_write_read_type_assert(&bitset);
    }

    #[test]
    fn fixed_bitset_1() {
        let mut bitset = FixedBitSet::from([0b01000111, 0b00010110]);

        assert_eq!("[0, 1, 2, 6, 9, 10, 12]", format!("{bitset:?}"));
        assert_eq!(7, bitset.cardinality());
        assert_eq!(13, bitset.length());

        ser_write_read_type_assert(&bitset);

        bitset.set(7);
        bitset.set(8);
        bitset.set(15);
        bitset.clear(10);

        assert_eq!("[0, 1, 2, 6, 7, 8, 9, 12, 15]", format!("{bitset:?}"));
        assert_eq!(9, bitset.cardinality());
        assert_eq!(16, bitset.length());

        ser_write_read_type_assert(&bitset);
    }

    #[test]
    fn fixed_bitset_2() {
        let mut bitset = FixedBitSet::<2>::new();
        bitset.set(0);
        bitset.set(1);
        bitset.set(2);
        bitset.set(3);
        bitset.set(7);

        bitset.set(10);
        bitset.set(11);
        bitset.set(13);

        assert!(bitset.get(0).unwrap());
        assert!(bitset.get(1).unwrap());
        assert!(bitset.get(2).unwrap());
        assert!(bitset.get(3).unwrap());
        assert!(!bitset.get(4).unwrap());
        assert!(!bitset.get(5).unwrap());
        assert!(!bitset.get(6).unwrap());
        assert!(bitset.get(7).unwrap());
        assert!(!bitset.get(8).unwrap());
        assert!(!bitset.get(9).unwrap());
        assert!(bitset.get(10).unwrap());
        assert!(bitset.get(11).unwrap());
        assert!(!bitset.get(12).unwrap());
        assert!(bitset.get(13).unwrap());
        assert!(!bitset.get(14).unwrap());
        assert!(!bitset.get(15).unwrap());

        ser_write_read_type_assert(&bitset);

        assert_eq!("[0, 1, 2, 3, 7, 10, 11, 13]", format!("{bitset:?}"));
        assert_eq!(8, bitset.cardinality());
        assert_eq!(14, bitset.length());

        bitset.clear(2);
        bitset.clear(10);
        bitset.clear(13);

        assert_eq!("[0, 1, 3, 7, 11]", format!("{bitset:?}"));
        assert_eq!(5, bitset.cardinality());
        assert_eq!(12, bitset.length());

        ser_write_read_type_assert(&bitset);
    }
}
