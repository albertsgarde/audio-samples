use anyhow::{Context, Result};
use audio_samples::data::DataPointLabel;
use hound::{SampleFormat, WavReader, WavWriter};
use rand::Rng;
use std::{collections::HashMap, fs::File};

const RUN_NAMES: &[&str] = &["loud", "quiet"];

fn normalize_samples<A>(samples: A) -> Vec<f32>
where
    A: AsRef<[f32]>,
{
    let samples = samples.as_ref();
    let max = samples
        .iter()
        .map(|sample| sample.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    samples.iter().map(|sample| sample / max).collect()
}

fn sample_data_points(
    samples: &[f32],
    num_data_points: usize,
    data_point_samples: usize,
    rng: &mut impl Rng,
) -> Vec<Vec<f32>> {
    (0..num_data_points)
        .map(|_| {
            let start_point = rng.gen_range(0..samples.len() - data_point_samples);
            normalize_samples(&samples[start_point..start_point + data_point_samples])
        })
        .collect()
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();

    let start_note: u32 = 12;
    let end_note = 91;

    let data_point_samples = 256;
    let data_points_per_note = 4;

    let note_length = 1.;

    let src_path = r#"C:\Users\alber\Google Drive\Music (Albert)\Studio One\Songs\Audio Samples\Mixdown\sampled_strings.wav"#;
    let dest_path = r#"C:\Users\alber\Google Drive\DTU\Deep Learning\project\deep-learning\data\sampled_strings"#;

    let mut reader = WavReader::open(src_path).context("Could not open source file.")?;
    let spec = reader.spec();
    let samples: Vec<_> = if spec.channels != 1 {
        panic!("Unsupported number of channels: {}", spec.channels);
    } else if spec.bits_per_sample != 32 {
        panic!("Unsupported bit depth: {}", spec.bits_per_sample);
    } else if spec.sample_format != SampleFormat::Float {
        panic!("Unsupported sample format: {:?}", spec.sample_format);
    } else {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    };

    let sample_rate = spec.sample_rate;
    let samples_per_note = (sample_rate as f32 * note_length) as usize;

    let label_iterator = RUN_NAMES
        .iter()
        .flat_map(|name| (start_note..=end_note).map(move |note| (*name, note)));

    let samples = samples
        .chunks(samples_per_note)
        .zip(label_iterator)
        .flat_map(|(chunk, label)| {
            sample_data_points(chunk, data_points_per_note, data_point_samples, &mut rng)
                .into_iter()
                .map(move |samples| (samples, label))
                .enumerate()
        });

    let (data_points, labels): (Vec<_>, HashMap<_, _>) = samples
        .into_iter()
        .map(|(sub_index, (samples, (run_name, note_number)))| {
            let base_frequency_map = audio_samples::note_number_to_map(note_number as f32);
            let base_frequency = audio_samples::map_to_frequency(base_frequency_map);

            let label = DataPointLabel {
                sample_rate,
                base_frequency_map,
                base_frequency,
                num_samples: samples.len() as u64,
            };

            let data_point_name = format!("{}_{}_{}", run_name, note_number, sub_index);

            ((data_point_name.clone(), samples), (data_point_name, label))
        })
        .unzip();

    for (data_point_name, samples) in data_points {
        let file_path = format!("{dest_path}/{data_point_name}.wav",);
        let mut writer =
            WavWriter::create(file_path, spec).context("Could not create WavWriter.")?;

        for &sample in samples.iter() {
            writer
                .write_sample(sample)
                .context("Failed to write sample.")?;
        }
    }

    let label_path = format!("{dest_path}/labels.json",);
    let label_file = File::create(label_path).context("Could not create labels file.")?;
    serde_json::to_writer_pretty(label_file, &labels)?;

    Ok(())
}
