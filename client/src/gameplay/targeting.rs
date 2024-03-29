use sdl2::keyboard::Keycode;

use game::inventory::ContainerId;
use game::landscaping::Surface;
use game::math::{ArrayIndex, Tile, TileMath, VectorMath};
use game::model::{
    Cementer, Composter, Construction, Corpse, Creature, Crop, Door, Equipment, Rest, Stack,
    Theodolite,
};
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
    Ground(Tile),
    Construction(Construction),
    Equipment(Equipment),
    Theodolite(Theodolite),
    Wall([usize; 2]),
    Crop(Crop),
    Door(Door),
    Rest(Rest),
    Stack(Stack),
    Cementer(Cementer),
    CementerContainer(Cementer, ContainerId),
    Composter(Composter),
    ComposterContainer(Composter, ContainerId),
    Creature(Creature),
    Corpse(Corpse),
    Device(DeviceId),
    Waterbody(Tile),
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
                return vec![Target::Stack(stack.entity)];
            }
        }

        for construction in self.constructions.values() {
            if construction.tile == tile {
                return vec![Target::Construction(construction.entity)];
            }
        }

        for equipment in self.equipments.values() {
            if equipment.position.to_tile() == tile {
                return vec![Target::Equipment(equipment.entity)];
            }
        }

        for theodolite in self.theodolites.values() {
            if theodolite.position.to_tile() == tile {
                return vec![Target::Theodolite(theodolite.entity)];
            }
        }

        for creature in self.creatures.values() {
            if creature.estimated_position.to_tile() == tile {
                return vec![Target::Creature(creature.entity)];
            }
        }

        for corpse in self.corpses.values() {
            if corpse.position.to_tile() == tile {
                return vec![Target::Corpse(corpse.entity)];
            }
        }

        for crop in self.crops.values() {
            if crop.position.to_tile() == tile {
                return vec![Target::Crop(crop.entity), Target::Ground(tile)];
            }
        }

        for door in self.doors.values() {
            if door.position.to_tile() == tile {
                return vec![Target::Door(door.entity)];
            }
        }

        for rest in self.rests.values() {
            if rest.position.to_tile() == tile {
                return vec![Target::Rest(rest.entity)];
            }
        }

        for cementer in self.cementers.values() {
            let pivot = cementer.position.to_tile();
            if tile == pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.input_offset)) {
                return vec![Target::CementerContainer(
                    cementer.entity,
                    cementer.entity.input,
                )];
            }
            if tile == pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.output_offset)) {
                return vec![Target::CementerContainer(
                    cementer.entity,
                    cementer.entity.output,
                )];
            }
            if tile == pivot {
                return vec![
                    Target::Cementer(cementer.entity),
                    Target::Device(cementer.entity.device),
                ];
            }
        }

        for composter in self.composters.values() {
            let pivot = composter.position.to_tile();
            if tile == pivot.add_offset(composter.rotation.apply_i8(composter.kind.input_offset)) {
                return vec![Target::ComposterContainer(
                    composter.entity,
                    composter.entity.input,
                )];
            }
            if tile == pivot.add_offset(composter.rotation.apply_i8(composter.kind.output_offset)) {
                return vec![Target::ComposterContainer(
                    composter.entity,
                    composter.entity.output,
                )];
            }
            if tile == pivot {
                return vec![
                    Target::Composter(composter.entity),
                    Target::Device(composter.entity.device),
                ];
            }
        }

        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();
            let [x, y] = tile;
            if farmland.surface[tile.fit(farmland.kind.land.width)] == Surface::BASIN {
                return vec![Target::Waterbody(tile)];
            }

            let cell = farmland.cells[y][x];
            if cell.wall {
                return vec![Target::Wall(tile)];
            }
        }

        vec![Target::Ground(tile)]
    }
}
