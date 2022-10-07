use rand::{prelude::Distribution, Rng};

#[derive(Debug, Clone)]
pub struct LogUniform {
    min: f32,
    max: f32,
}

impl LogUniform {
    pub fn from_tuple(mut range: (f32, f32)) -> Self {
        assert!(range.0 >= 0.0, "Range must be non-negative.");
        if range.0 == 0.0 {
            range.0 = 1e-6;
        }
        assert!(range.1 >= range.0);
        Self {
            min: range.0,
            max: range.1,
        }
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn map_value(&self, frequency: f32) -> f32 {
        assert_ne!(self.min, self.max, "Cannot map a range of 0.");
        (frequency.ln() - self.min.ln()) / (self.max.ln() - self.min.ln()) * 2. - 1.
    }

    pub fn unmap(&self, map: f32) -> f32 {
        if self.min == self.max {
            self.min
        } else {
            ((self.max.ln() - self.min.ln()) * (map + 1.) / 2. + self.min.ln()).exp()
        }
    }

    /// Returns a tuple `(map, sample)` where `map` is uniformly distributed in `[-1.;1.]` while `sample` is the mapping from `map` onto the distribution's domain.
    pub fn sample_with_map<R>(&self, rng: &mut R) -> (f32, f32)
    where
        R: Rng + ?Sized,
    {
        let map = rng.gen_range(-1. ..=1.);
        (map, self.unmap(map))
    }
}

impl Distribution<f32> for LogUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        self.sample_with_map(rng).1
    }
}
