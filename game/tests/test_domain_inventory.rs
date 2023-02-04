use crate::testing::InventoryTestScenario;

mod testing;

#[test]
fn test_something() {
    InventoryTestScenario::new()
        .given_container_kind("hands", 3)
        .given_container("hands", "Alice");
}
