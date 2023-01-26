use std::collections::HashMap;

use log::{error, info};

use crate::collections::Shared;
use crate::math::{detect_collision, Collider, VectorMath};

pub const MAX_SPACES: usize = 128;

pub struct PhysicsDomain {
    pub known_spaces: HashMap<SpaceKey, Shared<SpaceKind>>,
    pub known_bodies: HashMap<BodyKey, Shared<BodyKind>>,
    pub known_barriers: HashMap<BarrierKey, Shared<BarrierKind>>,
    pub spaces: Vec<Space>,
    pub bodies: Vec<Vec<Body>>,
    pub barriers: Vec<Vec<Barrier>>,
}

impl Default for PhysicsDomain {
    fn default() -> Self {
        Self {
            known_spaces: Default::default(),
            known_bodies: Default::default(),
            known_barriers: Default::default(),
            spaces: vec![],
            bodies: vec![vec![]; MAX_SPACES],
            barriers: vec![vec![]; MAX_SPACES],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceKey(pub usize);

pub struct SpaceKind {
    pub id: SpaceKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SpaceId(pub usize);

pub struct Space {
    pub id: SpaceId,
    pub kind: Shared<SpaceKind>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyKey(pub usize);

pub struct BodyKind {
    pub id: BodyKey,
    pub name: String,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BodyId(pub usize);

#[derive(Clone)]
pub struct Body {
    pub id: BodyId,
    pub kind: Shared<BodyKind>,
    pub position: [f32; 2],
    pub direction: [f32; 2],
    pub space: SpaceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BarrierKey(pub usize);

pub struct BarrierKind {
    pub id: BarrierKey,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BarrierId(pub usize);

#[derive(Clone)]
pub struct Barrier {
    pub id: BarrierId,
    pub kind: Shared<BarrierKind>,
    pub position: [f32; 2],
    pub space: SpaceId,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Physics {
    BodyPositionChanged {
        id: BodyId,
        space: SpaceId,
        position: [f32; 2],
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PhysicsError {
    SpaceNotFound { space: SpaceId },
}

pub struct BarrierCreation<'a> {
    pub barrier: Barrier,
    barriers: &'a mut Vec<Barrier>,
}

impl PhysicsDomain {
    pub fn create_barrier(
        &mut self,
        space: SpaceId,
        kind: Shared<BarrierKind>,
        position: [f32; 2],
    ) -> Result<BarrierCreation, PhysicsError> {
        let barriers = &mut self.barriers[space.0];
        let creation = BarrierCreation {
            barrier: Barrier {
                id: BarrierId(barriers.len() + 10),
                kind,
                position,
                space,
            },
            barriers,
        };
        Ok(creation)
    }
}

impl<'a> BarrierCreation<'a> {
    pub fn complete(self) -> Vec<Physics> {
        self.barriers.push(self.barrier);
        vec![]
    }
}

impl PhysicsDomain {
    pub fn get_body_mut(&mut self, id: BodyId) -> Option<&mut Body> {
        for bodies in self.bodies.iter_mut() {
            for body in bodies {
                if body.id == id {
                    return Some(body);
                }
            }
        }
        return None;
    }

    pub fn get_body(&self, id: BodyId) -> Option<&Body> {
        for bodies in self.bodies.iter() {
            for body in bodies {
                if body.id == id {
                    return Some(body);
                }
            }
        }
        return None;
    }

    pub fn get_barrier(&self, id: BarrierId) -> Option<&Barrier> {
        for barriers in self.barriers.iter() {
            for barrier in barriers {
                if barrier.id == id {
                    return Some(barrier);
                }
            }
        }
        return None;
    }

    pub fn handle_translate(&mut self, _space: SpaceId, body: BodyId, position: [f32; 2]) {
        match self.get_body_mut(body) {
            None => {}
            Some(body) => {
                body.position = position;
            }
        }
    }

    pub fn move_body2(&mut self, id: BodyId, direction: [f32; 2]) {
        match self.get_body_mut(id) {
            Some(body) => {
                body.direction = direction;
            }
            None => {
                error!("Unable to move body {:?}, not found", id);
            }
        }
    }

    pub fn update(&mut self, time: f32) -> Vec<Physics> {
        let mut events = vec![];

        for space in self.spaces.iter() {
            let bodies = &mut self.bodies[space.id.0];
            let barriers = &mut self.barriers[space.id.0];
            for index in 0..bodies.len() {
                let _id = bodies[index].id;

                let body = &bodies[index];
                let delta = body.kind.speed * time;

                let destination = body.direction;

                let distance = body.position.distance(destination);

                if distance > 0.00001 {
                    let position = if delta > distance {
                        destination
                    } else {
                        let movement = body.position.direction(destination).mul(delta);
                        body.position.add(movement)
                    };

                    if let Some(position) = detect_collision(&body, position, &barriers) {
                        let body = &mut bodies[index];
                        body.position = position;
                        events.push(Physics::BodyPositionChanged {
                            id: body.id.into(),
                            space: body.space.into(),
                            position: body.position,
                        })
                    }
                }

                /*
                // crowd control
                let mut sum = 0.0;
                for other in bodies.iter() {
                    if other.id != id {
                        sum += other.position[0] * time * other.kind.speed;
                    }
                }

                // collision detection
                for barrier in barriers.iter() {
                    if barrier.kind.bounds[0] < sum {
                        sum = barrier.kind.bounds[0];
                    }
                }

                if sum > 0.0 {
                    let body = &mut bodies[index];
                    body.position = [sum, sum];
                    events.push(Physics::BodyPositionChanged {
                        id: body.id.into(),
                        space: body.space.into(),
                        position: body.position,
                    })
                }*/
            }
        }

        events
    }
}

impl Collider for &Body {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn bounds(&self) -> [f32; 2] {
        [0.5, 0.5]
    }
}

impl Collider for Barrier {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn bounds(&self) -> [f32; 2] {
        self.kind.bounds
    }
}
