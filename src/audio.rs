use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
    path::Path,
};

use anyhow::{Context, Ok, Result};
use flexblock_synth::modules::{Module, ModuleTemplate};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rustfft::{
    num_complex::{Complex, Complex32},
    FftPlanner,
};

#[derive(Debug, Clone)]
pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub enum AudioGenerationError {
    Clipping(u64),
}

impl Display for AudioGenerationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clipping(sample_index) => write!(
                f,
                "Audio generation failed due to clipping at sample {sample_index}.",
            ),
        }
    }
}

impl Error for AudioGenerationError {}

#[derive(Debug)]
pub enum UnsupportedWavSpec {
    Channels(u16),
    BitDepth(u16),
    SampleFormat(SampleFormat),
}

impl Display for UnsupportedWavSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Channels(channels) => write!(
                f,
                "Unsupported number of channels {channels}. Only mono is supported.",
            ),
            Self::BitDepth(bit_depth) => write!(
                f,
                "Unsupported bit depth {bit_depth}. Only 32-bit float is supported.",
            ),
            Self::SampleFormat(sample_format) => write!(
                f,
                "Unsupported sample format {sample_format:?}. Only 32-bit float is supported.",
            ),
        }
    }
}

impl Error for UnsupportedWavSpec {}

impl Audio {
    pub fn from_samples(samples: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
        }
    }

    pub fn from_module<M>(
        module: &ModuleTemplate<M>,
        sample_rate: u32,
        duration: f32,
    ) -> Result<Self, AudioGenerationError>
    where
        M: Module,
    {
        let num_samples = ((duration - duration.floor()) * sample_rate as f32) as u64
            + duration.floor() as u64 * sample_rate as u64;
        Self::samples_from_module(module, sample_rate, num_samples)
    }

    pub fn samples_from_module<M>(
        module: &ModuleTemplate<M>,
        sample_rate: u32,
        num_samples: u64,
    ) -> Result<Self, AudioGenerationError>
    where
        M: Module,
    {
        assert!(sample_rate != 0);
        let mut module = module.create_instance();
        let mut samples = Vec::with_capacity(num_samples as usize);
        for sample_num in 0..num_samples {
            let sample = module.next(sample_num as u64);
            if sample.abs() > 1. {
                return Err(AudioGenerationError::Clipping(sample_num as u64));
            }
            samples.push(sample)
        }
        Result::Ok(Self {
            samples,
            sample_rate,
        })
    }

    pub fn from_spectrum<A>(spectrum: A, sample_rate: u32) -> Self
    where
        A: AsRef<[Complex32]>,
    {
        let mut spectrum = spectrum.as_ref().to_vec();
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(spectrum.len());
        fft.process(&mut spectrum);
        let num_samples = spectrum.len();
        let samples = spectrum
            .into_iter()
            .map(|c| c.re / num_samples as f32)
            .collect();
        Self {
            samples,
            sample_rate,
        }
    }

    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }

    pub fn fft(&self) -> Vec<Complex32> {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.num_samples());

        let mut buffer: Vec<_> = self.samples.iter().map(|&x| Complex::new(x, 0.)).collect();
        fft.process(&mut buffer);
        buffer
    }

    pub fn to_wav<P>(&self, file_path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut writer = WavWriter::create(
            file_path,
            WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
        )
        .context("Could not create WavWriter.")?;

        for &sample in self.samples.iter() {
            writer
                .write_sample(sample)
                .context("Failed to write sample.")?;
        }
        Ok(())
    }

    pub fn from_wav<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut reader = WavReader::open(file_path)?;
        let spec = reader.spec();
        if spec.channels != 1 {
            Err(UnsupportedWavSpec::Channels(spec.channels).into())
        } else if spec.bits_per_sample != 32 {
            Err(UnsupportedWavSpec::BitDepth(spec.bits_per_sample).into())
        } else if spec.sample_format != SampleFormat::Float {
            Err(UnsupportedWavSpec::SampleFormat(spec.sample_format).into())
        } else {
            let samples = reader.samples::<f32>().map(|s| s.unwrap()).collect();
            Ok(Self {
                samples,
                sample_rate: spec.sample_rate,
            })
        }
    }

    pub fn to_csv<P>(&self, file_path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut file = File::create(file_path)?;
        writeln!(file, "Index,Sample")?;
        let mut result = "Index,Sample\n".to_owned();
        for (i, &sample) in self.samples.iter().enumerate() {
            result += &format!("{i},{sample}\n");
        }
        write!(file, "{result}")?;
        Ok(())
    }
}
