use crate::engine::sprites::TilemapUniform;
use crate::engine::Frame;
use crate::gameplay::{position_of, rendering_position_of, Gameplay, TILE_SIZE};
use game::building::{Grid, Room};
use game::math::VectorMath;
use game::model::{Activity, Purpose};
use lazy_static::lazy_static;
use rusty_spine::AnimationStateData;
use sdl2::libc::raise;
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
        for farmer in self.farmers.values_mut() {
            farmer.animate_position(frame.input.time);
        }
        METRIC_ANIMATION_SECONDS.observe_closure_duration(|| {
            for farmer in self.spines.iter_mut() {
                farmer.sprite.skeleton.update(frame.input.time);
            }
            for crop in self.crops.values_mut() {
                if let Some(mut impact_bone) = crop.spine.skeleton.skeleton.find_bone_mut("impact")
                {
                    if crop.impact > 0.0 {
                        impact_bone.set_rotation(360.0 - crop.impact * 90.0);
                    } else {
                        impact_bone.set_rotation(-crop.impact * 90.0);
                    }
                }
                let mut growth = crop
                    .spine
                    .skeleton
                    .animation_state
                    .track_at_index_mut(3)
                    .unwrap();
                growth.set_timescale(1.0);
                let f = 100.0 * (1.0 / 30.0);
                growth.set_animation_start(f);
                growth.set_animation_end(f);

                let mut development = crop
                    .spine
                    .skeleton
                    .animation_state
                    .track_at_index_mut(1)
                    .unwrap();
                development.set_timescale(1.0);
                let f = 50.0 * (1.0 / 30.0);
                development.set_animation_start(f);
                development.set_animation_end(f);

                crop.spine.skeleton.update(frame.input.time);
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
            let cursor_pos = 1 << (128 - cursor_x - 1);
            for shape in &farmland.rooms {
                if cursor_y >= shape.rows_y && cursor_y < shape.rows_y + shape.rows.len() {
                    let row = shape.rows[cursor_y - shape.rows_y];
                    if row & cursor_pos != 0 {
                        self.cursor_shape = shape.id;
                        break;
                    }
                }
            }

            renderer.render_ground(
                farmland.asset.texture.clone(),
                farmland.asset.sampler.share(),
                &farmland.map,
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
                &farmland.floor,
                [0.0, 0.0],
                0,
                TilemapUniform { map: floor_map },
            );
            renderer.render_tilemap(
                &farmland.roof,
                [0.0, -2.0 * TILE_SIZE],
                127,
                TilemapUniform { map: roof_map },
            );

            // renderer.render_roof(
            //     self.roof_texture.clone(),
            //     farmland.asset.sampler.share(),
            //     &farmland.map,
            //     &farmland.rooms,
            //     self.cursor_shape,
            // );
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

                        let tileset = if cell.marker.is_some() {
                            &farmland.asset.building_template_surveying.tiles
                        } else {
                            &farmland.asset.building_templates[cell.material.0 as usize].tiles
                        };

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

                        let is_half =
                            (y == (cursor_y + 1) || y == (cursor_y)) && neighbors == Neighbors::WE;
                        let is_half = false; // disable
                                             // half
                        if is_half {
                            tile = &tileset[15];
                            if cell.door {
                                tile = &tileset[22]; // 16 small
                            }
                            if cell.window {
                                tile = &tileset[17];
                            }
                        }

                        // exp
                        if neighbors == Neighbors::WE && line[x - 1].door {
                            tile = &tileset[20];
                            if is_half {
                                tile = &tileset[23];
                            }
                        }
                        if neighbors == Neighbors::WE && line[x + 1].door {
                            tile = &tileset[18];
                            if is_half {
                                tile = &tileset[21];
                            }
                        }

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

        for drop in self.drops.values() {
            let sprite_line = drop.position[1] as usize;
            let position = rendering_position_of(drop.position);
            renderer.render_sprite(&self.drop_sprite, position, sprite_line, 1.0);
            for (i, item) in self
                .items
                .get(&drop.entity.container)
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
            renderer.render_sprite(&self.drop_sprite, position, sprite_line, 1.0);
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

        for crop in self.crops.values() {
            renderer.render_spine(&crop.spine, rendering_position_of(crop.position));
        }

        METRIC_DRAW_REQUEST_SECONDS.observe_closure_duration(|| {
            for spine in &self.spines {
                renderer.render_spine(&spine.sprite, spine.position);
            }
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
