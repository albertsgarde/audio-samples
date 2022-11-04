use flexblock_synth::modules::Sum;
use serde::{Deserialize, Serialize};

use crate::{audio::AudioGenerationError, parameters::DataPointParameters, Audio};

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub parameters: DataPointParameters,
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

        let total_amplitude = params
            .oscillators
            .iter()
            .map(|oscillator_params| oscillator_params.amplitude())
            .sum::<f32>();

        let mut synth = Sum::new(oscillators).boxed();

        for effect in params.effects.iter() {
            synth = effect.apply_effect(synth, total_amplitude).boxed();
        }

        let audio = Audio::samples_from_module(&synth, params.sample_rate, params.num_samples)?;
        Ok(Self {
            audio,
            parameters: params,
        })
    }

    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    pub fn parameters(&self) -> &DataPointParameters {
        &self.parameters
    }

    pub fn label(&self) -> DataPointLabel {
        DataPointLabel::new(&self.parameters)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPointLabel {
    pub sample_rate: u32,
    pub base_frequency_map: f32,
    pub base_frequency: f32,
    pub num_samples: u64,
}

impl DataPointLabel {
    pub fn new(params: &DataPointParameters) -> Self {
        Self {
            sample_rate: params.sample_rate,
            base_frequency_map: params.frequency_map,
            base_frequency: params.frequency,
            num_samples: params.num_samples,
        }
    }
}
