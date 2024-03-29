use core::fmt::Debug;

use datamap::Storage;
use log::info;
use serde::de;

use crate::assembling::{Placement, PlacementId};
use crate::building::{
    Grid, GridId, GridKey, GridKind, Surveyor, SurveyorId, SurveyorKey, SurveyorKind,
};
use crate::collections::DictionaryError;
use crate::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, Item, ItemId, ItemKey, ItemKind,
};
use crate::landscaping::{Land, LandId, LandKey, LandKind};
use crate::model::{
    Assembly, AssemblyKey, AssemblyKind, AssemblyTarget, Cementer, CementerKey, CementerKind,
    Composter, ComposterKey, ComposterKind, Construction, Corpse, CorpseKey, CorpseKind, Creature,
    CreatureKey, CreatureKind, Crop, CropKey, CropKind, Door, DoorKey, DoorKind, Equipment,
    EquipmentKey, EquipmentKind, Farmer, FarmerKey, FarmerKind, Farmland, FarmlandKey,
    FarmlandKind, Knowledge, Player, PlayerId, Purpose, PurposeDescription, Rest, RestKey,
    RestKind, Stack, Theodolite, TheodoliteKey, TheodoliteKind, Tree, TreeKey, TreeKind,
};
use crate::physics::{
    Barrier, BarrierId, BarrierKey, BarrierKind, Body, BodyId, BodyKey, BodyKind, Sensor, SensorId,
    SensorKey, SensorKind, Space, SpaceId, SpaceKey, SpaceKind,
};
use crate::planting::{Plant, PlantId, PlantKey, PlantKind, Soil, SoilId, SoilKey, SoilKind};
use crate::raising::{Animal, AnimalId, AnimalKey, AnimalKind, Tether, TetherId};
use crate::timing::{Calendar, CalendarId, CalendarKey, CalendarKind, MinGameMinute};
use crate::working::{Device, DeviceId, DeviceKey, DeviceKind};
use crate::Game;

impl Game {
    pub fn load_knowledge(path: &str) -> Knowledge {
        let storage = Storage::open(path).unwrap();
        let mut game = Game::new(storage);
        game.load_game_knowledge().unwrap();
        game.known
    }

    pub fn load_game_knowledge(&mut self) -> Result<(), DataError> {
        info!("Starts game knowledge loading from {}", self.storage.path);
        let storage = self.storage.open_into();
        // timing
        for kind in storage.find_all(|row| self.load_calendar_kind(row))? {
            self.known
                .calendars
                .insert(kind.id, kind.name.clone(), kind);
        }
        // physics
        for kind in storage.find_all(|row| self.load_space_kind(row))? {
            self.known.spaces.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_body_kind(row))? {
            self.known.bodies.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_barrier_kind(row))? {
            self.known.barriers.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_sensor_kind(row))? {
            self.known.sensors.insert(kind.id, kind.name.clone(), kind);
        }
        // planting
        for kind in storage.find_all(|row| self.load_soil_kind(row))? {
            self.known.soils.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_plant_kind(row))? {
            self.known.plants.insert(kind.id, kind.name.clone(), kind);
        }
        // raising
        for kind in storage.find_all(|row| self.load_animal_kind(row))? {
            self.known.animals.insert(kind.id, kind.name.clone(), kind);
        }
        // building
        for kind in storage.find_all(|row| self.load_grid_kind(row))? {
            self.known.grids.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_surveyor_kind(row))? {
            self.known
                .surveyors
                .insert(kind.id, kind.name.clone(), kind);
        }
        // inventory
        for kind in storage.find_all(|row| self.load_container_kind(row))? {
            self.known
                .containers
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_item_kind(row))? {
            self.known.items.insert(kind.id, kind.name.clone(), kind);
        }
        // working
        for kind in storage.find_all(|row| self.load_device_kind(row))? {
            self.known.devices.insert(kind.id, kind.name.clone(), kind);
        }
        // landscaping
        for kind in storage.find_all(|row| self.load_land_kind(row))? {
            self.known.lands.insert(kind.id, kind.name.clone(), kind);
        }
        // universe
        for kind in storage.find_all(|row| self.load_tree_kind(row))? {
            self.known.trees.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_farmland_kind(row))? {
            self.known
                .farmlands
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_farmer_kind(row))? {
            self.known.farmers.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_equipment_kind(row))? {
            self.known
                .equipments
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_theodolite_kind(row))? {
            self.known
                .theodolites
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_crop_kind(row))? {
            self.known.crops.insert(kind.id, kind.name.clone(), kind);
        }
        // creature references:
        for kind in storage.find_all(|row| self.load_corpse_kind(row))? {
            self.known.corpses.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_creature_kind(row))? {
            self.known
                .creatures
                .insert(kind.id, kind.name.clone(), kind);
        }
        // assembly references:
        for kind in storage.find_all(|row| self.load_cementer_kind(row))? {
            self.known
                .cementers
                .insert(kind.key, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_composter_kind(row))? {
            self.known
                .composters
                .insert(kind.key, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_door_kind(row))? {
            self.known.doors.insert(kind.key, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_rest_kind(row))? {
            self.known.rests.insert(kind.key, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_assembly_kind(row))? {
            self.known
                .assembly
                .insert(kind.key, kind.name.clone(), kind);
        }
        info!("Ends game knowledge loading");

        Ok(())
    }

    pub fn load_game_state(&mut self) -> Result<(), DataError> {
        info!("Starts game state loading from {}", self.storage.path);
        let storage = self.storage.open_into();
        self.players = storage.find_all(|row| self.load_player(row))?;

        // timing
        let (calendars, _) = storage.get_sequence(|row| self.load_calendar(row))?;
        self.timing.load_calendars(calendars);

        // physics
        let (spaces, sequence) = storage.get_sequence(|row| self.load_space(row))?;
        self.physics.load_spaces(spaces, sequence);
        let (bodies, _) = storage.get_sequence(|row| self.load_body(row))?;
        self.physics.load_bodies(bodies);
        let (barriers, sequence) = storage.get_sequence(|row| self.load_barrier(row))?;
        self.physics.load_barriers(barriers, sequence);
        let (sensors, sequence) = storage.get_sequence(|row| self.load_sensor(row))?;
        self.physics.load_sensors(sensors, sequence);

        // planting
        let (soils, sequence) = storage.get_sequence(|row| self.load_soil(row))?;
        self.planting.load_soils(soils, sequence);
        let (plants, sequence) = storage.get_sequence(|row| self.load_plant(row))?;
        self.planting.load_plants(plants, sequence);

        // landscaping
        let (lands, _) = storage.get_sequence(|row| self.load_land(row))?;
        self.landscaping.load_lands(lands);

        // raising
        let (animals, sequence) = storage.get_sequence(|row| self.load_animal(row))?;
        self.raising.load_animals(animals, sequence);
        let (tethers, sequence) = storage.get_sequence(|row| self.load_tether(row))?;
        self.raising.load_tethers(tethers, sequence);

        // building
        let (grids, sequence) = storage.get_sequence(|row| self.load_grid(row))?;
        self.building.load_grids(grids, sequence);
        let (surveyors, sequence) = storage.get_sequence(|row| self.load_surveyor(row))?;
        self.building.load_surveyors(surveyors, sequence);

        // inventory
        let (containers, sequence) = storage.get_sequence(|row| self.load_container(row))?;
        self.inventory.load_containers(containers, sequence);
        let (items, sequence) = storage.get_sequence(|row| self.load_item(row))?;
        self.inventory.load_items(items, sequence);

        // assembling
        let (placements, sequence) = storage.get_sequence(|row| self.load_placement(row))?;
        self.assembling.load_placements(placements, sequence);

        // working
        let (devices, sequence) = storage.get_sequence(|row| self.load_device(row))?;
        self.working.load_devices(devices, sequence);

        // models
        let (trees, trees_id) = storage.get_sequence(|row| self.load_tree(row))?;
        self.universe.load_trees(trees, trees_id);
        let (farmlands, farmlands_id) = storage.get_sequence(|row| self.load_farmland(row))?;
        self.universe.load_farmlands(farmlands, farmlands_id);
        let (theodolites, theodolites_id) =
            storage.get_sequence(|row| self.load_theodolite(row))?;
        self.universe.load_theodolites(theodolites, theodolites_id);
        let (farmers, farmers_id) = storage.get_sequence(|row| self.load_farmer(row))?;
        self.universe.load_farmers(farmers, farmers_id);
        let (stacks, stacks_id) = storage.get_sequence(|row| self.load_stack(row))?;
        self.universe.load_stacks(stacks, stacks_id);
        let (constructions, id) = storage.get_sequence(|row| self.load_construction(row))?;
        self.universe.load_constructions(constructions, id);
        let (equipments, id) = storage.get_sequence(|row| self.load_equipment(row))?;
        self.universe.load_equipments(equipments, id);
        let (crops, id) = storage.get_sequence(|row| self.load_crop(row))?;
        self.universe.load_crops(crops, id);
        let (corpses, id) = storage.get_sequence(|row| self.load_corpse(row))?;
        self.universe.load_corpses(corpses, id);
        let (creatures, id) = storage.get_sequence(|row| self.load_creature(row))?;
        self.universe.load_creatures(creatures, id);
        // assembly references:
        let (doors, id) = storage.get_sequence(|row| self.load_door(row))?;
        self.universe.load_doors(doors, id);
        let (rests, id) = storage.get_sequence(|row| self.load_rest(row))?;
        self.universe.load_rests(rests, id);
        let (cementers, id) = storage.get_sequence(|row| self.load_cementer(row))?;
        self.universe.load_cementers(cementers, id);
        let (composters, id) = storage.get_sequence(|row| self.load_composter(row))?;
        self.universe.load_composters(composters, id);
        let (assembly, id) = storage.get_sequence(|row| self.load_assembly(row))?;
        self.universe.load_assembly(assembly, id);

        info!("Ends game state loading");

        Ok(())
    }

    pub fn save_game(&mut self) {}

    pub(crate) fn load_player(&mut self, row: &rusqlite::Row) -> Result<Player, DataError> {
        let data = Player {
            id: PlayerId(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_equipment_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<EquipmentKind, DataError> {
        let purpose = PurposeDescription::Tethering;
        let data = EquipmentKind {
            id: EquipmentKey(row.get("id")?),
            name: row.get("name")?,
            purpose,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            item: self.known.items.find_by(row, "item")?,
        };
        Ok(data)
    }

    pub(crate) fn load_equipment(&mut self, row: &rusqlite::Row) -> Result<Equipment, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let barrier = row.get("barrier")?;
        let purpose = Purpose::Moisture { sensor: 0 };
        let data = Equipment {
            id,
            key: EquipmentKey(kind),
            purpose,
            barrier: BarrierId(barrier),
        };
        Ok(data)
    }

    pub(crate) fn load_farmland_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<FarmlandKind, DataError> {
        let data = FarmlandKind {
            id: FarmlandKey(row.get("id")?),
            name: row.get("name")?,
            space: self.known.spaces.find_by(row, "space")?,
            soil: self.known.soils.find_by(row, "soil")?,
            grid: self.known.grids.find_by(row, "grid")?,
            land: self.known.lands.find_by(row, "land")?,
            calendar: self.known.calendars.find_by(row, "calendar")?,
        };
        Ok(data)
    }

    pub(crate) fn load_farmland(&mut self, row: &rusqlite::Row) -> Result<Farmland, DataError> {
        let data = Farmland {
            id: row.get("id")?,
            kind: FarmlandKey(row.get("kind")?),
            space: SpaceId(row.get("space")?),
            soil: SoilId(row.get("soil")?),
            grid: GridId(row.get("grid")?),
            land: LandId(row.get("land")?),
            calendar: CalendarId(row.get("calendar")?),
        };
        Ok(data)
    }

    pub(crate) fn load_theodolite_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<TheodoliteKind, DataError> {
        let data = TheodoliteKind {
            id: TheodoliteKey(row.get("id")?),
            name: row.get("name")?,
            surveyor: self.known.surveyors.find_by(row, "surveyor")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            item: self.known.items.find_by(row, "item")?,
        };
        Ok(data)
    }

    pub(crate) fn load_theodolite(&mut self, row: &rusqlite::Row) -> Result<Theodolite, DataError> {
        let data = Theodolite {
            id: row.get("id")?,
            key: TheodoliteKey(row.get("key")?),
            surveyor: SurveyorId(row.get("surveyor")?),
            barrier: BarrierId(row.get("barrier")?),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<FarmerKind, DataError> {
        let id = row.get("id")?;
        let body = row.get("body")?;
        let data = FarmerKind {
            id: FarmerKey(id),
            name: row.get("name")?,
            body: BodyKey(body),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer(&mut self, row: &rusqlite::Row) -> Result<Farmer, DataError> {
        let data = Farmer {
            id: row.get("id")?,
            kind: FarmerKey(row.get("kind")?),
            player: PlayerId(row.get("player")?),
            body: BodyId(row.get("body")?),
            hands: ContainerId(row.get("hands")?),
            backpack: ContainerId(row.get("backpack")?),
            tether: TetherId(row.get("tether")?),
        };
        Ok(data)
    }

    pub(crate) fn load_crop_kind(&mut self, row: &rusqlite::Row) -> Result<CropKind, DataError> {
        let data = CropKind {
            id: CropKey(row.get("id")?),
            name: row.get("name")?,
            plant: self.known.plants.find_by(row, "plant")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            sensor: self.known.sensors.find_by(row, "sensor")?,
            fruits: self.known.items.find_by(row, "fruits")?,
            residue: self.known.items.find_by(row, "residue")?,
        };
        Ok(data)
    }

    pub(crate) fn load_crop(&mut self, row: &rusqlite::Row) -> Result<Crop, DataError> {
        let data = Crop {
            id: row.get("id")?,
            key: CropKey(row.get("kind")?),
            plant: PlantId(row.get("plant")?),
            barrier: BarrierId(row.get("barrier")?),
            sensor: SensorId(row.get("sensor")?),
        };
        Ok(data)
    }

    pub(crate) fn load_creature_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<CreatureKind, DataError> {
        let data = CreatureKind {
            id: CreatureKey(row.get("id")?),
            name: row.get("name")?,
            body: self.known.bodies.find_by(row, "body")?,
            animal: self.known.animals.find_by(row, "animal")?,
            corpse: self.known.corpses.find_by(row, "corpse")?,
        };
        Ok(data)
    }

    pub(crate) fn load_creature(&mut self, row: &rusqlite::Row) -> Result<Creature, DataError> {
        let data = Creature {
            id: row.get("id")?,
            key: CreatureKey(row.get("kind")?),
            body: BodyId(row.get("body")?),
            animal: AnimalId(row.get("animal")?),
        };
        Ok(data)
    }

    pub(crate) fn load_corpse_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<CorpseKind, DataError> {
        let data = CorpseKind {
            id: CorpseKey(row.get("id")?),
            name: row.get("name")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            item: self.known.items.find_by(row, "item")?,
        };
        Ok(data)
    }

    pub(crate) fn load_corpse(&mut self, row: &rusqlite::Row) -> Result<Corpse, DataError> {
        let data = Corpse {
            id: row.get("id")?,
            key: CorpseKey(row.get("key")?),
            barrier: BarrierId(row.get("barrier")?),
        };
        Ok(data)
    }

    pub(crate) fn load_tree_kind(&mut self, row: &rusqlite::Row) -> Result<TreeKind, DataError> {
        let data = TreeKind {
            id: TreeKey(row.get("id")?),
            name: row.get("name")?,
            barrier: self.known.barriers.get_by(row, "barrier", BarrierKey)?,
            plant: self.known.plants.get_by(row, "plant", PlantKey)?,
        };
        Ok(data)
    }

    pub(crate) fn load_tree(&mut self, row: &rusqlite::Row) -> Result<Tree, DataError> {
        let data = Tree {
            id: row.get("id")?,
            kind: TreeKey(row.get("kind")?),
            plant: PlantId(row.get("plant")?),
            barrier: BarrierId(row.get("barrier")?),
        };
        Ok(data)
    }

    pub(crate) fn load_stack(&mut self, row: &rusqlite::Row) -> Result<Stack, DataError> {
        let data = Stack {
            id: row.get("id")?,
            barrier: BarrierId(row.get("barrier")?),
            container: ContainerId(row.get("container")?),
        };
        Ok(data)
    }

    pub(crate) fn load_construction(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<Construction, DataError> {
        let data = Construction {
            id: row.get("id")?,
            container: ContainerId(row.get("container")?),
            grid: GridId(row.get("grid")?),
            surveyor: SurveyorId(row.get("surveyor")?),
            stake: row.get("stake")?,
        };
        Ok(data)
    }

    pub(crate) fn load_assembly_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<AssemblyKind, DataError> {
        let target = if let Ok(Some(name)) = row.get("t_door") {
            let door = self.known.doors.find2(&name)?;
            AssemblyTarget::Door { door }
        } else if let Ok(Some(name)) = row.get("t_cementer") {
            let cementer = self.known.cementers.find2(&name)?;
            AssemblyTarget::Cementer { cementer }
        } else if let Ok(Some(name)) = row.get("t_rest") {
            let rest = self.known.rests.find2(&name)?;
            AssemblyTarget::Rest { rest }
        } else if let Ok(Some(name)) = row.get("t_composter") {
            let composter = self.known.composters.find2(&name)?;
            AssemblyTarget::Composter { composter }
        } else {
            return Err(DataError::NotSpecifiedVariant);
        };
        let data = AssemblyKind {
            key: AssemblyKey(row.get("id")?),
            name: row.get("name")?,
            target,
        };
        Ok(data)
    }

    pub(crate) fn load_assembly(&mut self, row: &rusqlite::Row) -> Result<Assembly, DataError> {
        let data = Assembly {
            id: row.get("id")?,
            key: AssemblyKey(row.get("key")?),
            placement: PlacementId(row.get("placement")?),
        };
        Ok(data)
    }

    pub(crate) fn load_door_kind(&mut self, row: &rusqlite::Row) -> Result<DoorKind, DataError> {
        let data = DoorKind {
            key: DoorKey(row.get("id")?),
            name: row.get("name")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            kit: self.known.items.find_by(row, "kit")?,
        };
        Ok(data)
    }

    pub(crate) fn load_door(&mut self, row: &rusqlite::Row) -> Result<Door, DataError> {
        let data = Door {
            id: row.get("id")?,
            key: DoorKey(row.get("key")?),
            barrier: BarrierId(row.get("barrier")?),
            placement: PlacementId(row.get("placement")?),
        };
        Ok(data)
    }

    pub(crate) fn load_rest_kind(&mut self, row: &rusqlite::Row) -> Result<RestKind, DataError> {
        let data = RestKind {
            key: RestKey(row.get("id")?),
            name: row.get("name")?,
            comfort: row.get("comfort")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            kit: self.known.items.find_by(row, "kit")?,
        };
        Ok(data)
    }

    pub(crate) fn load_rest(&mut self, row: &rusqlite::Row) -> Result<Rest, DataError> {
        let data = Rest {
            id: row.get("id")?,
            key: RestKey(row.get("key")?),
            barrier: BarrierId(row.get("barrier")?),
            placement: PlacementId(row.get("placement")?),
        };
        Ok(data)
    }

    pub(crate) fn load_cementer_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<CementerKind, DataError> {
        let data = CementerKind {
            key: CementerKey(row.get("id")?),
            name: row.get("name")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            device: self.known.devices.find_by(row, "device")?,
            input_offset: row.get_json("input_offset")?,
            input: self.known.containers.find_by(row, "input")?,
            output_offset: row.get_json("output_offset")?,
            output: self.known.containers.find_by(row, "output")?,
            kit: self.known.items.find_by(row, "kit")?,
            cement: self.known.items.find_by(row, "cement")?,
        };
        Ok(data)
    }

    pub(crate) fn load_cementer(&mut self, row: &rusqlite::Row) -> Result<Cementer, DataError> {
        let data = Cementer {
            id: row.get("id")?,
            key: CementerKey(row.get("kind")?),
            input: ContainerId(row.get("input")?),
            device: DeviceId(row.get("device")?),
            output: ContainerId(row.get("output")?),
            barrier: BarrierId(row.get("barrier")?),
            placement: PlacementId(row.get("placement")?),
        };
        Ok(data)
    }

    pub(crate) fn load_composter_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<ComposterKind, DataError> {
        let data = ComposterKind {
            key: ComposterKey(row.get("id")?),
            name: row.get("name")?,
            barrier: self.known.barriers.find_by(row, "barrier")?,
            device: self.known.devices.find_by(row, "device")?,
            input_offset: row.get_json("input_offset")?,
            input: self.known.containers.find_by(row, "input")?,
            output_offset: row.get_json("output_offset")?,
            output: self.known.containers.find_by(row, "output")?,
            kit: self.known.items.find_by(row, "kit")?,
            compost: self.known.items.find_by(row, "compost")?,
        };
        Ok(data)
    }

    pub(crate) fn load_composter(&mut self, row: &rusqlite::Row) -> Result<Composter, DataError> {
        let data = Composter {
            id: row.get("id")?,
            key: ComposterKey(row.get("kind")?),
            input: ContainerId(row.get("input")?),
            device: DeviceId(row.get("device")?),
            output: ContainerId(row.get("output")?),
            barrier: BarrierId(row.get("barrier")?),
            placement: PlacementId(row.get("placement")?),
        };
        Ok(data)
    }

    // physics

    pub(crate) fn load_space_kind(&mut self, row: &rusqlite::Row) -> Result<SpaceKind, DataError> {
        let data = SpaceKind {
            id: SpaceKey(row.get("id")?),
            name: row.get("name")?,
            bounds: row.get_json("bounds")?,
        };
        Ok(data)
    }

    pub(crate) fn load_space(&mut self, row: &rusqlite::Row) -> Result<Space, DataError> {
        let data = Space {
            id: SpaceId(row.get("id")?),
            kind: self.known.spaces.get_by(row, "kind", SpaceKey)?,
            holes: row.decode("holes")?,
        };
        Ok(data)
    }

    pub(crate) fn load_body_kind(&mut self, row: &rusqlite::Row) -> Result<BodyKind, DataError> {
        let data = BodyKind {
            id: BodyKey(row.get("id")?),
            name: row.get("name")?,
            speed: row.get("speed")?,
            radius: row.get("radius")?,
        };
        Ok(data)
    }

    pub(crate) fn load_body(&mut self, row: &rusqlite::Row) -> Result<Body, DataError> {
        let data = Body {
            id: BodyId(row.get("id")?),
            kind: self.known.bodies.get_by(row, "kind", BodyKey)?,
            position: row.get_json("position")?,
            destination: row.get_json("destination")?,
            space: SpaceId(row.get("space")?),
        };
        Ok(data)
    }

    pub(crate) fn load_barrier_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<BarrierKind, DataError> {
        let data = BarrierKind {
            id: BarrierKey(row.get("id")?),
            name: row.get("name")?,
            bounds: row.get_json("bounds")?,
        };
        Ok(data)
    }

    pub(crate) fn load_barrier(&mut self, row: &rusqlite::Row) -> Result<Barrier, DataError> {
        let data = Barrier {
            id: BarrierId(row.get("id")?),
            kind: self.known.barriers.get_by(row, "kind", BarrierKey)?,
            position: row.get_json("position")?,
            space: SpaceId(row.get("space")?),
            active: row.get("active")?,
        };
        Ok(data)
    }

    pub(crate) fn load_sensor_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<SensorKind, DataError> {
        let data = SensorKind {
            id: SensorKey(row.get("id")?),
            name: row.get("name")?,
            radius: row.get("radius")?,
        };
        Ok(data)
    }

    pub(crate) fn load_sensor(&mut self, row: &rusqlite::Row) -> Result<Sensor, DataError> {
        let data = Sensor {
            id: SensorId(row.get("id")?),
            kind: self.known.sensors.get_by(row, "kind", SensorKey)?,
            position: row.get_json("position")?,
            space: SpaceId(row.get("space")?),
            signals: row.get_json("signals")?,
            registered: Default::default(),
        };
        Ok(data)
    }

    // building

    pub(crate) fn load_grid_kind(&mut self, row: &rusqlite::Row) -> Result<GridKind, DataError> {
        let data = GridKind {
            id: GridKey(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_surveyor_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<SurveyorKind, DataError> {
        let data = SurveyorKind {
            id: SurveyorKey(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_grid(&mut self, row: &rusqlite::Row) -> Result<Grid, DataError> {
        let cells = row.decode("map")?;
        let rooms = Grid::calculate_rooms(&cells);
        let data = Grid {
            id: GridId(row.get("id")?),
            kind: self.known.grids.get_by(row, "kind", GridKey)?,
            cells,
            rooms,
        };
        Ok(data)
    }

    pub(crate) fn load_surveyor(&mut self, row: &rusqlite::Row) -> Result<Surveyor, DataError> {
        let data = Surveyor {
            id: SurveyorId(row.get("id")?),
            grid: GridId(row.get("grid")?),
            stake_id: 0,
            surveying: vec![],
            kind: self.known.surveyors.get_by(row, "kind", SurveyorKey)?,
            mode: 0,
        };
        Ok(data)
    }

    // inventory

    pub(crate) fn load_container_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<ContainerKind, DataError> {
        let data = ContainerKind {
            id: ContainerKey(row.get("id")?),
            name: row.get("name")?,
            capacity: row.get("capacity")?,
            filter: row.get_json("filter")?,
        };
        Ok(data)
    }

    pub(crate) fn load_container(&mut self, row: &rusqlite::Row) -> Result<Container, DataError> {
        let data = Container {
            id: ContainerId(row.get("id")?),
            kind: self.known.containers.get_by(row, "kind", ContainerKey)?,
            items: vec![],
        };
        Ok(data)
    }

    pub(crate) fn load_item_kind(&mut self, row: &rusqlite::Row) -> Result<ItemKind, DataError> {
        let data = ItemKind {
            id: ItemKey(row.get("id")?),
            name: row.get("name")?,
            stackable: row.get("stackable")?,
            max_quantity: row.get("max_quantity")?,
            functions: row.get_json("functions")?,
        };
        Ok(data)
    }

    pub(crate) fn load_item(&mut self, row: &rusqlite::Row) -> Result<Item, DataError> {
        let data = Item {
            id: ItemId(row.get("id")?),
            kind: self.known.items.get_by(row, "kind", ItemKey)?,
            container: ContainerId(row.get("container")?),
            quantity: row.get("quantity")?,
        };
        Ok(data)
    }

    // working

    pub(crate) fn load_device_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<DeviceKind, DataError> {
        let data = DeviceKind {
            id: DeviceKey(row.get("id")?),
            name: row.get("name")?,
            duration: row.get("duration")?,
            durability: row.get("durability")?,
        };
        Ok(data)
    }

    pub(crate) fn load_device(&mut self, row: &rusqlite::Row) -> Result<Device, DataError> {
        let data = Device {
            id: DeviceId(row.get("id")?),
            kind: self.known.devices.get_by(row, "kind", DeviceKey)?,
            enabled: row.get("enabled")?,
            progress: row.get("progress")?,
            input: row.get("input")?,
            output: row.get("output")?,
            deprecation: row.get("deprecation")?,
            broken: row.get("broken")?,
        };
        Ok(data)
    }

    // assembling

    pub(crate) fn load_placement(&mut self, row: &rusqlite::Row) -> Result<Placement, DataError> {
        let data = Placement {
            id: PlacementId(row.get("id")?),
            rotation: row.get_json("rotation")?,
            pivot: row.get_json("pivot")?,
            valid: row.get("valid")?,
        };
        Ok(data)
    }

    // landscaping

    pub(crate) fn load_land_kind(&mut self, row: &rusqlite::Row) -> Result<LandKind, DataError> {
        let data = LandKind {
            id: LandKey(row.get("id")?),
            name: row.get("name")?,
            width: row.get("width")?,
            height: row.get("height")?,
        };
        Ok(data)
    }

    pub(crate) fn load_land(&mut self, row: &rusqlite::Row) -> Result<Land, DataError> {
        let data = Land {
            id: LandId(row.get("id")?),
            kind: self.known.lands.get_by(row, "kind", LandKey)?,
            moisture: row.decode("moisture")?,
            moisture_capacity: row.decode("moisture_capacity")?,
            surface: row.decode("surface")?,
        };
        Ok(data)
    }

    // timing

    pub(crate) fn load_calendar_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<CalendarKind, DataError> {
        let data = CalendarKind {
            id: CalendarKey(row.get("id")?),
            name: row.get("name")?,
            day_duration: MinGameMinute(row.get("day_duration")?),
            seasons: row.get_json("seasons")?,
        };
        Ok(data)
    }

    pub(crate) fn load_calendar(&mut self, row: &rusqlite::Row) -> Result<Calendar, DataError> {
        let data = Calendar {
            id: CalendarId(row.get("id")?),
            kind: self.known.calendars.get_by(row, "key", CalendarKey)?,
            season: row.get("season")?,
            season_day: row.get("season_day")?,
            times_of_day: row.get("times_of_day")?,
        };
        Ok(data)
    }

    // planting

    pub(crate) fn load_soil_kind(&mut self, row: &rusqlite::Row) -> Result<SoilKind, DataError> {
        let data = SoilKind {
            id: SoilKey(row.get("id")?),
            name: row.get("name")?,
            width: row.get("width")?,
            height: row.get("height")?,
        };
        Ok(data)
    }

    pub(crate) fn load_soil(&mut self, row: &rusqlite::Row) -> Result<Soil, DataError> {
        let data = Soil {
            id: SoilId(row.get("id")?),
            kind: self.known.soils.get_by(row, "kind", SoilKey)?,
            fertility: row.decode("fertility")?,
        };
        Ok(data)
    }

    pub(crate) fn load_plant_kind(&mut self, row: &rusqlite::Row) -> Result<PlantKind, DataError> {
        let data = PlantKind {
            id: PlantKey(row.get("id")?),
            name: row.get("name")?,
            growth: row.get("growth")?,
            flexibility: row.get("flexibility")?,
            transpiration: row.get("transpiration")?,
            max_fruits: row.get("max_fruits")?,
        };
        Ok(data)
    }

    pub(crate) fn load_plant(&mut self, row: &rusqlite::Row) -> Result<Plant, DataError> {
        let data = Plant {
            id: PlantId(row.get("id")?),
            kind: self.known.plants.get_by(row, "kind", PlantKey)?,
            soil: SoilId(row.get("soil")?),
            impact: row.get("impact")?,
            thirst: row.get("thirst")?,
            hunger: row.get("hunger")?,
            health: row.get("health")?,
            growth: row.get("growth")?,
            fruits: row.get("fruits")?,
        };
        Ok(data)
    }

    pub(crate) fn load_animal_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<AnimalKind, DataError> {
        let data = AnimalKind {
            id: AnimalKey(row.get("id")?),
            name: row.get("name")?,
            hunger_speed: row.get("hunger_speed")?,
            thirst_speed: row.get("thirst_speed")?,
            hunger_damage: row.get("hunger_damage")?,
            thirst_damage: row.get("thirst_damage")?,
            death_threshold: row.get("death_threshold")?,
            voracity: row.get("voracity")?,
        };
        Ok(data)
    }

    pub(crate) fn load_animal(&mut self, row: &rusqlite::Row) -> Result<Animal, DataError> {
        let data = Animal {
            id: AnimalId(row.get("id")?),
            kind: self.known.animals.get_by(row, "kind", AnimalKey)?,
            age: row.get("age")?,
            weight: 0.0,
            thirst: row.get("thirst")?,
            hunger: row.get("hunger")?,
            voracity: row.get("voracity")?,
            health: row.get("health")?,
            stress: row.get("stress")?,
            behaviour: row.get_json("behaviour")?,
        };
        Ok(data)
    }

    pub(crate) fn load_tether(&mut self, row: &rusqlite::Row) -> Result<Tether, DataError> {
        let data = Tether {
            id: row.get_json("id")?,
            animal: row.get_json("animal")?,
        };
        Ok(data)
    }
}

#[derive(Debug)]
pub enum DataError {
    Json(serde_json::Error),
    Sql(rusqlite::Error),
    Bincode(bincode::error::DecodeError),
    Inconsistency(DictionaryError),
    NotSpecifiedVariant,
}

impl From<bincode::error::DecodeError> for DataError {
    fn from(error: bincode::error::DecodeError) -> Self {
        Self::Bincode(error)
    }
}

impl From<DictionaryError> for DataError {
    fn from(error: DictionaryError) -> Self {
        Self::Inconsistency(error)
    }
}

impl From<serde_json::Error> for DataError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<rusqlite::Error> for DataError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

pub trait JsonDeserializer {
    fn get_json<'a, T>(&self, index: &str) -> Result<T, DataError>
    where
        T: de::DeserializeOwned;
}

impl<'stmt> JsonDeserializer for rusqlite::Row<'stmt> {
    fn get_json<'a, T>(&self, index: &str) -> Result<T, DataError>
    where
        T: de::DeserializeOwned,
    {
        let value: String = self.get(index)?;
        let value = serde_json::from_str(&value)?;
        Ok(value)
    }
}

pub trait BincodeDeserializer {
    fn decode<T>(&self, index: &str) -> Result<T, DataError>
    where
        T: bincode::Decode;
}

impl<'stmt> BincodeDeserializer for rusqlite::Row<'stmt> {
    fn decode<T>(&self, index: &str) -> Result<T, DataError>
    where
        T: bincode::Decode,
    {
        let data: Vec<u8> = self.get(index)?;
        let config = bincode::config::standard();
        let (value, _) = bincode::decode_from_slice(&data, config)?;
        Ok(value)
    }
}
