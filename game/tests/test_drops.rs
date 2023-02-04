use crate::testing::GameTestScenario;

mod testing;

#[test]
fn test_something() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_construction("c00", "farmland", [0, 0]);
}
