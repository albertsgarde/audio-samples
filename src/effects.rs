use rustfft::num_complex::Complex32;

use crate::Audio;

pub fn low_pass(audio: &Audio, cutoff_freq: f32) -> Audio {
    let mut spectrum = audio.fft();
    let cutoff_index = (cutoff_freq / audio.sample_rate as f32 * spectrum.len() as f32) as usize;
    for value in spectrum.iter_mut().skip(cutoff_index) {
        *value = Complex32::new(0., 0.);
    }
    Audio::from_spectrum(spectrum, audio.sample_rate)
}
