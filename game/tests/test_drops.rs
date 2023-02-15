use game::api::Action::{DropItem, PutItem};
use game::api::ActionError;
use game::inventory::ContainerId;
use game::inventory::Inventory::{ContainerCreated, ItemAdded, ItemRemoved};
use game::inventory::InventoryError::{ContainerIsFull, ItemNotFoundByIndex};
use game::model::Drop;
use game::model::Universe::DropAppeared;
use game::physics::BarrierId;
use game::physics::Physics::BarrierCreated;
use game::physics::PhysicsError::BarrierCreationOverlaps;

use crate::testing::{at, GameTestScenario, ANYWHERE};

mod testing;

#[test]
fn test_drop_item_with_empty_hands() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_drop("d00", "farmland", ANYWHERE)
        .given_item("wood-material", "i01", "d00")
        .when_farmer_perform("Alice", |_| DropItem { tile: ANYWHERE })
        .then_action_should_fail(|given| {
            ActionError::Inventory(ItemNotFoundByIndex {
                container: given.container("Alice:hands"),
                index: -1,
            })
        });
}

#[test]
fn test_drop_item_on_occupied_tile() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_drop("stack", "farmland", [0, 0])
        .given_item("wood-material", "i01", "stack")
        .given_item("wood-material", "i02", "Alice:hands")
        .when_farmer_perform("Alice", |_| DropItem { tile: [0, 0] })
        .then_action_should_fail(|_| ActionError::Physics(BarrierCreationOverlaps));
}

#[test]
fn test_drop_item_under_feet() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_item("wood-material", "item", "Alice:hands")
        .when_farmer_perform("Alice", |_| DropItem { tile: [1, 1] })
        .then_action_should_fail(|_| ActionError::Physics(BarrierCreationOverlaps));
}

#[test]
fn test_regular_drop_item() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", at(0, 0))
        .given_item("wood-material", "crate", "Alice:hands")
        .when_farmer_perform("Alice", |_| DropItem { tile: [1, 1] })
        .then_events_should_be(|given| {
            vec![
                vec![BarrierCreated {
                    id: BarrierId(1),
                    space: given.space("farmland"),
                    position: [192.0, 192.0],
                    bounds: [128.0, 128.0],
                }]
                .into(),
                vec![
                    ItemRemoved {
                        item: given.item("crate"),
                        container: given.container("Alice:hands"),
                    },
                    ContainerCreated { id: ContainerId(2) },
                    ItemAdded {
                        item: given.item("crate"),
                        kind: given.item_key("wood-material"),
                        container: ContainerId(2),
                    },
                ]
                .into(),
                vec![DropAppeared {
                    drop: Drop {
                        id: 1,
                        container: ContainerId(2),
                        barrier: BarrierId(1),
                    },
                    position: [192.0, 192.0],
                }]
                .into(),
            ]
        });
}

#[test]
fn test_put_item_in_drop_with_no_capacity() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_drop("stack", "farmland", ANYWHERE)
        .given_items("stack", ["wood-material"; 5])
        .given_item("wood-material", "item", "Alice:hands")
        .when_farmer_perform("Alice", |given| PutItem {
            drop: given.drop("stack"),
        })
        .then_action_should_fail(|given| {
            ActionError::Inventory(ContainerIsFull {
                id: given.container("stack"),
            })
        });
}

#[test]
fn test_take_one_of_drop_items() {}

#[test]
fn test_take_last_item_from_drop() {}

#[test]
fn test_take_one_of_drop_items_with_busy_hands() {}
