use std::{collections::HashMap, fs::File};

use anyhow::{Context, Result};
use audio_samples::parameters::DataParameters;

const DATA_SET_SIZE: usize = 1000;

fn main() -> Result<()> {
    let output_path = r#"data/"#;

    let parameters = DataParameters::new(44100, (50., 2000.), [0], 256);

    let data_point_iterator = (0..).map(|i| parameters.generate(i as u64).generate().unwrap());

    let (data_points, labels): (Vec<_>, HashMap<_, _>) = data_point_iterator
        .take(DATA_SET_SIZE)
        .enumerate()
        .map(|(i, data_point)| {
            let data_point_name = format!("{i}");
            (
                (data_point_name.clone(), data_point.audio().clone()),
                (data_point_name, data_point.label()),
            )
        })
        .unzip();

    for (data_point_name, audio) in data_points {
        let file_path = format!("{output_path}/{data_point_name}.wav",);
        audio.to_wav(file_path).context("Failed to write sample.")?;
    }

    let label_path = format!("{output_path}/labels.json",);
    let label_file = File::create(label_path).context("Could not create labels file.")?;
    serde_json::to_writer_pretty(label_file, &labels)?;

    Ok(())
}
