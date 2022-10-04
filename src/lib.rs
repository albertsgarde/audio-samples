mod audio;
mod log_uniform;
pub use audio::Audio;
use audio::AudioGenerationError;
use log_uniform::LogUniform;

use flexblock_synth::modules::SineOscillator;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct DataParameters {
    sample_rate: u32,
    frequency: LogUniform,
    num_samples: u64,
}

impl DataParameters {
    pub fn new(sample_rate: u32, frequency_range: (f32, f32), num_samples: u64) -> Self {
        Self {
            sample_rate,
            frequency: LogUniform::from_tuple(frequency_range),
            num_samples,
        }
    }

    fn generate(&self, rng: &mut impl Rng) -> DataPointParameters {
        DataPointParameters::new(self, rng)
    }
}

#[derive(Debug, Clone)]
pub struct DataPointParameters {
    pub sample_rate: u32,
    pub frequency_map: f32,
    pub frequency: f32,
    pub num_samples: u64,
}

impl DataPointParameters {
    fn new(data_parameters: &DataParameters, rng: &mut impl Rng) -> Self {
        let (frequency_map, frequency) = data_parameters.frequency.sample_with_map(rng);

        Self {
            sample_rate: data_parameters.sample_rate,
            frequency_map,
            frequency,
            num_samples: data_parameters.num_samples,
        }
    }
}

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub label: DataPointParameters,
}

impl DataPoint {
    pub fn new(params: DataPointParameters, _seed: u64) -> Result<Self, AudioGenerationError> {
        let synth = SineOscillator::new(params.frequency, params.sample_rate);
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
    rng: Pcg64Mcg,
    data_point_num: u64,
}

impl DataGenerator {
    pub fn new(data_parameters: DataParameters, seed: u64) -> Self {
        Self {
            data_parameters,
            rng: Pcg64Mcg::seed_from_u64(seed),
            data_point_num: 0,
        }
    }
}

impl Iterator for DataGenerator {
    type Item = Result<DataPoint, AudioGenerationError>;

    fn next(&mut self) -> Option<Self::Item> {
        let params = self.data_parameters.generate(&mut self.rng);
        let data_point = DataPoint::new(params, self.rng.gen());
        self.data_point_num += 1;
        Some(data_point)
    }
}
