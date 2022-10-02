use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Ok, Result};
use flexblock_synth::modules::{Module, ModuleTemplate};
use hound::{SampleFormat, WavSpec, WavWriter};

#[derive(Debug, Clone)]
pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub enum AudioGenerationError {
    Clipping(u64),
}

impl Audio {
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
        let mut samples = vec![0.; num_samples as usize];
        for (sample_num, sample_ref) in samples.iter_mut().enumerate() {
            let sample = module.next(sample_num as u64);
            if sample.abs() > 1. {
                return Err(AudioGenerationError::Clipping(sample_num as u64));
            }
            *sample_ref = sample;
        }
        Result::Ok(Self {
            samples,
            sample_rate,
        })
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
