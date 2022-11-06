use crate::engine::animatoro::{AnimationAsset, Machine, State, StateId};
use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::sprites::SpriteRenderer;
use crate::engine::Input;
use crate::gameplay::camera::Camera;
use crate::gameplay::objects::{BarrierHint, FarmerBehaviour, FarmlandBehaviour, TreeBehaviour};
use crate::{Assets, Frame, Mode, SceneRenderer};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::detect_collision;
use game::model::{FarmerId, FarmlandId, TreeId};
use game::Game;
use glam::{Mat4, Vec2, Vec3};
use log::{error, info};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;

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
        }
    }

    pub fn handle_server_responses(&mut self, assets: &mut Assets, renderer: &mut SceneRenderer) {
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
                            Event::FarmlandAppeared { id, kind } => {
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

                                self.farmlands
                                    .insert(id, FarmlandBehaviour { id, kind, asset });
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
    }

    pub fn render2d(&self, renderer: &mut SpriteRenderer, assets: &mut Assets) {
        renderer.clear();
        renderer.look_at();
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
        }
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
                42,
                &farmer.machine.pose_buffer,
            );
            renderer.bounds(
                Mat4::from_translation(Vec3::new(
                    farmer.last_sync_position.x,
                    0.5,
                    farmer.last_sync_position.y,
                )),
                farmer.asset.mesh.bounds(),
            );
        }
    }
}

impl Mode for Gameplay {
    fn start(&mut self, _assets: &mut Assets) {
        self.knowledge.load_game_knowledge();
    }

    fn update(&mut self, frame: Frame) {
        self.handle_server_responses(frame.assets, frame.scene);
        self.handle_user_input(&frame.input);
        self.animate(&frame.input);
        self.render(frame.scene);
        self.render2d(frame.sprites, frame.assets);
    }
}
