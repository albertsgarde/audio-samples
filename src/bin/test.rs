#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use audio_samples::{Audio, DataGenerator, DataParameters, DataPointParameters};
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
    let synth = SawOscillator::new(440.0, SAMPLE_RATE);

    let audio = Audio::from_module(&synth, SAMPLE_RATE, 1.)?;

    let audio = audio_samples::effects::low_pass(&audio, 1000.);

    let file_path = "test.wav";
    audio.to_wav(file_path)?;
    Ok(())
}
