use crate::engine::Frame;
use crate::gameplay::{rendering_position_of, Gameplay, InputMethod, TILE_SIZE};
use game::api::{Cheat, FarmerBound};
use game::math::{test_collisions, VectorMath};
use game::model::Activity;
use game::physics::generate_holes;
use glam::vec3;
use log::error;
use sdl2::keyboard::Keycode;

impl Gameplay {
    pub fn handle_user_input(&mut self, frame: &mut Frame) {
        let farmer = match self.get_my_farmer_mut() {
            Some(farmer) => unsafe { &mut *farmer },
            None => {
                // error!("Farmer behaviour not initialized yet");
                return;
            }
        };

        let input = &frame.input;

        let cursor = input.mouse_position(self.camera.position(), TILE_SIZE);
        let tile = cursor.tile;

        if input.pressed(Keycode::P) {
            self.players_index = (self.players_index + 1) % self.players.len();
        }

        let targets = self.get_targets_at(tile);

        if input.ctrl_pressed(Keycode::G) {
            self.send_cheat(Cheat::GrowthUpCrops {
                growth: 2.5,
                radius: 3.0,
            });
        }
        if input.ctrl_pressed(Keycode::H) {
            self.send_cheat(Cheat::SetCreaturesHealth {
                health: 0.0,
                radius: 3.0,
            });
        }

        // if input.pressed(Keycode::E) {
        //     if let Target::Crop(crop) = target {
        //         let creature = self.creatures.values_mut().nth(0).unwrap();
        //         let entity = creature.entity;
        //         self.send_action_as_ai(Action::EatCrop {
        //             crop,
        //             creature: entity,
        //         });
        //     }
        // }
        //
        // if input.pressed(Keycode::R) {
        //     if let Target::Ground { .. } = target {
        //         let creature = self.creatures.values().nth(0).unwrap().entity;
        //         self.send_action_as_ai(Action::MoveCreature {
        //             destination: cursor.position,
        //             creature,
        //         });
        //     }
        // }

        if let Some(intention) = input.recognize_intention(cursor) {
            for target in targets {
                let item = self
                    .items
                    .get(&farmer.entity.hands)
                    .and_then(|hands| hands.values().nth(0));
                let functions = match item {
                    None => vec![],
                    Some(item) => item.kind.functions.clone(),
                };
                self.interact_with(farmer, functions, target, intention);
            }
        }

        match farmer.activity {
            Activity::Idle | Activity::Usage | Activity::Assembling { .. } => {}
            _ => {
                // not movement allowed
                return;
            }
        }

        let mut direction = [0.0, 0.0];
        if input.down(Keycode::A) {
            direction[0] -= 1.0;
        }
        if input.down(Keycode::D) {
            direction[0] += 1.0;
        }
        if input.down(Keycode::W) {
            direction[1] -= 1.0;
        }
        if input.down(Keycode::S) {
            direction[1] += 1.0;
        }
        let delta = direction.normalize().mul(input.time * farmer.body.speed);
        let estimated_position = delta.add(farmer.rendering_position);

        let farmland = match self.current_farmland {
            None => {
                error!("Current farmland not specified yet");
                return;
            }
            Some(farmland) => farmland,
        };

        let farmland = self.farmlands.get(&farmland).unwrap();

        // client side physics pre-calculation to prevent
        // obvious movement errors
        // Collision Detection
        let holes = generate_holes(estimated_position, farmer.body.radius, &farmland.holes);
        let holes_offsets = match test_collisions(farmer, estimated_position, &holes) {
            Some(offsets) => offsets,
            None => vec![],
        };
        if holes_offsets.len() < 2 {
            let offsets = match test_collisions(farmer, estimated_position, &self.barriers_hint) {
                None => holes_offsets,
                Some(mut barrier_offsets) => {
                    barrier_offsets.extend(holes_offsets);
                    barrier_offsets
                }
            };
            if offsets.len() < 2 {
                let estimated_position = if offsets.len() == 1 {
                    estimated_position.add(offsets[0])
                } else {
                    estimated_position
                };
                farmer.estimated_position = estimated_position;
                if delta.length() > 0.0 {
                    self.send_action(FarmerBound::Move {
                        destination: estimated_position,
                    });
                }
            }
        }

        // TODO: move camera after farmer rendering position changed
        let width = frame.scene.screen.width() as f32 * frame.scene.zoom;
        let height = frame.scene.screen.height() as f32 * frame.scene.zoom;
        let farmer_rendering_position = rendering_position_of(farmer.rendering_position);
        self.camera.eye = vec3(
            farmer_rendering_position[0] - width / 2.0,
            farmer_rendering_position[1] - height / 2.0,
            0.0,
        );
    }
}
