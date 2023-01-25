use crate::engine::animatoro::{AnimationAsset, Machine, State, StateId};
use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::sprites::SpineSpriteController;
use crate::engine::{Input, SpineAsset, SpriteAsset, TextureAsset, TilesetAsset};
use crate::gameplay::behaviours::{
    BarrierHint, DropBehaviour, FarmerBehaviour, FarmlandBehaviour, TreeBehaviour,
};
use crate::gameplay::camera::Camera;
use crate::{Frame, Mode, SceneRenderer};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::{detect_collision, VectorMath};
use game::model::{Construction, Drop, Farmer, Farmland, ItemView, Position, Tile, Tree, Universe};
use game::Game;
use glam::{vec3, Mat4, Vec2, Vec3};
use log::{error, info};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;

use game::building::Building;
use game::inventory::{ContainerId, ItemId};
use game::model::Universe::{
    BarrierHintAppeared, DropAppeared, DropVanished, FarmerAppeared, FarmerVanished,
    FarmlandAppeared, FarmlandVanished, TreeAppeared, TreeVanished,
};
use game::physics::Physics;
use game::planting::Planting;
use lazy_static::lazy_static;

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

pub struct Gameplay {
    _server: Option<LocalServerThread>,
    client: TcpClient,
    action_id: usize,
    pub knowledge: Game,
    pub barriers: Vec<BarrierHint>,
    pub farmlands: HashMap<Farmland, FarmlandBehaviour>,
    pub trees: HashMap<Tree, TreeBehaviour>,
    pub farmers: HashMap<Farmer, FarmerBehaviour>,
    pub drops: HashMap<Drop, DropBehaviour>,
    pub items: HashMap<ContainerId, HashMap<ItemId, ItemView>>,
    pub camera: Camera,
    pub farmers2d: Vec<Farmer2d>,
    pub cursor: SpriteAsset,
    pub cursor_shape: usize,
    pub players: Vec<SpriteAsset>,
    pub players_index: usize,
    pub building_tiles: TilesetAsset,
    pub building_tiles_marker: TilesetAsset,
    pub roof_texture: TextureAsset,
    pub drop_sprite: SpriteAsset,
}

impl Gameplay {
    pub fn new(server: Option<LocalServerThread>, client: TcpClient, frame: &mut Frame) -> Self {
        let assets = &mut frame.assets;
        let mut camera = Camera::new();
        camera.eye = vec3(0.0, 0.0, -1.0);

        let mut knowledge = Game::new(Storage::open("./assets/database.sqlite").unwrap());
        knowledge.load_game_knowledge();
        let cursor = assets.sprite("cursor");
        let players = vec![
            assets.sprite("player"),
            assets.sprite("player-2"),
            assets.sprite("player-3"),
            assets.sprite("player-4"),
        ];

        Self {
            _server: server,
            client,
            action_id: 0,
            knowledge,
            barriers: Default::default(),
            farmlands: Default::default(),
            trees: HashMap::new(),
            farmers: Default::default(),
            drops: Default::default(),
            items: Default::default(),
            camera,
            farmers2d: vec![],
            cursor,
            cursor_shape: 0,
            players,
            building_tiles: assets.tileset("building"),
            building_tiles_marker: assets.tileset("building-marker"),
            players_index: 0,
            roof_texture: assets.texture("./assets/texture/building-roof-template-2.png"),
            drop_sprite: assets.sprite("<drop>"),
        }
    }

    pub fn handle_event(&mut self, frame: &mut Frame, event: Event) {
        match event {
            Event::Universe(events) => {
                for event in events {
                    self.handle_universe_event(frame, event);
                }
            }
            Event::Physics(events) => {}
            Event::Building(events) => {
                for event in events {
                    self.handle_building_event(frame, event);
                }
            }
            Event::Inventory(events) => {}
            Event::Planting(events) => {
                for event in events {
                    self.handle_planting_event(frame, event);
                }
            }
        }
    }

    pub fn handle_building_event(&mut self, frame: &mut Frame, event: Building) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
        match event {
            Building::GridChanged { grid, cells, rooms } => {
                for (farmland, behaviour) in self.farmlands.iter_mut() {
                    if farmland.grid == grid {
                        behaviour.cells = cells;
                        behaviour.rooms = rooms;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_planting_event(&mut self, frame: &mut Frame, event: Planting) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
        match event {
            Planting::LandChanged { land, map } => {
                for (farmland, behaviour) in self.farmlands.iter_mut() {
                    if farmland.land == land {
                        behaviour.map = map;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_physics_event(&mut self, frame: &mut Frame, event: Physics) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
        match event {
            Physics::BodyPositionChanged {
                id,
                position,
                space,
            } => {
                // FarmerMoved { id, position } => {
                //     match self.farmers.get_mut(&id) {
                //         None => {}
                //         Some(farmer) => {
                //             let position = Vec2::from(position);
                //             let rendering =
                //                 Vec2::new(farmer.rendering_position.x, farmer.rendering_position.z);
                //             // des
                //             farmer.last_sync_position = position.into();
                //             let error = position.distance(rendering);
                //             if error > 0.25 {
                //                 // info!(
                //                 //     "Correct farmer {:?} position (error {}) {} -> {}",
                //                 //     id, error, farmer.estimated_position, position
                //                 // );
                //             };
                //         }
                //     }
                // }
            }
        }
    }

    pub fn handle_universe_event(&mut self, frame: &mut Frame, event: Universe) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
        match event {
            TreeAppeared {
                tree,
                position,
                growth,
            } => {
                let kind = self
                    .knowledge
                    .universe
                    .known
                    .trees
                    .get(&tree.kind)
                    .unwrap()
                    .clone();
                info!(
                    "Appear tree {:?} kind='{}' at {:?} (g {})",
                    tree, kind.name, position, growth
                );

                let prefab = assets.tree(&kind.name);

                self.trees.insert(
                    tree,
                    TreeBehaviour {
                        tree,
                        kind,
                        asset: prefab,
                        position: Vec3::new(position[0], 0.0, position[1]),
                        direction: Vec2::ZERO,
                    },
                );
            }
            TreeVanished(id) => {
                info!("Vanish tree {:?}", id);
                self.trees.remove(&id);
                // self.barriers.remove(&id.into());
            }
            FarmlandAppeared {
                farmland,
                map,
                cells,
                rooms,
            } => {
                let kind = self
                    .knowledge
                    .universe
                    .known
                    .farmlands
                    .get(&farmland.kind)
                    .unwrap()
                    .clone();
                info!("Appear farmland {:?} kind='{}'", farmland, kind.name);

                let asset = assets.farmland(&kind.name);

                self.farmlands.insert(
                    farmland,
                    FarmlandBehaviour {
                        farmland,
                        kind,
                        asset,
                        map,
                        cells,
                        rooms,
                    },
                );
            }
            FarmlandVanished(id) => {
                info!("Vanish farmland {:?}", id);
                self.farmlands.remove(&id);
            }
            FarmerAppeared {
                farmer,
                position,
                player,
            } => {
                let kind = self
                    .knowledge
                    .universe
                    .known
                    .farmers
                    .get(&farmer.kind)
                    .unwrap()
                    .clone();
                info!(
                    "Appear farmer {:?}({}) kind='{}' at {:?}",
                    farmer, player, kind.name, position
                );

                let asset = assets.spine(&kind.name);

                info!("TEST 2d");

                let max_y = 7 * 2;
                let max_x = 14 * 2;
                let colors = [
                    [1.00, 1.00, 1.00, 1.0],
                    [0.64, 0.49, 0.40, 1.0],
                    [0.45, 0.40, 0.36, 1.0],
                    [0.80, 0.52, 0.29, 1.0],
                ];
                let pool = 256;
                let mut variant = 0;
                // for y in 0..max_y {
                //     for x in 0..max_x {
                //         let c0 = variant / 64;
                //         let c1 = (variant % 64) / 16;
                //         let c2 = (variant % 16) / 4;
                //         let c3 = variant % 4;
                //         variant = ((5 * variant) + 1) % pool;
                //         let asset = asset.share();
                //         let variant = x + y * max_x;
                //         let head = format!("head/head-{}", variant % 4);
                //         let tile = format!("tail/tail-{}", variant % 3);
                //         let sprite = frame.sprites.instantiate(
                //             &asset,
                //             [head, tile],
                //             [colors[c0], colors[c1], colors[c2], colors[c3]],
                //         );
                //         let position = [
                //             64.0 + 128.0 + 128.0 * x as f32,
                //             64.0 + 256.0 + 128.0 * y as f32,
                //         ];
                //         self.farmers2d.push(Farmer2d {
                //             asset,
                //             sprite,
                //             position,
                //             variant,
                //         });
                //     }
                // }

                let asset = assets.farmer(&kind.name);

                info!("Mesh bounds: {:?}", asset.mesh.bounds());

                self.farmers.insert(
                    farmer,
                    FarmerBehaviour {
                        farmer,
                        kind,
                        player,
                        asset,
                        estimated_position: position.into(),
                        rendering_position: Vec3::new(position[0], 0.0, position[1]),
                        last_sync_position: position.into(),
                        direction: Vec2::ZERO,
                        machine: Machine {
                            parameters: Default::default(),
                            states: vec![State {
                                id: StateId(42),
                                name: "idle".to_string(),
                                fps: 10.0,
                                motion: AnimationAsset::from_space3(
                                    "./assets/mesh/male@idle.space3",
                                ),
                                looped: true,
                                frame: 0,
                                frame_time: 0.0,
                                transitions: vec![],
                            }],
                            current: 0,
                            transform: [Mat4::IDENTITY; 64],
                            pose_buffer: PoseBuffer::create::<PoseUniform>(
                                renderer.device.clone(),
                                &renderer.device_memory,
                                1,
                            ),
                        },
                    },
                );
            }
            FarmerVanished(id) => {
                info!("Vanish farmer {:?}", id);
                self.farmers.remove(&id);
            }
            BarrierHintAppeared {
                id,
                kind,
                position,
                bounds,
            } => {
                self.barriers.push(BarrierHint {
                    id,
                    kind,
                    position,
                    bounds,
                });
            }
            DropAppeared {
                drop,
                position,
                items,
            } => {
                info!("Appear drop {:?} at {:?}", drop, position,);
                self.drops.insert(drop, DropBehaviour { drop, position });
                for item in items {
                    let container = self.items.entry(item.container).or_insert(HashMap::new());
                    container.insert(item.id, item);
                }
            }
            DropVanished(drop) => {
                self.drops.remove(&drop);
            }
        }
    }

    pub fn handle_server_responses(&mut self, frame: &mut Frame) {
        let responses: Vec<GameResponse> = self.client.responses().collect();
        for response in responses {
            match response {
                GameResponse::Heartbeat => {}
                GameResponse::Events { events } => {
                    for event in events {
                        self.handle_event(frame, event);
                    }
                }
                GameResponse::Login { result } => {
                    error!("Unexpected game login response result={:?}", result);
                }
                GameResponse::ActionError { action_id, error } => {
                    error!("Action {} error {:?}", action_id, error);
                }
            }
        }
    }

    fn send_action(&mut self, action: Action) {
        self.action_id += 1;
        self.client.send(PlayerRequest::Perform {
            action,
            action_id: self.action_id,
        })
    }

    pub fn handle_user_input(&mut self, input: &Input) {
        self.camera.update(input);

        let mouse_position = input
            .mouse_position()
            .position
            .add([self.camera.eye.x, -self.camera.eye.y]);
        let cursor_position = Position {
            x: mouse_position[0],
            y: mouse_position[1],
        };
        let cursor = Tile {
            x: (mouse_position[0] / 128.0).floor() as usize,
            y: (mouse_position[1] / 128.0).floor() as usize,
        };

        if input.pressed(Keycode::Kp1) {
            self.send_action(Action::DoSomething);
        }
        if input.pressed(Keycode::Tab) {
            self.players_index = (self.players_index + 1) % self.players.len();
        }
        if input.left_click() {
            let farmland = self.farmlands.values().nth(0).unwrap();
            let row = cursor.y.max(127);
            let column = cursor.x.max(127);
            if !farmland.cells[row][column].marker && !farmland.cells[row][column].wall {
                let action = Action::Survey { target: cursor };
                self.send_action(action);
            }
        }
        if let Some(farmer) = self
            .farmers
            .values_mut()
            .find(|farmer| farmer.player == self.client.player)
        {
            let mut direction = Vec2::ZERO;
            if input.down(Keycode::Left) {
                direction.x -= 1.0;
            }
            if input.down(Keycode::Right) {
                direction.x += 1.0;
            }
            if input.down(Keycode::Up) {
                direction.y += 1.0;
            }
            if input.down(Keycode::Down) {
                direction.y -= 1.0;
            }
            let delta = direction.normalize_or_zero() * input.time * 7.0;
            let destination =
                delta + Vec2::new(farmer.rendering_position.x, farmer.rendering_position.z);

            // client side physics pre-calculation to prevent
            // obvious movement errors
            if let Some(destination) = detect_collision(farmer, destination.into(), &self.barriers)
            {
                farmer.estimated_position = Vec2::from(destination);
                if delta.length() > 0.0 {
                    self.client.send(PlayerRequest::Perform {
                        action_id: 0,
                        action: Action::MoveFarmer { destination },
                    })
                }
            }
        }
    }

    pub fn animate(&mut self, input: &Input) {
        for farmer in self.farmers.values_mut() {
            farmer.machine.update(input.time);
            farmer.rendering_position = Vec3::new(
                farmer.estimated_position.x,
                0.0,
                farmer.estimated_position.y,
            );
        }
        METRIC_ANIMATION_SECONDS.observe_closure_duration(|| {
            for farmer in self.farmers2d.iter_mut() {
                farmer.sprite.skeleton.update(input.time);
            }
        });
    }

    pub fn render2d(&mut self, frame: &mut Frame) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.sprites;
        renderer.clear();
        renderer.look_at(self.camera.eye);

        let mouse_position = frame
            .input
            .mouse_position()
            .position
            .add([self.camera.eye.x, -self.camera.eye.y]);
        let [mouse_x, mouse_y] = mouse_position;
        let cursor_x = (mouse_x / 128.0).floor() as usize;
        let cursor_y = (mouse_y / 128.0).floor() as usize;

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
            renderer.render_floor(
                self.roof_texture.clone(),
                farmland.asset.sampler.share(),
                &farmland.map,
                &farmland.rooms,
            );
            renderer.render_roof(
                self.roof_texture.clone(),
                farmland.asset.sampler.share(),
                &farmland.map,
                &farmland.rooms,
                self.cursor_shape,
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

                        let tileset = if cell.marker {
                            &self.building_tiles_marker.tiles
                        } else {
                            &self.building_tiles.tiles
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
                        let position = [x as f32 * 128.0, 128.0 + y as f32 * 128.0];
                        renderer.render_sprite(
                            tile,
                            position,
                            (position[1] / 128.0) as usize,
                            highlight,
                        );
                    }

                    if x == cursor_x && y == cursor_y {
                        let position = [mouse_x, mouse_y];
                        renderer.render_sprite(
                            &self.players[self.players_index],
                            position,
                            (position[1] / 128.0) as usize,
                            1.0,
                        );
                    }
                }
            }
        }
        let cursor_x = cursor_x as f32 * 128.0 + 64.0;
        let cursor_y = cursor_y as f32 * 128.0 + 64.0;
        let position = [cursor_x, cursor_y];
        renderer.render_sprite(&self.cursor, position, (position[1] / 128.0) as usize, 1.0);

        for drop in self.drops.values() {
            let sprite_line = (drop.position[1] / 128.0) as usize;
            renderer.render_sprite(&self.drop_sprite, drop.position, sprite_line, 1.0);
            for (i, item) in self
                .items
                .get(&drop.drop.container)
                .unwrap()
                .values()
                .enumerate()
            {
                let kind = self
                    .knowledge
                    .inventory
                    .known_items
                    .get(&item.kind)
                    .unwrap();
                let asset = assets.item(&kind.name);
                let offset = [
                    0.0,
                    -24.0 + (48.0 * (i % 2) as f32) - (48.0 * (i / 2) as f32),
                ];
                renderer.render_sprite(&asset.sprite, drop.position.add(offset), sprite_line, 1.0);
            }
        }

        METRIC_DRAW_REQUEST_SECONDS.observe_closure_duration(|| {
            for farmer in &self.farmers2d {
                renderer.render_spine(&farmer.sprite, farmer.position, [mouse_x, mouse_y]);
            }
        });

        renderer.set_point_light([1.0, 0.0, 0.0, 1.0], 512.0, mouse_position);

        renderer.set_point_light([1.0, 0.0, 0.0, 1.0], 512.0, [1024.0, 256.0]);
        renderer.set_point_light([0.0, 1.0, 0.0, 1.0], 512.0, [256.0, 1024.0]);
        renderer.set_point_light([0.0, 0.0, 1.0, 1.0], 512.0, [1024.0, 1024.0]);
    }

    pub fn render(&self, renderer: &mut SceneRenderer) {
        renderer.clear();
        renderer.look_at(self.camera.uniform(
            renderer.screen.width() as f32,
            renderer.screen.height() as f32,
        ));
        for farmland in self.farmlands.values() {
            for props in &farmland.asset.props {
                let matrix = Mat4::from_translation(props.position.into())
                    * Mat4::from_scale(props.scale.into())
                    // todo: rework rotation
                    * Mat4::from_rotation_x(props.rotation[0].to_radians())
                    * Mat4::from_rotation_y(props.rotation[1].to_radians())
                    * Mat4::from_rotation_z(props.rotation[2].to_radians());
                renderer.draw(matrix, &props.asset.mesh, &props.asset.texture);
            }
        }
        for tree in self.trees.values() {
            renderer.draw(
                Mat4::from_translation(tree.position),
                &tree.asset.mesh,
                &tree.asset.texture,
            );
        }
        for farmer in self.farmers.values() {
            renderer.animate(
                Mat4::from_translation(farmer.rendering_position),
                &farmer.asset.mesh,
                &farmer.asset.texture,
                &farmer.machine.pose_buffer,
            );
        }
    }
}

impl Mode for Gameplay {
    fn update(&mut self, frame: &mut Frame) {
        self.handle_server_responses(frame);
        self.handle_user_input(&frame.input);
        self.animate(&frame.input);
        // self.render(frame.scene);
        self.render2d(frame);
    }
}

pub struct Farmer2d {
    pub asset: SpineAsset,
    pub sprite: SpineSpriteController,
    pub position: [f32; 2],
    pub variant: u32,
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
