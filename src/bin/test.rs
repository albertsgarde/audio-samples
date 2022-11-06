#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use audio_samples::{
    parameters::{oscillators::OscillatorTypeDistribution, DataParameters, DataPointParameters},
    Audio,
};
use flexblock_synth::modules::{
    lowpass_filter, ConvolutionFilter, Envelope, Module, ModuleTemplate, PulseOscillator,
    SawOscillator, SineOscillator, TriangleOscillator,
};
use rand::{distributions::Uniform, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

const SAMPLE_RATE: u32 = 44100;
const SEED: u64 = 0;
const DATA_POINT_LENGTH: u64 = 9184;
const MIN_FREQUENCY: f32 = 20.0;
const MAX_FREQUENCY: f32 = 20000.0;

fn main() -> Result<()> {
    let parameters = DataParameters::new(
        SAMPLE_RATE,
        (MIN_FREQUENCY, MAX_FREQUENCY),
        [0u32],
        DATA_POINT_LENGTH,
    )
    .with_oscillator(OscillatorTypeDistribution::Sine, (0.0, 0.4))
    .with_oscillator(OscillatorTypeDistribution::Saw, (0.00, 0.15))
    .with_oscillator(
        OscillatorTypeDistribution::Pulse(Uniform::new_inclusive(0.0, 0.9)),
        (0.0, 0.2),
    )
    .with_oscillator(OscillatorTypeDistribution::Noise, (0., 0.25))
    .with_seed_offset(SEED);

    for (i, data_point) in (0..30)
        .map(|i| parameters.generate(i).generate())
        .enumerate()
    {
        let file_path = format!("output/{i}.wav");
        data_point?.audio().to_wav(file_path)?;
    }
    Ok(())
}
