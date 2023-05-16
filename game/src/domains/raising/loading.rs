use crate::raising::domain::{Animal, RaisingDomain, Tether};

impl RaisingDomain {
    pub fn load_animals(&mut self, animals: Vec<Animal>, sequence: usize) {
        self.animals_id = sequence;
        for animal in animals {
            self.animals.push(animal);
        }
    }
    
    pub fn load_tethers(&mut self, tethers: Vec<Tether>, sequence: usize) {
        self.tethers_id = sequence;
        self.tethers.extend(tethers);
    }
}