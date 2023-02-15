use crate::testing::PhysicsTestScenario;
use game::physics::Physics::{BarrierCreated, BodyPositionChanged};
use game::physics::{BarrierId, PhysicsError};

mod testing;

const DEFAULT_SPACE_BOUNDS: [f32; 2] = [128.0, 128.0];
const SMALL_SPACE: [f32; 2] = [10.0, 10.0];
const NORMAL_TICK: f32 = 0.02;


#[test]
fn test_create_barrier_in_connection_with_others() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "top", "land", [1.5, 1.5])
        .given_barrier("crate", "bottom", "land", [1.5, 3.5])
        .when_create_barrier("crate", "center", "land", [1.5, 2.5])
        .then_events(
            |given| {
                vec![BarrierCreated {
                    id: given.barrier("center"),
                    space: given.space("land"),
                    position: [1.5, 2.5],
                    bounds: [1.0, 1.0],
                }]
            },
            |scenario| scenario.debug_space("land").present(),
        );
}

#[test]
fn test_create_barrier_overlaps_other() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "other", "land", [1.5, 1.5])
        .when_create_barrier("crate", "crate", "land", [1.5, 1.5])
        .then_error(|given| PhysicsError::BarrierCreationOverlaps {
            other: given.barrier("other"),
        });
}

#[test]
fn test_move_into_barriers_connection() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [1.5, 3.4])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.5])
        .given_barrier("crate", "b", "land", [2.5, 3.5])
        .when_move_body("Alice", [3.5, 3.4])
        .when_update(10, NORMAL_TICK)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.55, 3.0],
                }]
            },
            |scenario| {
                scenario
                    .debug_space("land")
                    .debug_body_movement("Alice", true)
                    .present()
            },
        );
}

#[test]
fn test_move_into_barrier() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", SMALL_SPACE)
        .given_space("default", "land")
        .given_body_kind("farmer", 1.0, 0.5)
        .given_body("farmer", "Alice", "land", [2.2, 1.5])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.5])
        .when_move_body("Alice", [1.7, 2.5])
        .when_update(4, 0.25)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.55, 3.0],
                }]
            },
            |scenario| {
                scenario
                    .debug_space("land")
                    .debug_body_movement("Alice", true)
                    .present()
            },
        );
}

#[test]
fn test_move_into_gap_between_barriers() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [1.4, 3.0])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.25])
        .given_barrier("crate", "b", "land", [2.5, 3.75])
        .when_move_body("Alice", [3.5, 3.0])
        .when_update(10, 0.5)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.551, 3.0],
                }]
            },
            |scenario| {
                scenario
                    .debug_space("land")
                    .debug_body_movement("Alice", true)
                    .present()
            },
        );
}
