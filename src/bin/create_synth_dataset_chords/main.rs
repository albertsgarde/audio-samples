use std::fs::File;

use anyhow::{Context, Result};
use audio_samples::{
    data::LABELS_FILE_NAME,
    log_uniform::LogUniform,
    parameters::{
        effects::EffectTypeDistribution, oscillators::OscillatorTypeDistribution, DataParameters,
        OctaveParameters, WaveForms,
    },
    UniformF,
};

const DATA_SET_SIZE: usize = 1000;

fn main() -> Result<()> {
    let output_path = r#"C:\Users\alber\Google Drive\DTU\Deep Learning\project\deep-learning\data\synth_chord_data"#;

    let octave_parameters = OctaveParameters::new(0.5, 0.3, 90., 10_000.);
    let wave_forms = WaveForms::new().load_dir_and_add("assets/custom_oscillators");
    let parameters = DataParameters::new(
        44100,
        (50., 2000.),
        (0.5, 3.),
        [1, 2, 3, 4, 5, 6],
        octave_parameters,
        wave_forms,
        256,
    );
    let parameters = parameters.with_oscillator(OscillatorTypeDistribution::Sine, 0.5, (0.1, 0.2));
    let parameters = parameters.with_oscillator(OscillatorTypeDistribution::Saw, 0.5, (0.1, 0.2));
    let parameters = parameters.with_oscillator(
        OscillatorTypeDistribution::Pulse(UniformF::new(0.1, 0.9)),
        0.5,
        (0.1, 0.2),
    );
    let parameters =
        parameters.with_oscillator(OscillatorTypeDistribution::Triangle, 0.5, (0.1, 0.2));
    let parameters = parameters.with_oscillator(OscillatorTypeDistribution::Noise, 0.5, (0.1, 0.2));
    let parameters = parameters.with_effect(
        EffectTypeDistribution::Distortion(LogUniform::from_tuple((0.1, 20.))),
        0.5,
    );
    let parameters = parameters.with_effect(EffectTypeDistribution::Normalize, 1.);

    let data_point_iterator = (0..).map(|i| parameters.generate(i as u64).generate().unwrap());

    let (data_points, labels): (Vec<_>, Vec<_>) = data_point_iterator
        .take(DATA_SET_SIZE)
        .enumerate()
        .map(|(i, data_point)| {
            let label = data_point.label();
            let data_point_name = format!("synth_chord_{i}");
            (
                (data_point_name.clone(), data_point.audio().clone()),
                (data_point_name, label),
            )
        })
        .unzip();

    for (data_point_name, audio) in data_points {
        let file_path = format!("{output_path}/{data_point_name}.wav",);
        audio.to_wav(file_path).context("Failed to write sample.")?;
    }

    let label_path = format!("{output_path}/{LABELS_FILE_NAME}",);
    let label_file = File::create(label_path).context("Could not create labels file.")?;
    serde_json::to_writer_pretty(label_file, &labels)?;

    Ok(())
}
