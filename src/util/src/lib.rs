use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use dyn_clone::DynClone;
use paste::paste;

pub mod hex;
pub mod nbt;
pub mod f32_triplet;
pub mod sha1;
pub mod time;
pub mod slice;
pub mod math;
pub mod array;

macro_rules! float_eq_impl {
    ($t:ty) => {
        paste! {
            pub fn [<$t _eq >](a: $t, b: $t, decimal_places: u8) -> bool {
                let p = [< 10 $t >].powi(-(decimal_places as i32));
                (a - b).abs() < p
            }
        }
    };
}

float_eq_impl!(f32);

#[macro_export]
macro_rules! assert_f32_eq {
    ($left:expr, $right:expr, $decimal_places:expr) => {
        if !spherix_util::f32_eq($left, $right, $decimal_places) {
            panic!(
                r#"assertion `left == right` failed
left: {}
right: {}"#,
                $left,
                $right
            )
        }
    };
}

float_eq_impl!(f64);

#[macro_export]
macro_rules! assert_f64_eq {
    ($left:expr, $right:expr, $decimal_places:expr) => {
        if !spherix_util::f64_eq($left, $right, $decimal_places) {
            panic!(
                r#"assertion `left == right` failed
left: {}
right: {}"#,
                $left,
                $right
            )
        }
    };
}

pub fn iters_equal_anyorder<T: Eq + Hash>(i1: impl Iterator<Item=T>, i2: impl Iterator<Item=T>) -> bool {
    fn get_lookup<T: Eq + Hash>(iter: impl Iterator<Item=T>) -> HashMap<T, usize> {
        let mut lookup = HashMap::<T, usize>::new();
        for value in iter {
            match lookup.entry(value) {
                Entry::Occupied(entry) => { *entry.into_mut() += 1; }
                Entry::Vacant(entry) => { entry.insert(0); }
            }
        }
        lookup
    }
    get_lookup(i1) == get_lookup(i2)
}

pub trait CloneableIterator: Iterator + DynClone {}

impl<T> CloneableIterator for T where T: Iterator + DynClone {}

dyn_clone::clone_trait_object!(<T> CloneableIterator<Item = T>);

#[cfg(test)]
mod tests {
    use crate::{f32_eq, f64_eq, iters_equal_anyorder};

    #[test]
    fn test_f32_eq() {
        assert!(f32_eq(0.43025392, 0.43025383, 7));
        assert!(!f32_eq(0.43025392, 0.43025383, 8));
    }

    #[test]
    fn test_f64_eq() {
        assert!(f64_eq(0.340209528758382, 0.340209528758365, 13));
        assert!(!f64_eq(0.340209528758382, 0.340209528758365, 14));
    }

    #[test]
    fn test_iters_equal_anyorder() {
        assert!(iters_equal_anyorder(vec![1, 2, 3, 4].into_iter(), vec![1, 2, 3, 4].into_iter()));
        assert!(iters_equal_anyorder(vec![1, 2, 3, 4].into_iter(), vec![4, 3, 2, 1].into_iter()));
        assert!(iters_equal_anyorder(vec![2, 3, 1, 4].into_iter(), vec![3, 4, 2, 1].into_iter()));

        assert!(!iters_equal_anyorder(vec![2, 3, 1, 4].into_iter(), vec![3, 4, 1].into_iter()));
        assert!(!iters_equal_anyorder(vec![3, 7, 4].into_iter(), vec![3, 4, 1].into_iter()));
    }
}
