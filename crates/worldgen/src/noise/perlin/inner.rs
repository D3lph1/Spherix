use crate::rng::Rng;

const PERMUTATION_TABLE_SIZE: usize = 256;

/// Minecraft does not use predefined permutation table that Ken Perlin suggests
/// to use. Instead, it uses randomly shuffled permutation table.
type PermutationTable = [u8; PERMUTATION_TABLE_SIZE];

/// It is performance crucial to use floats, not integers for gradient vectors. Because
/// casting the gradient components to f64 in order to multiply them by point vector
/// components leads to performance degradation.
type GradientVec = (f64, f64, f64);

#[derive(Clone)]
pub struct NoiseInner {
    pub perm: PermutationTable,
    /// offsets
    pub x_o: f64,
    pub y_o: f64,
    pub z_o: f64,
}

impl NoiseInner {
    pub const GRADIENTS: [GradientVec; 16] = [
        (1.0, 1.0, 0.0), (-1.0, 1.0, 0.0), (1.0, -1.0, 0.0), (-1.0, -1.0, 0.0),
        (1.0, 0.0, 1.0), (-1.0, 0.0, 1.0), (1.0, 0.0, -1.0), (-1.0, 0.0, -1.0),
        (0.0, 1.0, 1.0), (0.0, -1.0, 1.0), (0.0, 1.0, -1.0), (0.0, -1.0, -1.0),
        (1.0, 1.0, 0.0), (0.0, -1.0, 1.0), (-1.0, 1.0, 0.0), (0.0, -1.0, -1.0)
    ];

    pub fn new<R: Rng>(rng: &mut R) -> Self {
        // Initialize offsets with random values
        let x_o = rng.next_f64() * PERMUTATION_TABLE_SIZE as f64;
        let y_o = rng.next_f64() * PERMUTATION_TABLE_SIZE as f64;
        let z_o = rng.next_f64() * PERMUTATION_TABLE_SIZE as f64;
        let mut perm = [0; PERMUTATION_TABLE_SIZE];

        // Just fill the table with ordered numbers
        for i in 0..PERMUTATION_TABLE_SIZE {
            perm[i] = i as u8;
        }

        // Shuffle the table
        for i in 0..PERMUTATION_TABLE_SIZE {
            let rand_i = rng.next_u32(PERMUTATION_TABLE_SIZE as u32 - i as u32) as usize;
            perm.swap(i, i + rand_i);
        }

        Self {
            perm,
            x_o,
            y_o,
            z_o,
        }
    }

    /// Get value from the permutation array with cyclical access
    #[inline]
    pub fn permutation(&self, i: i32) -> i32 {
        (self.perm[(i & 255) as usize] & 255) as i32
    }

    /// Calculate the dot product of the gradient vector and the offset vector
    #[inline]
    pub fn dot(grad: GradientVec, x: f64, y: f64, z: f64) -> f64 {
        grad.0 * x + grad.1 * y + grad.2 * z
    }
}
