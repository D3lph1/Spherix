use rstar::Point;

#[derive(Clone, PartialEq, Debug)]
pub struct ClimatePoint {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64
}

impl Point for ClimatePoint {
    type Scalar = i64;
    const DIMENSIONS: usize = 6;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self {
            temperature: generator(0),
            humidity: generator(1),
            continentalness: generator(2),
            erosion: generator(3),
            depth: generator(4),
            weirdness: generator(5),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.temperature,
            1 => self.humidity,
            2 => self.continentalness,
            3 => self.erosion,
            4 => self.depth,
            5 => self.weirdness,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.temperature,
            1 => &mut self.humidity,
            2 => &mut self.continentalness,
            3 => &mut self.erosion,
            4 => &mut self.depth,
            5 => &mut self.weirdness,
            _ => unreachable!(),
        }
    }
}
