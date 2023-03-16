use ai::Behaviours;
use std::sync::{Arc, RwLock};

pub struct BehavioursAsset {
    data: Arc<RwLock<Behaviours>>,
}

impl BehavioursAsset {
    pub fn from(data: Behaviours) -> Self {
        BehavioursAsset {
            data: Arc::new(RwLock::new(data)),
        }
    }

    pub fn share(&self) -> BehavioursAsset {
        BehavioursAsset {
            data: self.data.clone(),
        }
    }

    pub fn share_data(&self) -> Arc<RwLock<Behaviours>> {
        self.data.clone()
    }

    pub fn update(&mut self, data: Behaviours) {
        let mut ptr = self.data.write().unwrap();
        *ptr = data;
    }
}
