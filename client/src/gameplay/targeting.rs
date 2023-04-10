use game::inventory::ContainerId;
use sdl2::keyboard::Keycode;

use game::math::{TileMath, VectorMath};
use game::model::{Assembly, Cementer, Construction, Creature, Crop, Door, Equipment, Stack};
use game::working::DeviceId;

use crate::engine::{Cursor, Input};
use crate::gameplay::Gameplay;

#[derive(Debug, Copy, Clone)]
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
    Container(ContainerId),
    Assembly(Assembly),
    Construction(Construction),
    Equipment(Equipment),
    Wall([usize; 2]),
    Crop(Crop),
    Door(Door),
    Cementer(Cementer),
    Creature(Creature),
    Device(DeviceId),
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
    pub fn get_targets_at(&self, tile: [usize; 2]) -> Vec<Target> {
        for stack in self.stacks.values() {
            if stack.position.to_tile() == tile {
                return vec![Target::Container(stack.entity.container)];
            }
        }

        for construction in self.constructions.values() {
            if construction.tile == tile {
                return vec![
                    Target::Construction(construction.entity),
                    Target::Container(construction.entity.container),
                ];
            }
        }

        for equipment in self.equipments.values() {
            if equipment.position.to_tile() == tile {
                return vec![Target::Equipment(equipment.entity)];
            }
        }

        for creature in self.creatures.values() {
            if creature.estimated_position.to_tile() == tile {
                return vec![Target::Creature(creature.entity)];
            }
        }

        for crop in self.crops.values() {
            if crop.position.to_tile() == tile {
                return vec![Target::Crop(crop.entity)];
            }
        }

        for door in self.doors.values() {
            if door.position.to_tile() == tile {
                return vec![Target::Door(door.entity)];
            }
        }

        for cementer in self.cementers.values() {
            let pivot = cementer.position.to_tile();
            if tile == pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.input_offset)) {
                return vec![
                    Target::Container(cementer.entity.input),
                    Target::Cementer(cementer.entity),
                    Target::Device(cementer.entity.device),
                ];
            }
            if tile == pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.output_offset)) {
                return vec![
                    Target::Container(cementer.entity.output),
                    Target::Cementer(cementer.entity),
                    Target::Device(cementer.entity.device),
                ];
            }
            if tile == pivot {
                return vec![
                    Target::Cementer(cementer.entity),
                    Target::Device(cementer.entity.device),
                ];
            }
        }

        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();

            let cell = farmland.cells[tile[1]][tile[0]];
            if cell.wall {
                return vec![Target::Wall(tile)];
            }
        }

        vec![Target::Ground { tile }]
    }
}
