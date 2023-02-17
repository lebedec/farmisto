use crate::engine::sprites::SpineSpriteController;
use crate::engine::{Input, SamplerAsset, SpineAsset, SpriteAsset, TextureAsset};
use crate::gameplay::camera::Camera;
use crate::gameplay::representation::{
    BarrierHint, ConstructionRep, DropRep, EquipmentRep, FarmerRep, FarmlandRep, TreeRep,
};
use crate::{Frame, Mode};
use datamap::Storage;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::{test_collisions, VectorMath};
use game::model::{
    Activity, Construction, Drop, Equipment, Farmer, Farmland, ItemRep, Knowledge, Purpose, Tree,
    Universe,
};
use game::Game;
use glam::vec3;
use log::{error, info};
use network::TcpClient;
use sdl2::keyboard::Keycode;
use server::LocalServerThread;
use std::collections::HashMap;

use game::building::{Building, Marker};
use game::inventory::{ContainerId, Inventory, ItemId};
use game::model::Universe::{
    BarrierHintAppeared, ConstructionAppeared, ConstructionVanished, DropAppeared, DropVanished,
    FarmerAppeared, FarmerVanished, FarmlandAppeared, FarmlandVanished, TreeAppeared, TreeVanished,
};
use game::physics::{generate_holes, Physics};
use game::planting::Planting;
use lazy_static::lazy_static;

pub const TILE_SIZE: f32 = 128.0;

#[inline]
pub fn rendering_position_of(position: [f32; 2]) -> [f32; 2] {
    position.mul(TILE_SIZE)
}

#[inline]
pub fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}

pub enum Intention {
    Use,
    Put,
    Swap,
}

pub enum Target {
    Ground([usize; 2]),
    Drop(Drop),
    Construction(Construction),
    Equipment(Equipment),
    Wall([usize; 2]),
}

pub trait InputMethod {
    fn recognize_intention(&self) -> Option<Intention>;
}

impl InputMethod for Input {
    fn recognize_intention(&self) -> Option<Intention> {
        if self.left_click() {
            Some(Intention::Use)
        } else if self.right_click() {
            Some(Intention::Put)
        } else if self.pressed(Keycode::Tab) {
            Some(Intention::Swap)
        } else {
            None
        }
    }
}

pub struct Gameplay {
    _server: Option<LocalServerThread>,
    pub client: TcpClient,
    pub action_id: usize,
    pub known: Knowledge,
    pub barriers: Vec<BarrierHint>,
    pub farmlands: HashMap<Farmland, FarmlandRep>,
    pub current_farmland: Option<Farmland>,
    pub trees: HashMap<Tree, TreeRep>,
    pub farmers: HashMap<Farmer, FarmerRep>,
    pub drops: HashMap<Drop, DropRep>,
    pub equipments: HashMap<Equipment, EquipmentRep>,
    pub constructions: HashMap<Construction, ConstructionRep>,
    pub items: HashMap<ContainerId, HashMap<ItemId, ItemRep>>,
    pub camera: Camera,
    pub spines: Vec<Farmer2d>,
    pub cursor: SpriteAsset,
    pub cursor_shape: usize,
    pub players: Vec<SpriteAsset>,
    pub players_index: usize,
    pub roof_texture: TextureAsset,
    pub drop_sprite: SpriteAsset,
    pub theodolite_sprite: SpriteAsset,
    pub theodolite_gui_sprite: SpriteAsset,
    pub theodolite_gui_select_sprite: SpriteAsset,
    pub gui_controls: SpriteAsset,

    pub tilemap_roof_texture: TextureAsset,
    pub tilemap_texture: TextureAsset,
    pub tilemap_sampler: SamplerAsset,
}

impl Gameplay {
    pub fn new(server: Option<LocalServerThread>, client: TcpClient, frame: &mut Frame) -> Self {
        let assets = &mut frame.assets;
        let mut camera = Camera::new();
        camera.eye = vec3(0.0, 0.0, -1.0);

        let mut knowledge = Game::new(Storage::open("./assets/database.sqlite").unwrap());
        knowledge.load_game_knowledge();
        let knowledge = knowledge.known;

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
            known: knowledge,
            barriers: Default::default(),
            farmlands: Default::default(),
            current_farmland: None,
            trees: HashMap::new(),
            farmers: Default::default(),
            drops: Default::default(),
            equipments: Default::default(),
            constructions: Default::default(),
            items: Default::default(),
            camera,
            spines: vec![],
            cursor,
            cursor_shape: 0,
            players,
            players_index: 0,
            roof_texture: assets.texture("./assets/texture/building-roof-template-2.png"),
            drop_sprite: assets.sprite("<drop>"),
            theodolite_sprite: assets.sprite("theodolite"),
            theodolite_gui_sprite: assets.sprite("building-gui"),
            theodolite_gui_select_sprite: assets.sprite("building-gui-select"),
            gui_controls: assets.sprite("gui-controls"),
            tilemap_roof_texture: assets.texture("./assets/texture/tiles-roof-template.png"),
            tilemap_texture: assets.texture("./assets/texture/tiles-floor-template.png"),
            tilemap_sampler: assets.sampler("pixel-perfect"),
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
                    error!("Action {} error response {:?}", action_id, response);
                    self.farmers.get_mut(&response.farmer).unwrap().activity = response.correction;
                }
            }
        }
    }

    pub fn send_action(&mut self, action: Action) -> usize {
        self.action_id += 1;
        if let Action::MoveFarmer { .. } = action {
        } else {
            info!("Sends {:?}", action);
        }
        self.client.send(PlayerRequest::Perform {
            action,
            action_id: self.action_id,
        });
        self.action_id
    }

    pub fn get_target_at(&self, tile: [usize; 2]) -> Target {
        for drop in self.drops.values() {
            if drop.position.to_tile() == tile {
                return Target::Drop(drop.entity);
            }
        }

        for construction in self.constructions.values() {
            if construction.tile == tile {
                return Target::Construction(construction.entity);
            }
        }

        for equipment in self.equipments.values() {
            if equipment.position.to_tile() == tile {
                return Target::Equipment(equipment.entity);
            }
        }

        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();

            let cell = farmland.cells[tile[1]][tile[0]];
            if cell.wall && cell.marker.is_none() {
                return Target::Wall(tile);
            }
        }

        Target::Ground(tile)
    }

    pub fn handle_user_input(&mut self, frame: &mut Frame) {
        let farmer = match self
            .farmers
            .values_mut()
            .find(|farmer| farmer.player == self.client.player)
        {
            None => {
                error!("Farmer behaviour not initialized yet");
                return;
            }
            Some(farmer) => {
                let ptr = farmer as *mut FarmerRep;
                unsafe {
                    // TODO: safe farmer behaviour mutation
                    &mut *ptr
                }
            }
        };

        let input = &frame.input;

        let cursor = input.mouse_position(self.camera.position(), TILE_SIZE);
        let tile = cursor.tile;

        if input.pressed(Keycode::P) {
            self.players_index = (self.players_index + 1) % self.players.len();
        }

        let target = self.get_target_at(tile);

        if let Some(intention) = input.recognize_intention() {
            self.interact_with(farmer, target, intention);
        }

        match farmer.activity {
            Activity::Instrumenting
            | Activity::Idle
            | Activity::Delivery
            | Activity::Installing { .. } => {}
            _ => {
                // not movement allowed
                return;
            }
        }

        let mut direction = [0.0, 0.0];
        if input.down(Keycode::A) {
            direction[0] -= 1.0;
        }
        if input.down(Keycode::D) {
            direction[0] += 1.0;
        }
        if input.down(Keycode::W) {
            direction[1] -= 1.0;
        }
        if input.down(Keycode::S) {
            direction[1] += 1.0;
        }
        let delta = direction.normalize().mul(input.time * farmer.body.speed);
        let estimated_position = delta.add(farmer.rendering_position);

        let farmland = match self.current_farmland {
            None => {
                error!("Current farmland not specified yet");
                return;
            }
            Some(farmland) => farmland,
        };

        let farmland = self.farmlands.get(&farmland).unwrap();

        // client side physics pre-calculation to prevent
        // obvious movement errors
        // Collision Detection
        let holes = generate_holes(estimated_position, farmer.body.radius, &farmland.holes);
        let holes_offsets = match test_collisions(farmer, estimated_position, &holes) {
            Some(offsets) => offsets,
            None => vec![],
        };
        if holes_offsets.len() < 2 {
            let offsets = match test_collisions(farmer, estimated_position, &self.barriers) {
                None => holes_offsets,
                Some(mut barrier_offsets) => {
                    barrier_offsets.extend(holes_offsets);
                    barrier_offsets
                }
            };
            if offsets.len() < 2 {
                let estimated_position = if offsets.len() == 1 {
                    estimated_position.add(offsets[0])
                } else {
                    estimated_position
                };
                farmer.estimated_position = estimated_position;
                if delta.length() > 0.0 {
                    self.send_action(Action::MoveFarmer {
                        destination: estimated_position,
                    });
                }
            }
        }

        // TODO: move camera after farmer rendering position changed
        let width = frame.sprites.screen.width() as f32 * frame.sprites.zoom;
        let height = frame.sprites.screen.height() as f32 * frame.sprites.zoom;
        let farmer_rendering_position = rendering_position_of(farmer.rendering_position);
        self.camera.eye = vec3(
            (farmer_rendering_position[0] - width / 2.0),
            (farmer_rendering_position[1] - height / 2.0),
            0.0,
        );
    }
}

impl Mode for Gameplay {
    fn update(&mut self, frame: &mut Frame) {
        self.handle_server_responses(frame);
        self.handle_user_input(frame);
        self.animate(frame);
        self.render(frame);
    }
}

pub struct Farmer2d {
    pub asset: SpineAsset,
    pub sprite: SpineSpriteController,
    pub position: [f32; 2],
    pub variant: u32,
}
