use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Store f32 value as 3 integers: mantissa (u64), exponent (i16) and sign (i8).
///
/// This is very helpful for structures and algorithms that require
/// [`Eq`] trait implementation (which f32 numbers have no).
#[derive(PartialEq, Eq, Hash)]
pub struct F32Triplet(u64, i16, i8);

impl F32Triplet {
    /// Returns the mantissa, exponent and sign as integers.
    /// Code from the official [`Rust repository`]. It was removed from std.
    ///
    /// [`Rust repository`]: https://github.com/rust-lang/rust/blob/5c674a11471ec0569f616854d715941757a48a0a/src/libcore/num/f32.rs#L203-L216
    fn integer_decode(val: f32) -> (u64, i16, i8) {
        let bits: u32 = unsafe { std::mem::transmute(val) };
        let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
        let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
        let mantissa = if exponent == 0 {
            (bits & 0x7fffff) << 1
        } else {
            (bits & 0x7fffff) | 0x800000
        };
        // Exponent bias + mantissa shift
        exponent -= 127 + 23;
        (mantissa as u64, exponent, sign)
    }

    #[inline]
    pub fn integer_encode(&self) -> f32 {
        (self.2 as f32) * (self.0 as f32) * (2f32.powf(self.1 as f32))
    }
}

impl From<f32> for F32Triplet {
    fn from(value: f32) -> Self {
        let (mantissa, exponent, sign) = Self::integer_decode(value);

        Self (mantissa, exponent, sign)
    }
}

impl From<F32Triplet> for f32 {
    fn from(value: F32Triplet) -> Self {
        value.integer_encode()
    }
}

impl Serialize for F32Triplet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.integer_encode().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for F32Triplet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        Ok(f32::deserialize(deserializer)?.into())
    }
}

impl Clone for F32Triplet {
    fn clone(&self) -> Self {
        Self(self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use crate::f32_triplet::F32Triplet;

    #[test]
    fn test() {
        let x: F32Triplet = 1.0.into();
        assert_eq!(1.0f32, x.into());

        let x: F32Triplet = (-4.8124).into();
        assert_eq!(-4.8124f32, x.into());

        let x: F32Triplet = 0.0.into();
        assert_eq!(0.0f32, x.into());

        let x: F32Triplet = 1538.00094.into();
        assert_eq!(1538.00094f32, x.into());
    }
}
