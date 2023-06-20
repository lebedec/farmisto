use crate::building::{BuildingDomain, Grid, Surveyor};

impl BuildingDomain {
    pub fn load_grids(&mut self, grids: Vec<Grid>, sequence: usize) {
        self.grids_sequence = sequence;
        self.grids.extend(grids);
    }

    pub fn load_surveyors(&mut self, surveyors: Vec<Surveyor>, sequence: usize) {
        self.surveyors_sequence = sequence;
        self.surveyors.extend(surveyors);
    }
}
