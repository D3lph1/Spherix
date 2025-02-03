use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use crate::noise::math::clamped_lerp;
use crate::noise::perlin::noise::{LegacyNoise, SupremumNoise};
use crate::noise::perlin::octave::{wrap, MultiOctaveNoiseFactory, MultiOctaveNoiseParameters};
use crate::noise::perlin::DefaultNoise;
use crate::noise::perlin::LegacyMultiOctaveGridNoise;
use crate::rng::RngForkable;
use spherix_math::vector::{Vector3, Vector3f};
use std::fmt::{Debug, Formatter};
use std::hint::black_box;
use std::rc::Rc;

pub struct NoiseDensityFunction {
    noise: Rc<NoiseHolder<DefaultNoise>>,
    xz_scale: f32,
    y_scale: f32,
}

impl Clone for NoiseDensityFunction {
    fn clone(&self) -> Self {
        Self {
            noise: Rc::new(self.noise.as_ref().clone()),
            xz_scale: self.xz_scale,
            y_scale: self.y_scale,
        }
    }
}

impl NoiseDensityFunction {
    pub fn new(
        noise: Rc<NoiseHolder<DefaultNoise>>,
        xz_scale: f32,
        y_scale: f32,
    ) -> Self {
        Self {
            noise,
            xz_scale,
            y_scale,
        }
    }

    #[inline]
    pub fn noise(&self) -> &NoiseHolder<DefaultNoise> {
        &self.noise
    }

    #[inline]
    pub fn xz_scale(&self) -> f32 {
        self.xz_scale
    }

    #[inline]
    pub fn y_scale(&self) -> f32 {
        self.y_scale
    }
}

impl Debug for NoiseDensityFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Noise (xz_scale: {}, y_scale: {}, noise: (name: {}))", self.xz_scale, self.y_scale, self.noise.tag)
    }
}

impl DensityFunction for NoiseDensityFunction {
    fn sample(&self, at: Vector3, _: &mut DensityFunctionContext) -> f64 {
        self.noise.sample(
            Vector3f::new(
                at.x as f64 * self.xz_scale as f64,
                at.y as f64 * self.y_scale as f64,
                at.z as f64 * self.xz_scale as f64,
            ),
            0.0,
            0.0
        )
    }

    #[inline]
    fn min_value(&self) -> f64 {
        -self.max_value()
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.noise.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(DensityFunctions::Noise(self))
    }
}

#[derive(Clone)]
pub struct OldBlendedNoise {
    min_limit_noise: LegacyMultiOctaveGridNoise,
    max_limit_noise: LegacyMultiOctaveGridNoise,
    pub main_noise: LegacyMultiOctaveGridNoise,
    xz_scale: f64,
    y_scale: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
    //
    xz_multiplier: f64,
    y_multiplier: f64,
    max_value: f64,
}

impl OldBlendedNoise {
    pub fn new(
        min_limit_noise: LegacyMultiOctaveGridNoise,
        max_limit_noise: LegacyMultiOctaveGridNoise,
        main_noise: LegacyMultiOctaveGridNoise,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let y_multiplier = 684.412 * y_scale;
        let max_value = min_limit_noise.max_broken_value(y_multiplier);

        Self {
            min_limit_noise,
            max_limit_noise,
            main_noise,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            //
            xz_multiplier: 684.412 * xz_scale,
            y_multiplier,
            max_value,
        }
    }

    pub fn create<R: RngForkable>(
        rng: &mut R,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64
    ) -> Self {
        Self::new(
            LegacyMultiOctaveGridNoise::from_i32_amplitudes_range(rng, -15..=0),
            LegacyMultiOctaveGridNoise::from_i32_amplitudes_range(rng, -15..=0),
            LegacyMultiOctaveGridNoise::from_i32_amplitudes_range(rng, -7..=0),
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier
        )
    }

    pub fn with_new_rng<R: RngForkable>(&self, rng: &mut R) -> Self {
        Self::create(rng, self.xz_scale, self.y_scale, self.xz_factor, self.y_factor, self.smear_scale_multiplier)
    }
}

impl Debug for OldBlendedNoise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OldBlendedNoise (xz_scale: {}, y_scale: {}, xz_factor: {}, y_factor: {}, smear_scale_multiplier: {})",
            self.xz_scale,
            self.y_scale,
            self.xz_factor,
            self.y_factor,
            self.smear_scale_multiplier
        )
    }
}

impl DensityFunction for OldBlendedNoise {
    fn sample(&self, at: Vector3, _: &mut DensityFunctionContext) -> f64 {
        let d0 = at.x as f64 * self.xz_multiplier;
        let d1 = at.y as f64 * self.y_multiplier;
        let d2 = at.z as f64 * self.xz_multiplier;

        let d3 = d0 / self.xz_factor;
        let d4 = d1 / self.y_factor;
        let d5 = d2 / self.xz_factor;

        let v = Vector3f::new(d3, d4, d5);

        let d6 = self.y_multiplier * self.smear_scale_multiplier;
        let d7 = d6 / self.y_factor;
        let mut d8 = 0.0;
        let mut d9 = 0.0;
        let mut d10 = 0.0;
        let mut d11 = 1.0;

        for i in 0..8 {
            let oct = self.main_noise.octave(i);
            if oct.is_some() {
                let oct = oct.unwrap().inner();
                let x = oct.sample(&v * d11, d7 * d11, d4 * d11);
                d10 += x / d11;
            }

            d11 /= 2.0;
        }

        let d16 = (d10 / 10.0 + 1.0) / 2.0;
        let flag1 = d16 >= 1.0;
        let flag2 = d16 <= 0.0;
        d11 = 1.0;

        for j in 0..16 {
            let d12 = wrap(d0 * d11);
            let d13 = wrap(d1 * d11);
            let d14 = wrap(d2 * d11);
            let d15 = d6 * d11;
            if !flag1 {
                let oct1 = self.min_limit_noise.octave(j);
                if oct1.is_some() {
                    let oct1 = oct1.unwrap().inner();
                    d8 += oct1.sample(Vector3f::new(d12, d13, d14), d15, d1 * d11) / d11;
                }
            }

            if !flag2 {
                let oct2 = self.max_limit_noise.octave(j);
                if oct2.is_some() {
                    let oct2 = oct2.unwrap().inner();
                    d9 += oct2.sample(Vector3f::new(d12, d13, d14), d15, d1 * d11) / d11;
                }
            }

            d11 /= 2.0;
        }

        clamped_lerp(d8 / 512.0, d9 / 512.0, d16) / 128.0
    }

    #[inline]
    fn min_value(&self) -> f64 {
        -self.max_value()
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(DensityFunctions::OldBlendedNoise(self))
    }
}

#[derive(Clone)]
pub struct BlendDensity {
    input: DensityFunctions
}

impl BlendDensity {
    pub fn new(input: DensityFunctions) -> Self {
        Self {
            input,
        }
    }

    fn transform(&self, at: Vector3, min: f64, ctx: &mut DensityFunctionContext) -> f64 {
        ctx.blender.blend_density(at, min)
    }
}

impl Debug for BlendDensity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlendDensity")
    }
}

impl DensityFunction for BlendDensity {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let sampled = self.input.sample(at, ctx);

        ctx.blender.blend_density(at, sampled)
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.input.fill_array(arr, ctx);

        for i in 0..arr.len() {
            ctx.for_index(i as i32);
            arr[i] = ctx.blender.blend_density(ctx.pos(), arr[i])
        }
    }

    fn min_value(&self) -> f64 {
        f64::NEG_INFINITY
    }

    fn max_value(&self) -> f64 {
        f64::INFINITY
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::BlendDensity(
                Box::new(
                    Self::new(self.input.map(mapper))
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct NoiseHolder<N: SupremumNoise + MultiOctaveNoiseFactory + Clone> {
    tag: String,
    params: MultiOctaveNoiseParameters,
    noise: Option<N>
}

impl <N: SupremumNoise + MultiOctaveNoiseFactory + Clone> NoiseHolder<N> {
    pub fn new(tag: String, params: MultiOctaveNoiseParameters, noise: Option<N>) -> Self {
        Self {
            tag,
            params,
            noise
        }
    }

    #[inline]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    #[inline]
    pub fn params(&self) -> MultiOctaveNoiseParameters {
        self.params.clone()
    }

    pub fn with_rng<R: RngForkable>(&self, rng: &mut R) -> Self {
        Self::new(
            self.tag.clone(),
            self.params(),
            Some(N::create(
                rng,
                &self.params.amplitudes,
                self.params.first_octave
            ))
        )
    }
}

impl<N> LegacyNoise<Vector3f> for NoiseHolder<N>
where 
    N: LegacyNoise<Vector3f> + SupremumNoise + MultiOctaveNoiseFactory + Clone
{
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        if self.noise.is_some() {
            self.noise.as_ref().unwrap().sample(at, y_amp, y_min)
        } else {
            0.0
        }
    }
}

impl<N> crate::noise::perlin::Noise<Vector3f> for NoiseHolder<N>
where
    N: crate::noise::perlin::Noise<Vector3f> + SupremumNoise + MultiOctaveNoiseFactory + Clone
{
    fn sample(&self, at: Vector3f) -> f64 {
        self.noise.as_ref().unwrap().sample(at)
    }
}

impl<N: SupremumNoise + MultiOctaveNoiseFactory + Clone> SupremumNoise for NoiseHolder<N> {
    fn max_value(&self) -> f64 {
        if self.noise.is_some() {
            self.noise.as_ref().unwrap().max_value()
        } else {
            2.0
        }
    }
}

pub struct ShiftedNoise {
    noise: Rc<NoiseHolder<DefaultNoise>>,
    shift_x: DensityFunctions,
    shift_y: DensityFunctions,
    shift_z: DensityFunctions,
    xz_scale: f64,
    y_scale: f64,
}

impl Clone for ShiftedNoise {
    fn clone(&self) -> Self {
        Self {
            noise: Rc::new(self.noise.as_ref().clone()),
            shift_x: self.shift_x.clone(),
            shift_y: self.shift_y.clone(),
            shift_z: self.shift_z.clone(),
            xz_scale: self.xz_scale,
            y_scale: self.y_scale,
        }
    }
}

impl ShiftedNoise {
    pub fn new(
        noise: Rc<NoiseHolder<DefaultNoise>>,
        shift_x: DensityFunctions,
        shift_y: DensityFunctions,
        shift_z: DensityFunctions,
        xz_scale: f64,
        y_scale: f64,
    ) -> Self {
        Self {
            noise,
            shift_x,
            shift_y,
            shift_z,
            xz_scale,
            y_scale,
        }
    }

    #[inline]
    pub fn noise(&self) -> &NoiseHolder<DefaultNoise> {
        &self.noise
    }

    #[inline]
    pub fn shifts(self) -> (DensityFunctions, DensityFunctions, DensityFunctions) {
        (self.shift_x, self.shift_y, self.shift_z)
    }

    #[inline]
    pub fn xz_scale(&self) -> f64 {
        self.xz_scale
    }

    #[inline]
    pub fn y_scale(&self) -> f64 {
        self.y_scale
    }
}

impl Debug for ShiftedNoise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ShiftedNoise (xz_scale: {}, y_scale: {}, noise: (name: {}))",
            self.xz_scale,
            self.y_scale,
            self.noise.tag
        )
    }
}

impl DensityFunction for ShiftedNoise {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let d0 = at.x() as f64 * self.xz_scale + self.shift_x.sample(at, ctx);
        let d1 = at.y() as f64 * self.y_scale + self.shift_y.sample(at, ctx);
        let d2 = at.z() as f64 * self.xz_scale + self.shift_z.sample(at, ctx);

        let val = self.noise.sample(Vector3f::new(d0, d1, d2), 0.0, 0.0);

        black_box(val)
    }

    fn min_value(&self) -> f64 {
        -self.max_value()
    }

    fn max_value(&self) -> f64 {
        self.noise.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::ShiftedNoise(
                Box::new(
                    Self::new(
                        self.noise,
                        self.shift_x.map(mapper),
                        self.shift_y.map(mapper),
                        self.shift_z.map(mapper),
                        self.xz_scale,
                        self.y_scale
                    )
                )
            )
        )
    }
}

pub struct ShiftA {
    noise: Rc<NoiseHolder<DefaultNoise>>
}

impl Clone for ShiftA {
    fn clone(&self) -> Self {
        Self {
            noise: Rc::new(self.noise.as_ref().clone()),
        }
    }
}

impl ShiftA {
    pub fn new(noise: Rc<NoiseHolder<DefaultNoise>>) -> Self {
        Self {
            noise
        }
    }

    #[inline]
    pub fn noise(&self) -> &NoiseHolder<DefaultNoise> {
        &self.noise
    }
}

impl Debug for ShiftA {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShiftA (noise (name: {}))", self.noise.tag)
    }
}

impl DensityFunction for ShiftA {
    fn sample(&self, at: Vector3, _: &mut DensityFunctionContext) -> f64 {
        self.noise.sample(
            Vector3f::new(at.x() as f64 * 0.25, 0.0, at.z() as f64 * 0.25),
            0.0,
            0.0
        ) * 4.0
    }

    fn min_value(&self) -> f64 {
        -self.max_value()
    }

    fn max_value(&self) -> f64 {
        self.noise.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(DensityFunctions::ShiftA(self))
    }
}

pub struct ShiftB {
    noise: Rc<NoiseHolder<DefaultNoise>>
}

impl Clone for ShiftB {
    fn clone(&self) -> Self {
        Self {
            noise: Rc::new(self.noise.as_ref().clone()),
        }
    }
}

impl ShiftB {
    pub fn new(noise: Rc<NoiseHolder<DefaultNoise>>) -> Self {
        Self {
            noise
        }
    }

    #[inline]
    pub fn noise(&self) -> &NoiseHolder<DefaultNoise> {
        &self.noise
    }
}

impl Debug for ShiftB {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShiftB (noise (name: {}))", self.noise.tag)
    }
}

impl DensityFunction for ShiftB {
    fn sample(&self, at: Vector3, _: &mut DensityFunctionContext) -> f64 {
        self.noise.sample(
            Vector3f::new(at.z() as f64 * 0.25, at.x() as f64 * 0.25, 0.0),
            0.0,
            0.0
        ) * 4.0
    }

    fn min_value(&self) -> f64 {
        -self.max_value()
    }

    fn max_value(&self) -> f64 {
        self.noise.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(DensityFunctions::ShiftB(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::density::density::{DensityFunction, DensityFunctionContext};
    use crate::noise::density::noise::OldBlendedNoise;
    use crate::rng::XoroShiro;
    use spherix_math::vector::Vector3;
    use spherix_util::assert_f64_eq;

    #[test]
    fn blended_noise_density_function_sample() {
        let mut rng = XoroShiro::new(0x301D04);

        let noise = OldBlendedNoise::create(
            &mut rng,
            0.25,
            0.125,
            80.0,
            160.0,
            8.0
        );

        let s = noise.sample(
            Vector3::new(2, -4, 15),
            &mut DensityFunctionContext::default()
        );

        assert_f64_eq!(-0.06628792977026834, s, 10);
    }
}
