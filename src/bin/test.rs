#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use audio_samples::{Audio, DataGenerator, DataParameters};
use flexblock_synth::modules::{
    lowpass_filter, ConvolutionFilter, Envelope, Module, ModuleTemplate, PulseOscillator,
    SawOscillator, SineOscillator, TriangleOscillator,
};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

const SAMPLE_RATE: u32 = 44100;
const SEED: u64 = 0;
const DATA_POINT_SAMPLES: u64 = 9184;
const MIN_FREQUENCY: f32 = 20.0;
const MAX_FREQUENCY: f32 = 20000.0;

fn main() -> Result<()> {
    let data_params = DataParameters {
        sample_rate: SAMPLE_RATE,
        frequency_range: (MIN_FREQUENCY, MAX_FREQUENCY),
        num_samples: DATA_POINT_SAMPLES,
    };

    let mut data_generator = DataGenerator::new(data_params, SEED);

    for i in 0..10 {
        let data_point = data_generator.next().unwrap().unwrap();
        let file_path = format!("output/data_point_{}.wav", i);
        data_point.audio().to_wav(file_path)?;
    }
    Ok(())
}
