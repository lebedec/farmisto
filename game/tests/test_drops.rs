use game::api::Action::DropItem;
use game::api::Event::{Inventory, Physics, Universe};
use game::inventory::ContainerId;
use game::inventory::Inventory::{ContainerCreated, ItemAdded, ItemRemoved};
use game::model::Drop;
use game::model::Universe::DropAppeared;
use game::physics::BarrierId;

use crate::testing::{at, GameTestScenario};

mod testing;

#[test]
fn test_something() {
    GameTestScenario::new()
        .given_farmland("test", "farmland")
        .given_farmer("farmer", "Alice", at(0, 0))
        .given_drop("d00", "farmland", [0, 0])
        .given_item("d00", "wood-material", "i01")
        .given_item("d00", "wood-material", "i02")
        .given_item("d00", "wood-material", "i03")
        .given_item("Alice:hands", "wood-material", "crate")
        .given_construction("c00", "farmland", [0, 0])
        .when_farmer_perform("Alice", DropItem { tile: [0, 0] })
        .then_action_events_should_be(|given| {
            vec![
                Physics(vec![]),
                Inventory(vec![
                    ItemRemoved {
                        item: given.item("crate"),
                        container: given.container("Alice:hands"),
                    },
                    ContainerCreated { id: ContainerId(4) },
                    ItemAdded {
                        item: given.item("crate"),
                        kind: given.item_key("wood-material"),
                        container: ContainerId(4),
                    },
                ]),
                Universe(vec![DropAppeared {
                    drop: Drop {
                        id: 2,
                        container: ContainerId(4),
                        barrier: BarrierId(2),
                    },
                    position: [64.0, 64.0],
                }]),
            ]
        });
}
