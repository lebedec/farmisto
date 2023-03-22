use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Consideration<I>
where
    I: Sized + Serialize,
{
    pub input: I,
    pub curve: Curve,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Decision<I, A>
where
    A: Copy + Sized + Debug + Serialize,
    I: Sized + Serialize,
{
    pub action: A,
    pub considerations: Vec<Consideration<I>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviour<D>
where
    D: Sized,
{
    pub name: String,
    pub decisions: Vec<D>,
}

/// Represents a decision making report via last update.
#[derive(Default, Clone, serde::Serialize)]
pub struct Thinking {
    pub reasons: HashMap<String, f32>,
    pub current_set: usize,
}

impl Thinking {
    pub fn reason(&mut self, key: String, score: f32) {
        let set = self.current_set;
        self.reasons.insert(format!("{set}:{key}"), score);
    }
}

pub fn make_decision<S, C, A>(behaviour_sets: &Vec<S>, consider: C) -> (Option<A>, Thinking)
where
    C: Fn(&S, &mut Thinking) -> (f32, A),
{
    let mut thinking = Thinking::default();
    let mut best_action = None;
    let mut best_action_scores = 0.0;
    for (set_index, set) in behaviour_sets.iter().enumerate() {
        thinking.current_set = set_index;
        let (scores, action) = consider(set, &mut thinking);
        if scores > best_action_scores {
            best_action = Some(action);
            best_action_scores = scores;
        }
    }
    (best_action, thinking)
}

pub fn consider<I, T, F, A>(
    behaviours: &Vec<Behaviour<Decision<I, A>>>,
    targets: &Vec<T>,
    input: F,
    thinking: &mut Thinking,
) -> (usize, usize, usize, f32)
where
    A: Copy + Sized + Debug + Serialize,
    I: Copy + Sized + Serialize,
    F: Fn(I, &T) -> f32,
{
    let mut best_behaviour = 0;
    let mut best_behaviour_decision = 0;
    let mut best_behaviour_target = 0;
    let mut best_behaviour_scores = 0.0;
    for (behaviour_index, behaviour) in behaviours.iter().enumerate() {
        let mut best_target = 0;
        let mut best_target_decision = 0;
        let mut best_target_scores = 0.0;
        for (target_index, target) in targets.iter().enumerate() {
            let mut best_decision = 0;
            let mut best_decision_scores = 0.0;
            for (decision_index, decision) in behaviour.decisions.iter().enumerate() {
                let mut scores = 1.0;
                for (index, consideration) in decision.considerations.iter().enumerate() {
                    let x = input(consideration.input, target);
                    let score = consideration.curve.respond(x);
                    {
                        // TODO: exclude from release build
                        let key =
                            format!("{behaviour_index}:{target_index}:{decision_index}:{index}");
                        thinking.reason(key, x);
                    }
                    scores *= score;
                    if scores == 0.0 {
                        // optimization:
                        // skip considerations for obviously zero scored decision
                        break;
                    }
                }
                if scores > best_decision_scores {
                    best_decision_scores = scores;
                    best_decision = decision_index;
                }
                if best_decision_scores > 0.95 {
                    // optimization:
                    // no need to compare decisions very precisely if we found one good enough
                    break;
                }
            }
            if best_decision_scores > best_target_scores {
                best_target = target_index;
                best_target_decision = best_decision;
                best_target_scores = best_decision_scores;
            }
            if best_target_scores > 0.95 {
                // optimization:
                // no need to choose a target very precisely if we found one good enough
                break;
            }
        }
        if best_target_scores > best_behaviour_scores {
            best_behaviour = behaviour_index;
            best_behaviour_decision = best_target_decision;
            best_behaviour_target = best_target;
            best_behaviour_scores = best_target_scores;
        }
        if best_behaviour_scores > 0.95 {
            // optimization:
            // not need to consider every behaviour if we found one appropriate enough
            break;
        }
    }
    (
        best_behaviour,
        best_behaviour_target,
        best_behaviour_decision,
        best_behaviour_scores,
    )
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Curve {
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Curve {
    pub fn respond(&self, mut t: f32) -> f32 {
        if t < 0.0 {
            t = 0.0;
        }
        if t > 1.0 {
            t = 1.0;
        }
        for (index, x) in self.x.iter().enumerate() {
            let x = *x;
            if x > t || x >= 1.0 {
                let start = index - 1;
                let end = index;
                let segment = self.x[end] - self.x[start];
                let progress = (t - self.x[start]) / segment;
                let delta = self.y[end] - self.y[start];
                let value = self.y[start] + delta * progress;
                return value;
            }
        }
        1.0
    }
}
