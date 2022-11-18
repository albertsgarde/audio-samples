use std::path::Path;

use anyhow::Context;
use rand::{distributions, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::{Deserialize, Serialize};

use crate::{audio::AudioGenerationError, parameters::DataPointParameters, Audio};

pub const LABELS_FILE_NAME: &str = "_labels.json";

#[derive(Clone)]
pub struct DataPoint {
    pub audio: Audio,
    pub parameters: DataPointParameters,
}

impl DataPoint {
    fn generate_from_oscillators(parameters: &DataPointParameters) -> Vec<f32> {
        let mut samples = vec![0.; parameters.num_samples as usize];

        let (_, chord_type) = crate::CHORD_TYPES[parameters.chord_type as usize];

        let mut rng = Pcg64Mcg::seed_from_u64(parameters.frequency_walk_seed);

        for &frequency in parameters.frequencies.iter() {
            for oscillator_params in parameters.oscillators.iter() {
                oscillator_params.write(
                    frequency,
                    parameters.frequency_std_dev,
                    rng.sample(distributions::Standard),
                    parameters.sample_rate,
                    &mut samples,
                );
            }
        }

        let amplitude_factor = 1. / chord_type.num_notes() as f32;
        for sample in samples.iter_mut() {
            *sample *= amplitude_factor;
        }

        samples
    }

    fn apply_effects(parameters: &DataPointParameters, buffer: &mut [f32]) {
        let total_amplitude = parameters
            .oscillators
            .iter()
            .map(|oscillator_params| oscillator_params.amplitude())
            .sum::<f32>();

        for effect in parameters.effects.iter() {
            effect.apply_to_buffer(buffer, total_amplitude);
        }
    }

    pub fn new(parameters: DataPointParameters) -> Result<Self, AudioGenerationError> {
        let mut samples = Self::generate_from_oscillators(&parameters);

        Self::apply_effects(&parameters, &mut samples);

        let audio = Audio::from_samples(samples, parameters.sample_rate);
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
    pub base_frequency: Option<f32>,
    pub frequencies: Option<Vec<f32>>,
    pub note: Option<u32>,
    pub chord_type: u32,
    pub num_samples: u64,
}

impl DataPointLabel {
    pub fn new(params: &DataPointParameters) -> Self {
        Self {
            sample_rate: params.sample_rate,
            base_frequency: Some(params.base_frequency),
            frequencies: Some(params.frequencies.clone()),
            note: Some(crate::note_number_to_note(crate::frequency_to_note_number(
                params.base_frequency,
            ))),
            chord_type: params.chord_type,
            num_samples: params.num_samples,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn base_frequency(&self) -> Option<f32> {
        self.base_frequency
    }

    pub fn base_frequency_map(&self) -> Option<f32> {
        self.base_frequency.map(crate::frequency_to_map)
    }

    pub fn frequencies(&self) -> Option<&[f32]> {
        self.frequencies.as_deref()
    }

    pub fn note_number(&self) -> Option<f32> {
        self.base_frequency.map(crate::frequency_to_note_number)
    }

    pub fn note(&self) -> Option<u32> {
        self.note
    }

    pub fn chord_type(&self) -> u32 {
        self.chord_type
    }

    pub fn num_samples(&self) -> u64 {
        self.num_samples
    }
}

pub fn load_dir<P>(path: P) -> anyhow::Result<Vec<(Audio, DataPointLabel)>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let labels_path = path.join(LABELS_FILE_NAME);
    let labels_file = std::fs::File::open(labels_path)?;
    let labels: Vec<(String, DataPointLabel)> = serde_json::from_reader(labels_file)?;
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
