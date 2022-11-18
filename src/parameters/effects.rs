use float_ord::FloatOrd;
use rand::{prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};

use crate::log_uniform::LogUniform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectTypeDistribution {
    Distortion(LogUniform),
    Normalize,
}

impl EffectTypeDistribution {
    pub fn distortion(power_range: (f32, f32)) -> Self {
        Self::Distortion(LogUniform::from_tuple(power_range))
    }

    pub fn normalize() -> Self {
        Self::Normalize
    }
}

impl Distribution<EffectParameters> for EffectTypeDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EffectParameters {
        match self {
            EffectTypeDistribution::Distortion(power_distribution) => {
                EffectParameters::Distortion(power_distribution.sample(rng))
            }
            EffectTypeDistribution::Normalize => EffectParameters::Normalize,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDistribution {
    effect_type_distribution: EffectTypeDistribution,
    probability: f64,
}

impl EffectDistribution {
    pub fn new(effect_type_distribution: EffectTypeDistribution, probability: f64) -> Self {
        assert!(probability > 0.0, "Probability must be positive.");
        assert!(
            probability <= 1.0,
            "Probability must be less than or equal to 1."
        );
        Self {
            effect_type_distribution,
            probability,
        }
    }

    pub fn probability(&self) -> f64 {
        self.probability
    }
}

impl Distribution<Option<EffectParameters>> for EffectDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<EffectParameters> {
        rng.gen_bool(self.probability)
            .then(|| self.effect_type_distribution.sample(rng))
    }
}

#[derive(Debug, Clone)]
pub enum EffectParameters {
    Distortion(f32),
    Normalize,
}

impl EffectParameters {
    pub fn apply_to_buffer(&self, buffer: &mut [f32], signal_amplitude: f32) {
        match self {
            EffectParameters::Distortion(power) => buffer.iter_mut().for_each(|sample| {
                *sample = flexblock_synth::effects::distortion(*sample, *power, signal_amplitude)
            }),
            EffectParameters::Normalize => {
                let max_amplitude = buffer
                    .iter()
                    .map(|sample| sample.abs())
                    .max_by_key(|&x| FloatOrd(x))
                    .expect("Buffer must have at least one sample to normalize.");
                assert!(
                    max_amplitude >= 0.,
                    "Max amplitude must be non-negative. Max amplitude: {}",
                    max_amplitude
                );
                if max_amplitude > 0. {
                    buffer
                        .iter_mut()
                        .for_each(|sample| *sample /= max_amplitude);
                }
                let &max_amplitude = buffer
                    .iter()
                    .max_by_key(|&&x| FloatOrd(x.abs()))
                    .expect("Buffer must have at least one sample to normalize.");
                assert!(max_amplitude <= 1.);
            }
        }
    }
}
