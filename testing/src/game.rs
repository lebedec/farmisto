use crate::ffi::{PyString, PyStringToString};

pub struct GameTestScenario {
    data: String,
}

#[no_mangle]
pub unsafe extern "C" fn change_data(scenario: &mut GameTestScenario, data: PyString) {
    scenario.data = data.to_string();
}

#[no_mangle]
pub unsafe extern "C" fn create() -> *mut GameTestScenario {
    let scenario = GameTestScenario {
        data: String::from("<empty>"),
    };
    Box::into_raw(Box::new(scenario))
}

#[no_mangle]
pub unsafe extern "C" fn dispose(scenario: *mut GameTestScenario) {
    drop(Box::from_raw(scenario));
}
