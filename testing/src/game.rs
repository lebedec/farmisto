#![allow(improper_ctypes_definitions)]

use std::ffi::CString;
use std::fs;
use std::mem::take;

use datamap::Storage;
use game::api::{ActionError, Event};
use game::building::{Grid, GridId, Material, Stake, Structure, SurveyorId};
use game::inventory::{ContainerId, ItemId};
use game::math::VectorMath;
use game::model::{
    Construction, Creature, CreatureKey, Crop, Farmer, Farmland, Theodolite, Universe,
};
use game::physics::BodyId;
use game::raising::AnimalId;
use game::{occur, Game};

use crate::ffi::{PyString, PyStringToString, PyTuple, PyTupleToSlice};

pub struct Scenario {
    pub game: Game,
    pub events: Vec<Event>,
    pub errors: Option<ActionError>,
}

#[no_mangle]
pub unsafe extern "C" fn create(database: PyString) -> *mut Scenario {
    let storage = Storage::open(database.to_str()).unwrap();
    let mut game = Game::new(storage);
    game.load_game_knowledge().unwrap();
    let scenario = Scenario {
        game,
        events: vec![],
        errors: None,
    };
    Box::into_raw(Box::new(scenario))
}

#[no_mangle]
pub unsafe extern "C" fn perform_action(scenario: &mut Scenario, player: PyString, data: PyString) {
    let action = serde_json::from_str(data.to_str()).unwrap();
    match scenario.game.perform_action(player.to_str(), action) {
        Ok(events) => {
            scenario.events.extend(events);
        }
        Err(error) => {
            scenario.errors = Some(error);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn update(scenario: &mut Scenario, time: f32) {
    scenario.game.update(time);
}

#[no_mangle]
pub unsafe extern "C" fn add_farmland(scenario: &mut Scenario, kind: PyString) -> Farmland {
    let events = scenario.game.create_farmland(kind.to_str()).unwrap();
    for event in events {
        if let Event::UniverseStream(events) = event {
            for event in events {
                if let Universe::FarmlandAppeared { farmland, .. } = event {
                    return farmland;
                }
            }
        }
    }
    panic!("Unable to add farmland, event with farmland id not found");
}

#[no_mangle]
pub unsafe extern "C" fn add_farmer(
    scenario: &mut Scenario,
    name: PyString,
    kind: PyString,
    farmland: Farmland,
    position: PyTuple,
) -> Farmer {
    let events = scenario
        .game
        .create_farmer(name.to_str(), kind.to_str(), farmland, position.to_slice())
        .unwrap();
    for event in events {
        if let Event::UniverseStream(events) = event {
            for event in events {
                if let Universe::FarmerAppeared { farmer, .. } = event {
                    return farmer;
                }
            }
        }
    }
    panic!("Unable to add farmer, event with farmer id not found");
}

#[no_mangle]
pub unsafe extern "C" fn set_farmer_activity(
    scenario: &mut Scenario,
    farmer: Farmer,
    activity: PyString,
) {
    let activity = serde_json::from_str(activity.to_str()).expect("failed activity");
    scenario.game.universe.change_activity(farmer, activity);
}

#[no_mangle]
pub unsafe extern "C" fn add_theodolite(
    scenario: &mut Scenario,
    kind: PyString,
    farmland: Farmland,
    position: PyTuple,
) -> Theodolite {
    let kind = scenario.game.known.theodolites.find(kind.to_str()).unwrap();
    let events = scenario
        .game
        .create_theodolite(kind.id, farmland, position.to_slice().to_tile())
        .unwrap();
    for event in events {
        if let Event::UniverseStream(events) = event {
            for event in events {
                if let Universe::TheodoliteAppeared { id, .. } = event {
                    return id;
                }
            }
        }
    }
    panic!("Unable to add theodolite, event with theodolite id not found");
}

#[no_mangle]
pub unsafe extern "C" fn add_item(scenario: &mut Scenario, kind: PyString, container: ContainerId) {
    let kind = scenario
        .game
        .known
        .items
        .find(kind.to_str())
        .expect("failed kind");
    let id = scenario.game.inventory.items_id.introduce().one(ItemId);
    let create_item = scenario
        .game
        .inventory
        .create_item(id, &kind, container, 1)
        .expect("failed created_item");
    create_item();
}

#[no_mangle]
pub unsafe extern "C" fn add_crop(
    scenario: &mut Scenario,
    kind: PyString,
    farmland: Farmland,
    position: PyTuple,
) -> Crop {
    let kind = scenario
        .game
        .known
        .crops
        .find(kind.to_str())
        .expect("failed kind");
    let (barrier, sensor, create_barrier_sensor) = scenario
        .game
        .physics
        .create_barrier_sensor(
            farmland.space,
            &kind.barrier,
            &kind.sensor,
            position.to_slice(),
            false,
        )
        .expect("failed physics");
    let (plant, create_plant) = scenario
        .game
        .planting
        .create_plant(farmland.soil, &kind.plant, 0.0)
        .expect("failed planting");
    let events = occur![
        create_barrier_sensor(),
        create_plant(),
        scenario
            .game
            .appear_crop(kind.id, barrier, sensor, plant)
            .expect("appear"),
    ];
    for event in events {
        if let Event::UniverseStream(events) = event {
            for event in events {
                if let Universe::CropAppeared { entity, .. } = event {
                    return entity;
                }
            }
        }
    }
    panic!("Unable to add crop, event with crop id not found");
}

#[no_mangle]
pub unsafe extern "C" fn add_construction(
    scenario: &mut Scenario,
    surveyor: SurveyorId,
    marker: PyString,
    grid: GridId,
    position: PyTuple,
) -> Construction {
    let cell = position.to_slice().to_tile();
    let marker = serde_json::from_str(marker.to_str()).expect("failed marker");
    let (stake, survey) = scenario
        .game
        .building
        .survey(surveyor, marker, cell)
        .expect("failed survey");
    let container_kind = scenario
        .game
        .known
        .containers
        .find("<construction>")
        .unwrap();
    let container = scenario
        .game
        .inventory
        .containers_id
        .introduce()
        .one(ContainerId);
    let create_container = scenario
        .game
        .inventory
        .add_empty_container(container, &container_kind)
        .unwrap();
    let events = occur![
        survey(),
        create_container(),
        scenario
            .game
            .appear_construction(container, grid, surveyor, stake)
            .expect("failed appear"),
    ];
    for event in events {
        if let Event::UniverseStream(events) = event {
            for event in events {
                if let Universe::ConstructionAppeared { id, .. } = event {
                    return id;
                }
            }
        }
    }
    panic!("Unable to add construction, event with construction id not found");
}

#[no_mangle]
pub unsafe extern "C" fn get_grid(scenario: &mut Scenario, id: GridId) -> PyString {
    let grid = scenario
        .game
        .building
        .get_grid(id)
        .expect("failed get grid");
    let data = serde_json::to_string(&grid.rooms).expect("failed json");
    CString::new(data).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn get_constructions(
    scenario: &mut Scenario,
    _farmland: Farmland,
) -> PyString {
    let data = serde_json::to_string(&scenario.game.universe.constructions).expect("failed json");
    CString::new(data).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn rebuild_grid(scenario: &mut Scenario, farmland: Farmland) {
    let grid = scenario
        .game
        .building
        .get_mut_grid(farmland.grid)
        .expect("failed get grid");
    grid.rooms = Grid::calculate_rooms(&grid.cells);
}

#[no_mangle]
pub unsafe extern "C" fn add_building(
    scenario: &mut Scenario,
    farmland: Farmland,
    position: PyTuple,
    material: Material,
    structure: PyString,
) {
    let tile = position.to_tile();
    let grid = scenario
        .game
        .building
        .get_mut_grid(farmland.grid)
        .expect("failed get grid");
    let [x, y] = tile;
    let structure: Structure = serde_json::from_str(structure.to_str()).expect("structure");
    let cell = &mut grid.cells[y][x];
    cell.material = material;
    match structure {
        Structure::Wall => {
            cell.wall = true;
        }
        Structure::Window => {
            cell.wall = true;
            cell.window = true;
        }
        Structure::Door => {
            cell.wall = true;
            cell.door = true;
        }
        Structure::Fence => {
            cell.wall = true;
        }
    }
    let size = if material.index() == Material::PLANKS {
        2
    } else {
        1
    };
    let create_hole = scenario
        .game
        .physics
        .create_hole(farmland.space, tile, size)
        .expect("failed create hole");
    if structure != Structure::Door {
        create_hole();
    }
}

#[no_mangle]
pub unsafe extern "C" fn take_events(scenario: &mut Scenario) -> PyString {
    let events = take(&mut scenario.events);
    let data = serde_json::to_string(&events).unwrap();
    CString::new(data).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn take_errors(scenario: &mut Scenario) -> PyString {
    let errors = take(&mut scenario.errors);
    let data = serde_json::to_string(&errors).unwrap();
    CString::new(data).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn dispose(scenario: *mut Scenario) {
    drop(Box::from_raw(scenario));
}
