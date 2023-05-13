use nanorand::{Rng, WyRand};

pub struct Random {
    generator: WyRand,
}

impl Random {
    pub fn new() -> Self {
        Self {
            generator: WyRand::new(),
        }
    }

    pub fn max(&mut self, max: f32) -> f32 {
        max * self.generator.generate::<f32>()
    }

    pub fn generate(&mut self) -> f32 {
        self.generator.generate()
    }
}
