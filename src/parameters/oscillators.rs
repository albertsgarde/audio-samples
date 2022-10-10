use flexblock_synth::modules::{
    BoxedModule, Clamp, ModuleTemplate, NoiseOscillator, PulseOscillator, SawOscillator,
    SineOscillator, TriangleOscillator,
};
use rand::{distributions::Uniform, prelude::Distribution, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::log_uniform::LogUniform;

#[derive(Debug, Clone)]
pub enum OscillatorTypeDistribution {
    Sine,
    Saw,
    Pulse(Uniform<f32>),
    Triangle,
    Noise,
}

impl Distribution<OscillatorType> for OscillatorTypeDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OscillatorType {
        match self {
            OscillatorTypeDistribution::Sine => OscillatorType::Sine,
            OscillatorTypeDistribution::Saw => OscillatorType::Saw,
            OscillatorTypeDistribution::Pulse(pulse_width_distribution) => {
                OscillatorType::Pulse(pulse_width_distribution.sample(rng))
            }
            OscillatorTypeDistribution::Triangle => OscillatorType::Triangle,
            OscillatorTypeDistribution::Noise => OscillatorType::Noise(rng.next_u64()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OscillatorDistribution {
    oscillator_type_distribution: OscillatorTypeDistribution,
    amplitude_distribution: LogUniform,
}

impl OscillatorDistribution {
    pub fn new(
        oscillator_type_distribution: OscillatorTypeDistribution,
        amplitude_range: (f32, f32),
    ) -> Self {
        assert!(
            amplitude_range.0 >= 0.0,
            "Amplitude range must be non-negative."
        );
        assert!(
            amplitude_range.1 >= amplitude_range.0,
            "Amplitude range must be non-empty."
        );
        assert!(
            amplitude_range.1 <= 1.0,
            "Amplitude range must be less than 1."
        );
        Self {
            oscillator_type_distribution,
            amplitude_distribution: LogUniform::from_tuple(amplitude_range),
        }
    }

    pub fn maximum_amplitude(&self) -> f32 {
        self.amplitude_distribution.max()
    }
}

impl Distribution<OscillatorParameters> for OscillatorDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OscillatorParameters {
        OscillatorParameters {
            oscillator_type: self.oscillator_type_distribution.sample(rng),
            amplitude: self.amplitude_distribution.sample(rng),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OscillatorType {
    Sine,
    Saw,
    Pulse(f32),
    Triangle,
    // Contains the seed for the noise generator.
    Noise(u64),
}

#[derive(Debug, Clone)]
pub struct OscillatorParameters {
    oscillator_type: OscillatorType,
    amplitude: f32,
}

impl OscillatorParameters {
    pub fn create_oscillator(
        &self,
        frequency: f32,
        sample_rate: u32,
    ) -> ModuleTemplate<BoxedModule> {
        match self.oscillator_type {
            OscillatorType::Sine => SineOscillator::new(frequency, sample_rate).boxed(),
            OscillatorType::Saw => SawOscillator::new(frequency, sample_rate).boxed(),
            OscillatorType::Pulse(pulse_width) => {
                PulseOscillator::new(frequency, pulse_width, sample_rate).boxed()
            }
            OscillatorType::Triangle => TriangleOscillator::new(frequency, sample_rate).boxed(),
            OscillatorType::Noise(seed) => {
                Clamp::new(NoiseOscillator::new(Pcg64Mcg::seed_from_u64(seed)), -1., 1.).boxed()
            }
        }
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }
}
