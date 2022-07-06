use crate::engine::{Input, MeshAsset, TextureAsset, Transform, TreePrefab};
use crate::gameplay::camera::Camera;
use crate::{Assets, Mode, MyRenderer};
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{TreeId, TreeKind};
use game::persistence::{Known, Shared, Storage};
use glam::{vec3, Mat4};
use log::{error, info};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;
use std::path::PathBuf;

mod camera;

pub struct Gameplay {
    server: Option<LocalServerThread>,
    client: TcpClient,
    action_id: usize,
    storage: Storage,
    knowledge: KnowledgeBase,
    trees: HashMap<TreeId, TreeBehaviour>,
    camera: Camera,
}

impl Gameplay {
    pub fn new(
        server: Option<LocalServerThread>,
        client: TcpClient,
        viewport: [f32; 2],
    ) -> Box<Self> {
        Box::new(Self {
            server,
            client,
            action_id: 0,
            storage: Storage::open("./assets/database.sqlite").unwrap(),
            knowledge: KnowledgeBase::default(),
            trees: HashMap::new(),
            camera: Camera::new(viewport),
        })
    }
}

impl Mode for Gameplay {
    // fn start(&mut self, assets: &mut Assets) {
    //     self.tree_tex = Some((
    //         assets.mesh("./assets/viking_room.space3"),
    //         assets.texture("./assets/viking_room.png"),
    //     ));
    // }

    fn update(&mut self, input: &Input, renderer: &mut MyRenderer, assets: &mut Assets) {
        self.knowledge.load(&self.storage);

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

                                let path = PathBuf::from(&kind.name).with_extension("yaml");
                                let path = PathBuf::from("./assets/trees").join(path);
                                let prefab = assets.tree(path);

                                self.trees.insert(id, TreeBehaviour { id, kind, prefab });
                            }
                            Event::TreeVanished(id) => {
                                info!("Vanish tree {:?}", id);
                                self.trees.remove(&id);
                            }
                            Event::TreeUpdated { id } => {
                                info!("Update tree {:?} [not implemented yet]", id);
                            }
                        }
                    }
                }
                GameResponse::Login { result } => {
                    error!("Unexpected game login response result={:?}", result);
                }
            }
        }

        self.camera.update(input);

        if input.pressed(Keycode::Kp1) {
            self.action_id += 1;
            let action = Action::DoSomething;
            self.client.send(PlayerRequest::Perform {
                action,
                action_id: self.action_id,
            })
        }

        if input.pressed(Keycode::P) {
            self.client.disconnect();
        }

        if input.pressed(Keycode::O) {
            if let Some(thread) = self.server.as_mut() {
                thread.terminate();
            }
        }

        // RENDER
        renderer.clear();
        renderer.look_at(self.camera.uniform());
        for tree in self.trees.values() {
            renderer.draw(
                Transform {
                    matrix: Mat4::from_translation(vec3(0.0, 0.0, 0.0))
                        * Mat4::from_rotation_y(10.0_f32.to_radians()),
                },
                tree.prefab.mesh(),
                tree.prefab.texture(),
            );
            // for x in 0..1000 {
            //     let x1 = x % 100;
            //     let y1 = x / 100;
            //     let x1 = x1 as f32;
            //     let y1 = y1 as f32;
            //     renderer.draw(
            //         Transform {
            //             matrix: Mat4::from_translation(vec3(x1, 0.0, y1))
            //                 * Mat4::from_rotation_y(10.0_f32.to_radians()),
            //         },
            //         tree.prefab.mesh(),
            //         tree.prefab.texture(),
            //     );
            // }
        }
    }
}

pub struct TreeBehaviour {
    id: TreeId,
    kind: Shared<TreeKind>,
    prefab: TreePrefab,
}

#[derive(Default)]
pub struct KnowledgeBase {
    trees: Known<TreeKind>,
}

impl KnowledgeBase {
    pub fn load(&mut self, storage: &Storage) {
        self.trees.load(storage);
    }
}
