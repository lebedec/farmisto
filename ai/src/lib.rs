use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use game::api::Action;
use game::inventory::{ContainerId, FunctionsQuery, ItemId};
use game::math::{cast_ray, cast_ray2, Array, ArrayIndex, VectorMath};
use game::model::{Farmer, Knowledge};
use game::physics::{BarrierId, BarrierKey, SpaceId};
use game::raising::TetherId;
pub use thread::*;

use crate::decision_making::{make_decision, react, Behaviour, Reaction, Thinking};
use crate::fauna::{
    CreatureAgent, CreatureCropDecision, CreatureDecision, CreatureFoodDecision,
    CreatureGroundDecision, Targeting,
};
use crate::perception::{BarrierView, ContainerView, CreatureView, CropView, FarmerView, FoodView, ItemView, TetherView};

mod api;
mod decision_making;
mod fauna;
mod machine;
mod perception;
mod queries;
mod thread;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRef {
    id: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviours {
    creatures: Vec<CreatureBehaviourSet>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum CreatureBehaviourSet {
    Me {
        name: String,
        behaviours: Vec<Behaviour<CreatureDecision>>,
    },
    Crop {
        name: String,
        behaviours: Vec<Behaviour<CreatureCropDecision>>,
    },
    Ground {
        name: String,
        behaviours: Vec<Behaviour<CreatureGroundDecision>>,
    },
    Food {
        name: String,
        behaviours: Vec<Behaviour<CreatureFoodDecision>>,
    },
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    creatures: Vec<CreatureView>,
    holes_map: HashMap<SpaceId, Vec<u8>>,
    barriers_map: HashMap<SpaceId, Vec<u8>>,
    barriers: HashMap<BarrierId, BarrierView>,
    containers: HashMap<ContainerId, ContainerView>,
    items: HashMap<ContainerId, HashMap<ItemId, ItemView>>,
    farmers: HashMap<Farmer, FarmerView>,
    tethers: HashMap<TetherId, TetherView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
    colonization_date: f32,
    // shared
    feeding_map: Vec<f32>,
    //
    known: Knowledge,
}

impl Nature {
    pub fn update(&mut self) {}

    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];

        let mut holes = HashMap::new();
        for (space, view) in &self.holes_map {
            let mut data = view.clone();
            data.add(
                128,
                [0, 0, 128, 128],
                &self.barriers_map.get(space).unwrap(),
            );
            holes.insert(*space, data);
        }

        self.feeding_map = vec![0.0; 128 * 128];
        for (container, items) in &self.items {
            let container = match self.containers.get(container) {
                Some(container) => container,
                None => {
                    error!("Unable to get {container:?}, not registered");
                    continue;
                }
            };
            let mut is_food = false;
            for item in items.values() {
                if item.kind.functions.as_food().is_ok() {
                    is_food = true;
                    break;
                }
            }
            if is_food {
                let rect = container.position.to_tile().rect([128, 128], [9, 9]);
                let [x, y, w, h] = rect;
                let patch = vec![0.5; w * h];
                self.feeding_map.add(128, rect, &patch);
            }
        }

        for index in 0..self.creature_agents.len() {
            let agent = &self.creature_agents[index];
            let sets = &behaviours.creatures;
            let tiles = self.get_tiles_around(
                agent.position.to_tile(),
                agent.radius as usize,
                holes.get(&agent.farmland.space).expect("agent holes"),
            );

            let mut foods = vec![];
            for (container, items) in &self.items {
                let container = match self.containers.get(container) {
                    Some(container) => container,
                    None => {
                        error!("Unable to get {container:?}, not registered");
                        continue;
                    }
                };
                if container.position.distance(agent.position) > agent.interaction {
                    continue;
                }
                let holes = holes.get(&agent.farmland.space).unwrap();
                let contacts = cast_ray2(agent.position, container.position, holes);
                let is_food_reachable = contacts.is_empty();
                if is_food_reachable {
                    for item in items.values() {
                        // TODO: common borrowing structure
                        if item.kind.functions.as_food().is_ok() {
                            foods.push(FoodView {
                                item: item.item,
                                owner: container.owner.clone(),
                                quantity: item.quantity,
                                max_quantity: item.kind.max_quantity,
                                position: container.position,
                            });
                        }
                    }
                }
            }
            let mut crops = vec![];
            for crop in &self.crops {
                if crop.position.distance(agent.position) > agent.interaction {
                    continue;
                }
                let holes = holes.get(&agent.farmland.space).unwrap();
                let contacts = cast_ray2(agent.position, crop.position, holes);
                let is_food_reachable = contacts.is_empty();
                if is_food_reachable {
                    // TODO: common borrowing structure
                    crops.push(crop.clone());
                }
            }

            let me = vec![CreatureView {
                _entity: agent.entity,
            }];

            let targeting = Targeting {
                crops: crops.iter().map(|crop| crop.entity.id).collect(),
                tiles: tiles.clone(),
                foods: foods.iter().map(|food| food.item.0).collect(),
                me: vec![agent.entity.id],
            };

            let (reaction, thinking) = make_decision(sets, |_, set, thinking| match set {
                CreatureBehaviourSet::Me { behaviours, .. } => react(
                    self,
                    agent,
                    &behaviours,
                    &me,
                    CreatureAgent::me,
                    CreatureAgent::react_me,
                    thinking,
                    &agent.disabling,
                ),
                CreatureBehaviourSet::Crop { behaviours, .. } => react(
                    self,
                    agent,
                    &behaviours,
                    &crops,
                    CreatureAgent::crop,
                    CreatureAgent::react_crop,
                    thinking,
                    &agent.disabling,
                ),
                CreatureBehaviourSet::Ground { behaviours, .. } => react(
                    self,
                    agent,
                    &behaviours,
                    &tiles,
                    CreatureAgent::ground,
                    CreatureAgent::react_ground,
                    thinking,
                    &agent.disabling,
                ),
                CreatureBehaviourSet::Food { behaviours, .. } => react(
                    self,
                    agent,
                    &behaviours,
                    &foods,
                    CreatureAgent::food,
                    CreatureAgent::react_food,
                    thinking,
                    &agent.disabling,
                ),
            });
            let agent = &mut self.creature_agents[index];
            agent.thinking = thinking;
            agent.targeting = targeting;

            if let Some(best) = agent.thinking.best.as_ref() {
                for tag in &best.decision_tags {
                    agent.cooldowns.insert(tag.clone(), self.colonization_date);
                }
            }

            match reaction {
                Some(Reaction::Action(action)) => actions.push(action),
                Some(Reaction::Tuning(tuning)) => agent.tune(tuning),
                None => {}
            }
        }
        actions
    }

    pub fn get_creature_agent(&self, id: usize) -> Option<&CreatureAgent> {
        self.creature_agents
            .iter()
            .find(|agent| agent.entity.id == id)
    }
}
