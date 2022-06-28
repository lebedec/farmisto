use crate::persistence::{Grouping, Persisted};
use log::info;
use std::rc::Rc;

pub struct PhysicsDomain {
    pub bodies: Grouping<usize, Body, BodyKind>,
    pub barriers: Grouping<usize, Barrier, BarrierKind>,
}

#[derive(Debug, Persisted)]
pub struct SpaceKind {
    id: usize,
    name: String,
}

#[derive(Debug, Persisted)]
pub struct Space {
    id: usize,
    kind: Rc<SpaceKind>,
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
    pub kind: Rc<BodyKind>,
    #[group]
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
    pub kind: Rc<BarrierKind>,
    #[group]
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
    pub fn new() -> Self {
        Self {
            bodies: Grouping::new(),
            barriers: Grouping::new(),
        }
    }

    pub fn load(&mut self, connection: &rusqlite::Connection) {
        self.bodies.load(connection);
    }

    pub fn handle_create_barrier(&mut self, space: usize, kind: usize, position: [f32; 2]) {
        let id = self.barriers.next_id();
        let kind = self.barriers.get_kind(kind).unwrap();
        info!("barrier kind name is {:?}", kind.name);
        self.barriers.insert(
            space,
            Barrier {
                id,
                kind,
                space,
                position,
            },
        )
    }

    pub fn update(&mut self, time: f32) -> Vec<Physics> {
        let mut events = vec![];

        // for space in [0, 1, 3] {
        //     let bodies = self.bodies.iter_mut(space).unwrap();
        //     let barriers = self.barriers.iter_mut(space).unwrap();
        //     for index in 0..bodies.len() {
        //         let id = bodies[index].id;
        //
        //         // crowd control
        //         let mut sum = 0.0;
        //         for other in bodies.iter() {
        //             if other.id != id {
        //                 sum += other.position[0] * time * other.kind.speed;
        //             }
        //         }
        //
        //         // collision detection
        //         for barrier in barriers.iter() {
        //             if barrier.kind.bounds[0] < sum {
        //                 sum = barrier.kind.bounds[0];
        //             }
        //         }
        //
        //         if sum > 0.0 {
        //             let body = &mut bodies[index];
        //             body.position = [sum, sum];
        //             events.push(Physics::BodyPositionChanged {
        //                 id: body.id,
        //                 space: body.space,
        //                 position: body.position,
        //             })
        //         }
        //     }
        // }

        events
    }
}
