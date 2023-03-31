use core::fmt::Debug;

use log::info;
use serde::de;

use crate::building::{
    Grid, GridId, GridKey, GridKind, Surveyor, SurveyorId, SurveyorKey, SurveyorKind,
};
use crate::collections::{DictionaryError, Shared};
use crate::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, Item, ItemId, ItemKey, ItemKind,
};
use crate::model::{
    Construction, Creature, CreatureKey, CreatureKind, Crop, CropKey, CropKind, Equipment,
    EquipmentKey, EquipmentKind, Farmer, FarmerKey, FarmerKind, Farmland, FarmlandKey,
    FarmlandKind, Player, PlayerId, Purpose, PurposeDescription, Stack, Tree, TreeKey, TreeKind,
};
use crate::physics::{
    Barrier, BarrierId, BarrierKey, BarrierKind, Body, BodyId, BodyKey, BodyKind, Sensor, SensorId,
    SensorKey, SensorKind, Space, SpaceId, SpaceKey, SpaceKind,
};
use crate::planting::{Plant, PlantId, PlantKey, PlantKind, Soil, SoilId, SoilKey, SoilKind};
use crate::raising::{Animal, AnimalId, AnimalKey, AnimalKind};
use crate::Game;

impl Game {
    pub fn load_game_knowledge(&mut self) {
        info!("Begin game knowledge loading from ...");
        let storage = self.storage.open_into();
        // physics
        for kind in storage.find_all(|row| self.load_space_kind(row).unwrap()) {
            self.known.spaces.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_body_kind(row).unwrap()) {
            self.known.bodies.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_barrier_kind(row).unwrap()) {
            self.known.barriers.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_sensor_kind(row).unwrap()) {
            self.known.sensors.insert(kind.id, kind.name.clone(), kind);
        }
        // planting
        for kind in storage.find_all(|row| self.load_land_kind(row).unwrap()) {
            self.known.soils.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_plant_kind(row).unwrap()) {
            self.known.plants.insert(kind.id, kind.name.clone(), kind);
        }
        // raising
        for kind in storage.find_all(|row| self.load_animal_kind(row).unwrap()) {
            self.known.animals.insert(kind.id, kind.name.clone(), kind);
        }
        // building
        for kind in storage.find_all(|row| self.load_grid_kind(row).unwrap()) {
            self.known.grids.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_surveyor_kind(row).unwrap()) {
            self.known
                .surveyors
                .insert(kind.id, kind.name.clone(), kind);
        }
        // inventory
        for kind in storage.find_all(|row| self.load_container_kind(row).unwrap()) {
            self.known
                .containers
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_item_kind(row).unwrap()) {
            self.known.items.insert(kind.id, kind.name.clone(), kind);
        }
        // universe
        for kind in storage.find_all(|row| self.load_tree_kind(row).unwrap()) {
            self.known.trees.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_farmland_kind(row).unwrap()) {
            self.known
                .farmlands
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_farmer_kind(row).unwrap()) {
            self.known.farmers.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_equipment_kind(row).unwrap()) {
            self.known
                .equipments
                .insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_crop_kind(row).unwrap()) {
            self.known.crops.insert(kind.id, kind.name.clone(), kind);
        }
        for kind in storage.find_all(|row| self.load_creature_kind(row).unwrap()) {
            self.known
                .creatures
                .insert(kind.id, kind.name.clone(), kind);
        }
        info!("End game knowledge loading");
    }

    pub fn load_game_state(&mut self) -> Result<(), DataError> {
        info!("Begin game state loading from ...");
        let storage = self.storage.open_into();
        self.players = storage.find_all(|row| self.load_player(row).unwrap());

        // physics
        let (spaces, sequence) = storage.get_sequence(|row| self.load_space(row))?;
        self.physics.load_spaces(spaces, sequence);
        let (bodies, sequence) = storage.get_sequence(|row| self.load_body(row))?;
        self.physics.load_bodies(bodies, sequence);
        let (barriers, sequence) = storage.get_sequence(|row| self.load_barrier(row))?;
        self.physics.load_barriers(barriers, sequence);
        let (sensors, sequence) = storage.get_sequence(|row| self.load_sensor(row))?;
        self.physics.load_sensors(sensors, sequence);

        // planting
        let (lands, sequence) = storage.get_sequence(|row| self.load_land(row))?;
        self.planting.load_soils(lands, sequence);
        let (plants, sequence) = storage.get_sequence(|row| self.load_plant(row))?;
        self.planting.load_plants(plants, sequence);

        // raising
        let (animals, sequence) = storage.get_sequence(|row| self.load_animal(row))?;
        self.raising.load_animals(animals, sequence);

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

        // models
        let (trees, trees_id) = storage.get_sequence(|row| self.load_tree(row))?;
        self.universe.load_trees(trees, trees_id);
        let (farmlands, farmlands_id) = storage.get_sequence(|row| self.load_farmland(row))?;
        self.universe.load_farmlands(farmlands, farmlands_id);
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
        let (creatures, id) = storage.get_sequence(|row| self.load_creature(row))?;
        self.universe.load_creatures(creatures, id);
        info!("End game state loading");

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
        let purpose = if let Ok(Some(kind)) = row.get::<_, Option<String>>("p_surveyor") {
            let surveyor = self.known.surveyors.find(&kind)?.id;
            PurposeDescription::Surveying { surveyor }
        } else {
            PurposeDescription::Moisture { sensor: 0 }
        };
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
        let purpose = if let Ok(Some(id)) = row.get("p_surveyor") {
            Purpose::Surveying {
                surveyor: SurveyorId(id),
            }
        } else {
            Purpose::Moisture { sensor: 0 }
        };
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
            space: SpaceKey(row.get("space")?),
            soil: SoilKey(row.get("soil")?),
            grid: GridKey(row.get("grid")?),
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
            marker: row.get_json("marker")?,
            cell: row.get_json("cell")?,
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
            surveying: vec![],
            kind: self.known.surveyors.get_by(row, "kind", SurveyorKey)?,
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

    // planting

    pub(crate) fn load_land_kind(&mut self, row: &rusqlite::Row) -> Result<SoilKind, DataError> {
        let data = SoilKind {
            id: SoilKey(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_land(&mut self, row: &rusqlite::Row) -> Result<Soil, DataError> {
        let data = Soil {
            id: SoilId(row.get("id")?),
            kind: self.known.soils.get_by(row, "kind", SoilKey)?,
            map: row.decode("map")?,
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
        };
        Ok(data)
    }

    pub(crate) fn load_animal(&mut self, row: &rusqlite::Row) -> Result<Animal, DataError> {
        let data = Animal {
            id: AnimalId(row.get("id")?),
            kind: self.known.animals.get_by(row, "kind", AnimalKey)?,
            age: row.get("age")?,
            thirst: row.get("thirst")?,
            hunger: row.get("hunger")?,
            health: row.get("health")?,
            stress: row.get("stress")?,
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
