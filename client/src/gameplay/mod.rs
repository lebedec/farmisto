use crate::editor::Editor;
use crate::engine::{FarmlandAsset, Input, TreeAsset};
use crate::gameplay::camera::Camera;
use crate::{Assets, Mode, MyRenderer};
use datamap::{Known, Shared, Storage};
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{FarmlandId, FarmlandKind, TreeId, TreeKind};
use glam::{vec3, Mat4, Vec3};
use log::{error, info};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;
use std::hash::Hash;

mod camera;

pub struct Gameplay {
    server: Option<LocalServerThread>,
    editor: Option<Editor>,
    client: TcpClient,
    action_id: usize,
    storage: Storage,
    knowledge: KnowledgeBase,
    farmlands: HashMap<FarmlandId, FarmlandBehaviour>,
    trees: HashMap<TreeId, TreeBehaviour>,
    camera: Camera,
}

impl Gameplay {
    pub fn new(
        server: Option<LocalServerThread>,
        editor: Option<Editor>,
        client: TcpClient,
        viewport: [f32; 2],
    ) -> Box<Self> {
        Box::new(Self {
            server,
            editor,
            client,
            action_id: 0,
            storage: Storage::open("./assets/database.sqlite").unwrap(),
            knowledge: KnowledgeBase::default(),
            farmlands: Default::default(),
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

        if let Some(editor) = self.editor.as_mut() {
            editor.update(input, renderer, assets);
        }

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
                        }
                    }
                }
                GameResponse::Login { result } => {
                    error!("Unexpected game login response result={:?}", result);
                }
            }
        }

        self.camera.update(input);

        if input.click() {
            let (_, pos) = self.camera.cast_ray(input.mouse_position());

            if let Some(pos) = pos {
                let mut best = f32::INFINITY;
                let mut best_position = Vec3::ZERO;
                for farmland in self.farmlands.values() {
                    for transform in &farmland.asset.data.borrow().props {
                        let distance = transform.position().distance(pos);
                        if distance < best {
                            best = distance;
                            best_position = transform.position();
                        }
                    }
                }
                info!("SELECTION: {}", best_position);
            }
        }

        if input.pressed(Keycode::Kp1) {
            self.action_id += 1;
            let action = Action::DoSomething;
            self.client.send(PlayerRequest::Perform {
                action,
                action_id: self.action_id,
            })
        }

        // RENDER
        renderer.clear();
        renderer.look_at(self.camera.uniform());
        for farmland in self.farmlands.values() {
            for props in &farmland.asset.data.borrow().props {
                let matrix = Mat4::from_translation(props.position.into())
                    * Mat4::from_scale(props.scale.into())
                    // todo: rework rotation
                    * Mat4::from_rotation_x(props.rotation[0].to_radians())
                    * Mat4::from_rotation_y(props.rotation[1].to_radians())
                    * Mat4::from_rotation_z(props.rotation[2].to_radians());
                renderer.draw(matrix, props.asset.mesh(), props.asset.texture());
                renderer.bounds(matrix, props.asset.mesh().bounds());
            }
        }
        for tree in self.trees.values() {
            renderer.draw(
                Mat4::from_translation(vec3(0.0, 0.0, 0.0))
                    * Mat4::from_rotation_y(10.0_f32.to_radians()),
                tree.asset.mesh(),
                tree.asset.texture(),
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

pub struct FarmlandBehaviour {
    id: FarmlandId,
    kind: Shared<FarmlandKind>,
    asset: FarmlandAsset,
}

pub struct TreeBehaviour {
    id: TreeId,
    kind: Shared<TreeKind>,
    asset: TreeAsset,
}

#[derive(Default)]
pub struct KnowledgeBase {
    trees: Known<TreeKind>,
    farmlands: Known<FarmlandKind>,
}

impl KnowledgeBase {
    pub fn load(&mut self, storage: &Storage) {
        self.trees.load(storage);
        self.farmlands.load(storage);
    }
}
