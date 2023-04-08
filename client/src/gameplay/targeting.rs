use sdl2::keyboard::Keycode;

use game::math::VectorMath;
use game::model::{Assembly, Construction, Creature, Crop, Door, Equipment, Stack};

use crate::engine::{Cursor, Input};
use crate::gameplay::Gameplay;

pub enum Intention {
    Use,
    Put,
    Swap,
    Aim([usize; 2]),
    Move,
    QuickSwap(u8),
}

#[derive(Clone)]
pub enum Target {
    Ground { tile: [usize; 2] },
    Stack(Stack),
    Assembly(Assembly),
    Construction(Construction),
    Equipment(Equipment),
    Wall([usize; 2]),
    Crop(Crop),
    Door(Door),
    Creature(Creature),
}

pub trait InputMethod {
    fn recognize_intention(&self, cursor: Cursor) -> Option<Intention>;
}

impl InputMethod for Input {
    fn recognize_intention(&self, cursor: Cursor) -> Option<Intention> {
        if self.left_click() {
            Some(Intention::Use)
        } else if self.right_click() {
            Some(Intention::Put)
        } else if self.pressed(Keycode::Tab) {
            Some(Intention::Swap)
        } else if self.pressed(Keycode::Num1) {
            Some(Intention::QuickSwap(0))
        } else if self.pressed(Keycode::Num2) {
            Some(Intention::QuickSwap(1))
        } else if self.pressed(Keycode::Num3) {
            Some(Intention::QuickSwap(2))
        } else if self.pressed(Keycode::Num4) {
            Some(Intention::QuickSwap(3))
        } else if self.down(Keycode::A)
            || self.down(Keycode::S)
            || self.down(Keycode::D)
            || self.down(Keycode::W)
        {
            Some(Intention::Move)
        } else if cursor.tile != cursor.previous_position.to_tile() {
            Some(Intention::Aim(cursor.tile))
        } else {
            None
        }
    }
}

impl Gameplay {
    pub fn get_target_at(&self, tile: [usize; 2]) -> Target {
        for stack in self.stacks.values() {
            if stack.position.to_tile() == tile {
                return Target::Stack(stack.entity);
            }
        }

        for construction in self.constructions.values() {
            if construction.tile == tile {
                return Target::Construction(construction.entity);
            }
        }

        for equipment in self.equipments.values() {
            if equipment.position.to_tile() == tile {
                return Target::Equipment(equipment.entity);
            }
        }

        for creature in self.creatures.values() {
            if creature.estimated_position.to_tile() == tile {
                return Target::Creature(creature.entity);
            }
        }

        for crop in self.crops.values() {
            if crop.position.to_tile() == tile {
                return Target::Crop(crop.entity);
            }
        }

        for door in self.doors.values() {
            if door.position.to_tile() == tile {
                return Target::Door(door.entity);
            }
        }

        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();

            let cell = farmland.cells[tile[1]][tile[0]];
            if cell.wall {
                return Target::Wall(tile);
            }
        }

        Target::Ground { tile }
    }
}
