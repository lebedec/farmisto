use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::space3;
use crate::engine::space3::S3Animation;
use glam::{Mat4, Quat, Vec3, Vec4};
use log::{error, info};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(pub usize);

pub struct Armature {}

pub struct AnimationAsset {
    frames: Vec<Frame>,
}

impl AnimationAsset {
    pub fn from_space3<P: AsRef<Path>>(path: P) -> Self {
        let mut scene = space3::read_scene_from_file(path).unwrap();

        // todo: optimize struct, remove translation and collect
        let animation = std::mem::replace(&mut scene.animation, S3Animation::default());
        AnimationAsset {
            frames: animation
                .keyframes
                .into_iter()
                .map(|frame| Frame {
                    channels: frame
                        .channels
                        .into_iter()
                        .map(|channel| Channel {
                            parent: if channel.parent > -1 {
                                Some(channel.parent as usize)
                            } else {
                                None
                            },
                            position: channel.position,
                            rotation: channel.rotation,
                            scale: channel.scale,
                            matrix: Mat4::from_cols_array_2d(&channel.matrix),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

pub struct Channel {
    parent: Option<usize>,
    position: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    matrix: Mat4,
}

pub struct Frame {
    channels: Vec<Channel>,
}

pub struct State {
    pub id: StateId,
    pub name: String,
    pub fps: f32,
    pub motion: AnimationAsset,
    pub looped: bool,
    pub frame: usize,
    pub frame_time: f32,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterId(usize);

pub enum ParameterType {
    Float,
    Int,
    Bool,
    Trigger,
}

pub struct Parameter {
    id: ParameterId,
    value: f32,
}

pub enum ConditionOperator {
    Less,
    Greater,
    Equal,
}

pub struct Condition {
    parameter: ParameterId,
    operator: ConditionOperator,
    value: f32,
}

pub struct Transition {
    conditions: Vec<Condition>,
    next: StateId,
}

pub struct Machine {
    pub parameters: HashMap<ParameterId, Parameter>,
    pub states: Vec<State>,
    pub current: usize,
    pub transform: [Mat4; 64],
    pub pose_buffer: PoseBuffer,
}

impl Machine {
    pub fn set_float(&mut self, name: &str, value: f32) {
        unimplemented!()
    }

    pub fn set_int(&mut self, name: &str, value: i32) {
        unimplemented!()
    }

    pub fn set_bool(&mut self, name: &str, value: f32) {
        unimplemented!()
    }

    pub fn set_trigger(&mut self, name: &str) {
        unimplemented!()
    }

    pub fn update(&mut self, time: f32) {
        let state = &mut self.states[self.current];

        let frame_time = 1.0 / state.fps;
        state.frame_time += time;
        let mut exit = false;
        let mut need_update = false;
        while state.frame_time >= frame_time {
            need_update = true;
            state.frame_time -= frame_time;
            state.frame += 1;
            if state.frame >= state.motion.frames.len() {
                if state.looped {
                    state.frame = 0;
                } else {
                    state.frame = state.motion.frames.len() - 1;
                    exit = true;
                }
            }
        }

        if need_update {
            let blend = &state.motion.frames[state.frame];

            let transform = &mut self.transform;
            for (bone, channel) in blend.channels.iter().enumerate() {
                let pos = Vec3::from(channel.position);
                let quat = Quat::from_vec4(Vec4::from([
                    channel.rotation[0],
                    channel.rotation[1],
                    channel.rotation[2],
                    channel.rotation[3],
                ]));
                let scale = Vec3::from(channel.scale);
                let mut local = Mat4::from_scale_rotation_translation(scale, quat, pos);

                if let Some(parent) = channel.parent {
                    // local = transform[parent] * local;
                }
                transform[bone] = local;
            }
            // move transform to GPU buffer
            let uniform = PoseUniform {
                bones: transform.clone(),
            };
            self.pose_buffer.update::<PoseUniform>(0, uniform);
        }

        if exit {
            let state = &self.states[self.current];
            for transition in state.transitions.iter() {
                for condition in transition.conditions.iter() {
                    if !self.check_condition(condition) {
                        continue;
                    }
                }

                self.current = self.index(transition.next);
            }
        }
    }

    fn check_condition(&self, condition: &Condition) -> bool {
        let parameter = match self.get_parameter(condition.parameter) {
            Some(parameter) => parameter,
            None => {
                error!(
                    "Unable to check condition parameter {:?} not found",
                    condition.parameter
                );
                return false;
            }
        };
        match condition.operator {
            ConditionOperator::Less => parameter.value < condition.value,
            ConditionOperator::Greater => parameter.value > condition.value,
            ConditionOperator::Equal => (parameter.value - condition.value).abs() < 0.00001,
        }
    }

    fn get_parameter(&self, id: ParameterId) -> Option<&Parameter> {
        self.parameters.get(&id)
    }

    fn index(&self, id: StateId) -> usize {
        match self.states.iter().position(|state| state.id == id) {
            Some(index) => index,
            None => {
                error!("Unable to index state {:?}, use first", id);
                0
            }
        }
    }
}
