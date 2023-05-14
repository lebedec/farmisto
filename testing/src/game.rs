use crate::ffi::{PyString, PyStringToString};
use datamap::Storage;
use game::api::{ActionError, Event};
use game::Game;
use std::ffi::CString;
use std::fs;
use std::mem::take;

pub struct Scenario {
    pub game: Game,
    pub events: Vec<Event>,
    pub errors: Option<ActionError>,
}

#[no_mangle]
pub unsafe extern "C" fn perform_action(scenario: &mut Scenario, data: PyString) {
    let action = serde_json::from_str(data.to_str()).unwrap();
    println!("ACTION!!!: {action:?}");
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
pub unsafe extern "C" fn create(database: PyString) -> *mut Scenario {
    println!("LS ./");
    let paths = fs::read_dir("./").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    println!("LS ../");
    let paths = fs::read_dir("../").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    println!("LS ../assets");
    let paths = fs::read_dir("../assets").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    let storage = Storage::open(database.to_str()).unwrap();
    let mut game = Game::new(storage);
    game.load_game_knowledge().unwrap();
    println!("KNOWN SPACES: {}", game.known.spaces.len());
    let scenario = Scenario {
        game,
        events: vec![],
        errors: None,
    };
    Box::into_raw(Box::new(scenario))
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
