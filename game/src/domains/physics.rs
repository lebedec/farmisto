use crate::persistence::{group, Grouping, Mutable, MutableGrouping, Persisted, Readonly};
use std::rc::Rc;

pub struct PhysicsDomain {
    pub space_kinds: Readonly<SpaceKind>,
    pub spaces: Mutable<Space>,
    pub body_kinds: Readonly<BodyKind>,
    pub bodies: Mutable<Body>,
    pub spaces2: MutableGrouping<usize, Body>,
}

#[derive(Debug, Persisted)]
pub struct SpaceKind {
    id: usize,
    name: String,
}

#[derive(Debug, Persisted)]
pub struct Space {
    id: usize,
    kind: usize,
}

pub struct Space2 {
    id: usize,
    kind: usize,
    bodies: Vec<Body>,
    barriers: Vec<Barrier>,
    bodies2: Grouping<usize, Body, BodyKind>,
}

impl Space2 {
    pub fn update(&mut self, domain: &PhysicsDomain, time: f32) -> Vec<Physics> {
        let mut events = Vec::with_capacity(self.bodies.len());
        let body1 = self.bodies2.iter_mut(0);
        let body2 = self.bodies2.iter_mut(1);
        for index in 0..self.bodies.len() {
            let id = self.bodies[index].id;
            let kind = self.bodies[index].kind;
            let kind = domain.body_kinds.get(kind).unwrap();

            // crowd manager
            let mut sum = 0.0;
            for other in self.bodies.iter() {
                if other.id != id {
                    sum += other.position[0] * time * kind.speed;
                }
            }

            // collision detection
            for barrier in self.barriers.iter() {
                if barrier.position[0] < sum {
                    sum = barrier.position[0]
                }
            }

            if sum > 0.0 {
                let body = &mut self.bodies[index];
                body.position = [sum, sum];
                events.push(Physics::BodyPositionChanged {
                    id,
                    space: body.space,
                    position: body.position,
                })
            }
        }
        events
    }
}

#[derive(Debug, Persisted)]
pub struct BodyKind {
    pub id: usize,
    pub name: String,
    pub speed: f32,
}

#[derive(Debug, Persisted)]
pub struct Body {
    pub id: usize,
    pub kind: usize,
    pub space: usize,
    pub position: [f32; 2],
}

#[derive(Debug, Persisted)]
pub struct BarrierKind {
    pub id: usize,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Persisted)]
pub struct Barrier {
    pub id: usize,
    pub kind: usize,
    pub space: usize,
    pub position: [f32; 2],
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Physics {
    BodyPositionChanged {
        id: usize,
        space: usize,
        position: [f32; 2],
    },
}

impl PhysicsDomain {
    /*
    pub fn load_state(&mut self, connection: &rusqlite::Connection) -> Vec<Shapes> {
        let mut events = vec![];
        let my_events =
            self.triangles
                .load(connection, Shapes::insert_triangle, Shapes::remove_triangle);
        events.extend(my_events);
        events
    }

    pub fn update_knowledge(&mut self, connection: &rusqlite::Connection) {
        self.triangle_kinds.update(connection);
        self.quad_kinds.update(connection);
    }*/
}

impl PhysicsDomain {
    pub fn new() -> Self {
        Self {
            space_kinds: Readonly::new(),
            spaces: Mutable::new(),
            body_kinds: Readonly::new(),
            bodies: Mutable::new(),
            spaces2: MutableGrouping::new(),
        }
    }

    pub fn load(&mut self, connection: &rusqlite::Connection) {}

    pub fn update(&mut self, time: f32) -> Vec<Physics> {
        let mut events = vec![];

        // let body = self.spaces2.get(0, 0);

        for (space, bodies) in self.spaces2.iter_groups() {
            for index in 0..bodies.len() {
                let id = bodies[index].id;

                let mut sum = 0.0;
                for other in bodies.iter() {
                    if other.id != id {
                        sum += other.position[0];
                    }
                }

                bodies[index].position = [sum, sum];
            }
        }

        for (space, bodies) in self.spaces2.iter_mut() {
            // update space
            /*for body in bodies.values_mut() {
                let mut sum = 0.0;
                for other in bodies.values() {
                    if other.id != body.id {
                        sum += other.position[0]
                    }
                }

                body.position = [1.0, sum];
            }*/
        }

        for space in self.spaces.iter_mut() {
            for body in self.bodies.iter_mut() {
                let kind = self.body_kinds.get_unchecked(body.kind);
                let delta = vec_scale([1.0, 0.0], time * kind.speed);
                body.position = vec_add(body.position, delta);
                events.push(Physics::BodyPositionChanged {
                    id: body.id,
                    space: body.space,
                    position: body.position,
                })
            }
        }
        events
    }
}

#[inline]
fn vec_scale(value: [f32; 2], scalar: f32) -> [f32; 2] {
    [value[0] * scalar, value[1] * scalar]
}

#[inline]
fn vec_add(left: [f32; 2], right: [f32; 2]) -> [f32; 2] {
    [left[0] + right[0], left[1] + right[1]]
}
