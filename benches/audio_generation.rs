use audio_samples::parameters::{
    effects::EffectDistribution, oscillators::OscillatorTypeDistribution, DataParameters,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::distributions::Uniform;

fn bench_parameters(c: &mut Criterion, label: &str, parameters: &DataParameters) {
    let data_point_parameters = parameters.generate(0);
    c.bench_function(label, |b| {
        b.iter(|| {
            data_point_parameters.clone().generate().unwrap();
        })
    });
}

pub fn bench(c: &mut Criterion) {
    let single_note_parameters = DataParameters::new(44100, (50., 2000.), [0], 256);
    let large_chord_parameters = DataParameters::new(44100, (50., 2000.), [5], 256);
    let parameters = [
        ("empty", single_note_parameters.clone()),
        ("empty_chord", large_chord_parameters.clone()),
        (
            "sine",
            single_note_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.5, 0.7)),
        ),
        (
            "sine_chord",
            large_chord_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.5, 0.7)),
        ),
        (
            "sine_dist",
            single_note_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.5, 0.7))
                .with_effect(EffectDistribution::distortion((0.2, 20.))),
        ),
        (
            "sine_chord_dist",
            large_chord_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.5, 0.7))
                .with_effect(EffectDistribution::distortion((0.2, 20.))),
        ),
        (
            "all_osc",
            single_note_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, (0.1, 0.2)),
        ),
        (
            "all_osc_chord",
            large_chord_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, (0.1, 0.2)),
        ),
        (
            "all_osc_dist",
            single_note_parameters
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, (0.1, 0.2))
                .with_effect(EffectDistribution::distortion((0.2, 20.))),
        ),
        (
            "all_osc_chord_dist",
            large_chord_parameters
                .with_oscillator(OscillatorTypeDistribution::Sine, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, (0.1, 0.2))
                .with_effect(EffectDistribution::distortion((0.2, 20.))),
        ),
    ];
    for (label, parameters) in parameters.iter() {
        bench_parameters(c, label, parameters);
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
