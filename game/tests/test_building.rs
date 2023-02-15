use crate::testing::{any, GameTestScenario, ANYWHERE};
use game::api::Action;
use game::building::{Building, Cell, Marker};
use game::building::{GridId, Room};
use game::inventory::Inventory;
use game::model::Universe;
use game::physics::Physics;

mod testing;

#[test]
fn test_first_construction() {
    GameTestScenario::new(scenario!())
        .given_farmland("test", "land")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_theodolite("level", ANYWHERE)
        .when_farmer_perform("Alice", |given| Action::Survey {
            theodolite: given.theodolite("level"),
            tile: [1, 1],
            marker: Marker::Wall,
        })
        .debug(|scenario| scenario.debug_buildings("land").present())
        .then_events_should_be(|given| vec![]);
}

#[test]
fn test_complete_room_with_no_doors_and_windows() {
    GameTestScenario::new(scenario!())
        .given_farmland("test", "land")
        .given_farmer("farmer", "Alice", ANYWHERE)
        .given_buildings(
            r#"
            . . . . . . .
            . # # # # # .
            . # . . . # .
            . # # . # # .
            . . . . . . .
        "#,
        )
        .given_construction("wall", [3, 3])
        .given_item("wood-material", "wood", "wall")
        .when_farmer_perform("Alice", |given| Action::Construct {
            construction: given.construction("wall"),
        })
        .debug(|scenario| scenario.debug_buildings("land").present())
        .then_events_should_be(|given| {
            events![
                Inventory::ItemRemoved {
                    item: given.item("wood"),
                    container: given.container("wall")
                },
                Building::GridChanged {
                    grid: given.grid("land"),
                    cells: any(),
                    rooms: any(),
                },
                Physics::SpaceUpdated {
                    id: given.space("land"),
                    holes: any()
                },
                Universe::ConstructionVanished {
                    id: given.construction("wall"),
                },
            ]
        });
}
