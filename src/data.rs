use flexblock_synth::modules::Sum;

use crate::{
    audio::AudioGenerationError,
    parameters::{DataParameters, DataPointParameters},
    Audio,
};

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub label: DataPointParameters,
}

impl DataPoint {
    pub fn new(params: DataPointParameters) -> Result<Self, AudioGenerationError> {
        let mut oscillators = vec![];
        for oscillator_params in params.oscillators.iter() {
            let oscillator_module = oscillator_params
                .create_oscillator(params.frequency, params.sample_rate)
                * oscillator_params.amplitude();
            oscillators.push(oscillator_module);
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
