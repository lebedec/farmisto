use crate::raising::{Raising, RaisingDomain};

impl RaisingDomain {
    pub fn update(&mut self, _time: f32) -> Vec<Raising> {
        let events = vec![];
        for _animal in self.animals.iter_mut() {}
        events
    }
}
