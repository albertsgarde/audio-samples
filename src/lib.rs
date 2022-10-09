mod audio;
pub mod effects;
mod log_uniform;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub use audio::Audio;
use audio::AudioGenerationError;
use log_uniform::LogUniform;

use flexblock_synth::modules::{
    BoxedModule, Clamp, ModuleTemplate, NoiseOscillator, PulseOscillator, SawOscillator,
    SineOscillator, Sum, TriangleOscillator,
};
use rand::{distributions::Uniform, prelude::Distribution, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

fn hash(x: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

pub fn cent_diff(freq1: f32, freq2: f32) -> f32 {
    1200.0 * (freq2 / freq1).log2()
}

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
struct OscillatorDistribution {
    oscillator_type_distribution: OscillatorTypeDistribution,
    amplitude_distribution: LogUniform,
}

impl OscillatorDistribution {
    fn new(
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
pub struct DataParameters {
    sample_rate: u32,
    frequency_distribution: LogUniform,
    oscillators: Vec<OscillatorDistribution>,
    num_samples: u64,
    seed_offset: u64,
}

impl DataParameters {
    pub fn new(sample_rate: u32, frequency_range: (f32, f32), num_samples: u64) -> Self {
        Self {
            sample_rate,
            frequency_distribution: LogUniform::from_tuple(frequency_range),
            oscillators: vec![],
            num_samples,
            seed_offset: hash(hash(0)),
        }
    }

    pub fn frequency_to_map(&self, frequency: f32) -> f32 {
        self.frequency_distribution.map_value(frequency)
    }

    pub fn map_to_frequency(&self, map: f32) -> f32 {
        self.frequency_distribution.unmap(map)
    }

    pub fn with_seed_offset(mut self, seed_offset: u64) -> Self {
        self.seed_offset = hash(hash(seed_offset));
        self
    }

    pub fn with_oscillator(
        mut self,
        oscillator_type_distribution: OscillatorTypeDistribution,
        amplitude_range: (f32, f32),
    ) -> Self {
        self.oscillators.push(OscillatorDistribution::new(
            oscillator_type_distribution,
            amplitude_range,
        ));
        let osc_amplitude_sum = self
            .oscillators
            .iter()
            .map(|oscillator_distr| oscillator_distr.amplitude_distribution.max())
            .sum::<f32>();
        if osc_amplitude_sum > 1. {
            panic!(
                "The sum of oscillator amplitudes must not exceed 1. Current: {osc_amplitude_sum}"
            );
        }
        self
    }

    pub fn generate(&self, index: u64) -> DataPointParameters {
        let seed = hash(index).wrapping_add(self.seed_offset);
        DataPointParameters::new(self, seed)
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
    fn create_oscillator(&self, frequency: f32, sample_rate: u32) -> ModuleTemplate<BoxedModule> {
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
}

#[derive(Debug, Clone)]
pub struct DataPointParameters {
    pub sample_rate: u32,
    pub frequency_map: f32,
    pub frequency: f32,
    pub oscillators: Vec<OscillatorParameters>,
    pub num_samples: u64,
}

impl DataPointParameters {
    fn new(data_parameters: &DataParameters, seed: u64) -> Self {
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let (frequency_map, frequency) = data_parameters
            .frequency_distribution
            .sample_with_map(&mut rng);

        Self {
            sample_rate: data_parameters.sample_rate,
            frequency_map,
            frequency,
            oscillators: data_parameters
                .oscillators
                .iter()
                .map(|oscillator_distribution| oscillator_distribution.sample(&mut rng))
                .collect(),
            num_samples: data_parameters.num_samples,
        }
    }

    pub fn generate(self) -> Result<DataPoint, AudioGenerationError> {
        DataPoint::new(self)
    }
}

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub label: DataPointParameters,
}

impl DataPoint {
    pub fn new(params: DataPointParameters) -> Result<Self, AudioGenerationError> {
        let mut oscillators = vec![];
        for oscillator in params.oscillators.iter() {
            let oscillator = oscillator.create_oscillator(params.frequency, params.sample_rate)
                * oscillator.amplitude;
            oscillators.push(oscillator);
        }

        let synth = Sum::new(oscillators);

        let audio = Audio::samples_from_module(&synth, params.sample_rate, params.num_samples)?;
        Ok(Self {
            audio,
            label: params,
        })
    }

    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    pub fn label(&self) -> &DataPointParameters {
        &self.label
    }
}

pub struct DataGenerator {
    data_parameters: DataParameters,
    data_point_num: u64,
}

impl DataGenerator {
    pub fn new(data_parameters: DataParameters) -> Self {
        Self {
            data_parameters,
            data_point_num: 0,
        }
    }
}

impl Iterator for DataGenerator {
    type Item = Result<DataPoint, AudioGenerationError>;

    fn next(&mut self) -> Option<Self::Item> {
        let data_point = self
            .data_parameters
            .generate(self.data_point_num)
            .generate();
        self.data_point_num += 1;
        Some(data_point)
    }
}
