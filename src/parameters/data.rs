use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::{
    audio::AudioGenerationError,
    data::DataPoint,
    hash,
    parameters::oscillators::{
        OscillatorDistribution, OscillatorParameters, OscillatorTypeDistribution,
    },
};

use super::effects::{EffectDistribution, EffectParameters};

#[derive(Debug, Clone)]
pub struct DataParameters {
    sample_rate: u32,
    frequency_distribution: Uniform<f32>,
    oscillators: Vec<OscillatorDistribution>,
    effects: Vec<EffectDistribution>,
    num_samples: u64,
    seed_offset: u64,
}

impl DataParameters {
    pub fn new(sample_rate: u32, frequency_range: (f32, f32), num_samples: u64) -> Self {
        let min_frequency_map = crate::frequency_to_map(frequency_range.0);
        let max_frequency_map = crate::frequency_to_map(frequency_range.1);
        Self {
            sample_rate,
            frequency_distribution: Uniform::new(min_frequency_map, max_frequency_map),
            oscillators: vec![],
            effects: vec![],
            num_samples,
            seed_offset: hash(hash(0)),
        }
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

    pub fn with_effect(mut self, effect_distribution: EffectDistribution) -> Self {
        self.effects.push(effect_distribution);
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
    pub effects: Vec<EffectParameters>,
    pub num_samples: u64,
}

impl DataPointParameters {
    fn new(data_parameters: &DataParameters, seed: u64) -> Self {
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let frequency_map = data_parameters.frequency_distribution.sample(&mut rng);
        let frequency = crate::map_to_frequency(frequency_map);

        Self {
            sample_rate: data_parameters.sample_rate,
            frequency_map,
            frequency,
            oscillators: data_parameters
                .oscillators
                .iter()
                .map(|oscillator_distribution| oscillator_distribution.sample(&mut rng))
                .collect(),
            effects: data_parameters
                .effects
                .iter()
                .map(|effect_distribution| effect_distribution.sample(&mut rng))
                .collect(),
            num_samples: data_parameters.num_samples,
        }
    }

    pub fn generate(self) -> Result<DataPoint, AudioGenerationError> {
        DataPoint::new(self)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn map_to_note_number() {
        let cases = vec![(80., 830.61), (60., 261.63), (40., 82.41), (20., 25.96)];
        for (note_number, frequency) in cases {
            let map = crate::note_number_to_map(note_number);
            let frequency_from_map = crate::map_to_frequency(map);
            assert!((frequency_from_map - frequency).abs() < 0.01, "Note number: {note_number}  Frequency: {frequency}  Map: {map}  Frequency from map: {frequency_from_map}");

            let map = crate::frequency_to_map(frequency);
            let note_number_from_map = crate::map_to_note_number(map);
            assert!((note_number_from_map - note_number).abs() < 0.01, "Note number: {note_number}  Frequency: {frequency}  Map: {map}  Note number from map: {note_number_from_map}");
        }
    }
}
