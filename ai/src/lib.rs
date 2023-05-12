use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use game::api::Action;
use game::inventory::{ContainerId, ItemId};
use game::math::{cast_ray, VectorMath};
use game::model::{Farmer, Knowledge};
use game::physics::SpaceId;
pub use thread::*;

use crate::decision_making::{make_decision, react, Behaviour, Reaction, Thinking};
use crate::fauna::{
    CreatureAgent, CreatureCropDecision, CreatureDecision, CreatureFoodDecision,
    CreatureGroundDecision, Targeting,
};
use crate::perception::{
    ContainerView, CreatureView, CropView, FarmerView, FoodView, ItemView, TileView,
};

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
    tiles: HashMap<SpaceId, Vec<Vec<TileView>>>,
    containers: HashMap<ContainerId, ContainerView>,
    items: HashMap<ContainerId, HashMap<ItemId, ItemView>>,
    farmers: HashMap<Farmer, FarmerView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
    colonization_date: f32,
}

impl Nature {
    pub fn update(&mut self) {}

    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];

        let mut holes = HashMap::new();
        for (space, view) in &self.tiles {
            let mut data = vec![vec![0u8; 128]; 128];
            for y in 0..128 {
                for x in 0..128 {
                    data[y][x] = if view[y][x].has_hole || view[y][x].has_barrier {
                        1
                    } else {
                        0
                    }
                }
            }
            holes.insert(*space, data);
        }

        for index in 0..self.creature_agents.len() {
            let agent = &self.creature_agents[index];
            let sets = &behaviours.creatures;
            let tiles = self.get_tiles_around(
                agent.farmland.space,
                agent.position.to_tile(),
                agent.radius as usize,
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
                if container.position.distance(agent.position) > agent.radius {
                    continue;
                }
                let holes = holes.get(&agent.farmland.space).unwrap();
                let contacts = cast_ray(agent.position, container.position, holes);
                let is_food_reachable = contacts.is_empty();
                if is_food_reachable {
                    for item in items.values() {
                        // TODO: common borrowing structure
                        foods.push(FoodView {
                            item: item.item,
                            owner: container.owner.clone(),
                            quantity: item.quantity,
                            max_quantity: item.max_quantity,
                            position: container.position,
                        });
                    }
                }
            }
            let mut crops = vec![];
            for crop in &self.crops {
                if crop.position.distance(agent.position) > agent.radius {
                    continue;
                }
                let holes = holes.get(&agent.farmland.space).unwrap();
                let contacts = cast_ray(agent.position, crop.position, holes);
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
                    agent,
                    &behaviours,
                    &me,
                    CreatureAgent::me,
                    CreatureAgent::react_me,
                    thinking,
                ),
                CreatureBehaviourSet::Crop { behaviours, .. } => react(
                    agent,
                    &behaviours,
                    &crops,
                    CreatureAgent::crop,
                    CreatureAgent::react_crop,
                    thinking,
                ),
                CreatureBehaviourSet::Ground { behaviours, .. } => react(
                    agent,
                    &behaviours,
                    &tiles,
                    CreatureAgent::ground,
                    CreatureAgent::react_ground,
                    thinking,
                ),
                CreatureBehaviourSet::Food { behaviours, .. } => react(
                    agent,
                    &behaviours,
                    &foods,
                    CreatureAgent::food,
                    CreatureAgent::react_food,
                    thinking,
                ),
            });
            let agent = &mut self.creature_agents[index];
            agent.thinking = thinking;
            agent.targeting = targeting;

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
