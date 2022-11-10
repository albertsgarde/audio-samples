use flexblock_synth::modules::{BoxedModule, Distortion, Module, ModuleTemplate};
use rand::{prelude::Distribution, Rng};

use crate::log_uniform::LogUniform;

#[derive(Debug, Clone)]
pub enum EffectDistribution {
    Distortion(LogUniform),
}

impl EffectDistribution {
    pub fn distortion(power_range: (f32, f32)) -> Self {
        Self::Distortion(LogUniform::from_tuple(power_range))
    }
}

impl Distribution<EffectParameters> for EffectDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EffectParameters {
        match self {
            EffectDistribution::Distortion(power_distribution) => {
                EffectParameters::Distortion(power_distribution.sample(rng))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EffectParameters {
    Distortion(f32),
}

impl EffectParameters {
    pub fn apply_effect<M>(
        &self,
        module: ModuleTemplate<M>,
        signal_amplitude: f32,
    ) -> ModuleTemplate<BoxedModule>
    where
        M: Module,
    {
        match self {
            EffectParameters::Distortion(power) => {
                Distortion::new(module, signal_amplitude, *power).boxed()
            }
        }
    }

    pub fn apply_to_buffer(&self, buffer: &mut [f32], signal_amplitude: f32) {
        match self {
            EffectParameters::Distortion(power) => buffer.iter_mut().for_each(|sample| {
                *sample = flexblock_synth::effects::distortion(*sample, *power, signal_amplitude)
            }),
        }
    }
}
