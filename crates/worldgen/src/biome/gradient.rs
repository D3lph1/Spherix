use crate::biome::accessor::BiomeAccessor;
use crate::noise::math::floor_mod;
use crypto_hashes::sha2::{Digest, Sha256};
use spherix_math::vector::{Vector3, Vector3f};
use spherix_world::chunk::biome::Biome;
use std::cell::RefCell;
use std::sync::Arc;

/// The struct assesses neighboring biomes and calculates a pseudo-random distance.
/// The biome that minimizes this distance is then selected, resulting in smooth and
/// natural transitions between different biomes.
pub struct BiomeGradient<'a> {
    seed: i64,
    biome_accessor: &'a BiomeAccessor
}

impl<'a> BiomeGradient<'a> {
    #[inline]
    pub fn new(seed: i64, biome_accessor: &'a BiomeAccessor) -> Self {
        Self {
            seed,
            biome_accessor
        }
    }

    #[inline]
    pub fn with_hashed_seed(seed: i64, biome_accessor: &'a BiomeAccessor) -> Self {
        Self::new(Self::hash_seed(seed), biome_accessor)
    }

    #[inline]
    fn hash_seed(seed: i64) -> i64 {
        let hash = Sha256::digest(seed.to_le_bytes());

        i64::from_le_bytes(hash[0..8].try_into().unwrap())
    }

    /// Gets the biome for a given block position, applying smoothing by evaluating
    /// neighboring biomes.
    ///
    /// It receives single argument - the position of the block to get the biome for.
    pub fn biome(&self, block_pos: &Vector3) -> Arc<Biome> {
        // Get the block position and offset them by 2 for centering the biome sample.
        let block_pos = block_pos - 2;
        let biome_pos = block_pos >> 2;
        // offset of the block within biome
        let offset = (block_pos & 3) / 4.0;

        let mut best_biome_candidate_index = 0;
        let mut min_distance = f64::INFINITY;

        // Loop through the 8 vertices of the cube (neighboring biomes) for evaluation
        for candidate_index in 0..8 {
            // Determine if the current vertex is on the positive or negative side of each axis
            let is_x_positive = (candidate_index & 4) == 0;
            let is_y_positive = (candidate_index & 2) == 0;
            let is_z_positive = (candidate_index & 1) == 0;

            // Get the biome position for the current candidate
            let candidate_biome_pos = Vector3::new(
                if is_x_positive { biome_pos.x } else { biome_pos.x + 1 },
                if is_y_positive { biome_pos.y } else { biome_pos.y + 1 },
                if is_z_positive { biome_pos.z } else { biome_pos.z + 1 },
            );

            // Get the offset from the center of the current biome for the candidate
            let candidate_offset = Vector3f::new(
                if is_x_positive { offset.x } else { offset.x - 1.0 },
                if is_y_positive { offset.y }  else { offset.y - 1.0 },
                if is_z_positive { offset.z } else { offset.z - 1.0 }
            );
            
            let distance = Self::fiddled_distance(self.seed, candidate_biome_pos, candidate_offset);
            if min_distance > distance {
                best_biome_candidate_index = candidate_index;
                min_distance = distance;
            }
        }

        let best_biome = Vector3::new(
            if (best_biome_candidate_index & 4) == 0 { biome_pos.x } else { biome_pos.x + 1 },
            if (best_biome_candidate_index & 2) == 0 { biome_pos.y } else { biome_pos.y + 1 },
            if (best_biome_candidate_index & 1) == 0 { biome_pos.z } else { biome_pos.z + 1 }
        );

        self.biome_accessor.biome_at(&best_biome)
    }

    fn fiddled_distance(seed: i64, quarter: Vector3, offset: Vector3f) -> f64 {
        let mut lcg = Lcg {
            state: seed,
        };
        lcg.next(quarter.x as i64);
        lcg.next(quarter.y as i64);
        lcg.next(quarter.z as i64);
        lcg.next(quarter.x as i64);
        lcg.next(quarter.y as i64);
        let r = lcg.next(quarter.z as i64);
        let shift_x = Self::fiddle(r);
        let r = lcg.next(seed);
        let shift_y = Self::fiddle(r);
        let r = lcg.next(seed);
        let shift_z = Self::fiddle(r);

        // Return the sum of squared shifted offsets, which represents the distance in the noise space.
        (offset.z + shift_z).powi(2) + (offset.y + shift_y).powi(2) + (offset.x + shift_x).powi(2)
    }

    #[inline]
    fn fiddle(x: i64) -> f64 {
        let d0 = floor_mod(x >> 24, 1024) as f64 / 1024.0;

        (d0 - 0.5) * 0.9
    }
}

struct Lcg {
    state: i64
}

impl Lcg {
    const MULTIPLIER: i64 = 6364136223846793005;
    const INCREMENT: i64 = 1442695040888963407;

    #[inline]
    fn next(&mut self, term: i64) -> i64 {
        self.state = self.state.wrapping_mul(self.state.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::INCREMENT));
        self.state = self.state.wrapping_add(term);

        self.state
    }
}

pub struct LazyCachedBiomeGradient<'a> {
    biome_gradient: &'a BiomeGradient<'a>,
    at: RefCell<Vector3>,
    cache: RefCell<Option<Arc<Biome>>>
}

impl<'a> LazyCachedBiomeGradient<'a> {
    #[inline]
    pub fn new(biome_gradient: &'a BiomeGradient<'a>) -> LazyCachedBiomeGradient<'a> {
        LazyCachedBiomeGradient {
            biome_gradient,
            at: RefCell::new(Vector3::new(0, 0, 0)),
            cache: RefCell::new(None),
        }
    }

    #[inline]
    pub fn at(&self, at: Vector3) {
        self.at.replace(at);
        self.cache.replace(None);
    }

    pub fn biome(&self) -> Arc<Biome> {
        let mut borrow = self.cache.borrow_mut();
        
        match *borrow {
            Some(ref biome) => biome.clone(),
            None => {
                let biome = self.biome_gradient.biome(&*self.at.borrow());
                borrow.replace(biome.clone());
                
                biome
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::biome::gradient::{BiomeGradient, Lcg};
    use spherix_math::vector::{Vector3, Vector3f};

    #[test]
    fn lcg() {
        let mut lcg = Lcg {
            state: 12
        };
        assert_eq!(-7035991034581378769, lcg.next(43));
        assert_eq!(8795629728386739657, lcg.next(-5));
        assert_eq!(8428620334462335289, lcg.next(117));
    }

    #[test]
    fn hash_seed() {
        assert_eq!(-6467378160175308932, BiomeGradient::hash_seed(1));
        assert_eq!(1019781227919247049, BiomeGradient::hash_seed(34523));
        assert_eq!(-7689220722315728222, BiomeGradient::hash_seed(-723600001));
    }

    #[test]
    fn fiddled_distance() {
        assert_eq!(
            17.602254361343384,
            BiomeGradient::fiddled_distance(351, Vector3::new(-40, 7, 12), Vector3f::new(0.84, 1.7, -3.19))
        );

        assert_eq!(
            7.441707526779174,
            BiomeGradient::fiddled_distance(-1193476802, Vector3::new(55, -31, -173), Vector3f::new(1.89, -2.54, 0.02))
        );
    }
}
