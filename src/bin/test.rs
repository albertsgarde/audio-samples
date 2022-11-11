#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use audio_samples::{
    log_uniform::LogUniform,
    parameters::{
        effects::{EffectDistribution, EffectTypeDistribution},
        oscillators::OscillatorTypeDistribution,
        DataParameters, DataPointParameters,
    },
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
const DATA_POINT_LENGTH: u64 = SAMPLE_RATE as u64 * 5;
const MIN_FREQUENCY: f32 = 50.0;
const MAX_FREQUENCY: f32 = 2000.0;

fn main() -> Result<()> {
    let parameters = DataParameters::new(
        SAMPLE_RATE,
        (MIN_FREQUENCY, MAX_FREQUENCY),
        [0, 1, 2, 3, 4, 5, 6],
        DATA_POINT_LENGTH,
    )
    .with_oscillator(OscillatorTypeDistribution::Sine, 0.5, (0.1, 0.2))
    .with_oscillator(OscillatorTypeDistribution::Saw, 0.5, (0.1, 0.2))
    .with_oscillator(
        OscillatorTypeDistribution::Pulse(Uniform::new_inclusive(0.1, 0.9)),
        0.5,
        (0.1, 0.2),
    )
    .with_oscillator(OscillatorTypeDistribution::Triangle, 0.5, (0.1, 0.2))
    .with_oscillator(OscillatorTypeDistribution::Noise, 0.5, (0.01, 0.04))
    .with_effect(
        EffectTypeDistribution::Distortion(LogUniform::from_tuple((0.1, 20.))),
        0.5,
    )
    .with_effect(EffectTypeDistribution::Normalize, 1.)
    .with_seed_offset(SEED);

    for i in 0..100 {
        let data_parameters = parameters.generate(i);
        let data_point = data_parameters.generate().unwrap();
        data_point.audio().to_wav(format!("data/test_{i}.wav"))?;
        println!("Written test_{i}.wav");
    }

    let data_point_parameters = parameters.generate(0);
    println!("{data_point_parameters:?}");
    let data_point = data_point_parameters.generate().unwrap();

    println!("{}", data_point.audio().num_samples());
    Ok(())
}
