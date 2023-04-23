use std::collections::HashMap;
use std::fmt::format;
use std::time::Instant;

use glam::vec3;
use log::{error, info};
use sdl2::keyboard::Keycode;

use ai::AiThread;
use datamap::Storage;
use game::api::{Action, FarmerBound, GameResponse, PlayerRequest};
use game::inventory::{ContainerId, ItemId};
use game::math::{test_collisions, VectorMath};
use game::model::Crop;
use game::model::Equipment;
use game::model::Farmer;
use game::model::Farmland;
use game::model::Knowledge;
use game::model::Stack;
use game::model::Tree;
use game::model::{Activity, Assembly, Door};
use game::model::{Cementer, Construction};
use game::model::{Creature, Rest};
use game::physics::{generate_holes, Barrier};
use game::Game;
use network::TcpClient;
use server::LocalServerThread;

use crate::assets::{SamplerAsset, SpriteAsset, TextureAsset};
use crate::engine::rendering::TextController;
use crate::engine::Input;
use crate::gameplay::camera::Camera;
use crate::gameplay::representation::{
    AssemblyRep, CementerRep, ConstructionRep, CreatureRep, CropRep, DoorRep, EquipmentRep,
    FarmerRep, FarmlandRep, ItemRep, RestRep, StackRep, TreeRep,
};
use crate::gameplay::{GameplayMetrics, InputMethod, Target};
use crate::monitoring::{Timer, TimerIntegration};
use crate::{Frame, Mode};

pub const TILE_SIZE: f32 = 128.0;

#[inline]
pub fn rendering_position_of(position: [f32; 2]) -> [f32; 2] {
    position.mul(TILE_SIZE)
}

#[inline]
pub fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}

pub struct Host {
    pub server: LocalServerThread,
    pub ai: AiThread,
}

pub struct Gameplay {
    pub host: Option<Host>,
    pub client: TcpClient,
    pub action_id: usize,
    pub sent_actions: HashMap<usize, FarmerBound>,
    pub known: Knowledge,
    pub barriers_hint: Vec<Barrier>,
    pub farmlands: HashMap<Farmland, FarmlandRep>,
    pub current_farmland: Option<Farmland>,
    pub trees: HashMap<Tree, TreeRep>,
    pub farmers: HashMap<Farmer, FarmerRep>,
    pub stacks: HashMap<Stack, StackRep>,
    pub equipments: HashMap<Equipment, EquipmentRep>,
    pub assembly: HashMap<Assembly, AssemblyRep>,
    pub doors: HashMap<Door, DoorRep>,
    pub rests: HashMap<Rest, RestRep>,
    pub cementers: HashMap<Cementer, CementerRep>,
    pub constructions: HashMap<Construction, ConstructionRep>,
    pub crops: HashMap<Crop, CropRep>,
    pub creatures: HashMap<Creature, CreatureRep>,
    pub items: HashMap<ContainerId, HashMap<ItemId, ItemRep>>,
    pub camera: Camera,
    pub cursor: SpriteAsset,
    pub players: Vec<SpriteAsset>,
    pub players_index: usize,
    pub roof_texture: TextureAsset,
    pub stack_sprite: SpriteAsset,
    pub theodolite_sprite: SpriteAsset,
    pub theodolite_gui_sprite: SpriteAsset,
    pub theodolite_gui_select_sprite: SpriteAsset,
    pub gui_controls: SpriteAsset,
    pub watch_display: TextController,
    pub colonization_date: f32,
    pub speed: f32,
    pub default_sampler: SamplerAsset,
    pub metrics: GameplayMetrics,
}

impl Gameplay {
    pub fn new(
        host: Option<Host>,
        client: TcpClient,
        frame: &mut Frame,
        metrics: GameplayMetrics,
    ) -> Self {
        let assets = &mut frame.assets;
        let mut camera = Camera::new();
        camera.eye = vec3(0.0, 0.0, -1.0);

        let mut knowledge = Game::new(Storage::open(&frame.config.save_file).unwrap());
        knowledge.load_game_knowledge().unwrap();
        let knowledge = knowledge.known;

        let cursor = assets.sprite("cursor");
        let players = vec![
            assets.sprite("player-Alice"),
            assets.sprite("player-Boris"),
            assets.sprite("player-Carol"),
            assets.sprite("player-David"),
        ];

        let test_text = frame.scene.instantiate_text(
            1024,
            128,
            String::from("Hello 0!"),
            assets.fonts_default.share(),
            frame.scene.ui_element_sampler.share(),
        );

        Self {
            host,
            client,
            action_id: 0,
            sent_actions: Default::default(),
            known: knowledge,
            barriers_hint: Default::default(),
            farmlands: Default::default(),
            current_farmland: None,
            trees: HashMap::new(),
            farmers: Default::default(),
            stacks: Default::default(),
            equipments: Default::default(),
            assembly: Default::default(),
            doors: Default::default(),
            rests: Default::default(),
            cementers: Default::default(),
            constructions: Default::default(),
            crops: Default::default(),
            creatures: Default::default(),
            items: Default::default(),
            camera,
            cursor,
            players,
            players_index: 0,
            roof_texture: assets.texture("./assets/texture/building-roof-template-2.png"),
            stack_sprite: assets.sprite("<drop>"),
            theodolite_sprite: assets.sprite("theodolite"),
            theodolite_gui_sprite: assets.sprite("building-gui"),
            theodolite_gui_select_sprite: assets.sprite("building-gui-select"),
            gui_controls: assets.sprite("gui-controls"),
            watch_display: test_text,
            colonization_date: 0.0,
            speed: 0.0,
            default_sampler: assets.sampler("default"),
            metrics,
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
                GameResponse::ActionError {
                    action_id,
                    response,
                } => {
                    let action = match self.sent_actions.get(&action_id) {
                        Some(action) => format!("{action:?}"),
                        None => format!("id={action_id}"),
                    };
                    error!("Action {action} error {response:?}");
                    self.farmers.get_mut(&response.farmer).unwrap().activity = response.correction;
                }
                GameResponse::Trip { id } => self.client.send(PlayerRequest::Trip { id }),
            }
        }
    }

    pub fn send_action(&mut self, action: FarmerBound) -> usize {
        self.action_id += 1;
        self.sent_actions.insert(self.action_id, action.clone());
        if self.sent_actions.len() > 100 {
            self.sent_actions.remove(&(self.action_id - 100));
        }
        match action {
            FarmerBound::Move { .. } => {
                // do not spam logs with real time movement
            }
            _ => {
                // info!("Client sends id={} {:?}", self.action_id, action);
            }
        }
        self.client.send(PlayerRequest::Perform {
            action: Action::Farmer { action },
            action_id: self.action_id,
        });
        self.action_id
    }

    pub fn send_action_as_ai(&mut self, action: Action) -> usize {
        self.action_id += 1;
        info!("Client sends as AI id={} {:?}", self.action_id, action);
        self.client.send(PlayerRequest::Perform {
            action,
            action_id: self.action_id,
        });
        self.action_id
    }

    pub fn get_my_farmer_mut(&mut self) -> Option<*mut FarmerRep> {
        let _farmer = match self
            .farmers
            .values_mut()
            .find(|farmer| farmer.player == self.client.player)
        {
            None => {
                return None;
            }
            Some(farmer) => {
                let ptr = farmer as *mut FarmerRep;
                return Some(ptr);
            }
        };
    }
}

impl Mode for Gameplay {
    fn update(&mut self, frame: &mut Frame) {
        let timer = &mut Timer::now();
        self.handle_server_responses(frame);
        self.metrics.update.record("server-response", timer);
        self.handle_user_input(frame);
        self.metrics.update.record("user-input", timer);
        self.animate(frame);
        self.metrics.update.record("animation", timer);
        self.render(frame);
        self.metrics.update.record("render", timer);
        self.render_ui(frame);
        self.metrics.update.record("render-ui", timer);
    }
}
