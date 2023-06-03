#![allow(improper_ctypes_definitions)]

use std::ffi::CString;
use std::fs;
use std::mem::take;

use datamap::Storage;
use game::api::{ActionError, Event};
use game::model::{Creature, CreatureKey, Farmer, Farmland, Universe};
use game::physics::BodyId;
use game::raising::AnimalId;
use game::Game;

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
pub unsafe extern "C" fn perform_action(scenario: &mut Scenario, data: PyString) {
    let action = serde_json::from_str(data.to_str()).unwrap();
    match scenario.game.perform_action("Alice", action) {
        Ok(events) => {
            scenario.events.extend(events);
        }
        Err(error) => {
            scenario.errors = Some(error);
        }
    }
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
