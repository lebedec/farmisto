use std::collections::HashMap;

use log::{error, info};
use rand::prelude::*;

use game::assembling::Rotation;
use game::building::{Cell, Grid, Marker, Material, Room, Structure};
use game::inventory::{ContainerId, ItemId};
use game::math::{ArrayIndex, Position, Tile, TileMath, VectorMath};
use game::model::{Activity, CementerKind, ComposterKind, Purpose};

use crate::assets::{CementerAsset, ComposterAsset};
use crate::engine::rendering::{xy, Scene, TilemapUniform, VISIBLE_MAP_X, VISIBLE_MAP_Y};
use crate::engine::Frame;
use crate::gameplay::representation::{AssemblyTargetAsset, CreatureRep, CropRep, ItemRep};
use crate::gameplay::{position_of, rendering_position_of, Gameplay, TILE_SIZE};

impl Gameplay {
    pub fn animate(&mut self, frame: &mut Frame) {
        let time = frame.input.time * self.game_speed;
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
                .track_at_index_mut(CreatureRep::ANIMATION_WALK as usize)
                .unwrap();
            walk.set_alpha(alpha);
            walk.set_timescale(self.game_speed * 1.5);

            let mut idle = creature
                .spine
                .skeleton
                .animation_state
                .track_at_index_mut(CreatureRep::ANIMATION_BASE as usize)
                .unwrap();
            idle.set_alpha(1.0 - alpha);
            idle.set_timescale(self.game_speed);

            let mut age = creature
                .spine
                .skeleton
                .animation_state
                .track_at_index_mut(CreatureRep::ANIMATION_AGE as usize)
                .unwrap();
            age.set_alpha(1.0);
            age.set_timescale(1.0);
            let age_t = creature.age;
            let age_t = if age_t < 24.0 {
                age_t / 24.0 * 0.33
            } else if age_t < 48.0 {
                0.33 + (age_t - 24.0) / 24.0 * 0.33
            } else {
                0.67
            };
            age.set_animation_start(age_t);
            age.set_animation_end(age_t);

            let mut weight = creature
                .spine
                .skeleton
                .animation_state
                .track_at_index_mut(CreatureRep::ANIMATION_WEIGHT as usize)
                .unwrap();
            let weight_t = creature.weight.min(1.0);
            weight.set_animation_start(weight_t);
            weight.set_animation_end(weight_t);
        }
        for creature in self.creatures.values_mut() {
            creature
                .spine
                .skeleton
                .skeleton
                .set_scale_x(creature.direction[0].signum());
            creature.spine.skeleton.update(time);
        }
        for crop in self.crops.values_mut() {
            let current_spine = crop.spine();
            if let Some(mut impact_bone) = crop.spines[current_spine]
                .skeleton
                .skeleton
                .find_bone_mut("impact")
            {
                info!("IMPACT REnDER {}", crop.impact);
                if crop.impact > 0.0 {
                    impact_bone.set_rotation(360.0 - crop.impact * 90.0);
                } else if crop.impact < 0.0 {
                    impact_bone.set_rotation(-crop.impact * 90.0);
                }
            }
            let mut growth = crop.spines[current_spine]
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
            crop.spines[current_spine].skeleton.update(time);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let scene = &mut frame.scene;
        scene.look_at(self.camera.eye);

        let cursor = frame
            .input
            .mouse_position(self.camera.position(), TILE_SIZE);
        let [cursor_x, cursor_y] = cursor.tile;

        let farmer = match self.get_my_farmer_mut() {
            Some(farmer) => unsafe { &mut *farmer }.rendering_position.to_tile(),
            None => [0, 0],
        };

        let render_offset = self.camera.position().div(TILE_SIZE).to_tile();
        let [render_offset_x, render_offset_y] = render_offset;

        for farmland in self.farmlands.values_mut() {
            let mut cursor_room = 0;
            let mut all_cursor_rooms = vec![];
            let mut farmer_room = 0;

            for (_i, room) in farmland.rooms.iter().enumerate() {
                if room.contains(cursor.tile) {
                    cursor_room = room.id;
                    all_cursor_rooms.push(room.id);
                }
                if room.contains([cursor.tile[0], cursor.tile[1] + 1]) {
                    all_cursor_rooms.push(room.id);
                }
                if room.contains([cursor.tile[0], cursor.tile[1] + 2]) {
                    all_cursor_rooms.push(room.id);
                }
                if room.contains(farmer) {
                    farmer_room = room.id;
                }
            }

            // TODO: client interpolation without heavy server events
            // let not_synchronized_heat = 0.00233 * frame.input.time;
            // let moisture = &mut farmland.moisture;
            // for y in 0..LAND_HEIGHT {
            //     for x in 0..LAND_WIDTH {
            //         let value = moisture[y][x] as f32 / 255.0;
            //         moisture[y][x] = ((value - not_synchronized_heat).max(0.0) * 255.0) as u8;
            //     }
            // }

            let camera_position = [self.camera.eye.x, self.camera.eye.y];
            let [input_size_x, input_size_y] =
                [farmland.kind.land.width, farmland.kind.land.height];
            let offset_step = camera_position.div(TILE_SIZE).floor();
            let offset_step = offset_step.clamp(
                [0.0, 0.0],
                [
                    (input_size_x - VISIBLE_MAP_X) as f32,
                    (input_size_y - VISIBLE_MAP_Y) as f32,
                ],
            );
            let offset = offset_step.mul(TILE_SIZE);
            let mut map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y] = Default::default();
            for y in 0..VISIBLE_MAP_Y {
                for x in 0..VISIBLE_MAP_X {
                    let [step_x, step_y] = offset_step;
                    let iy = y + step_y as usize;
                    let ix = x + step_x as usize;
                    let index = [ix, iy].fit(farmland.kind.land.width);
                    let moisture = farmland.moisture[index];
                    let capacity = farmland.moisture_capacity[index];
                    let fertility = farmland.fertility[index];
                    map[y][x] = [capacity, moisture, fertility, 0.0];
                }
            }
            scene.render_ground(
                farmland.asset.texture.share(),
                farmland.asset.sampler.share(),
                map,
                TILE_SIZE,
                offset,
            );

            let mut surface_map = [[[0; 4]; 31]; 18];
            for y in 0..18 {
                let y = y + render_offset_y;
                for x in 0..31 {
                    let x = x + render_offset_x;
                    let s = farmland.surface[[x, y].fit(farmland.kind.land.width)];
                    surface_map[y - render_offset_y][x - render_offset_x] = [s as u32, 0, 0, 0];
                }
            }

            let mut floor_map = [[[0; 4]; 31]; 18];
            for (room_index, room) in farmland.rooms.iter().enumerate() {
                if room.id == Room::EXTERIOR_ID {
                    continue;
                }
                'no_flooring: for (i, row) in room.area.iter().enumerate() {
                    let y = room.area_y + i;
                    if y >= render_offset_y && (y - render_offset_y) < floor_map.len() {
                        for x in 0..31 {
                            let x = x + render_offset_x;

                            let interior_bit = 1 << (Grid::COLUMNS - x - 1);
                            if row & interior_bit == interior_bit {
                                // TODO: room material detection
                                let material = farmland.cells[y][x].material.index();
                                if material == Material::PLANKS || material == Material::GLASS {
                                    break 'no_flooring;
                                }
                                floor_map[y - render_offset_y][x - render_offset_x] =
                                    [1, 0, 0, room_index as u32];
                            }
                        }
                    }
                }
            }

            let mut roof_map = [[[0; 4]; 31]; 18];
            for room in &farmland.rooms {
                if room.id == Room::EXTERIOR_ID
                    || room.id == cursor_room
                    || room.id == farmer_room
                    || all_cursor_rooms.contains(&room.id)
                {
                    continue;
                }
                'no_roofing: for (i, row) in room.area.iter().enumerate() {
                    let y = room.area_y + i;
                    if y >= render_offset_y && (y - render_offset_y) < roof_map.len() {
                        for x in 0..31 {
                            let x = x + render_offset_x;
                            let interior_bit = 1 << (Grid::COLUMNS - x - 1);
                            if row & interior_bit == interior_bit {
                                // TODO: room material detection
                                let material = farmland.cells[y][x].material.index();
                                if material == Material::PLANKS || material == Material::GLASS {
                                    break 'no_roofing;
                                }
                                roof_map[y - render_offset_y][x - render_offset_x] = [1, 0, 0, 0];
                            }
                        }
                    }
                }
            }
            scene.render_tilemap(
                &farmland.construction.floor, // TODO: detect floor,
                [
                    render_offset_x as f32 * TILE_SIZE,
                    render_offset_y as f32 * TILE_SIZE,
                ],
                0,
                TilemapUniform { map: floor_map },
            );
            scene.render_tilemap(
                &farmland.surface_tilemap,
                [
                    render_offset_x as f32 * TILE_SIZE,
                    render_offset_y as f32 * TILE_SIZE,
                ],
                0,
                TilemapUniform { map: surface_map },
            );
            scene.render_tilemap(
                &farmland.construction.roof, // TODO: detect roof
                [
                    render_offset_x as f32 * TILE_SIZE,
                    render_offset_y as f32 * TILE_SIZE + -2.0 * TILE_SIZE,
                ],
                127,
                TilemapUniform { map: roof_map },
            );

            let mut rendering_cells = farmland.cells.clone();

            let mut surveying = HashMap::new();
            for construction in self.constructions.values() {
                surveying.insert(construction.tile, construction.marker);
                let [column, row] = construction.tile;
                // create walls from markers via rendering process
                // to make correct tiling calculation
                rendering_cells[row][column].wall = true;
                match construction.marker {
                    Marker::Construction(structure) => {
                        rendering_cells[row][column].window = structure == Structure::Window;
                        rendering_cells[row][column].door = structure == Structure::Door;
                    }
                    Marker::Reconstruction(structure) => {
                        rendering_cells[row][column].window = structure == Structure::Window;
                        rendering_cells[row][column].door = structure == Structure::Door;
                    }
                    Marker::Deconstruction => {}
                }
            }

            for (y, line) in rendering_cells.iter().enumerate() {
                for (x, _cell) in line.iter().enumerate() {
                    let highlight = if y == cursor_y as usize && x == cursor_x as usize {
                        [1.5; 4]
                    } else if y > cursor_y
                        && y <= cursor_y + 2
                        && x as isize >= (cursor_x as isize) - 2
                        && x <= cursor_x + 2
                    {
                        [1.0, 1.0, 1.0, 0.5]
                    } else {
                        [1.0; 4]
                    };
                    let position = [x as f32 * TILE_SIZE, TILE_SIZE + y as f32 * TILE_SIZE];
                    let sorting = (position[1] - TILE_SIZE / 2.0) as isize;
                    if farmland.cells[y][x].wall {
                        // if origin wall
                        let neighbors = Neighbors::of(x, y, &farmland.cells);
                        let mut tile_index = neighbors.to_tile_index();
                        let material = farmland.cells[y][x].material.index();
                        let tileset = &farmland.buildings.get(&material).unwrap().asset.walls.tiles;

                        if y >= render_offset_y && x >= render_offset_x {
                            let y = y - render_offset_y;
                            let x = x - render_offset_x;
                            if y + 1 < 18 && x < 31 {
                                let mut is_exterior_around = floor_map[y + 1][x][3] == 0;
                                if x > 1 {
                                    is_exterior_around =
                                        is_exterior_around || floor_map[y + 1][x - 1][3] == 0;
                                }
                                if x + 1 < 31 {
                                    is_exterior_around =
                                        is_exterior_around || floor_map[y + 1][x + 1][3] == 0;
                                }
                                if !is_exterior_around {
                                    // shift index to interior tiles
                                    tile_index += 32;
                                }
                            }
                        }
                        let tile = &tileset[tile_index];
                        scene.render_sprite_colored(tile, xy(position).sorting(sorting), highlight);
                    }
                    if let Some(marker) = surveying.get(&[x, y]) {
                        let tileset = match marker {
                            Marker::Construction(_) => &farmland.construction.asset.walls.tiles,
                            Marker::Reconstruction(_) => &farmland.reconstruction.asset.walls.tiles,
                            Marker::Deconstruction => &farmland.deconstruction.asset.walls.tiles,
                        };
                        let neighbors = Neighbors::of(x, y, &rendering_cells);
                        let tile_index = neighbors.to_tile_index();
                        let tile = &tileset[tile_index];
                        scene.render_sprite_colored(tile, xy(position).sorting(sorting), highlight);

                        let construction = self
                            .constructions
                            .values()
                            .find(|construction| construction.tile == [x, y])
                            .unwrap();
                        let position = position_of(construction.tile);
                        let position = rendering_position_of(position);
                        render_items_stack(
                            &self.items,
                            construction.entity.container,
                            position,
                            scene,
                        );
                    }
                }
            }
        }
        let cursor_x = cursor_x as f32 * TILE_SIZE + 64.0;
        let cursor_y = cursor_y as f32 * TILE_SIZE + 64.0;
        let cursor_position = [cursor_x, cursor_y];
        scene.render_sprite(&self.cursor, xy(cursor_position));

        for assembly in self.assembly.values() {
            let position = position_of(assembly.pivot);
            let mut rendering_position = rendering_position_of(position);
            let highlight = if assembly.valid {
                [1.0; 4]
            } else {
                [1.0, 0.5, 0.5, 1.0]
            };
            match &assembly.asset {
                AssemblyTargetAsset::Door { door } => {
                    // fix door offset
                    rendering_position[1] += TILE_SIZE / 2.0;
                    let index = assembly.rotation.index();
                    let sprite = &door.sprites.tiles[index];
                    scene.render_sprite_colored(sprite, xy(rendering_position), highlight)
                }
                AssemblyTargetAsset::Cementer { cementer, kind } => {
                    render_cementer(
                        assembly.pivot,
                        assembly.rotation,
                        cementer,
                        &kind,
                        false,
                        false,
                        true,
                        false,
                        highlight,
                        scene,
                    );
                }
                AssemblyTargetAsset::Composter { composter, kind } => {
                    render_composter(
                        assembly.pivot,
                        assembly.rotation,
                        composter,
                        &kind,
                        false,
                        false,
                        true,
                        false,
                        highlight,
                        scene,
                    );
                }
                AssemblyTargetAsset::Rest { rest } => {
                    let index = assembly.rotation.index();
                    let sprite = &rest.sprites.tiles[index];
                    scene.render_sprite_colored(sprite, xy(rendering_position), highlight)
                }
            }
        }

        for door in self.doors.values() {
            let mut rendering_position = rendering_position_of(door.position);
            // fix door offset
            rendering_position[1] += TILE_SIZE / 2.0;
            let mut index = door.rotation.index();
            if door.open {
                index += 4;
            }
            let sprite = &door.asset.sprites.tiles[index];
            scene.render_sprite(sprite, xy(rendering_position))
        }

        for rest in self.rests.values() {
            let rendering_position = rendering_position_of(rest.position);
            let index = rest.rotation.index();
            let sprite = &rest.asset.sprites.tiles[index];
            scene.render_sprite(sprite, xy(rendering_position))
        }

        for corpse in self.corpses.values() {
            let rendering_position = rendering_position_of(corpse.position);
            scene.render_sprite(&corpse.asset.sprite, xy(rendering_position));
        }

        for cementer in self.cementers.values() {
            render_cementer(
                cementer.position.to_tile(),
                cementer.rotation,
                &cementer.asset,
                &cementer.kind,
                cementer.enabled,
                cementer.broken,
                cementer.input,
                cementer.output,
                [1.0; 4],
                scene,
            );

            let pivot = cementer.position.to_tile();
            let input_position =
                pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.input_offset));
            let input_position = rendering_position_of(input_position.position());
            let output_position =
                pivot.add_offset(cementer.rotation.apply_i8(cementer.kind.output_offset));
            let output_position = rendering_position_of(output_position.position());
            render_items_stack(&self.items, cementer.entity.input, input_position, scene);
            render_items_stack(&self.items, cementer.entity.output, output_position, scene);
        }

        for composter in self.composters.values() {
            render_composter(
                composter.position.to_tile(),
                composter.rotation,
                &composter.asset,
                &composter.kind,
                composter.enabled,
                composter.broken,
                composter.input,
                composter.output,
                [1.0; 4],
                scene,
            );

            let pivot = composter.position.to_tile();
            let input_position =
                pivot.add_offset(composter.rotation.apply_i8(composter.kind.input_offset));
            let input_position = rendering_position_of(input_position.position());
            let output_position =
                pivot.add_offset(composter.rotation.apply_i8(composter.kind.output_offset));
            let output_position = rendering_position_of(output_position.position());
            render_items_stack(&self.items, composter.entity.input, input_position, scene);
            render_items_stack(&self.items, composter.entity.output, output_position, scene);
        }

        for farmer in self.farmers.values() {
            let rendering_position = rendering_position_of(farmer.rendering_position);
            let item_sorting = rendering_position[1] as isize;

            for (i, item) in self
                .items
                .entry(farmer.entity.backpack)
                .or_insert(HashMap::new())
                .values()
                .enumerate()
            {
                let offset = [32.0, -192.0 - (32.0 * i as f32)];
                scene.render_sprite(
                    &item.sprite(),
                    xy(rendering_position.add(offset)).sorting(item_sorting - 1),
                );
            }

            let highlight = if let Activity::Resting { .. } = farmer.activity {
                [0.8, 0.8, 1.0, 1.0]
            } else {
                [1.0; 4]
            };

            scene.render_sprite_colored(
                &self.players[farmer.entity.id - 1],
                xy(rendering_position),
                highlight,
            );

            for (i, item) in self
                .items
                .entry(farmer.entity.hands)
                .or_insert(HashMap::new())
                .values()
                .enumerate()
            {
                let offset = [0.0, -128.0 - (32.0 * i as f32)];
                scene.render_sprite(
                    &item.sprite(),
                    xy(rendering_position.add(offset)).sorting(item_sorting + 1),
                );
            }

            let last_sync_position = rendering_position_of(farmer.last_sync_position);
            scene.render_sprite_colored(&self.cursor, xy(last_sync_position), [0.5; 4]);
        }

        for stack in self.stacks.values() {
            let position = rendering_position_of(stack.position);
            scene.render_sprite(&self.stack_sprite, xy(position));
            render_items_stack(&self.items, stack.entity.container, position, scene);
        }

        for theodolite in self.theodolites.values() {
            let position = rendering_position_of(theodolite.position);
            scene.render_sprite(&theodolite.item.sprite, xy(position));
        }

        for equipment in self.equipments.values() {
            match equipment.entity.purpose {
                Purpose::Moisture { .. } => {}
                Purpose::Tethering { tether } => {
                    let position = rendering_position_of(equipment.position);
                    scene.render_sprite(&equipment.item.sprite, xy(position));

                    let start = position;
                    for creature in self.creatures.values() {
                        if creature.tether == Some(tether) {
                            let end = match creature.spine.skeleton.skeleton.find_bone("tether") {
                                None => {
                                    error!(
                                        "Can't find creature tether node of {}",
                                        creature.kind.name
                                    );
                                    rendering_position_of(creature.rendering_position)
                                }
                                Some(node) => {
                                    let node = node.applied_position();
                                    let node = [node.x, -node.y];
                                    rendering_position_of(creature.rendering_position).add(node)
                                }
                            };
                            scene.render_line(start, end, scene.rope.share());
                        }
                    }
                }
            }
        }

        for farmer in self.farmers.values() {
            if let Activity::Surveying { theodolite } = farmer.activity {
                let theodolite = self.theodolites.get(&theodolite).unwrap();
                let position = rendering_position_of(theodolite.position);
                let sorting = position[1] as isize;
                scene.render_sprite(
                    &self.theodolite_gui_sprite,
                    xy(position.add([0.0, -32.0])).sorting(sorting),
                );
                scene.render_sprite(
                    &self.theodolite_gui_select_sprite,
                    xy(position.add([-196.0 + 128.0 * (theodolite.mode as f32), -224.0]))
                        .sorting(sorting),
                );
            }
            if let Activity::Tethering { creature } = farmer.activity {
                let creature = self.creatures.get(&creature).unwrap();
                let start = rendering_position_of(farmer.rendering_position).add([20.0, -108.0]);
                let end = match creature.spine.skeleton.skeleton.find_bone("tether") {
                    None => {
                        error!("Can't find creature tether node of {}", creature.kind.name);
                        rendering_position_of(creature.rendering_position)
                    }
                    Some(node) => {
                        let node = node.applied_position();
                        let node = [node.x, -node.y];
                        rendering_position_of(creature.rendering_position).add(node)
                    }
                };
                scene.render_line(start, end, scene.rope.share());
            }
        }

        for creature in self.creatures.values() {
            scene.render_animal(
                &creature.spine,
                &creature.asset.coloration,
                rendering_position_of(creature.rendering_position),
                creature.health,
            );
        }

        for crop in self.crops.values() {
            let mut random = StdRng::seed_from_u64(crop.entity.id as u64);
            let offset_x: f32 = random.gen_range(-0.05..0.05);
            let offset_y: f32 = random.gen_range(-0.05..0.05);
            let offset = [offset_x, offset_y];
            scene.render_plant(
                &crop.spines[crop.spine()],
                crop.asset.damage_mask.share(),
                rendering_position_of(crop.position.add(offset)),
                crop.health,
                crop.thirst,
                [
                    [1.0, 1.0 - crop.thirst * 0.5, 1.0 - crop.thirst * 0.75, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ],
            );
        }
        scene.set_point_light(
            [1.0, 0.0, 0.0, 1.0],
            512.0,
            rendering_position_of(cursor.position),
        );
        scene.set_point_light([1.0, 0.0, 0.0, 1.0], 512.0, [1024.0, 256.0]);
        scene.set_point_light([0.0, 1.0, 0.0, 1.0], 512.0, [256.0, 1024.0]);
        scene.set_point_light([0.0, 0.0, 1.0, 1.0], 512.0, [1024.0, 1024.0]);

        scene.render_sprite(&self.gui_controls, xy([-512.0, 512.0]));
    }

    pub fn render_ui(&mut self, frame: &mut Frame) {
        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();
            let m = self.colonization_date.floor();
            let s = ((self.colonization_date - m) * 60.0) as u8;
            let m = m as u32;
            let cd = format!("{m}:{s:02}");
            let cd_text = frame.translator.format("CD_$1", [cd]);

            let calendar = &farmland.kind.calendar;
            let season = &calendar.seasons[farmland.season as usize];
            let d = farmland.season_day / calendar.day_duration.as_f32();
            let d = d as u8 + 1;
            let dl = season.duration.as_f32() / calendar.day_duration.as_f32();
            let dl = dl as u8;
            let season_key = format!("Season_{}_$1_of_$2", season.key);
            let season_text = frame.translator.format(&season_key, [d, dl]);

            let time_text = match farmland.times_of_day {
                value if value < 0.125 => {
                    let dt = (((value + 0.125) / 0.25) * 100.0) as u8;
                    frame.translator.format("Time_night_$1", [dt])
                }
                value if value < 0.25 => {
                    let dt = (((value - 0.125) / 0.125) * 100.0) as u8;
                    frame.translator.format("Time_morning_$1", [dt])
                }
                value if value < 0.75 => {
                    let dt = (((value - 0.25) / 0.5) * 100.0) as u8;
                    frame.translator.format("Time_day_$1", [dt])
                }
                value if value < 0.875 => {
                    let dt = (((value - 0.75) / 0.125) * 100.0) as u8;
                    frame.translator.format("Time_evening_$1", [dt])
                }
                value => {
                    let dt = (((value - 0.875) / 0.25) * 100.0) as u8;
                    frame.translator.format("Time_night_$1", [dt])
                }
            };

            // simple night overlay
            // TODO: lights system
            let alpha = match farmland.times_of_day {
                value if value < 0.25 => (1.0 - (value / 0.25)) * 0.65,
                value if value >= 0.75 => ((value - 0.75) / 0.25) * 0.65,
                _ => 0.0,
            };
            if alpha > 0.0 {
                let color = [0.07, 0.05, 0.2, alpha];
                frame.scene.render_texture(
                    frame.assets.texture_white(),
                    [0, 0],
                    frame.scene.screen.size_f32().mul(frame.scene.zoom),
                    color,
                );
            }

            self.watch_display
                .set_text(format!("{cd_text}\n{season_text}, {time_text}"));
            frame.scene.render_text(&mut self.watch_display, [16, 16]);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Neighbors {
    LeftRight,
    TopDown,
    Full,
    TopLeft,
    TopRight,
    DownRight,
    DownLeft,
    TopDownLeft,
    TopDownRight,
    DownLeftRight,
    TopLeftRight,
    //
    WindowLeftRight,
    WindowRight,
    WindowHorizontal,
    WindowLeft,
    WindowTopDown,
    WindowTop,
    WindowVertical,
    WindowDown,
    //
    DoorLeftRight,
    DoorRight,
    DoorHorizontal,
    DoorLeft,
    DoorTopDown,
    DoorTop,
    DoorVertical,
    DoorDown,
}

pub fn peek(x: isize, y: isize, cells: &Vec<Vec<Cell>>) -> (bool, bool, bool) {
    if x < 0 || x >= cells[0].len() as isize || y < 0 || y >= cells.len() as isize {
        (false, false, false)
    } else {
        let cell = &cells[y as usize][x as usize];
        (cell.wall, cell.door, cell.window)
    }
}

impl Neighbors {
    pub fn of(x: usize, y: usize, cells: &Vec<Vec<Cell>>) -> Neighbors {
        let cell = &cells[y][x];
        let (is_door, is_window) = (cell.door, cell.window);

        let x = x as isize;
        let y = y as isize;

        let (left, left_door, left_window) = peek(x - 1, y, cells);
        let (right, right_door, right_window) = peek(x + 1, y, cells);
        let (top, top_door, top_window) = peek(x, y - 1, cells);
        let (down, down_door, down_window) = peek(x, y + 1, cells);

        if is_window {
            return match (left_window, top_window, right_window, down_window) {
                (false, false, true, false) => Neighbors::WindowRight,
                (true, false, true, false) => Neighbors::WindowHorizontal,
                (true, false, false, false) => Neighbors::WindowLeft,
                (false, true, false, false) => Neighbors::WindowTop,
                (false, true, false, true) => Neighbors::WindowVertical,
                (false, false, false, true) => Neighbors::WindowDown,
                _ => {
                    if top && down {
                        Neighbors::WindowTopDown
                    } else {
                        Neighbors::WindowLeftRight
                    }
                }
            };
        }

        if is_door {
            return match (left_door, top_door, right_door, down_door) {
                (false, false, true, false) => Neighbors::DoorRight,
                (true, false, true, false) => Neighbors::DoorHorizontal,
                (true, false, false, false) => Neighbors::DoorLeft,
                (false, true, false, false) => Neighbors::DoorTop,
                (false, true, false, true) => Neighbors::DoorVertical,
                (false, false, false, true) => Neighbors::DoorDown,
                _ => {
                    if top && down {
                        Neighbors::DoorTopDown
                    } else {
                        Neighbors::DoorLeftRight
                    }
                }
            };
        }

        match (left, top, right, down) {
            (true, true, true, true) => Neighbors::Full,
            (false, true, false, true) => Neighbors::TopDown,
            (true, false, true, false) => Neighbors::LeftRight,
            (true, true, false, false) => Neighbors::TopLeft,
            (false, true, true, false) => Neighbors::TopRight,
            (false, false, true, true) => Neighbors::DownRight,
            (true, false, false, true) => Neighbors::DownLeft,
            (true, true, true, false) => Neighbors::TopLeftRight,
            (true, true, false, true) => Neighbors::TopDownLeft,
            (true, false, true, true) => Neighbors::DownLeftRight,
            (false, true, true, true) => Neighbors::TopDownRight,
            // unimplemented
            (true, false, false, false) => Neighbors::LeftRight,
            (false, true, false, false) => Neighbors::TopDown,
            (false, false, false, true) => Neighbors::TopDown,
            (false, false, true, false) => Neighbors::LeftRight,
            (false, false, false, false) => Neighbors::Full,
        }
    }

    pub fn to_tile_index(&self) -> usize {
        match self {
            Neighbors::LeftRight => 4 + 1 * 16,
            Neighbors::TopDown => 4,
            Neighbors::Full => 0 + 4 * 16,
            Neighbors::TopLeft => 1,
            Neighbors::TopRight => 0,
            Neighbors::DownRight => 2,
            Neighbors::DownLeft => 3,
            Neighbors::TopDownLeft => 1 + 1 * 16,
            Neighbors::TopDownRight => 0 + 1 * 16,
            Neighbors::DownLeftRight => 3 + 1 * 16,
            Neighbors::TopLeftRight => 2 + 1 * 16,
            //
            Neighbors::WindowLeftRight => 8 + 1 * 16,
            Neighbors::WindowRight => 9 + 1 * 16,
            Neighbors::WindowHorizontal => 10 + 1 * 16,
            Neighbors::WindowLeft => 11 + 1 * 16,
            Neighbors::WindowTopDown => 8,
            Neighbors::WindowTop => 9,
            Neighbors::WindowVertical => 10,
            Neighbors::WindowDown => 11,
            //
            Neighbors::DoorLeftRight => 12 + 1 * 16,
            Neighbors::DoorRight => 13 + 1 * 16,
            Neighbors::DoorHorizontal => 14 + 1 * 16,
            Neighbors::DoorLeft => 15 + 1 * 16,
            Neighbors::DoorTopDown => 12,
            Neighbors::DoorTop => 13,
            Neighbors::DoorVertical => 14,
            Neighbors::DoorDown => 15,
        }
    }
}

fn render_items_stack(
    items: &HashMap<ContainerId, HashMap<ItemId, ItemRep>>,
    container: ContainerId,
    center: Position,
    scene: &mut Scene,
) {
    match items.get(&container) {
        None => {}
        Some(container) => {
            for (i, item) in container.values().enumerate() {
                let offset = [
                    0.0,
                    -24.0 + (48.0 * (i % 2) as f32) - (48.0 * (i / 2) as f32),
                ];
                scene.render_sprite(
                    item.sprite(),
                    xy(center.add(offset)).sorting(center[1] as isize),
                );
            }
        }
    }
}

fn render_cementer(
    pivot: Tile,
    rotation: Rotation,
    cementer: &CementerAsset,
    kind: &CementerKind,
    enabled: bool,
    broken: bool,
    input: bool,
    output: bool,
    color: [f32; 4],
    scene: &mut Scene,
) {
    let position = position_of(pivot);
    let rendering_position = rendering_position_of(position);

    let index = if broken {
        3
    } else if enabled {
        if output || !input {
            1
        } else {
            0
        }
    } else {
        2
    };

    let sprite = &cementer.sprites.tiles[index];
    let rot = rotation.index();

    scene.render_sprite_colored(sprite, xy(rendering_position), color);

    let input_offset = rotation.apply_i8(kind.input_offset);
    let input_sprite = &cementer.sprites.tiles[4 + rot];
    let input_tile = pivot.add_offset(input_offset);
    let input_pos = rendering_position_of(input_tile.position());
    scene.render_sprite_colored(input_sprite, xy(input_pos), color);

    let output_offset = rotation.apply_i8(kind.output_offset);
    let output_sprite = &cementer.sprites.tiles[8 + rot];
    let output_tile = pivot.add_offset(output_offset);
    let output_pos = rendering_position_of(output_tile.position());
    scene.render_sprite_colored(output_sprite, xy(output_pos), color);
}

fn render_composter(
    pivot: Tile,
    rotation: Rotation,
    cementer: &ComposterAsset,
    kind: &ComposterKind,
    enabled: bool,
    broken: bool,
    input: bool,
    output: bool,
    color: [f32; 4],
    scene: &mut Scene,
) {
    let position = position_of(pivot);
    let rendering_position = rendering_position_of(position);

    let index = if broken {
        3
    } else if enabled {
        if output || !input {
            1
        } else {
            0
        }
    } else {
        2
    };

    let sprite = &cementer.sprites.tiles[index];
    let rot = rotation.index();

    scene.render_sprite_colored(sprite, xy(rendering_position), color);

    let input_offset = rotation.apply_i8(kind.input_offset);
    let input_sprite = &cementer.sprites.tiles[4 + rot];
    let input_tile = pivot.add_offset(input_offset);
    let input_pos = rendering_position_of(input_tile.position());
    scene.render_sprite_colored(input_sprite, xy(input_pos), color);

    let output_offset = rotation.apply_i8(kind.output_offset);
    let output_sprite = &cementer.sprites.tiles[8 + rot];
    let output_tile = pivot.add_offset(output_offset);
    let output_pos = rendering_position_of(output_tile.position());
    scene.render_sprite_colored(output_sprite, xy(output_pos), color);
}
