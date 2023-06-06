#![allow(improper_ctypes_definitions)]

use std::ffi::CString;

use serde_json::json;

use game::physics::{Barrier, BarrierId, BodyId, Space, SpaceId};

use crate::ffi::{PyString, PyStringToString, PyTuple, PyTupleToSlice};
use crate::game::Scenario;

#[no_mangle]
pub unsafe extern "C" fn add_space(scenario: &mut Scenario, kind: PyString) -> SpaceId {
    println!("add_space {}", kind.to_str());
    let physics = &mut scenario.game.physics;
    let kind = scenario.game.known.spaces.find(kind.to_str()).unwrap();
    let id = SpaceId(physics.spaces_sequence + 1);
    let space = Space {
        id,
        kind,
        holes: vec![],
    };
    physics.load_spaces(vec![space], id.0);
    id
}

#[no_mangle]
pub unsafe extern "C" fn set_body_position(
    scenario: &mut Scenario,
    body: BodyId,
    position: PyTuple,
) {
    let body = scenario.game.physics.get_body_mut(body).unwrap();
    body.position = position.to_slice();
    body.destination = position.to_slice();
}

#[no_mangle]
pub unsafe extern "C" fn add_barrier(
    scenario: &mut Scenario,
    kind: PyString,
    space: usize,
    position: PyTuple,
    active: bool,
) -> BarrierId {
    println!(
        "add_barrier {}, {space:?} {active} {:?}",
        kind.to_str(),
        position.to_slice()
    );
    let physics = &mut scenario.game.physics;
    let kind = scenario.game.known.barriers.find(kind.to_str()).unwrap();
    let id = BarrierId(physics.barriers_sequence + 1);
    let barrier = Barrier {
        id,
        kind,
        space: space.into(),
        position: position.to_slice(),
        active,
    };
    physics.load_barriers(vec![barrier], id.0);
    id
}

#[no_mangle]
pub unsafe extern "C" fn change_barrier(scenario: &mut Scenario, id: BarrierId, active: bool) {
    let physics = &mut scenario.game.physics;
    let command = physics.change_barrier(id, active).unwrap();
    let events = command();
    scenario.events.push(events.into());
}

#[no_mangle]
pub unsafe extern "C" fn get_barrier(scenario: &mut Scenario, id: BarrierId) -> PyString {
    println!("get_barrier {id:?}");
    let physics = &mut scenario.game.physics;
    let barrier = physics.get_barrier(id).unwrap();
    let repr = json!({
        "kind": barrier.kind.name,
        "position": barrier.position,
        "active": barrier.active
    });
    let s: String = serde_json::to_string(&repr).unwrap();
    println!("s {s}");
    CString::new(s).unwrap().into_raw()
}
