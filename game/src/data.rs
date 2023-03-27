use log::info;

use crate::building::{
    Grid, GridId, GridKey, GridKind, Marker, Surveyor, SurveyorId, SurveyorKey, SurveyorKind,
};
use crate::collections::Shared;
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
        let id = row.get("id")?;
        let barrier: String = row.get("barrier")?;
        let purpose = if let Ok(Some(kind)) = row.get::<_, Option<String>>("p_surveyor") {
            let surveyor = self.known.surveyors.find(&kind).unwrap().id;
            PurposeDescription::Surveying { surveyor }
        } else {
            PurposeDescription::Moisture { sensor: 0 }
        };
        let data = EquipmentKind {
            id: EquipmentKey(id),
            name: row.get("name")?,
            purpose,
            barrier: self.known.barriers.find(&barrier).unwrap().id,
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
            kind: EquipmentKey(kind),
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
        let id = row.get("id")?;
        let plant: String = row.get("plant")?;
        let barrier: String = row.get("barrier")?;
        let sensor: String = row.get("sensor")?;
        let data = CropKind {
            id: CropKey(id),
            name: row.get("name")?,
            plant: self.known.plants.find(&plant).unwrap().id,
            barrier: self.known.barriers.find(&barrier).unwrap().id,
            sensor: self.known.sensors.find(&sensor).unwrap().id,
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
        let body: String = row.get("body")?;
        let animal: String = row.get("animal")?;
        let data = CreatureKind {
            id: CreatureKey(row.get("id")?),
            name: row.get("name")?,
            body: self.known.bodies.find(&body).unwrap().id,
            animal: self.known.animals.find(&animal).unwrap().id,
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
        let id = row.get("id")?;
        let barrier = row.get("barrier")?;
        let plant = row.get("plant")?;
        let data = TreeKind {
            id: TreeKey(id),
            name: row.get("name")?,
            barrier: BarrierKey(barrier),
            plant: PlantKey(plant),
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
        let cell: String = row.get("cell")?;
        let marker: String = row.get("marker")?;
        let data = Construction {
            id: row.get("id")?,
            container: ContainerId(row.get("container")?),
            grid: GridId(row.get("grid")?),
            surveyor: SurveyorId(row.get("surveyor")?),
            marker: serde_json::from_str(&marker)?,
            cell: serde_json::from_str(&cell)?,
        };
        Ok(data)
    }

    // physics

    pub(crate) fn load_space_kind(&mut self, row: &rusqlite::Row) -> Result<SpaceKind, DataError> {
        let bounds: String = row.get("bounds")?;
        let data = SpaceKind {
            id: SpaceKey(row.get("id")?),
            name: row.get("name")?,
            bounds: serde_json::from_str(&bounds)?,
        };
        Ok(data)
    }

    pub(crate) fn load_space(&mut self, row: &rusqlite::Row) -> Result<Space, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let holes: Vec<u8> = row.get("holes")?;
        let config = bincode::config::standard();
        let (holes, _) = bincode::decode_from_slice(&holes, config).unwrap();
        let data = Space {
            id: SpaceId(id),
            kind: self.known.spaces.get(SpaceKey(kind)).unwrap(),
            holes,
        };
        Ok(data)
    }

    pub(crate) fn load_body_kind(&mut self, row: &rusqlite::Row) -> Result<BodyKind, DataError> {
        let id = row.get("id")?;
        let data = BodyKind {
            id: BodyKey(id),
            name: row.get("name")?,
            speed: row.get("speed")?,
            radius: row.get("radius")?,
        };
        Ok(data)
    }

    pub(crate) fn load_body(&mut self, row: &rusqlite::Row) -> Result<Body, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let space = row.get("space")?;
        let position: String = row.get("position")?;
        let destination: String = row.get("destination")?;
        let data = Body {
            id: BodyId(id),
            kind: self.known.bodies.get(BodyKey(kind)).unwrap(),
            position: serde_json::from_str(&position)?,
            destination: serde_json::from_str(&destination)?,
            space: SpaceId(space),
        };
        Ok(data)
    }

    pub(crate) fn load_barrier_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<BarrierKind, DataError> {
        let id = row.get("id")?;
        let bounds: String = row.get("bounds")?;
        let data = BarrierKind {
            id: BarrierKey(id),
            name: row.get("name")?,
            bounds: serde_json::from_str(&bounds)?,
        };
        Ok(data)
    }

    pub(crate) fn load_barrier(&mut self, row: &rusqlite::Row) -> Result<Barrier, DataError> {
        let id = row.get("id")?;
        let key = BarrierKey(row.get("kind")?);
        let space = row.get("space")?;
        let position: String = row.get("position")?;
        let data = Barrier {
            id: BarrierId(id),
            kind: self.known.barriers.get(key).unwrap(),
            position: serde_json::from_str(&position)?,
            space: SpaceId(space),
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
        let key = SensorKey(row.get("kind")?);
        let space = row.get("space")?;
        let position: String = row.get("position")?;
        let signals: String = row.get("signals")?;
        let data = Sensor {
            id: SensorId(row.get("id")?),
            kind: self.known.sensors.get(key).unwrap(),
            position: serde_json::from_str(&position)?,
            space: SpaceId(space),
            signals: serde_json::from_str(&signals)?,
        };
        Ok(data)
    }

    // building

    pub(crate) fn load_grid_kind(&mut self, row: &rusqlite::Row) -> Result<GridKind, DataError> {
        let id = row.get("id")?;
        let materials: String = row.get("materials")?;
        let data = GridKind {
            id: GridKey(id),
            name: row.get("name")?,
            materials: serde_json::from_str(&materials)?,
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
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let config = bincode::config::standard();
        let (cells, _) = bincode::decode_from_slice(&map, config).unwrap();
        let rooms = Grid::calculate_rooms(&cells);
        let data = Grid {
            id: GridId(id),
            kind: self.known.grids.get(GridKey(kind)).unwrap(),
            cells,
            rooms,
        };
        Ok(data)
    }

    pub(crate) fn load_surveyor(&mut self, row: &rusqlite::Row) -> Result<Surveyor, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let grid = row.get("grid")?;
        let data = Surveyor {
            id: SurveyorId(id),
            grid: GridId(grid),
            surveying: vec![],
            kind: self.known.surveyors.get(SurveyorKey(kind)).unwrap(),
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
        let key = row.get("kind")?;
        let data = Container {
            id: ContainerId(row.get("id")?),
            kind: self.known.containers.get(ContainerKey(key)).unwrap(),
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
        };
        Ok(data)
    }

    pub(crate) fn load_item(&mut self, row: &rusqlite::Row) -> Result<Item, DataError> {
        let functions: String = row.get("functions")?;
        let key = row.get("kind")?;
        let data = Item {
            id: ItemId(row.get("id")?),
            kind: self.known.items.get(ItemKey(key)).unwrap(),
            container: ContainerId(row.get("container")?),
            functions: serde_json::from_str(&functions)?,
            quantity: row.get("quantity")?,
        };
        Ok(data)
    }

    // planting

    pub(crate) fn load_land_kind(&mut self, row: &rusqlite::Row) -> Result<SoilKind, DataError> {
        let id = row.get("id")?;
        let data = SoilKind {
            id: SoilKey(id),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_land(&mut self, row: &rusqlite::Row) -> Result<Soil, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let config = bincode::config::standard();
        let (map, _) = bincode::decode_from_slice(&map, config).unwrap();
        let data = Soil {
            id: SoilId(id),
            kind: self.known.soils.get(SoilKey(kind)).unwrap(),
            map,
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
        let kind = row.get("kind")?;
        let data = Plant {
            id: PlantId(row.get("id")?),
            kind: self.known.plants.get(PlantKey(kind)).unwrap(),
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
        let kind = row.get("kind")?;
        let data = Animal {
            id: AnimalId(row.get("id")?),
            kind: self.known.animals.get(AnimalKey(kind)).unwrap(),
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
    Consistency,
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
