use flexblock_synth::modules::{
    Module, NoiseOscillator, PulseOscillator, RandomWalk, SawOscillator, SineOscillator,
    TriangleOscillator,
};
use rand::{prelude::Distribution, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::{Deserialize, Serialize};

use crate::{log_uniform::LogUniform, Uniform};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscillatorTypeDistribution {
    Sine,
    Saw,
    Pulse(Uniform),
    Triangle,
    Noise,
}

impl OscillatorTypeDistribution {
    pub fn has_frequency(&self) -> bool {
        match self {
            OscillatorTypeDistribution::Sine => true,
            OscillatorTypeDistribution::Saw => true,
            OscillatorTypeDistribution::Pulse(_) => true,
            OscillatorTypeDistribution::Triangle => true,
            OscillatorTypeDistribution::Noise => false,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillatorDistribution {
    oscillator_type_distribution: OscillatorTypeDistribution,
    probability: f64,
    amplitude_distribution: LogUniform,
}

impl OscillatorDistribution {
    pub fn new(
        oscillator_type_distribution: OscillatorTypeDistribution,
        probability: f64,
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
        assert!(probability > 0.0, "Probability must be positive.");
        assert!(probability <= 1.0, "Probability must be less than 1.");
        Self {
            oscillator_type_distribution,
            probability,
            amplitude_distribution: LogUniform::from_tuple(amplitude_range),
        }
    }

    pub fn maximum_amplitude(&self) -> f32 {
        self.amplitude_distribution.max()
    }

    pub fn has_frequency(&self) -> bool {
        self.oscillator_type_distribution.has_frequency()
    }
}

impl Distribution<Option<OscillatorParameters>> for OscillatorDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<OscillatorParameters> {
        rng.gen_bool(self.probability)
            .then(|| OscillatorParameters {
                oscillator_type: self.oscillator_type_distribution.sample(rng),
                amplitude: self.amplitude_distribution.sample(rng),
            })
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
    fn write_oscillator(mut oscillator: impl Module, amplitude: f32, buffer: &mut [f32]) {
        for (sample_num, sample) in buffer.iter_mut().enumerate() {
            *sample += oscillator.next(sample_num as u64) * amplitude;
        }
    }

    pub fn write(
        &self,
        frequency: f32,
        frequency_std_dev: f32,
        frequency_random_walk_seed: u64,
        sample_rate: u32,
        buffer: &mut [f32],
    ) {
        let amplitude = self.amplitude;

        let frequency_walk_dampening = 0.9;
        let rng = Pcg64Mcg::seed_from_u64(frequency_random_walk_seed);
        let walk_std_dev = frequency * ((2f32).powf(frequency_std_dev / 1200.) - 1.);
        let frequency_module =
            RandomWalk::new(rng, walk_std_dev, frequency_walk_dampening, sample_rate) + frequency;

        match self.oscillator_type {
            OscillatorType::Sine => Self::write_oscillator(
                SineOscillator::new(frequency_module, sample_rate).module(),
                amplitude,
                buffer,
            ),
            OscillatorType::Saw => Self::write_oscillator(
                SawOscillator::new(frequency_module, sample_rate).module(),
                amplitude,
                buffer,
            ),
            OscillatorType::Pulse(duty_cycle) => Self::write_oscillator(
                (PulseOscillator::new(frequency_module, duty_cycle, sample_rate)
                    + -(duty_cycle * 2. - 1.))
                    .module(),
                amplitude,
                buffer,
            ),
            OscillatorType::Triangle => Self::write_oscillator(
                TriangleOscillator::new(frequency_module, sample_rate).module(),
                amplitude,
                buffer,
            ),
            OscillatorType::Noise(seed) => Self::write_oscillator(
                NoiseOscillator::new(Pcg64Mcg::seed_from_u64(seed)).module(),
                amplitude,
                buffer,
            ),
        }
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn has_frequency(&self) -> bool {
        self.amplitude > 0.0
            && match self.oscillator_type {
                OscillatorType::Sine => true,
                OscillatorType::Saw => true,
                OscillatorType::Pulse(_) => true,
                OscillatorType::Triangle => true,
                OscillatorType::Noise(_) => false,
            }
    }
}
