use std::iter;

#[derive(Debug, Clone, Copy)]
pub struct ChordType {
    offsets: &'static [f32],
}

impl ChordType {
    pub const fn new(offsets: &'static [f32]) -> Self {
        Self { offsets }
    }

    pub fn num_notes(&self) -> usize {
        self.offsets.len() + 1
    }

    pub fn frequencies(&self, base_frequency: f32) -> impl Iterator<Item = f32> {
        iter::once(base_frequency).chain(
            self.offsets
                .iter()
                .map(move |&offset| base_frequency * offset),
        )
    }
}
