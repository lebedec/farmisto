use crate::engine::Input;
use crate::gameplay::camera::Camera;
use crate::gameplay::objects::{FarmerBehaviour, FarmlandBehaviour, KnowledgeBase, TreeBehaviour};
use crate::{Assets, Mode, MyRenderer};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{FarmerId, FarmlandId, TreeId};
use glam::{Mat4, Vec3};
use log::{error, info};
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
                                    },
                                );
                            }
                            Event::TreeVanished(id) => {
                                info!("Vanish tree {:?}", id);
                                self.trees.remove(&id);
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
                            Event::FarmerAppeared { id, kind, position } => {
                                let kind = self.knowledge.farmers.get(kind).unwrap();
                                info!(
                                    "Appear farmer {:?} kind='{}' at {:?}",
                                    id, kind.name, position
                                );

                                let asset = assets.farmer(&kind.name);

                                self.farmers.insert(
                                    id,
                                    FarmerBehaviour {
                                        id,
                                        kind,
                                        asset,
                                        considered_position: position.into(),
                                        position: Vec3::new(position[0], 0.0, position[1]),
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
                                    Some(farmer) => farmer.considered_position = position.into(),
                                }
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
    }

    pub fn render(&self, renderer: &mut MyRenderer) {
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
        for farmer in self.farmers.values() {
            renderer.draw(
                Mat4::from_translation(farmer.position),
                &farmer.asset.mesh,
                &farmer.asset.texture,
            )
        }
    }
}

impl Mode for Gameplay {
    fn start(&mut self, assets: &mut Assets) {
        self.knowledge.reload();
    }

    fn update(&mut self, input: &Input, renderer: &mut MyRenderer, assets: &mut Assets) {
        self.handle_server_responses(assets);
        self.handle_user_input(input);
        self.render(renderer);
    }
}
