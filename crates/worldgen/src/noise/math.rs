use std::ops::{Add, BitXor, Rem};

/// Linear interpolation
#[inline]
pub fn lerp(part: f64, from: f64, to: f64) -> f64 {
    from + part * (to - from)
}

#[inline]
pub fn lerp3(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, k: f64, l: f64, m: f64) -> f64 {
    lerp(c, lerp2(a, b, d, e, f, g), lerp2(a, b, h, k, l, m))
}

#[inline]
pub fn lerp2(x_part: f64, y_part: f64, x_from: f64, x_to: f64, y_from: f64, y_to: f64) -> f64 {
    lerp(y_part, lerp(x_part, x_from, x_to), lerp(x_part, y_from, y_to))
}

#[inline]
pub fn clamped_lerp(a: f64, b: f64, c: f64) -> f64 {
    if c < 0.0 {
        a
    } else {
        if c > 1.0 { b } else { lerp(c, a, b) }
    }
}

#[inline]
pub fn inverse_lerp(arg: f64, from_arg: f64, to_arg: f64) -> f64 {
    (arg - from_arg) / (to_arg - from_arg)
}

#[inline]
pub fn clamped_map(arg: f64, from_arg: f64, to_arg: f64, from_value: f64, to_value: f64) -> f64 {
    clamped_lerp(from_value, to_value, inverse_lerp(arg, from_arg, to_arg))
}

#[inline]
pub fn map(arg: f64, from_arg: f64, to_arg: f64, from_value: f64, to_value: f64) -> f64 {
    lerp(inverse_lerp(arg, from_arg, to_arg), from_value, to_value)
}

#[inline]
pub fn floor_mod<T>(a: T, b: T) -> T
where
    T: Rem<Output = T>
        + BitXor<Output = T>
        + PartialOrd
        + Add<Output = T>
        + Copy
        + PartialEq
        + From<i8>,
{
    let mut modulo = a % b;
    // if the signs are different and modulo not zero, adjust result
    if (modulo ^ b) < T::from(0) && modulo != T::from(0) {
        modulo = modulo + b;
    }

    modulo
}

#[inline]
pub fn floor_div(a: i32, b: i32) -> i32 {
    let mut r = a / b;
    // if the signs are different and modulo not zero, round down
    if (a ^ b) < 0 && (r * b != a) {
        r -= 1;
    }

    r
}

#[inline]
pub fn floor(x: f64) -> i32 {
    let z = x as i32;
    if x < z as f64 { z - 1 } else { z }
}

#[inline]
pub fn positive_mod(a: f64, b: f64) -> f64 {
    (a % b + b) % b
}
