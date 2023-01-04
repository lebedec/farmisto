use crate::engine::animatoro::{AnimationAsset, Machine, State, StateId};
use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::sprites::{SpineSpriteController, SpriteRenderer};
use crate::engine::{Input, SpineAsset, SpriteAsset, TextureAsset};
use crate::gameplay::camera::Camera;
use crate::gameplay::objects::{BarrierHint, FarmerBehaviour, FarmlandBehaviour, TreeBehaviour};
use crate::{Assets, Frame, Mode, SceneRenderer};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::{detect_collision, VectorMath};
use game::model::{FarmerId, FarmlandId, TreeId};
use game::Game;
use glam::{Mat4, Vec2, Vec3};
use log::{error, info};
use network::TcpClient;
use rusty_spine::controller::SkeletonController;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;
use std::time::Instant;

use prometheus::{Counter, Histogram, IntCounter, IntGauge};

use lazy_static::lazy_static;
use prometheus::{register_counter, register_histogram, register_int_counter, register_int_gauge};
use sdl2::sys::rand;

lazy_static! {
    static ref METRIC_ANIMATION_SECONDS: Histogram =
        register_histogram!("gameplay_animation_seconds", "gameplay_animation_seconds").unwrap();
    static ref METRIC_DRAW_REQUEST_SECONDS: Histogram = register_histogram!(
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
    pub farmlands: HashMap<FarmlandId, FarmlandBehaviour>,
    pub trees: HashMap<TreeId, TreeBehaviour>,
    pub farmers: HashMap<FarmerId, FarmerBehaviour>,
    pub camera: Camera,
    pub farmers2d: Vec<Farmer2d>,
    pub backgrounds: Vec<SpriteAsset>,
}

pub struct Farmer2d {
    pub asset: SpineAsset,
    pub sprite: SpineSpriteController,
    pub position: [f32; 2],
    pub variant: u32,
}

impl Gameplay {
    pub fn new(server: Option<LocalServerThread>, client: TcpClient) -> Self {
        Self {
            _server: server,
            client,
            action_id: 0,
            knowledge: Game::new(Storage::open("./assets/database.sqlite").unwrap()),
            barriers: Default::default(),
            farmlands: Default::default(),
            trees: HashMap::new(),
            farmers: Default::default(),
            camera: Camera::new(),
            farmers2d: vec![],
            backgrounds: vec![],
        }
    }

    pub fn handle_server_responses(&mut self, frame: &mut Frame) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
        for response in self.client.responses() {
            match response {
                GameResponse::Heartbeat => {}
                GameResponse::Events { events } => {
                    for event in events {
                        match event {
                            Event::TreeAppeared {
                                id,
                                kind,
                                position,
                                growth,
                            } => {
                                let kind = self
                                    .knowledge
                                    .universe
                                    .known
                                    .trees
                                    .get(&kind)
                                    .unwrap()
                                    .clone();
                                info!(
                                    "Appear tree {:?} kind='{}' at {:?} (g {})",
                                    id, kind.name, position, growth
                                );

                                let prefab = assets.tree(&kind.name);

                                self.trees.insert(
                                    id,
                                    TreeBehaviour {
                                        id,
                                        kind,
                                        asset: prefab,
                                        position: Vec3::new(position[0], 0.0, position[1]),
                                        direction: Vec2::ZERO,
                                    },
                                );
                            }
                            Event::TreeVanished(id) => {
                                info!("Vanish tree {:?}", id);
                                self.trees.remove(&id);
                                // self.barriers.remove(&id.into());
                            }
                            Event::TreeUpdated { id } => {
                                info!("Update tree {:?} [not implemented yet]", id);
                            }
                            Event::FarmlandAppeared { id, kind, map } => {
                                let kind = self
                                    .knowledge
                                    .universe
                                    .known
                                    .farmlands
                                    .get(&kind)
                                    .unwrap()
                                    .clone();
                                info!("Appear farmland {:?} kind='{}'", id, kind.name);

                                let asset = assets.farmland(&kind.name);

                                self.farmlands.insert(
                                    id,
                                    FarmlandBehaviour {
                                        id,
                                        kind,
                                        asset,
                                        map,
                                    },
                                );
                            }
                            Event::FarmlandUpdated { id, map } => {
                                let farmland = self.farmlands.get_mut(&id).unwrap();
                                farmland.map = map;
                            }
                            Event::FarmlandVanished(id) => {
                                info!("Vanish farmland {:?}", id);
                                self.farmlands.remove(&id);
                            }
                            Event::FarmerAppeared {
                                id,
                                kind,
                                position,
                                player,
                            } => {
                                let kind = self
                                    .knowledge
                                    .universe
                                    .known
                                    .farmers
                                    .get(&kind)
                                    .unwrap()
                                    .clone();
                                info!(
                                    "Appear farmer {:?}({}) kind='{}' at {:?}",
                                    id, player, kind.name, position
                                );

                                let asset = assets.spine(&kind.name);

                                info!("TEST 2d");

                                let max_y = 8;
                                let max_x = 14;
                                let colors = [
                                    [1.00, 1.00, 1.00, 1.0],
                                    [0.64, 0.49, 0.40, 1.0],
                                    [0.45, 0.40, 0.36, 1.0],
                                    [0.80, 0.52, 0.29, 1.0],
                                ];
                                let pool = 256;
                                let mut variant = 0;
                                for y in 0..max_y {
                                    for x in 0..max_x {
                                        let c0 = variant / 64;
                                        let c1 = (variant % 64) / 16;
                                        let c2 = (variant % 16) / 4;
                                        let c3 = variant % 4;
                                        variant = ((5 * variant) + 1) % pool;
                                        let asset = asset.share();
                                        let variant = x + y * max_x;
                                        let head = format!("head/head-{}", variant % 4);
                                        let tile = format!("tail/tail-{}", variant % 3);
                                        let sprite = frame.sprites.instantiate(
                                            &asset,
                                            [head, tile],
                                            [colors[c0], colors[c1], colors[c2], colors[c3]],
                                        );
                                        let position = [
                                            64.0 + 128.0 + 256.0 * x as f32,
                                            64.0 + 256.0 + 256.0 * y as f32,
                                        ];
                                        self.farmers2d.push(Farmer2d {
                                            asset,
                                            sprite,
                                            position,
                                            variant,
                                        });
                                    }
                                }

                                let asset = assets.farmer(&kind.name);

                                info!("Mesh bounds: {:?}", asset.mesh.bounds());

                                self.farmers.insert(
                                    id,
                                    FarmerBehaviour {
                                        id,
                                        kind,
                                        player,
                                        asset,
                                        estimated_position: position.into(),
                                        rendering_position: Vec3::new(
                                            position[0],
                                            0.0,
                                            position[1],
                                        ),
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
                            Event::FarmerVanished(id) => {
                                info!("Vanish farmer {:?}", id);
                                self.farmers.remove(&id);
                            }
                            Event::FarmerMoved { id, position } => {
                                match self.farmers.get_mut(&id) {
                                    None => {}
                                    Some(farmer) => {
                                        let position = Vec2::from(position);
                                        let rendering = Vec2::new(
                                            farmer.rendering_position.x,
                                            farmer.rendering_position.z,
                                        );
                                        // des
                                        farmer.last_sync_position = position.into();
                                        let error = position.distance(rendering);
                                        if error > 0.25 {
                                            // info!(
                                            //     "Correct farmer {:?} position (error {}) {} -> {}",
                                            //     id, error, farmer.estimated_position, position
                                            // );
                                        };
                                    }
                                }
                            }
                            Event::BarrierHintAppeared {
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
                        }
                    }
                }
                GameResponse::Login { result } => {
                    error!("Unexpected game login response result={:?}", result);
                }
            }
        }
    }

    pub fn handle_user_input(&mut self, input: &Input) {
        self.camera.update(input);
        if input.pressed(Keycode::Kp1) {
            self.action_id += 1;
            let action = Action::DoSomething;
            self.client.send(PlayerRequest::Perform {
                action,
                action_id: self.action_id,
            })
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
                // if farmer.variant < 10 {
                //     let scale = farmer.variant as f32 / 10.0;
                //     let mut bone = farmer
                //         .sprite
                //         .skeleton
                //         .skeleton
                //         .find_bone_mut("root")
                //         .unwrap();
                //     bone.set_scale([0.75 + scale * 0.5, 0.75 + scale * 0.5]);
                // }
                farmer.sprite.skeleton.update(input.time);
            }
        });
    }

    pub fn render2d(&self, renderer: &mut SpriteRenderer, assets: &mut Assets) {
        renderer.clear();
        renderer.look_at();
        // for background in &self.backgrounds {
        //     renderer.draw_sprite(&background, background.size.mul(0.5));
        // }
        /*
        renderer.draw(&assets.sprite("test"), [512.0, 512.0]);
        let mut trees: Vec<&TreeBehaviour> = self.trees.values().collect();
        trees.sort_by_key(|tree| tree.position.z as i32);
        for tree in trees {
            let sprite = assets.sprite(&tree.kind.name);
            let position = [
                512.0 + tree.position.x * 32.0,
                512.0 + tree.position.z * 32.0,
            ];
            renderer.draw(&sprite, position)
        }*/
        for farmland in self.farmlands.values() {
            renderer.draw_ground(
                farmland.asset.texture.clone(),
                farmland.asset.sampler.share(),
                &farmland.map,
            )
        }
        METRIC_DRAW_REQUEST_SECONDS.observe_closure_duration(|| {
            for farmer in &self.farmers2d {
                renderer.draw_spine(&farmer.sprite, farmer.position);
            }
        });
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
    fn start(&mut self, assets: &mut Assets) {
        self.knowledge.load_game_knowledge();
        self.backgrounds.push(assets.sprite("bg128.png"));
        //self.backgrounds.push(assets.sprite("lama-ref-384.png"));
    }

    fn update(&mut self, mut frame: Frame) {
        self.handle_server_responses(&mut frame);
        self.handle_user_input(&frame.input);
        self.animate(&frame.input);
        self.render(frame.scene);
        self.render2d(frame.sprites, frame.assets);
    }
}
