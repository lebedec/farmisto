use crate::api::{ActionError, Event};
use crate::inventory::{FunctionsQuery, Installation};
use crate::math::TileMath;
use crate::model::{Activity, Farmer, Farmland, TheodoliteKey};
use crate::{emit, Game};

impl Game {
    pub(crate) fn install_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let installation = item.kind.functions.as_installation()?;

        match installation {
            Installation::Theodolite(key) => {
                let position = tile.position();
                let use_item = self.inventory.use_items_from(farmer.hands)?;
                let kind = self.known.theodolites.get(TheodoliteKey(key))?;
                let (surveyor, create_surveyor) = self
                    .building
                    .create_surveyor(farmland.grid, kind.surveyor.clone())?;
                let (barrier, create_barrier) = self.physics.create_barrier(
                    farmland.space,
                    kind.barrier.clone(),
                    position,
                    true,
                    false,
                )?;
                emit![
                    use_item(),
                    create_surveyor(),
                    create_barrier(),
                    self.appear_theodolite(kind.id, surveyor, barrier)?
                ]
            }
            Installation::Peg(_key) => {
                unimplemented!()
            }
        }

        // let key = EquipmentKey(key);
        // let equipment_kind = self.known.equipments.get(key)?;
        // match equipment_kind.purpose {
        //     PurposeDescription::Surveying { surveyor } => {
        //         let position = tile.position();
        //         let use_item = self.inventory.use_items_from(farmer.hands)?;
        //         let kind = self.known.surveyors.get(surveyor).unwrap();
        //         let (surveyor, create_surveyor) =
        //             self.building.create_surveyor(farmland.grid, kind)?;
        //         let (barrier, create_barrier) = self.physics.create_barrier(
        //             farmland.space,
        //             equipment_kind.barrier.clone(),
        //             position,
        //             true,
        //             false,
        //         )?;
        //         let purpose = Purpose::Surveying { surveyor };
        //         let change_activity = self.universe.change_activity(farmer, Activity::Idle);
        //         emit![
        //             use_item(),
        //             create_surveyor(),
        //             create_barrier(),
        //             self.appear_equipment(equipment_kind.id, purpose, barrier),
        //             change_activity,
        //         ]
        //     }
        //     PurposeDescription::Moisture { .. } => Ok(vec![]),
        //     PurposeDescription::Tethering => {
        //         let position = tile.position();
        //         let use_item = self.inventory.use_items_from(farmer.hands)?;
        //         let (tether, create_tether) = self.raising.create_tether()?;
        //         let (barrier, create_barrier) = self.physics.create_barrier(
        //             farmland.space,
        //             equipment_kind.barrier.clone(),
        //             position,
        //             true,
        //             false,
        //         )?;
        //         let purpose = Purpose::Tethering { tether };
        //         let change_activity = self.universe.change_activity(farmer, Activity::Idle);
        //         emit![
        //             use_item(),
        //             create_tether(),
        //             create_barrier(),
        //             self.appear_equipment(equipment_kind.id, purpose, barrier),
        //             change_activity
        //         ]
        //     }
        // }
    }
}
