use crate::api::{ActionError, Event};
use crate::building::SurveyorId;
use crate::inventory::ItemId;
use crate::model::{Activity, Construction, Equipment, Farmer, Farmland, Purpose};
use crate::Universe;
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
                let equipment_kind = self.known.equipments.get(equipment.key)?;
                let item = self.inventory.items_id.introduce().one(ItemId);
                let create_item =
                    self.inventory
                        .create_item(item, &equipment_kind.item, farmer.hands, 1)?;
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

    fn teardown_constructions(
        &mut self,
        _farmer: Farmer,
        _farmland: Farmland,
        surveyor: SurveyorId,
    ) -> Result<Vec<Event>, ActionError> {
        let constructions: Vec<Construction> = self
            .universe
            .constructions
            .iter()
            .filter(|construction| construction.surveyor == surveyor)
            .cloned()
            .collect();

        let containers = constructions
            .iter()
            .map(|construction| construction.container)
            .collect();

        let destroy_containers = self.inventory.destroy_containers(containers, false)?;

        let events = occur![
            destroy_containers(),
            constructions
                .into_iter()
                .map(|id| self.universe.vanish_construction(id))
                .flatten()
                .collect::<Vec<Universe>>(),
        ];

        Ok(events)
    }
}
