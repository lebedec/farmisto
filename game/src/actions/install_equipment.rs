use crate::api::{ActionError, Event};
use crate::inventory::FunctionsQuery;
use crate::model::{Activity, EquipmentKey, Farmer, Farmland, Purpose, PurposeDescription};
use crate::{occur, position_of, Game};

impl Game {
    pub(crate) fn install_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let key = item.kind.functions.as_installation()?;
        let key = EquipmentKey(key);
        let equipment_kind = self.known.equipments.get(key)?;
        match equipment_kind.purpose {
            PurposeDescription::Surveying { surveyor } => {
                let position = position_of(tile);
                let use_item = self.inventory.use_items_from(farmer.hands)?;
                let kind = self.known.surveyors.get(surveyor).unwrap();
                let (surveyor, create_surveyor) =
                    self.building.create_surveyor(farmland.grid, kind)?;
                let kind = self.known.barriers.find("<equipment>").unwrap();
                let (barrier, create_barrier) =
                    self.physics
                        .create_barrier(farmland.space, kind, position, true, false)?;
                let purpose = Purpose::Surveying { surveyor };
                let change_activity = self.universe.change_activity(farmer, Activity::Idle);
                let events = occur![
                    use_item(),
                    create_surveyor(),
                    create_barrier(),
                    self.appear_equipment(equipment_kind.id, purpose, barrier),
                    change_activity,
                ];
                Ok(events)
            }
            PurposeDescription::Moisture { .. } => Ok(vec![]),
        }
    }
}
