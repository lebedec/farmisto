use crate::api::{ActionError, Event};
use crate::building::SurveyorId;
use crate::inventory::ItemId;
use crate::model::{Activity, Construction, Equipment, Farmer, Farmland, Purpose, Theodolite};
use crate::{emit, Universe};
use crate::{occur, Game};

impl Game {
    pub(crate) fn uninstall_theodolite(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        theodolite: Theodolite,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;

        // TODO: transactional
        let teardown = self.teardown_constructions(farmer, farmland, theodolite.surveyor)?;

        let destroy_surveyor = self.building.destroy_surveyor(theodolite.surveyor)?;
        let destroy_barrier = self.physics.destroy_barrier(theodolite.barrier)?;
        let kind = self.known.theodolites.get(theodolite.key)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_item = self
            .inventory
            .create_item(item, &kind.item, farmer.hands, 1)?;
        let vanish_theodolite = self.universe.vanish_theodolite(theodolite);
        let change_activity = self.universe.change_activity(farmer, Activity::Usage);

        let mut events = teardown;
        events.extend(occur![
            destroy_surveyor(),
            destroy_barrier(),
            create_item(),
            vanish_theodolite,
            change_activity,
        ]);
        Ok(events)
    }

    pub(crate) fn uninstall_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        match equipment.purpose {
            Purpose::Moisture { .. } => Ok(vec![]),
            Purpose::Tethering { tether } => {
                let destroy_tether = self.raising.destroy_tether(tether)?;
                let destroy_barrier = self.physics.destroy_barrier(equipment.barrier)?;
                let equipment_kind = self.known.equipments.get(equipment.key)?;
                let item = self.inventory.items_id.introduce().one(ItemId);
                let create_item =
                    self.inventory
                        .create_item(item, &equipment_kind.item, farmer.hands, 1)?;
                let vanish_equipment = self.universe.vanish_equipment(equipment);
                let change_activity = self.universe.change_activity(farmer, Activity::Usage);
                emit![
                    destroy_tether(),
                    destroy_barrier(),
                    create_item(),
                    vanish_equipment,
                    change_activity,
                ]
            }
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
