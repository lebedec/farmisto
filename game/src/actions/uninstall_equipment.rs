use crate::api::{ActionError, Event};
use crate::model::{Activity, Equipment, Farmer, Farmland, Purpose};
use crate::{occur, Game};

impl Game {
    pub(crate) fn uninstall_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        match equipment.purpose {
            Purpose::Surveying { surveyor } => {
                // TODO: transactional
                let teardown_constructions =
                    self.teardown_constructions(farmer, farmland, surveyor)?;

                let destroy_surveyor = self.building.destroy_surveyor(surveyor)?;
                let destroy_barrier = self.physics.destroy_barrier(equipment.barrier)?;
                let equipment_kind = self.known.equipments.get(equipment.key).unwrap();
                let (_item, create_item) =
                    self.inventory
                        .create_item(&equipment_kind.item, farmer.hands, 1)?;
                let vanish_equipment = self.universe.vanish_equipment(equipment);
                let change_activity = self.universe.change_activity(farmer, Activity::Usage);

                let mut events = teardown_constructions;
                events.extend(occur![
                    destroy_surveyor(),
                    destroy_barrier(),
                    create_item(),
                    vanish_equipment,
                    change_activity,
                ]);
                Ok(events)
            }
            Purpose::Moisture { .. } => Ok(vec![]),
        }
    }
}
