use std::mem::MaybeUninit;
use std::{mem, ptr};

pub fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N]
{
    let mut array: MaybeUninit<[T; N]> = MaybeUninit::uninit();
    unsafe {
        ptr::copy_nonoverlapping(v.as_ptr(), array.as_mut_ptr() as *mut T, N);
    }
    let array = unsafe { array.assume_init() };
    mem::forget(v);

    array
}
