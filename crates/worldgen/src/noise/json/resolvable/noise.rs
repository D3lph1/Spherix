use crate::noise::density::density::DensityFunctions;
use crate::noise::density::misc::Const;
use crate::noise::density::noise::NoiseHolder;
use crate::noise::json::resolvable::Resolvable;
use crate::noise::json::Resolver;
use crate::noise::perlin::octave::MultiOctaveNoiseParameters;
use crate::noise::perlin::DefaultNoise;
use anyhow::anyhow;
use serde_json::Value;
use std::rc::Rc;

impl<T> Resolvable<T> for NoiseHolder<DefaultNoise> {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self> {
        let amplitudes: Vec<f64> = resolver.resolve_field(val, "amplitudes")?;
        let first_octave = resolver.resolve_field(val, "firstOctave")?;

        Ok(NoiseHolder::new(
            resolver.contextual_name.take().unwrap(),
            MultiOctaveNoiseParameters::new(first_octave, amplitudes.clone()),
            None,
        ))
    }
}

impl<T> Resolvable<T> for (String, MultiOctaveNoiseParameters) {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let amplitudes: Vec<f64> = resolver.resolve_field(val, "amplitudes")?;
        let first_octave = resolver.resolve_field(val, "firstOctave")?;

        Ok((
            resolver.contextual_name.take().unwrap(),
            MultiOctaveNoiseParameters::new(first_octave, amplitudes.clone()),
        ))
    }
}

impl<T> Resolvable<T> for Rc<NoiseHolder<DefaultNoise>> {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self> {
        Ok(Rc::new(NoiseHolder::resolve(val, resolver)?))
    }
}

impl Resolvable<DensityFunctions> for DensityFunctions {
    fn resolve(val: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<Self> {
        match val {
            Value::String(alias) => resolver.deserialize(alias),
            Value::Object(_) => resolver.resolve(val),
            Value::Number(num) => Ok(DensityFunctions::Const(Const::new(num.as_f64().unwrap()))),
            v => Err(anyhow!(
                "Expected String, Object or Number, but given: {:?}",
                v
            )),
        }
    }
}
