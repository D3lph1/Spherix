use crate::noise::math::lerp3;
use crate::noise::perlin::inner::NoiseInner;
use crate::noise::perlin::{LegacyNoise, Noise};
use crate::rng::Rng;
use spherix_math::vector::Vector3f;

/// Improved 3-dimensional version of the noise that proposed by Ken Perlin in his [`article`].
///
/// Unlike [`crate::noise::perlin::simplex::noise::SimplexNoise`]'s simplex grid, this
/// algorithm uses cubic grid.
///
/// [`article`]: https://www.cs.cmu.edu/~jkh/462_s07/paper445.pdf
#[derive(Clone)]
pub struct GridNoise(NoiseInner);

impl GridNoise {
    /// Small value to prevent floating point errors when shifting coordinates
    const EPS: f64 = 1.0E-7;

    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self(NoiseInner::new(rng))
    }

    /// Sample the noise at the 8 corners of a cube and interpolate between them
    fn sample_and_lerp(&self, floor_x: i32, floor_y: i32, floor_z: i32, frac_x: f64, frac_y: f64, frac_z: f64, frac_y_orig: f64) -> f64 {
        let perm_x = self.0.permutation(floor_x);
        let perm_x1 = self.0.permutation(floor_x + 1);
        
        let perm_xy = self.0.permutation(perm_x + floor_y);
        let perm_xy1 = self.0.permutation(perm_x + floor_y + 1);

        let perm_x1_y = self.0.permutation(perm_x1 + floor_y);
        let perm_x1_y_inc = self.0.permutation(perm_x1 + floor_y + 1);

        // Calculate the dot products at each corner of the cube
        let value000 = Self::gradient_dot(self.0.permutation(perm_xy + floor_z), frac_x, frac_y, frac_z);
        let value100 = Self::gradient_dot(self.0.permutation(perm_x1_y + floor_z), frac_x - 1.0, frac_y, frac_z);
        let value010 = Self::gradient_dot(self.0.permutation(perm_xy1 + floor_z), frac_x, frac_y - 1.0, frac_z);
        let value110 = Self::gradient_dot(self.0.permutation(perm_x1_y_inc + floor_z), frac_x - 1.0, frac_y - 1.0, frac_z);

        let value001 = Self::gradient_dot(self.0.permutation(perm_xy + floor_z + 1), frac_x, frac_y, frac_z - 1.0);
        let value101 = Self::gradient_dot(self.0.permutation(perm_x1_y + floor_z + 1), frac_x - 1.0, frac_y, frac_z - 1.0);
        let value011 = Self::gradient_dot(self.0.permutation(perm_xy1 + floor_z + 1), frac_x, frac_y - 1.0, frac_z - 1.0);
        let value111 = Self::gradient_dot(self.0.permutation(perm_x1_y_inc + floor_z + 1), frac_x - 1.0, frac_y - 1.0, frac_z - 1.0);

        // Smooth the interpolation parameters using smoothstep function
        let smooth_x = Self::smooth_step_improved(frac_x);
        let smooth_y = Self::smooth_step_improved(frac_y_orig);
        let smooth_z = Self::smooth_step_improved(frac_z);

        // Perform a 3D interpolation using trilinear interpolation
        lerp3(smooth_x, smooth_y, smooth_z, value000, value100, value010, value110, value001, value101, value011, value111)
    }

    /// Calculate the dot product of a gradient vector and a coordinate vector
    #[inline]
    pub fn gradient_dot(index: i32, x: f64, y: f64, z: f64) -> f64 {
        NoiseInner::dot(NoiseInner::GRADIENTS[index as usize & 15], x, y, z)
    }

    /// Sigmoid function, proposed for Improved Noise
    ///
    /// It is defined as:
    ///     6x^5 - 15t^4 + 10t^3
    #[inline]
    fn smooth_step_improved(t: f64) -> f64 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
}

impl Noise<Vector3f> for GridNoise {
    fn sample(&self, at: Vector3f) -> f64 {
        LegacyNoise::sample(self, at, 0.0, 0.0)
    }
}

impl LegacyNoise<Vector3f> for GridNoise {
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        // Apply the pre-calculated random offsets to the coordinates
        let shifted_x = at.x + self.0.x_o;
        let shifted_y = at.y + self.0.y_o;
        let shifted_z = at.z + self.0.z_o;

        // Floor the shifted coordinates to get the integer grid positions.
        let floor_x = shifted_x.floor() as i32;
        let floor_y = shifted_y.floor() as i32;
        let floor_z = shifted_z.floor() as i32;

        // Calculate the fractional part of the shifted coordinates.
        let x_frac = shifted_x - floor_x as f64;
        let y_frac = shifted_y - floor_y as f64;
        let z_frac = shifted_z - floor_z as f64;

        let slice_value;
        // Handle vertical slicing
        if y_amp != 0.0 {
            let effective_y;
            // Adjust slice height based on if it's within the range of y_frac
            if y_min >= 0.0 && y_min < y_frac {
                effective_y = y_min;
            } else {
                effective_y = y_frac;
            }

            // Calculate the vertical slice position
            slice_value = (effective_y / y_amp + Self::EPS).floor() * y_amp;
        } else {
            // No vertical slicing
            slice_value = 0.0;
        }

        // Sample and interpolate the noise
        self.sample_and_lerp(floor_x, floor_y, floor_z, x_frac, y_frac - slice_value, z_frac, y_frac)
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::perlin::grid::noise::GridNoise;
    use crate::noise::perlin::noise::LegacyNoise;
    use crate::rng::XoroShiro;
    use spherix_math::vector::Vector3f;
    use spherix_util::assert_f64_eq;

    #[test]
    fn perlin_noise_sample() {
        let noise = GridNoise::new(&mut XoroShiro::new(0xD128FF383ED163EB));

        assert_f64_eq!(-0.41470253957180236, noise.sample(Vector3f::new(-1.859375, -1.859375, 0.0), 0.0, 0.0), 10);
        assert_f64_eq!(0.20404414295844375, noise.sample(Vector3f::new(-29.75, 0.0, -29.75), 0.0, 0.0), 10);
        assert_f64_eq!(-0.060215258364751438, noise.sample(Vector3f::new(-119.0, 0.0, -119.0), 0.0, 0.0), 10);
        assert_f64_eq!(0.051268904673249238, noise.sample(Vector3f::new(-476.0, 0.0, -476.0), 0.0, 0.0), 10);
        assert_f64_eq!(0.10763303000361138, noise.sample(Vector3f::new(0.015625, 0.0, -1.859375), 0.0, 0.0), 10);

        let noise = GridNoise::new(&mut XoroShiro::new(0xA809293));

        assert_f64_eq!(0.05841230530689322, noise.sample(Vector3f::new(1.0, 7.4, -23.6), 0.0, 0.0), 10);
        assert_f64_eq!(-0.5446696078278417, noise.sample(Vector3f::new(-5632781.4, 482357.5, 89928523.1), 0.0, 0.0), 10);
    }
}
