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

fn hash(x: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

pub fn cent_diff(freq1: f32, freq2: f32) -> f32 {
    1200.0 * (freq2 / freq1).log2()
}
