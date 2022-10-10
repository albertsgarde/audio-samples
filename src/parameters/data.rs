use rand::{prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::{
    audio::AudioGenerationError,
    data::DataPoint,
    hash,
    log_uniform::LogUniform,
    parameters::oscillators::{
        OscillatorDistribution, OscillatorParameters, OscillatorTypeDistribution,
    },
};

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
            .map(|oscillator_distr| oscillator_distr.maximum_amplitude())
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
