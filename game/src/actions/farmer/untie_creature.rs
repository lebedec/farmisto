use crate::api::{ActionError, Event};
use crate::model::{Activity, Creature, Farmer};
use crate::raising::TetherId;
use crate::{emit, Game};

impl Game {
    pub(crate) fn untie_creature2(
        &mut self,
        farmer: Farmer,
        tether: TetherId,
        _creature: Creature,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe
            .ensure_activity(farmer, Activity::Tethering2 { tether })?;
        let untie_animal = self.raising.untie_animal(tether)?;
        emit![untie_animal(),]
    }

    pub(crate) fn untie_creature(
        &mut self,
        farmer: Farmer,
        creature: Creature,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe
            .ensure_activity(farmer, Activity::Tethering { creature })?;
        let untie_animal = self.raising.untie_animal(farmer.tether)?;
        emit![
            untie_animal(),
            self.universe.change_activity(farmer, Activity::Usage)
        ]
    }
}
