#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use audio_samples::{
    log_uniform::LogUniform,
    parameters::{
        effects::{EffectDistribution, EffectTypeDistribution},
        oscillators::OscillatorTypeDistribution,
        DataParameters, DataPointParameters, OctaveParameters, WaveForms,
    },
    Audio, UniformF, UniformI,
};
use flexblock_synth::modules::{
    lowpass_filter, ConvolutionFilter, Envelope, Module, ModuleTemplate, PulseOscillator,
    SawOscillator, SineOscillator, TriangleOscillator,
};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

const SAMPLE_RATE: u32 = 44100;
const SEED: u64 = 0;
const DATA_POINT_LENGTH: u64 = SAMPLE_RATE as u64 * 5;
const MIN_FREQUENCY: f32 = 50.0;
const MAX_FREQUENCY: f32 = 100.0;

fn main() -> Result<()> {
    let oscillator_amp_range = (0.002, 0.01);
    let oscillator_prob = 0.3;

    let octave_parameters = OctaveParameters::new(0.5, 0.3, 90., 10_000.);
    let wave_forms = WaveForms::new().load_dir_and_add("assets/custom_oscillators");
    let parameters = DataParameters::new(
        SAMPLE_RATE,
        (MIN_FREQUENCY, MAX_FREQUENCY),
        (0.5, 3.),
        [0, 1, 2, 3, 4, 5, 6],
        octave_parameters,
        wave_forms,
        DATA_POINT_LENGTH,
    )
    .with_oscillator(
        OscillatorTypeDistribution::Sine,
        oscillator_prob,
        oscillator_amp_range,
    )
    .with_oscillator(
        OscillatorTypeDistribution::Saw,
        oscillator_prob,
        oscillator_amp_range,
    )
    .with_oscillator(
        OscillatorTypeDistribution::Pulse(UniformF::new_inclusive(0.1, 0.9)),
        oscillator_prob,
        oscillator_amp_range,
    )
    .with_oscillator(
        OscillatorTypeDistribution::Triangle,
        oscillator_prob,
        oscillator_amp_range,
    )
    .with_oscillator(
        OscillatorTypeDistribution::Noise,
        oscillator_prob,
        (0.0001, 0.001),
    )
    .with_effect(
        EffectTypeDistribution::Distortion(LogUniform::from_tuple((0.1, 20.))),
        oscillator_prob,
    )
    .with_effect(EffectTypeDistribution::Normalize, 1.)
    .with_seed_offset(SEED);

    let parameters = (0..parameters.num_wave_forms()).fold(parameters, |parameters, i| {
        parameters.with_oscillator(
            OscillatorTypeDistribution::Custom(UniformI::new(i, i + 1)),
            oscillator_prob,
            oscillator_amp_range,
        )
    });

    for i in 0..10 {
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
