use flexblock_synth::modules::Sum;

use crate::{audio::AudioGenerationError, parameters::DataPointParameters, Audio};

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
