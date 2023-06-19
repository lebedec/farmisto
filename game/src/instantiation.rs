use crate::api::ActionError;
use crate::assembling::PlacementId;
use crate::building::{GridId, Marker, SurveyorId};
use crate::inventory::ContainerId;
use crate::landscaping::LandId;
use crate::model::*;
use crate::physics::{BarrierId, BodyId, SensorId, SpaceId};
use crate::planting::{PlantId, SoilId};
use crate::raising::{AnimalId, TetherId};
use crate::timing::CalendarId;
use crate::working::DeviceId;
use crate::Game;

impl Game {
    pub fn appear_crop(
        &mut self,
        key: CropKey,
        barrier: BarrierId,
        sensor: SensorId,
        plant: PlantId,
    ) -> Result<Universe, ActionError> {
        self.universe.crops_id += 1;
        let entity = Crop {
            id: self.universe.crops_id,
            key,
            plant,
            barrier,
            sensor,
        };
        self.universe.crops.push(entity);
        self.inspect_crop(entity)
    }

    pub fn appear_farmer(
        &mut self,
        kind: FarmerKey,
        player: PlayerId,
        body: BodyId,
        hands: ContainerId,
        backpack: ContainerId,
        tether: TetherId,
    ) -> Result<Universe, ActionError> {
        self.universe.farmers_id += 1;
        let entity = Farmer {
            id: self.universe.farmers_id,
            kind,
            player,
            body,
            hands,
            backpack,
            tether,
        };
        self.universe
            .farmers_activity
            .insert(entity, Activity::Idle);
        self.universe.farmers.push(entity);
        self.inspect_farmer(entity)
    }

    pub fn appear_farmland(
        &mut self,
        kind: FarmlandKey,
        space: SpaceId,
        soil: SoilId,
        grid: GridId,
        land: LandId,
        calendar: CalendarId,
    ) -> Result<Universe, ActionError> {
        self.universe.farmlands_id += 1;
        let entity = Farmland {
            id: self.universe.farmlands_id,
            kind,
            space,
            soil,
            grid,
            land,
            calendar,
        };
        self.universe.farmlands.push(entity);
        self.inspect_farmland(entity)
    }

    pub fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        surveyor: SurveyorId,
        stake: usize,
    ) -> Result<Universe, ActionError> {
        self.universe.constructions_id += 1;
        let construction = Construction {
            id: self.universe.constructions_id,
            container,
            grid,
            surveyor,
            stake,
        };
        self.universe.constructions.push(construction);
        self.inspect_construction(construction)
    }

    pub fn appear_theodolite(
        &mut self,
        key: TheodoliteKey,
        surveyor: SurveyorId,
        barrier: BarrierId,
    ) -> Result<Universe, ActionError> {
        self.universe.theodolites_id += 1;
        let theodolite = Theodolite {
            id: self.universe.theodolites_id,
            key,
            surveyor,
            barrier,
        };
        self.universe.theodolites.push(theodolite);
        self.inspect_theodolite(theodolite)
    }

    pub fn appear_creature(
        &mut self,
        key: CreatureKey,
        body: BodyId,
        animal: AnimalId,
    ) -> Result<Universe, ActionError> {
        self.universe.creatures_id += 1;
        let entity = Creature {
            id: self.universe.creatures_id,
            key,
            body,
            animal,
        };
        self.universe.creatures.push(entity);
        self.inspect_creature(entity)
    }

    pub fn appear_corpse(
        &mut self,
        key: CorpseKey,
        barrier: BarrierId,
    ) -> Result<Universe, ActionError> {
        self.universe.corpses_id += 1;
        let entity = Corpse {
            id: self.universe.corpses_id,
            key,
            barrier,
        };
        self.universe.corpses.push(entity);
        self.inspect_corpse(entity)
    }

    pub fn appear_assembling_activity(
        &mut self,
        farmer: Farmer,
        key: AssemblyKey,
        placement: PlacementId,
    ) -> Vec<Universe> {
        self.universe.assembly_id += 1;
        let assembly = Assembly {
            id: self.universe.creatures_id,
            key,
            placement,
        };
        self.universe.assembly.push(assembly);
        let look_event = self.inspect_assembly(assembly);
        let activity = Activity::Assembling { assembly };
        let events = self.universe.change_activity(farmer, activity);
        let mut stream = vec![look_event];
        stream.push(events);
        stream
    }

    pub fn appear_door(
        &mut self,
        key: DoorKey,
        barrier: BarrierId,
        placement: PlacementId,
    ) -> Universe {
        self.universe.doors_id += 1;
        let entity = Door {
            id: self.universe.doors_id,
            key,
            barrier,
            placement,
        };
        self.universe.doors.push(entity);
        self.look_at_door(entity)
    }

    pub fn appear_rest(
        &mut self,
        key: RestKey,
        barrier: BarrierId,
        placement: PlacementId,
    ) -> Universe {
        self.universe.rests_id += 1;
        let entity = Rest {
            id: self.universe.doors_id,
            key,
            barrier,
            placement,
        };
        self.universe.rests.push(entity);
        self.look_at_rest(entity)
    }

    pub fn appear_cementer(
        &mut self,
        key: CementerKey,
        barrier: BarrierId,
        placement: PlacementId,
        input: ContainerId,
        device: DeviceId,
        output: ContainerId,
    ) -> Universe {
        self.universe.cementers_id += 1;
        let entity = Cementer {
            id: self.universe.cementers_id,
            key,
            input,
            device,
            output,
            barrier,
            placement,
        };
        self.universe.cementers.push(entity);
        self.inspect_cementer(entity)
    }

    pub fn appear_composter(
        &mut self,
        key: ComposterKey,
        barrier: BarrierId,
        placement: PlacementId,
        input: ContainerId,
        device: DeviceId,
        output: ContainerId,
    ) -> Universe {
        self.universe.composters_id += 1;
        let entity = Composter {
            id: self.universe.composters_id,
            key,
            input,
            device,
            output,
            barrier,
            placement,
        };
        self.universe.composters.push(entity);
        self.inspect_composter(entity)
    }

    pub fn appear_stack(&mut self, container: ContainerId, barrier: BarrierId) -> Universe {
        self.universe.stacks_id += 1;
        let stack = Stack {
            id: self.universe.stacks_id,
            container,
            barrier,
        };
        self.universe.stacks.push(stack);
        self.inspect_stack(stack)
    }

    pub fn appear_equipment(
        &mut self,
        kind: EquipmentKey,
        purpose: Purpose,
        barrier: BarrierId,
    ) -> Universe {
        self.universe.equipments_id += 1;
        let equipment = Equipment {
            id: self.universe.equipments_id,
            key: kind,
            purpose,
            barrier,
        };
        self.universe.equipments.push(equipment);
        self.look_at_equipment(equipment)
    }
}
