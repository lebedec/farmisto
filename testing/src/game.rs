use crate::ffi::{PyString, PyStringToString};
use datamap::Storage;
use game::api::Event;
use game::Game;
use std::ffi::CString;
use std::mem::take;

pub struct Scenario {
    data: String,
    pub game: Game,
    pub events: Vec<Event>,
}

#[no_mangle]
pub unsafe extern "C" fn change_data(scenario: &mut Scenario, data: PyString) {
    scenario.data = data.to_string();
}

#[no_mangle]
pub unsafe extern "C" fn perform_action(scenario: &mut Scenario, data: PyString) {
    scenario.data = data.to_string();
}

#[no_mangle]
pub unsafe extern "C" fn create(database: PyString) -> *mut Scenario {
    let storage = Storage::open(database.to_str()).unwrap();
    let mut game = Game::new(storage);
    game.load_game_knowledge().unwrap();
    println!("KNOWN SPACES: {}", game.known.spaces.len());
    let scenario = Scenario {
        data: String::from("<empty>"),
        game,
        events: vec![],
    };
    Box::into_raw(Box::new(scenario))
}

#[no_mangle]
pub unsafe extern "C" fn take_events(scenario: &mut Scenario) -> PyString {
    println!("hello");
    let events = take(&mut scenario.events);
    let data = format!("{:?}", events);
    println!("EVENTS: {data}");

    CString::new(data).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn dispose(scenario: *mut Scenario) {
    drop(Box::from_raw(scenario));
}
