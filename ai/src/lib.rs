use log::info;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use game::api::Action;
use game::inventory::{ContainerId, ItemId};
use game::math::VectorMath;
use game::model::Knowledge;
use game::physics::SpaceId;
pub use thread::*;

use crate::decision_making::{make_decision, react, Behaviour, Reaction, Thinking};
use crate::fauna::{
    CreatureAgent, CreatureCropDecision, CreatureDecision, CreatureFoodDecision,
    CreatureGroundDecision,
};
use crate::perception::{ContainerView, CreatureView, CropView, FoodView, TileView};

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
    foods: HashMap<ItemId, FoodView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
}

impl Nature {
    pub fn update(&mut self) {
        for food in self.foods.values_mut() {
            let container = self.containers.get(&food.container).unwrap();
            food.position = container.position;
        }
    }

    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];
        for index in 0..self.creature_agents.len() {
            let agent = &self.creature_agents[index];
            let sets = &behaviours.creatures;
            let tiles =
                self.get_tiles_around(agent.space, agent.position.to_tile(), agent.radius as usize);
            // TODO: gather targets in radius
            // TODO: common borrowing structure
            let foods: Vec<FoodView> = self.foods.values().map(|v| v.clone()).collect();
            let me = vec![CreatureView {
                _entity: agent.entity,
            }];
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
                    &self.crops,
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

            match reaction {
                Some(Reaction::Action(action)) => {
                    actions.push(action);
                    agent.last_action = Instant::now();
                }
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
