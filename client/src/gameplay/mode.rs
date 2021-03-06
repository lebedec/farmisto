use crate::engine::Input;
use crate::gameplay::camera::Camera;
use crate::gameplay::objects::{
    BarrierHint, FarmerBehaviour, FarmlandBehaviour, KnowledgeBase, TreeBehaviour,
};
use crate::{Assets, Mode, SceneRenderer};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::{detect_collision, Collider};
use game::model::{FarmerId, FarmlandId, TreeId};
use glam::{Mat4, Vec2, Vec3};
use log::{error, info, warn};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Gameplay {
    server: Option<LocalServerThread>,
    client: TcpClient,
    action_id: usize,
    pub knowledge: KnowledgeBase,
    pub barriers: Vec<BarrierHint>,
    pub farmlands: HashMap<FarmlandId, FarmlandBehaviour>,
    pub trees: HashMap<TreeId, TreeBehaviour>,
    pub farmers: HashMap<FarmerId, FarmerBehaviour>,
    pub camera: Camera,
}

impl Gameplay {
    pub fn new(server: Option<LocalServerThread>, client: TcpClient, viewport: [f32; 2]) -> Self {
        Self {
            server,
            client,
            action_id: 0,
            knowledge: KnowledgeBase::new(),
            barriers: Default::default(),
            farmlands: Default::default(),
            trees: HashMap::new(),
            farmers: Default::default(),
            camera: Camera::new(viewport),
        }
    }

    pub fn handle_server_responses(&mut self, assets: &mut Assets) {
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
                                let kind = self.knowledge.trees.get(kind).unwrap();
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
                                let kind = self.knowledge.farmlands.get(kind).unwrap();
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
                                let kind = self.knowledge.farmers.get(kind).unwrap();
                                info!(
                                    "Appear farmer {:?}({}) kind='{}' at {:?}",
                                    id, player, kind.name, position
                                );

                                let asset = assets.farmer(&kind.name);

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

    pub fn render(&mut self, renderer: &mut SceneRenderer) {
        renderer.clear();
        renderer.look_at(self.camera.uniform());
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
        for farmer in self.farmers.values_mut() {
            farmer.rendering_position = Vec3::new(
                farmer.estimated_position.x,
                0.0,
                farmer.estimated_position.y,
            );

            renderer.draw(
                Mat4::from_translation(farmer.rendering_position),
                &farmer.asset.mesh,
                &farmer.asset.texture,
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
    fn start(&mut self, assets: &mut Assets) {
        self.knowledge.reload();
    }

    fn update(&mut self, input: &Input, renderer: &mut SceneRenderer, assets: &mut Assets) {
        self.handle_server_responses(assets);
        self.handle_user_input(input);
        self.render(renderer);
    }
}
