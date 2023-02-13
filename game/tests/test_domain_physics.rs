use crate::testing::PhysicsTestScenario;
use game::physics::Physics::BodyPositionChanged;

mod testing;

const DEFAULT_SPACE_BOUNDS: [f32; 2] = [128.0, 128.0];

#[test]
fn test_something() {
    PhysicsTestScenario::new("test_something")
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [0.5, 1.5])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "1", "land", [0.5, 0.5])
        .given_barrier("crate", "2", "land", [1.5, 1.5])
        .given_barrier("crate", "3", "land", [2.5, 2.5])
        .given_barrier("crate", "4", "land", [3.5, 3.5])
        .then_events(|_| vec![], |scenario| scenario.debug_space("land"));
}

#[test]
fn test_move_into_barriers_connection() {
    PhysicsTestScenario::new("test_move_into_barriers_connection")
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [1.0, 3.0])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.5])
        .given_barrier("crate", "b", "land", [2.5, 3.5])
        .when_move_body("Alice", [3.5, 3.0])
        .when_update(3, 0.5)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.55, 3.0],
                }]
            },
            |scenario| scenario.debug_space("land"),
        );
}

#[test]
fn test_move_into_barrier() {
    PhysicsTestScenario::new(scenario!())
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [1.0, 2.0])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.5])
        .when_move_body("Alice", [1.95, 2.5])
        .when_update(1, 0.5)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.55, 3.0],
                }]
            },
            |scenario| scenario.debug_space("land"),
        );
}

#[test]
fn test_move_into_gap_between_barriers() {
    PhysicsTestScenario::new("test_move_into_gap_between_barriers")
        .given_space_kind("default", DEFAULT_SPACE_BOUNDS)
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5, 0.5)
        .given_body("farmer", "Alice", "land", [1.0, 3.0])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "a", "land", [2.5, 2.25])
        .given_barrier("crate", "b", "land", [2.5, 3.75])
        .when_move_body("Alice", [3.5, 3.0])
        .when_update(3, 0.5)
        .then_events(
            |given| {
                vec![BodyPositionChanged {
                    id: given.body("Alice"),
                    space: given.space("land"),
                    position: [1.551, 3.0],
                }]
            },
            |scenario| scenario.debug_space("land"),
        );
}
