use crate::raising::{Raising, RaisingDomain, RaisingError, Tether, TetherId};

impl RaisingDomain {
    pub fn create_tether(
        &mut self,
    ) -> Result<(TetherId, impl FnOnce() -> Vec<Raising> + '_), RaisingError> {
        let tether = TetherId(self.tethers_id + 1);
        let command = move || {
            self.tethers.push(Tether {
                id: tether,
                animal: None,
            });
            self.tethers_id += 1;
            vec![]
        };
        Ok((tether, command))
    }
}
