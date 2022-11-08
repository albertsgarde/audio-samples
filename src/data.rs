use std::{collections::HashMap, path::Path};

use anyhow::Context;
use flexblock_synth::modules::{BatchedBoxedModule, BufferedSum, Module, ModuleTemplate};
use serde::{Deserialize, Serialize};

use crate::{audio::AudioGenerationError, parameters::DataPointParameters, Audio};

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub parameters: DataPointParameters,
}

impl DataPoint {
    fn oscillators_from_params_with_frequency(
        frequency: f32,
        amplitude_factor: f32,
        parameters: &DataPointParameters,
    ) -> impl Iterator<Item = ModuleTemplate<impl Module>> + '_ {
        parameters.oscillators.iter().map(move |oscillator_params| {
            oscillator_params.create_oscillator(frequency, parameters.sample_rate)
                * oscillator_params.amplitude()
                * amplitude_factor
        })
    }

    pub fn new(parameters: DataPointParameters) -> Result<Self, AudioGenerationError> {
        let (_, chord_type) = crate::CHORD_TYPES[parameters.chord_type as usize];

        let amplitude_factor = 1. / chord_type.num_notes() as f32;

        let oscillators: Vec<_> = chord_type
            .frequencies(parameters.frequency)
            .flat_map(|frequency| {
                Self::oscillators_from_params_with_frequency(
                    frequency,
                    amplitude_factor,
                    &parameters,
                )
            })
            .collect();

        let total_amplitude = parameters
            .oscillators
            .iter()
            .map(|oscillator_params| oscillator_params.amplitude())
            .sum::<f32>();

        let mut synth = BatchedBoxedModule::new(BufferedSum::new(oscillators, 64), 64);

        for effect in parameters.effects.iter() {
            synth = BatchedBoxedModule::new(effect.apply_effect(synth, total_amplitude), 64);
        }

        let audio =
            Audio::samples_from_module(&synth, parameters.sample_rate, parameters.num_samples)?;
        Ok(Self { audio, parameters })
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
    pub chord_type: u32,
    pub num_samples: u64,
}

impl DataPointLabel {
    pub fn new(params: &DataPointParameters) -> Self {
        Self {
            sample_rate: params.sample_rate,
            base_frequency_map: params.frequency_map,
            base_frequency: params.frequency,
            chord_type: params.chord_type,
            num_samples: params.num_samples,
        }
    }
}

pub fn load_dir<P>(path: P) -> anyhow::Result<Vec<(Audio, DataPointLabel)>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let labels_path = path.join("labels.json");
    let labels_file = std::fs::File::open(labels_path)?;
    let labels: HashMap<String, DataPointLabel> = serde_json::from_reader(labels_file)?;
    labels
        .into_iter()
        .map(|(data_point_name, label)| load_data_point(path, data_point_name, label))
        .collect()
}

fn load_data_point(
    dir_path: &Path,
    data_point_name: String,
    label: DataPointLabel,
) -> anyhow::Result<(Audio, DataPointLabel)> {
    let data_point_path = dir_path.join(format!("{data_point_name}.wav"));
    let audio = Audio::from_wav(data_point_path).context(format!(
        "Failed to load audio for data point '{data_point_name}'."
    ))?;
    assert_eq!(audio.sample_rate, label.sample_rate);
    assert_eq!(audio.num_samples(), label.num_samples as usize);
    Ok((audio, label))
}
