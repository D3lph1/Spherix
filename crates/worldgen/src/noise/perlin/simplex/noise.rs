use crate::noise::math::floor;
use crate::noise::perlin::inner::NoiseInner;
use crate::noise::perlin::noise::Noise;
use crate::rng::Rng;
use spherix_math::vector::{Vector2f, Vector3f};

#[derive(Clone)]
pub struct SimplexNoise(NoiseInner);

impl SimplexNoise {
    const SQRT_3: f64 = 1.7320508075688772;
    /// Constant for converting coordinates from Cartesian to skewed for 2-d noise
    const F2: f64 = 0.5 * (Self::SQRT_3 - 1.0);
    /// Constant for converting coordinates from Cartesian to skewed for 3-d noise
    const F3: f64 = 0.3333333333333333;
    /// Constant for converting coordinates from skewed to Cartesian for 2-d noise
    const G2: f64 = (3.0 - Self::SQRT_3) / 6.0;
    /// Constant for converting coordinates from skewed to Cartesian for 3-d noise
    const G3: f64 = 0.16666666666666666;

    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self(NoiseInner::new(rng))
    }

    /// Calculate the influence of a corner point on the noise value
    fn corner_noise_3d(grad_i: usize, x: f64, y: f64, z: f64, falloff_factor: f64) -> f64 {
        // Calculate the square of the distance from the point to the grid vertex
        let mut dist = falloff_factor - x * x - y * y - z * z;

        // If the distance is negative, the influence of the vertex is 0
        if dist < 0.0 {
            0.0
        } else {
            // Otherwise, calculate the noise value by multiplying the dot product by a falloff function
            dist *= dist;
            dist * dist * NoiseInner::dot(NoiseInner::GRADIENTS[grad_i], x, y, z)
        }
    }
}

impl Noise<Vector3f> for SimplexNoise {
    fn sample(&self, at: Vector3f) -> f64 {
        // Transform coordinates to a skewed coordinate system
        let skew_value = (at.x + at.y + at.z) * Self::F3;
        // Determine the coordinates of the simplex cell
        let floor_x = floor(at.x + skew_value);
        let floor_y = floor(at.y + skew_value);
        let floor_z = floor(at.z + skew_value);
        // Transform coordinates back to the Cartesian system
        let unskew_value = (floor_x + floor_y + floor_z) as f64 * Self::G3;
        // Calculate offsets from the top-left corner of the simplex
        let offset_x = floor_x as f64 - unskew_value;
        let offset_y = floor_y as f64 - unskew_value;
        let offset_z = floor_z as f64 - unskew_value;
        // Calculate the offset of the point from the cell
        let in_cell_x = at.x - offset_x;
        let in_cell_y = at.y - offset_y;
        let in_cell_z = at.z - offset_z;

        // Determine the coordinates of the second, third, and fourth vertices of the simplex
        let (
            offset_vertex_2_x, offset_vertex_2_y, offset_vertex_2_z,
            offset_vertex_3_x, offset_vertex_3_y, offset_vertex_3_z
        ) = if in_cell_x >= in_cell_y {
            if in_cell_y >= in_cell_z {
                (1, 0, 0, 1, 1, 0)
            } else if in_cell_x >= in_cell_z {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if in_cell_y < in_cell_z {
            (0, 0, 1, 0, 1, 1)
        } else if in_cell_x < in_cell_z {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        // Calculate the coordinates of the second, third, and fourth vertices of the simplex
        let vertex_2_x = in_cell_x - offset_vertex_2_x as f64 + Self::G3;
        let vertex_2_y = in_cell_y - offset_vertex_2_y as f64 + Self::G3;
        let vertex_2_z = in_cell_z - offset_vertex_2_z as f64 + Self::G3;
        let vertex_3_x = in_cell_x - offset_vertex_3_x as f64 + Self::F3;
        let vertex_3_y = in_cell_y - offset_vertex_3_y as f64 + Self::F3;
        let vertex_3_z = in_cell_z - offset_vertex_3_z as f64 + Self::F3;
        let vertex_4_x = in_cell_x - 1.0 + 0.5;
        let vertex_4_y = in_cell_y - 1.0 + 0.5;
        let vertex_4_z = in_cell_z - 1.0 + 0.5;

        // Calculate the indices in the permutation array
        let perm_i_x = floor_x & 255;
        let perm_i_y = floor_y & 255;
        let perm_i_z = floor_z & 255;
        let grad_i_1 = self.0.permutation(perm_i_x + self.0.permutation(perm_i_y + self.0.permutation(perm_i_z))) % 12;
        let grad_i_2 = self.0.permutation(perm_i_x + offset_vertex_2_x + self.0.permutation(perm_i_y + offset_vertex_2_y + self.0.permutation(perm_i_z + offset_vertex_2_z))) % 12;
        let grad_i_3 = self.0.permutation(perm_i_x + offset_vertex_3_x + self.0.permutation(perm_i_y + offset_vertex_3_y + self.0.permutation(perm_i_z + offset_vertex_3_z))) % 12;
        let grad_i_4 = self.0.permutation(perm_i_x + 1 + self.0.permutation(perm_i_y + 1 + self.0.permutation(perm_i_z + 1))) % 12;

        // Calculate the noise values for each vertex
        let noise_1 = Self::corner_noise_3d(grad_i_1 as usize, in_cell_x, in_cell_y, in_cell_z, 0.6);
        let noise_2 = Self::corner_noise_3d(grad_i_2 as usize, vertex_2_x, vertex_2_y, vertex_2_z, 0.6);
        let noise_3 = Self::corner_noise_3d(grad_i_3 as usize, vertex_3_x, vertex_3_y, vertex_3_z, 0.6);
        let noise_4 = Self::corner_noise_3d(grad_i_4 as usize, vertex_4_x, vertex_4_y, vertex_4_z, 0.6);

        32.0 * (noise_1 + noise_2 + noise_3 + noise_4)
    }
}

impl Noise<Vector2f> for SimplexNoise {
    fn sample(&self, at: Vector2f) -> f64 {
        // Transform coordinates to a skewed coordinate system
        let skew_value = (at.x() + at.z()) * Self::F2;
        // Determine the coordinates of the simplex cell
        let floor_x = floor(at.x() + skew_value);
        let floor_y = floor(at.z() + skew_value);
        // Transform coordinates back to the Cartesian system
        let unskew_value = (floor_x + floor_y) as f64 * Self::G2;
        // Calculate offsets from the top-left corner of the simplex
        let offset_x = floor_x as f64 - unskew_value;
        let offset_y = floor_y as f64 - unskew_value;
        // Calculate the offset of the point from the cell
        let in_cell_x = at.x() - offset_x;
        let in_cell_y = at.z() - offset_y;
        // Determine the coordinates of the second vertex of the simplex
        let (offset_in_cell_x, offset_in_cell_y) = if in_cell_x > in_cell_y {
            (1, 0)
        } else {
            (0, 1)
        };

        // Calculate the coordinates of the second vertex of the simplex
        let vertex_2_x = in_cell_x - offset_in_cell_x as f64 + Self::G2;
        let vertex_2_y = in_cell_y - offset_in_cell_y as f64 + Self::G2;
        // Calculate the coordinates of the third vertex of the simplex
        let vertex_3_x = in_cell_x - 1.0 + 2.0 * Self::G2;
        let vertex_3_y = in_cell_y - 1.0 + 2.0 * Self::G2;

        // Calculate the indices in the permutation array
        let perm_i_x = floor_x & 255;
        let perm_i_y = floor_y & 255;
        let grad_i_1 = self.0.permutation(perm_i_x + self.0.permutation(perm_i_y)) % 12;
        let grad_i_2 = self.0.permutation(perm_i_x + offset_in_cell_x + self.0.permutation(perm_i_y + offset_in_cell_y)) % 12;
        let grad_i_3 = self.0.permutation(perm_i_x + 1 + self.0.permutation(perm_i_y + 1)) % 12;

        // Calculate the noise values for each vertex
        let noise_1 = Self::corner_noise_3d(grad_i_1 as usize, in_cell_x, in_cell_y, 0.0, 0.5);
        let noise_2 = Self::corner_noise_3d(grad_i_2 as usize, vertex_2_x, vertex_2_y, 0.0, 0.5);
        let noise_3 = Self::corner_noise_3d(grad_i_3 as usize, vertex_3_x, vertex_3_y, 0.0, 0.5);

        70.0 * (noise_1 + noise_2 + noise_3)
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::perlin::noise::Noise;
    use crate::noise::perlin::simplex::noise::SimplexNoise;
    use crate::rng::XoroShiro;
    use spherix_math::vector::{Vector2f, Vector3f};
    use spherix_util::assert_f64_eq;

    #[test]
    fn test_sample_2() {
        let noise = SimplexNoise::new(&mut XoroShiro::new(0xD08416));
        assert_f64_eq!(238.42554434432301, noise.0.x_o, 10);
        assert_f64_eq!(87.3961864853153, noise.0.y_o, 10);
        assert_f64_eq!(107.66230946779453, noise.0.z_o, 10);

        assert_f64_eq!(0.5984056652708516, noise.sample(Vector2f::new(24.11, -5.04)), 10);
        assert_f64_eq!(0.6531136137500556, noise.sample(Vector2f::new(32002.1, 29418.411)), 10);
    }

    #[test]
    fn test_sample_3() {
        let noise = SimplexNoise::new(&mut XoroShiro::new(0x532786fa));
        assert_f64_eq!(173.80106523756513, noise.0.x_o, 10);
        assert_f64_eq!(31.111246980867207, noise.0.y_o, 10);
        assert_f64_eq!(12.324905451877527, noise.0.z_o, 10);

        assert_f64_eq!(-0.6342398870682866, noise.sample(Vector3f::new(325.0, -7.21, -800.04)), 10);
        assert_f64_eq!(-0.5703277912748015, noise.sample(Vector3f::new(6151.0, 6154.2, -4194.5)), 10);
    }
}
