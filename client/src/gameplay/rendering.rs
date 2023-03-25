use crate::engine::rendering::TilemapUniform;
use crate::engine::Frame;
use crate::gameplay::representation::{CreatureRep, CropRep};
use crate::gameplay::{position_of, rendering_position_of, Gameplay, TILE_SIZE};
use game::building::{Grid, Room};
use game::math::VectorMath;
use game::model::{Activity, Purpose};
use lazy_static::lazy_static;
use rand::prelude::*;
use std::collections::HashMap;

lazy_static! {
    static ref METRIC_ANIMATION_SECONDS: prometheus::Histogram =
        prometheus::register_histogram!("gameplay_animation_seconds", "gameplay_animation_seconds")
            .unwrap();
    static ref METRIC_DRAW_REQUEST_SECONDS: prometheus::Histogram =
        prometheus::register_histogram!(
            "gameplay_draw_request_seconds",
            "gameplay_draw_request_seconds"
        )
        .unwrap();
}

impl Gameplay {
    pub fn animate(&mut self, frame: &mut Frame) {
        let time = frame.input.time;
        for farmer in self.farmers.values_mut() {
            farmer.animate_position(time);
        }
        for creature in self.creatures.values_mut() {
            creature.animate_position(time);
            let alpha = if creature.velocity.length() > 0.0 {
                1.0
            } else {
                0.0
            };
            let mut walk = creature
                .spine
                .skeleton
                .animation_state
                .track_at_index_mut(CreatureRep::ANIMATION_TRACK_WALK as usize)
                .unwrap();
            walk.set_alpha(alpha);
            let mut idle = creature
                .spine
                .skeleton
                .animation_state
                .track_at_index_mut(CreatureRep::ANIMATION_TRACK_IDLE as usize)
                .unwrap();
            idle.set_alpha(1.0 - alpha);
        }
        METRIC_ANIMATION_SECONDS.observe_closure_duration(|| {
            for creature in self.creatures.values_mut() {
                creature
                    .spine
                    .skeleton
                    .skeleton
                    .set_scale_x(creature.direction[0].signum());
                creature.spine.skeleton.update(time);
            }
            for crop in self.crops.values_mut() {
                crop.animate_growth(time);
                if let Some(mut impact_bone) = crop.spines[crop.spine]
                    .skeleton
                    .skeleton
                    .find_bone_mut("impact")
                {
                    if crop.impact > 0.0 {
                        impact_bone.set_rotation(360.0 - crop.impact * 90.0);
                    } else {
                        impact_bone.set_rotation(-crop.impact * 90.0);
                    }
                }
                let mut growth = crop.spines[crop.spine]
                    .skeleton
                    .animation_state
                    .track_at_index_mut(CropRep::ANIMATION_TRACK_GROWTH as usize)
                    .unwrap();
                growth.set_animation_start(crop.growth);
                growth.set_animation_end(crop.growth);
                // let mut growth = crop
                //     .spine
                //     .skeleton
                //     .animation_state
                //     .track_at_index_mut(3)
                //     .unwrap();
                // growth.set_timescale(1.0);
                // let f = 100.0 * (1.0 / 30.0);
                // growth.set_animation_start(f);
                // growth.set_animation_end(f);
                //
                // let mut drying = crop
                //     .spine
                //     .skeleton
                //     .animation_state
                //     .track_at_index_mut(2)
                //     .unwrap();
                // drying.set_timescale(1.0);
                // let f = (100.0 * crop.thirst) * (1.0 / 30.0);
                // drying.set_animation_start(f);
                // drying.set_animation_end(f);
                //
                // let mut development = crop
                //     .spine
                //     .skeleton
                //     .animation_state
                //     .track_at_index_mut(1)
                //     .unwrap();
                // development.set_timescale(1.0);
                // let f = 50.0 * (1.0 / 30.0);
                // development.set_animation_start(f);
                // development.set_animation_end(f);

                crop.spines[crop.spine].skeleton.update(time);
            }
        });
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.sprites;
        renderer.clear();
        renderer.look_at(self.camera.eye);

        let cursor = frame
            .input
            .mouse_position(self.camera.position(), TILE_SIZE);
        let [cursor_x, cursor_y] = cursor.tile;

        for farmland in self.farmlands.values() {
            self.cursor_shape = 0;
            let mut cursor_room = None;
            for (i, room) in farmland.rooms.iter().enumerate() {
                if room.contains(cursor.tile) {
                    self.cursor_shape = room.id;
                    if room.id != Room::EXTERIOR_ID {
                        cursor_room = Some(i);
                    }

                    break;
                }
            }

            renderer.render_ground(
                farmland.asset.texture.clone(),
                farmland.asset.sampler.share(),
                &farmland.soil_map,
                &farmland.rooms,
            );

            let mut floor_map = [[[0; 4]; 31]; 18];
            for room in &farmland.rooms {
                if room.id == Room::EXTERIOR_ID {
                    continue;
                }
                for (i, row) in room.rows.iter().enumerate() {
                    let y = room.rows_y + i;
                    if y < floor_map.len() {
                        for x in 0..31 {
                            let interior_bit = 1 << (Grid::COLUMNS - x - 1);
                            if row & interior_bit == interior_bit {
                                floor_map[y][x] = [1, 0, 0, 0];
                            }
                        }
                    }
                }
            }
            let mut roof_map = [[[0; 4]; 31]; 18];
            for room in &farmland.rooms {
                if room.id == Room::EXTERIOR_ID || room.id == self.cursor_shape {
                    continue;
                }
                for (i, row) in room.rows.iter().enumerate() {
                    let y = room.rows_y + i;
                    if y < roof_map.len() {
                        for x in 0..31 {
                            let interior_bit = 1 << (Grid::COLUMNS - x - 1);
                            if row & interior_bit == interior_bit {
                                roof_map[y][x] = [1, 0, 0, 0];
                            }
                        }
                    }
                }
            }
            renderer.render_tilemap(
                &farmland.building_marker.floor, // TODO: detect floor
                [0.0, 0.0],
                0,
                TilemapUniform { map: floor_map },
            );
            renderer.render_tilemap(
                &farmland.building_marker.roof, // TODO: detect roof
                [0.0, -2.0 * TILE_SIZE],
                127,
                TilemapUniform { map: roof_map },
            );

            for (y, line) in farmland.cells.iter().enumerate() {
                for (x, cell) in line.iter().enumerate() {
                    if cell.wall {
                        let west = x > 0 && line[x - 1].wall;
                        let east = x + 1 < line.len() && line[x + 1].wall;
                        let north = y > 0 && farmland.cells[y - 1][x].wall;
                        let south = y + 1 < farmland.cells.len() && farmland.cells[y + 1][x].wall;
                        let neighbors = match (west, north, east, south) {
                            (true, true, true, true) => Neighbors::Full,
                            (false, true, false, true) => Neighbors::NS,
                            (true, false, true, false) => Neighbors::WE,
                            (true, true, false, false) => Neighbors::NW,
                            (false, true, true, false) => Neighbors::NE,
                            (false, false, true, true) => Neighbors::SE,
                            (true, false, false, true) => Neighbors::SW,
                            (true, true, true, false) => Neighbors::WNE,
                            (true, true, false, true) => Neighbors::WNS,
                            (true, false, true, true) => Neighbors::ESW,
                            (false, true, true, true) => Neighbors::NES,
                            // unimplemented
                            (true, false, false, false) => Neighbors::WE,
                            (false, true, false, false) => Neighbors::NS,
                            (false, false, false, true) => Neighbors::NS,
                            (false, false, true, false) => Neighbors::WE,
                            (false, false, false, false) => Neighbors::Full,
                        };

                        let is_transparent = (y == (cursor_y + 1) || y == (cursor_y));
                        let is_transparent = is_transparent && cursor_room.is_some();

                        // skin
                        let tileset = if cell.marker.is_some() {
                            &farmland.building_marker.asset.walls.tiles
                        } else {
                            if is_transparent {
                                &farmland.buildings[cell.material.0 as usize]
                                    .asset
                                    .walls_transparency
                                    .tiles
                            } else {
                                &farmland.buildings[cell.material.0 as usize]
                                    .asset
                                    .walls
                                    .tiles
                            }
                        };

                        // tile change
                        let mut tile = match neighbors {
                            Neighbors::WE => &tileset[0],
                            Neighbors::NS => &tileset[1],
                            Neighbors::Full => &tileset[2],
                            Neighbors::NW => &tileset[3],
                            Neighbors::NE => &tileset[4],
                            Neighbors::SE => &tileset[5],
                            Neighbors::SW => &tileset[6],
                            Neighbors::WNS => &tileset[7],
                            Neighbors::NES => &tileset[8],
                            Neighbors::ESW => &tileset[9],
                            Neighbors::WNE => &tileset[10],
                        };

                        // tile change
                        if cell.door {
                            tile = match neighbors {
                                Neighbors::NS => &tileset[12],
                                _ => &tileset[19], // 11 small
                            }
                        }
                        if cell.window {
                            tile = match neighbors {
                                Neighbors::NS => &tileset[14],
                                _ => &tileset[13],
                            };
                        }

                        // expand door way
                        // if neighbors == Neighbors::WE && x > 1 && line[x - 1].door {
                        //     tile = &tileset[20];
                        //     if is_half {
                        //         tile = &tileset[23];
                        //     }
                        // }
                        // if neighbors == Neighbors::WE && line[x + 1].door {
                        //     tile = &tileset[18];
                        //     if is_half {
                        //         tile = &tileset[21];
                        //     }
                        // }

                        let highlight = if y == cursor_y as usize && x == cursor_x as usize {
                            1.5
                        } else {
                            1.0
                        };
                        let position = [x as f32 * TILE_SIZE, TILE_SIZE + y as f32 * TILE_SIZE];
                        renderer.render_sprite(
                            tile,
                            position,
                            (position[1] / TILE_SIZE) as usize,
                            highlight,
                        );
                    }
                }
            }
        }
        let cursor_x = cursor_x as f32 * TILE_SIZE + 64.0;
        let cursor_y = cursor_y as f32 * TILE_SIZE + 64.0;
        let cursor_position = [cursor_x, cursor_y];
        renderer.render_sprite(
            &self.cursor,
            cursor_position,
            (cursor_position[1] / TILE_SIZE) as usize,
            1.0,
        );

        for farmer in self.farmers.values() {
            let sprite_line = farmer.rendering_position[1] as usize;
            let rendering_position = rendering_position_of(farmer.rendering_position);

            for (i, item) in self
                .items
                .entry(farmer.entity.backpack)
                .or_insert(HashMap::new())
                .values()
                .enumerate()
            {
                let kind = self.known.items.get(item.kind).unwrap();
                let asset = assets.item(&kind.name);
                let offset = [0.0, -128.0 - (32.0 * i as f32)];
                renderer.render_sprite(
                    &asset.sprite,
                    rendering_position.add(offset),
                    sprite_line,
                    1.0,
                );
            }

            renderer.render_sprite(
                &self.players[farmer.entity.id],
                rendering_position,
                sprite_line,
                1.0,
            );

            for (i, item) in self
                .items
                .entry(farmer.entity.hands)
                .or_insert(HashMap::new())
                .values()
                .enumerate()
            {
                let kind = self.known.items.get(item.kind).unwrap();
                let asset = assets.item(&kind.name);
                let offset = [0.0, -128.0 - (32.0 * i as f32)];
                renderer.render_sprite(
                    &asset.sprite,
                    rendering_position.add(offset),
                    sprite_line,
                    1.0,
                );
            }

            let last_sync_position = rendering_position_of(farmer.last_sync_position);
            renderer.render_sprite(
                &self.cursor,
                last_sync_position,
                (last_sync_position[1] / TILE_SIZE) as usize,
                0.5,
            );
        }

        for stack in self.stacks.values() {
            let sprite_line = stack.position[1] as usize;
            let position = rendering_position_of(stack.position);
            renderer.render_sprite(&self.stack_sprite, position, sprite_line, 1.0);
            for (i, item) in self
                .items
                .get(&stack.entity.container)
                .unwrap()
                .values()
                .enumerate()
            {
                let kind = self.known.items.get(item.kind).unwrap();
                let asset = assets.item(&kind.name);
                let offset = [
                    0.0,
                    -24.0 + (48.0 * (i % 2) as f32) - (48.0 * (i / 2) as f32),
                ];
                renderer.render_sprite(&asset.sprite, position.add(offset), sprite_line, 1.0);
            }
        }

        for construction in self.constructions.values() {
            let sprite_line = construction.tile[1];
            let position = position_of(construction.tile);
            let position = rendering_position_of(position);
            renderer.render_sprite(&self.stack_sprite, position, sprite_line, 1.0);
            for (i, item) in self
                .items
                .entry(construction.entity.container)
                .or_insert(HashMap::new())
                .values()
                .enumerate()
            {
                let kind = self.known.items.get(item.kind).unwrap();
                let asset = assets.item(&kind.name);
                let offset = [
                    0.0,
                    -24.0 + (48.0 * (i % 2) as f32) - (48.0 * (i / 2) as f32),
                ];
                renderer.render_sprite(&asset.sprite, position.add(offset), sprite_line, 1.0);
            }
        }

        for equipment in self.equipments.values() {
            match equipment.entity.purpose {
                Purpose::Surveying { .. } => {
                    let sprite_line = equipment.position[1] as usize;
                    let position = rendering_position_of(equipment.position);
                    renderer.render_sprite(&self.theodolite_sprite, position, sprite_line, 1.0);
                }
                Purpose::Moisture { .. } => {}
            }
        }

        for farmer in self.farmers.values() {
            if let Activity::Surveying {
                equipment,
                selection,
            } = farmer.activity
            {
                let equipment = self.equipments.get(&equipment).unwrap();
                let sprite_line = equipment.position[1] as usize;
                let position = rendering_position_of(equipment.position);
                renderer.render_sprite(
                    &self.theodolite_gui_sprite,
                    position.add([0.0, -32.0]),
                    sprite_line,
                    1.0,
                );
                renderer.render_sprite(
                    &self.theodolite_gui_select_sprite,
                    position.add([-196.0 + 128.0 * (selection as f32), -224.0]),
                    sprite_line,
                    1.0,
                );
            }
        }

        for creature in self.creatures.values() {
            renderer.render_animal(
                &creature.spine,
                &creature.asset.coloration,
                rendering_position_of(creature.rendering_position),
            );
        }

        for crop in self.crops.values() {
            let mut random = StdRng::seed_from_u64(crop.entity.id as u64);
            let offset_x: f32 = random.gen_range(-0.05..0.05);
            let offset_y: f32 = random.gen_range(-0.05..0.05);
            let offset = [offset_x, offset_y];
            renderer.render_plant(
                &crop.spines[crop.spine],
                crop.asset.damage_mask.share(),
                rendering_position_of(crop.position.add(offset)),
                crop.health,
                crop.thirst,
                [
                    // [1.0, 1.0 - crop.thirst * 0.5, 1.0 - crop.thirst * 0.75, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ],
            );
        }

        METRIC_DRAW_REQUEST_SECONDS.observe_closure_duration(|| {
            // for spine in &self.spines {
            //     renderer.render_spine(
            //         &spine.sprite,
            //         spine.position,
            //         [
            //             [1.0, 1.0, 1.0, 1.0],
            //             [1.0, 1.0, 1.0, 1.0],
            //             [1.0, 1.0, 1.0, 1.0],
            //             [1.0, 1.0, 1.0, 1.0],
            //         ],
            //     );
            // }
        });
        renderer.set_point_light(
            [1.0, 0.0, 0.0, 1.0],
            512.0,
            rendering_position_of(cursor.position),
        );
        renderer.set_point_light([1.0, 0.0, 0.0, 1.0], 512.0, [1024.0, 256.0]);
        renderer.set_point_light([0.0, 1.0, 0.0, 1.0], 512.0, [256.0, 1024.0]);
        renderer.set_point_light([0.0, 0.0, 1.0, 1.0], 512.0, [1024.0, 1024.0]);

        renderer.render_sprite(&self.gui_controls, [-512.0, 512.0], 127, 1.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Neighbors {
    WE,
    NS,
    Full,
    NW,
    NE,
    SE,
    SW,
    WNS,
    NES,
    ESW,
    WNE,
}
