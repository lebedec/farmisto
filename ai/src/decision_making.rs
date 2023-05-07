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
    pub best_set: usize,
    pub best_behaviour: usize,
    pub best_decision: usize,
}

impl Thinking {
    pub fn reason(&mut self, key: String, score: f32) {
        self.reasons.insert(key, score);
    }
}

pub enum Reaction<A, T> {
    Action(A),
    Tuning(T),
}

pub fn make_decision<S, C, A, T>(
    behaviour_sets: &Vec<S>,
    consider: C,
) -> (Option<Reaction<A, T>>, Thinking)
where
    C: Fn(usize, &S, &mut Thinking) -> Option<(f32, Reaction<A, T>)>,
{
    let mut thinking = Thinking::default();
    let mut best_action = None;
    let mut best_action_scores = 0.0;
    for (set_index, set) in behaviour_sets.iter().enumerate() {
        if let Some(consideration) = consider(set_index, set, &mut thinking) {
            let (scores, action) = consideration;
            if scores > best_action_scores {
                best_action = Some(action);
                best_action_scores = scores;
            }
        }
    }
    (best_action, thinking)
}

#[derive(Default, Copy, Clone)]
pub struct Best {
    pub behaviour: usize,
    pub decision: usize,
    pub target: usize,
    pub scores: f32,
}

pub fn react<'t, I, T, E, F, A, Action, Tuning, Agent>(
    agent: &Agent,
    behaviours: &Vec<Behaviour<Decision<I, A>>>,
    targets: &'t Vec<T>,
    input: F,
    interact: E,
    thinking: &mut Thinking,
) -> Option<(f32, Reaction<Action, Tuning>)>
where
    A: Copy + Sized + Debug + Serialize,
    I: Copy + Sized + Serialize,
    F: Fn(&Agent, I, &T) -> f32,
    E: Fn(&Agent, A, &T) -> Reaction<Action, Tuning>,
{
    match consider(
        &behaviours,
        targets,
        |inp, target| input(agent, inp, target),
        thinking,
    ) {
        None => None,
        Some(best) => {
            let action = behaviours[best.behaviour].decisions[best.decision].action;
            let target = &targets[best.target];
            let action = interact(agent, action, target);
            Some((best.scores, action))
        }
    }
}

pub fn consider<'t, I, T, F, A>(
    behaviours: &Vec<Behaviour<Decision<I, A>>>,
    targets: &'t Vec<T>,
    input: F,
    thinking: &mut Thinking,
) -> Option<Best>
where
    A: Copy + Sized + Debug + Serialize,
    I: Copy + Sized + Serialize,
    F: Fn(I, &T) -> f32,
{
    let mut best: Option<Best> = None;
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
                    let x = x.clamp(0.0, 1.0);
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
        if best_target_scores > best.unwrap_or_default().scores {
            best = Some(Best {
                behaviour: behaviour_index,
                decision: best_target_decision,
                target: best_target,
                scores: best_target_scores,
            });
        }
        if best.unwrap_or_default().scores > 0.95 {
            // optimization:
            // not need to consider every behaviour if we found one appropriate enough
            break;
        }
    }
    best
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
