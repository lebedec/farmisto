use crate::persistence::{Collection, Grouping, Id, Knowledge, Persisted, Shared};
use log::info;

pub struct PhysicsDomain {
    pub spaces: Collection<Space, SpaceKind>,
    pub body_kinds: Knowledge<BodyKind>,
    pub bodies: Grouping<Body, SpaceId>,
    pub barrier_kinds: Knowledge<BarrierKind>,
    pub barriers: Grouping<Barrier, SpaceId>,
}

#[derive(Debug, Persisted)]
pub struct SpaceKind {
    id: usize,
    name: String,
}

#[derive(Debug, Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceId(usize);

#[derive(Debug, Persisted)]
pub struct Space {
    id: SpaceId,
    kind: Shared<SpaceKind>,
}

#[derive(Debug, Persisted)]
pub struct BodyKind {
    pub id: usize,
    pub name: String,
    pub speed: f32,
}

#[derive(Debug, Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(usize);

#[derive(Debug, Persisted)]
pub struct Body {
    pub id: BodyId,
    pub kind: Shared<BodyKind>,
    #[group]
    pub space: SpaceId,
    pub position: [f32; 2],
}

#[derive(Debug, Persisted)]
pub struct BarrierKind {
    pub id: usize,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarrierId(usize);

#[derive(Debug, Persisted)]
pub struct Barrier {
    pub id: BarrierId,
    pub kind: Shared<BarrierKind>,
    #[group]
    pub space: SpaceId,
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
            spaces: Collection::new(),
            body_kinds: Knowledge::new(),
            bodies: Grouping::new(),
            barrier_kinds: Knowledge::new(),
            barriers: Grouping::new(),
        }
    }

    pub fn load(&mut self, connection: &rusqlite::Connection) {
        self.spaces.load(connection);
        self.body_kinds.load(connection);
        self.bodies.load(connection, &self.body_kinds);
        self.barrier_kinds.load(connection);
        self.barriers.load(connection, &self.barrier_kinds);
    }

    pub fn handle_translate(&mut self, space: SpaceId, body: BodyId, position: [f32; 2]) {
        match self.bodies.get_mut(space, body) {
            None => {}
            Some(body) => {
                body.position = position;
            }
        }
    }

    pub fn handle_create_barrier(&mut self, space: SpaceId, kind: usize, position: [f32; 2]) {
        let id = self.barriers.next_id();
        let kind = self.barrier_kinds.get(kind).unwrap();
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

        for space in self.spaces.iter() {
            let mut empty = vec![];
            let bodies = self.bodies.iter_mut(space.id).unwrap_or(&mut empty);
            let mut empty = vec![];
            let barriers = self.barriers.iter_mut(space.id).unwrap_or(&mut empty);
            for index in 0..bodies.len() {
                let id = bodies[index].id;

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
                }
            }
        }

        events
    }
}
