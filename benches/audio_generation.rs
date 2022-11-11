use audio_samples::parameters::{
    effects::EffectTypeDistribution, oscillators::OscillatorTypeDistribution, DataParameters,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use flexblock_synth::modules::{ObjectSafeModule, PulseOscillator};
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
            single_note_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Sine,
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "sine_chord",
            large_chord_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Sine,
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "sine_dist",
            single_note_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.5, 0.7))
                .with_effect(EffectTypeDistribution::distortion((0.2, 20.)), 1.),
        ),
        (
            "sine_chord_dist",
            large_chord_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.5, 0.7))
                .with_effect(EffectTypeDistribution::distortion((0.2, 20.)), 1.),
        ),
        (
            "all_osc",
            single_note_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, 1., (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    1.,
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, 1., (0.1, 0.2)),
        ),
        (
            "all_osc_chord",
            large_chord_parameters
                .clone()
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, 1., (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    1.,
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, 1., (0.1, 0.2)),
        ),
        (
            "all_osc_dist",
            single_note_parameters
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, 1., (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    1.,
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, 1., (0.1, 0.2))
                .with_effect(EffectTypeDistribution::distortion((0.2, 20.)), 1.),
        ),
        (
            "all_osc_chord_dist",
            large_chord_parameters
                .with_oscillator(OscillatorTypeDistribution::Sine, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Saw, 1., (0.1, 0.2))
                .with_oscillator(OscillatorTypeDistribution::Triangle, 1., (0.1, 0.2))
                .with_oscillator(
                    OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                    1.,
                    (0.1, 0.2),
                )
                .with_oscillator(OscillatorTypeDistribution::Noise, 1., (0.1, 0.2))
                .with_effect(EffectTypeDistribution::distortion((0.2, 20.)), 1.),
        ),
    ];
    for (label, parameters) in parameters.iter() {
        bench_parameters(c, label, parameters);
    }
}

pub fn oscillators(c: &mut Criterion) {
    let base_parameters = DataParameters::new(44100, (50., 2000.), [0], 256);

    let parameters = [
        (
            "osc_sine",
            base_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Sine,
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "osc_saw",
            base_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Saw,
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "osc_triangle",
            base_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Triangle,
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "osc_pulse",
            base_parameters.clone().with_oscillator(
                OscillatorTypeDistribution::Pulse(Uniform::new(0.1, 0.9)),
                1.,
                (0.5, 0.7),
            ),
        ),
        (
            "osc_noise",
            base_parameters.with_oscillator(OscillatorTypeDistribution::Noise, 1., (0.5, 0.7)),
        ),
    ];

    for (label, parameters) in parameters.iter() {
        bench_parameters(c, label, parameters);
    }
}

pub fn module(c: &mut Criterion) {
    let frequency = 440.;
    let duty_cycle = 0.5;
    let inverse_sample_rate = 1. / 44100.;

    let mut buffer = vec![0.; 256];

    c.bench_function("direct", |b| {
        b.iter_batched(
            || (),
            |_| {
                let mut cur_pos = 0.;
                for sample in buffer.iter_mut() {
                    cur_pos += frequency * inverse_sample_rate;
                    cur_pos %= 1.;
                    if cur_pos < duty_cycle {
                        *sample = 1.;
                    } else {
                        *sample = -1.;
                    }
                }

                black_box(&buffer);
            },
            BatchSize::SmallInput,
        )
    });

    let module = PulseOscillator::new(frequency, duty_cycle, 44100).module();

    c.bench_function("module", |b| {
        b.iter_batched(
            || module.clone(),
            |mut module| {
                for (i, sample) in buffer.iter_mut().enumerate() {
                    *sample = module.next(i as u64);
                }
                black_box(&buffer);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
