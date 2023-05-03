use crate::raising::AnimalId;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

impl AnimalId {
    pub fn variant<const F: usize>(&self, features: [usize; F]) -> [usize; F] {
        let mut variants = features[0];
        for i in 1..F {
            variants *= features[i];
        }
        let seed = self.0 % variants;
        let mut random = StdRng::seed_from_u64(seed as u64);
        features.map(|size| {
            let range = random.gen_range(0..size);
            range
        })
    }
}
