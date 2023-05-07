use std::collections::HashMap;
use std::fmt::Debug;

use game::api::Action;
use game::math::VectorMath;
use game::physics::SpaceId;
pub use thread::*;

use crate::decision_making::{make_decision, react, Behaviour, Reaction, Thinking};
use crate::fauna::{CreatureAgent, CreatureCropDecision, CreatureGroundDecision};
use crate::perception::{CreatureView, CropView, TileView};

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
    Crop {
        name: String,
        behaviours: Vec<Behaviour<CreatureCropDecision>>,
    },
    Ground {
        name: String,
        behaviours: Vec<Behaviour<CreatureGroundDecision>>,
    },
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    creatures: Vec<CreatureView>,
    tiles: HashMap<SpaceId, Vec<Vec<TileView>>>,
    // agents
    creature_agents: Vec<CreatureAgent>,
}

impl Nature {
    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];
        for index in 0..self.creature_agents.len() {
            let agent = &self.creature_agents[index];
            let sets = &behaviours.creatures;
            let tiles = self.get_tiles_around(agent.space, agent.position.to_tile(), agent.radius);
            let (reaction, thinking) = make_decision(sets, |_, set, thinking| match set {
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
            });
            let agent = &mut self.creature_agents[index];
            agent.thinking = thinking;
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
            .find(|agent| agent.creature.id == id)
    }
}
