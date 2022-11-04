mod audio;
pub mod data;
pub mod effects;
mod log_uniform;
pub mod parameters;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub use audio::Audio;

const FREQUENCY_MAP_RANGE: (f32, f32) = (20., 20000.);
const A4_FREQUENCY: f32 = 440.0;
const A4_NOTE_NUMBER: f32 = 69.0;

fn hash(x: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

pub fn cent_diff(freq1: f32, freq2: f32) -> f32 {
    1200.0 * (freq2 / freq1).log2()
}

pub fn note_number_per_map() -> f32 {
    (FREQUENCY_MAP_RANGE.1 / FREQUENCY_MAP_RANGE.0).log2() * 12. * 0.5
}

pub fn frequency_to_map(frequency: f32) -> f32 {
    (frequency.ln() - FREQUENCY_MAP_RANGE.0.ln())
        / (FREQUENCY_MAP_RANGE.1.ln() - FREQUENCY_MAP_RANGE.0.ln())
        * 2.
        - 1.
}

pub fn map_to_frequency(map: f32) -> f32 {
    if FREQUENCY_MAP_RANGE.0 == FREQUENCY_MAP_RANGE.1 {
        FREQUENCY_MAP_RANGE.1
    } else {
        ((FREQUENCY_MAP_RANGE.1.ln() - FREQUENCY_MAP_RANGE.0.ln()) * (map + 1.) / 2.
            + FREQUENCY_MAP_RANGE.0.ln())
        .exp()
    }
}

pub fn map_to_note_number(map: f32) -> f32 {
    let a4_map = frequency_to_map(A4_FREQUENCY);
    A4_NOTE_NUMBER + (map - a4_map) * note_number_per_map()
}

pub fn note_number_to_map(note_number: f32) -> f32 {
    let a4_map = frequency_to_map(A4_FREQUENCY);
    a4_map + (note_number - A4_NOTE_NUMBER) / note_number_per_map()
}

pub fn frequency_to_note_number(frequency: f32) -> f32 {
    69.0 + 12.0 * (frequency / 440.0).log2()
}
