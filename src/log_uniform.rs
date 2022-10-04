use rand::{prelude::Distribution, Rng};

#[derive(Debug, Clone)]
pub struct LogUniform {
    min: f32,
    max: f32,
}

impl LogUniform {
    pub fn from_tuple(range: (f32, f32)) -> Self {
        Self {
            min: range.0,
            max: range.1,
        }
    }

    pub fn sample_from_map(&self, map: f32) -> f32 {
        ((self.max.ln() - self.min.ln()) * (map + 1.) / 2. + self.min.ln()).exp()
    }

    /// Returns a tuple `(map, sample)` where `map` is uniformly distributed in `[-1.;1.]` while `sample` is the mapping from `map` onto the distribution's domain.
    pub fn sample_with_map<R>(&self, rng: &mut R) -> (f32, f32)
    where
        R: Rng + ?Sized,
    {
        let map = rng.gen_range(-1. ..=1.);
        (map, self.sample_from_map(map))
    }
}

impl Distribution<f32> for LogUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        self.sample_with_map(rng).1
    }
}
