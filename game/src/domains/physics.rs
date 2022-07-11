use datamap::{Collection, Grouping, Id, Known, Persisted, Shared, Storage};
use log::info;

#[derive(Default)]
pub struct PhysicsDomain {
    pub known_spaces: Known<SpaceKind>,
    pub known_bodies: Known<BodyKind>,
    pub known_barriers: Known<BarrierKind>,
    pub spaces: Collection<Space>,
    pub bodies: Grouping<Body, SpaceId>,
    pub barriers: Grouping<Barrier, SpaceId>,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceKey(usize);

#[derive(Persisted)]
pub struct SpaceKind {
    id: SpaceKey,
    name: String,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceId(usize);

#[derive(Persisted)]
pub struct Space {
    id: SpaceId,
    kind: Shared<SpaceKind>,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyKey(usize);

#[derive(Persisted)]
pub struct BodyKind {
    pub id: BodyKey,
    pub name: String,
    pub speed: f32,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(usize);

#[derive(Persisted)]
pub struct Body {
    pub id: BodyId,
    pub kind: Shared<BodyKind>,
    pub position: [f32; 2],
    #[group]
    pub space: SpaceId,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarrierKey(usize);

#[derive(Persisted)]
pub struct BarrierKind {
    pub id: BarrierKey,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarrierId(usize);

#[derive(Persisted)]
pub struct Barrier {
    pub id: BarrierId,
    pub kind: Shared<BarrierKind>,
    pub position: [f32; 2],
    #[group]
    pub space: SpaceId,
}

pub enum Physics {
    BodyPositionChanged {
        id: usize,
        space: usize,
        position: [f32; 2],
    },
}

impl PhysicsDomain {
    pub fn load(&mut self, storage: &Storage) {
        self.known_spaces.load(storage);
        self.spaces.load(storage, &self.known_spaces);
        self.known_bodies.load(storage);
        self.bodies.load(storage, &self.known_bodies);
        self.known_barriers.load(storage);
        self.barriers.load(storage, &self.known_barriers);
    }

    pub fn handle_translate(&mut self, _space: SpaceId, body: BodyId, position: [f32; 2]) {
        match self.bodies.get_mut(body) {
            None => {}
            Some(body) => {
                body.position = position;
            }
        }
    }

    pub fn handle_create_barrier(&mut self, space: SpaceId, kind: BarrierKey, position: [f32; 2]) {
        // let id = self.barriers.next_id();
        let id = BarrierId(10);
        let kind = self.known_barriers.get(kind).unwrap();
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
