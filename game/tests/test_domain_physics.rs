use crate::testing::PhysicsTestScenario;

mod testing;

#[test]
fn test_something() {
    PhysicsTestScenario::new("test_something")
        .given_space_kind("default")
        .given_space("default", "land")
        .given_body_kind("farmer", 5.5)
        .given_body("farmer", "Alice", "land", [0.5, 1.5])
        .given_barrier_kind("crate", [1.0, 1.0])
        .given_barrier("crate", "1", "land", [0.5, 0.5])
        .given_barrier("crate", "2", "land", [1.5, 1.5])
        .given_barrier("crate", "3", "land", [2.5, 2.5])
        .given_barrier("crate", "4", "land", [3.5, 3.5])
        .then_events(|_| vec![], |scenario| scenario.debug_space("land"));
}
