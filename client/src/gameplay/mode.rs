use std::collections::HashMap;

use glam::vec3;
use log::{error, info};
use sdl2::keyboard::Keycode;

use ai::AiThread;
use datamap::Storage;
use game::api::{Action, FarmerBound, GameResponse, PlayerRequest};
use game::inventory::{ContainerId, ItemId};
use game::math::{test_collisions, VectorMath};
use game::model::Construction;
use game::model::Creature;
use game::model::Crop;
use game::model::Equipment;
use game::model::Farmer;
use game::model::Farmland;
use game::model::ItemRep;
use game::model::Knowledge;
use game::model::Stack;
use game::model::Tree;
use game::model::{Activity, Assembly, Door};
use game::physics::generate_holes;
use game::Game;
use network::TcpClient;
use server::LocalServerThread;

use crate::assets::{SpriteAsset, TextureAsset};
use crate::engine::Input;
use crate::gameplay::camera::Camera;
use crate::gameplay::representation::{
    AssemblyRep, BarrierHint, ConstructionRep, CreatureRep, CropRep, DoorRep, EquipmentRep,
    FarmerRep, FarmlandRep, StackRep, TreeRep,
};
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

pub enum Intention {
    Use,
    Put,
    Swap,
    Move,
    QuickSwap(u8),
}

#[derive(Clone)]
pub enum Target {
    Ground { tile: [usize; 2] },
    Stack(Stack),
    Construction(Construction),
    Equipment(Equipment),
    Wall([usize; 2]),
    Crop(Crop),
    Creature(Creature),
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
        } else if self.pressed(Keycode::Num1) {
            Some(Intention::QuickSwap(0))
        } else if self.pressed(Keycode::Num2) {
            Some(Intention::QuickSwap(1))
        } else if self.pressed(Keycode::Num3) {
            Some(Intention::QuickSwap(2))
        } else if self.pressed(Keycode::Num4) {
            Some(Intention::QuickSwap(3))
        } else if self.down(Keycode::A)
            || self.down(Keycode::S)
            || self.down(Keycode::D)
            || self.down(Keycode::W)
        {
            Some(Intention::Move)
        } else {
            None
        }
    }
}

pub struct Host {
    pub server: LocalServerThread,
    pub ai: AiThread,
}

pub struct Gameplay {
    pub host: Option<Host>,
    pub client: TcpClient,
    pub action_id: usize,
    pub known: Knowledge,
    pub barriers: Vec<BarrierHint>,
    pub farmlands: HashMap<Farmland, FarmlandRep>,
    pub current_farmland: Option<Farmland>,
    pub trees: HashMap<Tree, TreeRep>,
    pub farmers: HashMap<Farmer, FarmerRep>,
    pub stacks: HashMap<Stack, StackRep>,
    pub equipments: HashMap<Equipment, EquipmentRep>,
    pub assembly: HashMap<Assembly, AssemblyRep>,
    pub doors: HashMap<Door, DoorRep>,
    pub constructions: HashMap<Construction, ConstructionRep>,
    pub crops: HashMap<Crop, CropRep>,
    pub creatures: HashMap<Creature, CreatureRep>,
    pub items: HashMap<ContainerId, HashMap<ItemId, ItemRep>>,
    pub camera: Camera,
    pub cursor: SpriteAsset,
    pub cursor_room: usize,
    pub farmer_room: usize,
    pub players: Vec<SpriteAsset>,
    pub players_index: usize,
    pub roof_texture: TextureAsset,
    pub stack_sprite: SpriteAsset,
    pub theodolite_sprite: SpriteAsset,
    pub theodolite_gui_sprite: SpriteAsset,
    pub theodolite_gui_select_sprite: SpriteAsset,
    pub gui_controls: SpriteAsset,
}

impl Gameplay {
    pub fn new(host: Option<Host>, client: TcpClient, frame: &mut Frame) -> Self {
        let assets = &mut frame.assets;
        let mut camera = Camera::new();
        camera.eye = vec3(0.0, 0.0, -1.0);

        let mut knowledge = Game::new(Storage::open("./assets/database.sqlite").unwrap());
        knowledge.load_game_knowledge().unwrap();
        let knowledge = knowledge.known;

        let cursor = assets.sprite("cursor");
        let players = vec![
            assets.sprite("player-Alice"),
            assets.sprite("player-Boris"),
            assets.sprite("player-Carol"),
            assets.sprite("player-David"),
        ];

        Self {
            host,
            client,
            action_id: 0,
            known: knowledge,
            barriers: Default::default(),
            farmlands: Default::default(),
            current_farmland: None,
            trees: HashMap::new(),
            farmers: Default::default(),
            stacks: Default::default(),
            equipments: Default::default(),
            assembly: Default::default(),
            doors: Default::default(),
            constructions: Default::default(),
            crops: Default::default(),
            creatures: Default::default(),
            items: Default::default(),
            camera,
            cursor,
            cursor_room: 0,
            farmer_room: 0,
            players,
            players_index: 0,
            roof_texture: assets.texture("./assets/texture/building-roof-template-2.png"),
            stack_sprite: assets.sprite("<drop>"),
            theodolite_sprite: assets.sprite("theodolite"),
            theodolite_gui_sprite: assets.sprite("building-gui"),
            theodolite_gui_select_sprite: assets.sprite("building-gui-select"),
            gui_controls: assets.sprite("gui-controls"),
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

    pub fn send_action(&mut self, action: FarmerBound) -> usize {
        self.action_id += 1;
        match action {
            FarmerBound::Move { .. } => {
                // do not spam logs with real time movement
            }
            _ => {
                info!("Client sends id={} {:?}", self.action_id, action);
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

    pub fn get_target_at(&self, tile: [usize; 2]) -> Target {
        for stack in self.stacks.values() {
            if stack.position.to_tile() == tile {
                return Target::Stack(stack.entity);
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

        for creature in self.creatures.values() {
            if creature.estimated_position.to_tile() == tile {
                return Target::Creature(creature.entity);
            }
        }

        for crop in self.crops.values() {
            if crop.position.to_tile() == tile {
                return Target::Crop(crop.entity);
            }
        }

        if let Some(farmland) = self.current_farmland {
            let farmland = self.farmlands.get(&farmland).unwrap();

            let cell = farmland.cells[tile[1]][tile[0]];
            if cell.wall {
                return Target::Wall(tile);
            }
        }

        Target::Ground { tile }
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

    pub fn handle_user_input(&mut self, frame: &mut Frame) {
        let farmer = match self.get_my_farmer_mut() {
            Some(farmer) => unsafe { &mut *farmer },
            None => {
                error!("Farmer behaviour not initialized yet");
                return;
            }
        };

        let input = &frame.input;

        let cursor = input.mouse_position(self.camera.position(), TILE_SIZE);
        let tile = cursor.tile;

        if input.pressed(Keycode::P) {
            self.players_index = (self.players_index + 1) % self.players.len();
        }

        let target = self.get_target_at(tile);

        if input.pressed(Keycode::E) {
            if let Target::Crop(crop) = target {
                let creature = self.creatures.values_mut().nth(0).unwrap();
                let entity = creature.entity;
                self.send_action_as_ai(Action::EatCrop {
                    crop,
                    creature: entity,
                });
            }
        }

        if input.pressed(Keycode::R) {
            if let Target::Ground { .. } = target {
                let creature = self.creatures.values().nth(0).unwrap().entity;
                self.send_action_as_ai(Action::MoveCreature {
                    destination: cursor.position,
                    creature,
                });
            }
        }

        if let Some(intention) = input.recognize_intention() {
            let item = self
                .items
                .get(&farmer.entity.hands)
                .and_then(|hands| hands.values().nth(0));
            let functions = match item {
                None => vec![],
                Some(item) => {
                    let kind = self.known.items.get(item.kind).unwrap();
                    kind.functions.clone()
                }
            };
            self.interact_with(farmer, functions, target, intention);
        }

        match farmer.activity {
            Activity::Idle | Activity::Usage => {}
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
                    self.send_action(FarmerBound::Move {
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
            farmer_rendering_position[0] - width / 2.0,
            farmer_rendering_position[1] - height / 2.0,
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
