use rand::{distributions::Standard, prelude::Distribution, seq::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::{Deserialize, Serialize};

use crate::{
    audio::AudioGenerationError,
    data::DataPoint,
    hash,
    log_uniform::LogUniform,
    parameters::oscillators::{
        OscillatorDistribution, OscillatorParameters, OscillatorTypeDistribution,
    },
    Uniform,
};

use super::effects::{EffectDistribution, EffectParameters, EffectTypeDistribution};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OctaveParameters {
    add_root_octave_probability: f64,
    add_other_octave_probability: f64,
    min_frequency: f32,
    max_frequency: f32,
}

impl OctaveParameters {
    pub fn new(
        add_root_octave_probability: f64,
        add_other_octave_probability: f64,
        min_frequency: f32,
        max_frequency: f32,
    ) -> Self {
        assert!(
            add_root_octave_probability >= 0.,
            "Probabilities must be non-negative. Value: {add_root_octave_probability}"
        );
        assert!(
            add_root_octave_probability <= 1.,
            "Probabilities must be no more than 1. Value: {add_root_octave_probability}"
        );
        assert!(
            add_other_octave_probability >= 0.,
            "Probabilities must be non-negative. Value: {add_other_octave_probability}"
        );
        assert!(
            add_other_octave_probability <= 1.,
            "Probabilities must be no more than 1. Value: {add_other_octave_probability}"
        );

        assert!(
            min_frequency > 0.,
            "Minimum frequency must be positive. Value: {min_frequency}"
        );

        assert!(
            max_frequency >= min_frequency*2.,
            "Maximum frequency must be at least twice the minimum frequency. Max frequency: {max_frequency}"
        );

        Self {
            add_root_octave_probability,
            add_other_octave_probability,
            min_frequency,
            max_frequency,
        }
    }

    pub fn generate_octave(&self, rng: &mut impl Rng, frequency: f32) -> f32 {
        let lowest_octave = frequency / ((frequency / self.min_frequency).floor());
        let num_octaves = (1u32..40)
            .find(|&i| lowest_octave * (i as f32).exp2() > self.max_frequency)
            .expect("Could not find octave");
        let octave = rng.gen_range(0..num_octaves);

        frequency * (octave as f32).exp2()
    }

    pub fn generate_frequencies(
        &self,
        rng: &mut impl Rng,
        frequency: f32,
        root: bool,
    ) -> impl Iterator<Item = f32> {
        let given_octave = (-10i32..40)
            .find(|&i| frequency / (i as f32).exp2() < self.min_frequency)
            .unwrap()
            - 1;

        let lowest_octave = frequency / (given_octave as f32).exp2();

        let num_octaves = (1i32..40)
            .find(|&i| lowest_octave * (i as f32).exp2() > self.max_frequency)
            .expect("Could not find octave");

        let mut octaves = vec![if root {
            given_octave
        } else {
            rng.gen_range(0..num_octaves)
        }];
        let add_octave_probability = if root {
            self.add_root_octave_probability
        } else {
            self.add_other_octave_probability
        };
        while rng.gen_bool(add_octave_probability) {
            let octave = rng.gen_range(0..num_octaves);
            if !octaves.contains(&octave) {
                octaves.push(octave);
            }
        }

        octaves
            .into_iter()
            .map(move |octave| lowest_octave * (octave as f32).exp2())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataParameters {
    sample_rate: u32,
    frequency_distribution: Uniform,
    frequency_std_dev_distribution: LogUniform,
    possible_chords: Vec<u32>,
    octave_parameters: OctaveParameters,
    oscillators: Vec<OscillatorDistribution>,
    effects: Vec<EffectDistribution>,
    num_samples: u64,
    seed_offset: u64,
}

impl DataParameters {
    pub fn new<A>(
        sample_rate: u32,
        frequency_range: (f32, f32),
        frequency_std_dev_range: (f32, f32),
        possible_chords: A,
        octave_parameters: OctaveParameters,
        num_samples: u64,
    ) -> Self
    where
        A: AsRef<[u32]>,
    {
        assert!(
            frequency_range.0 < frequency_range.1,
            "Invalid frequency range."
        );
        assert!(num_samples > 0, "num_samples must be greater than 0.");
        assert!(sample_rate > 0, "sample_rate must be greater than 0.");
        assert!(
            frequency_std_dev_range.0 >= 0.0,
            "frequency_std_dev_range must be non-negative."
        );
        assert!(
            frequency_std_dev_range.1 >= frequency_std_dev_range.0,
            "frequency_std_dev_range maximum must be no less than the minimum."
        );
        let possible_chords: Vec<u32> = possible_chords.as_ref().to_vec();
        assert!(!possible_chords.is_empty(), "No chords provided.");

        for &chord_type in possible_chords.iter() {
            assert!(
                chord_type < crate::CHORD_TYPES.len() as u32,
                "Invalid chord type. Chord type must be less than {}.",
                crate::CHORD_TYPES.len()
            );
        }

        let min_frequency_map = crate::frequency_to_map(frequency_range.0);
        let max_frequency_map = crate::frequency_to_map(frequency_range.1);
        Self {
            sample_rate,
            frequency_distribution: Uniform::new(min_frequency_map, max_frequency_map),
            frequency_std_dev_distribution: LogUniform::from_tuple(frequency_std_dev_range),
            possible_chords,
            octave_parameters,
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
        probability: f64,
        amplitude_range: (f32, f32),
    ) -> Self {
        self.oscillators.push(OscillatorDistribution::new(
            oscillator_type_distribution,
            probability,
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

    pub fn with_effect(
        mut self,
        effect_distribution: EffectTypeDistribution,
        probability: f64,
    ) -> Self {
        self.effects
            .push(EffectDistribution::new(effect_distribution, probability));
        self
    }

    pub fn generate(&self, index: u64) -> DataPointParameters {
        assert!(
            self.oscillators.iter().any(|osc| osc.has_frequency()),
            "Cannot generate a signal without an oscillator with frequency."
        );
        let seed = hash(index).wrapping_add(self.seed_offset);
        DataPointParameters::new(self, seed)
    }
}

#[derive(Debug, Clone)]
pub struct DataPointParameters {
    pub sample_rate: u32,
    pub base_frequency: f32,
    pub frequency_std_dev: f32,
    pub frequency_walk_seed: u64,
    pub chord_type: u32,
    pub frequencies: Vec<f32>,
    pub oscillators: Vec<OscillatorParameters>,
    pub effects: Vec<EffectParameters>,
    pub num_samples: u64,
}

impl DataPointParameters {
    fn new(data_parameters: &DataParameters, seed: u64) -> Self {
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let frequency_map = data_parameters.frequency_distribution.sample(&mut rng);
        let base_frequency = crate::map_to_frequency(frequency_map);

        let oscillators = loop {
            let oscillators: Vec<_> = data_parameters
                .oscillators
                .iter()
                .flat_map(|oscillator_distribution| oscillator_distribution.sample(&mut rng))
                .collect();
            if oscillators
                .iter()
                .any(|oscillator| oscillator.has_frequency())
            {
                break oscillators;
            }
        };

        let chord_type = *data_parameters.possible_chords.choose(&mut rng).unwrap();

        let chord = crate::CHORD_TYPES[chord_type as usize].1;

        let octave_parameters = &data_parameters.octave_parameters;

        let frequencies: Vec<f32> = octave_parameters
            .generate_frequencies(&mut rng, base_frequency, true)
            .chain(
                chord
                    .frequencies(base_frequency)
                    .skip(1)
                    .flat_map(|frequency| {
                        octave_parameters.generate_frequencies(&mut rng, frequency, false)
                    }),
            )
            .collect();

        Self {
            sample_rate: data_parameters.sample_rate,
            base_frequency,
            frequency_std_dev: data_parameters
                .frequency_std_dev_distribution
                .sample(&mut rng),
            frequency_walk_seed: rng.sample(Standard),
            chord_type,
            frequencies,
            oscillators,
            effects: data_parameters
                .effects
                .iter()
                .flat_map(|effect_distribution| effect_distribution.sample(&mut rng))
                .collect(),
            num_samples: data_parameters.num_samples,
        }
    }

    pub fn has_frequency(&self) -> bool {
        self.oscillators.iter().any(|osc| osc.has_frequency())
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
