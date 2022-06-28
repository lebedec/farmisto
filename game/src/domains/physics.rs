use crate::persistence::{Mutable, Persisted, Readonly};

pub struct PhysicsDomain {
    pub space_kinds: Readonly<SpaceKind>,
    pub spaces: Mutable<Space>,
    pub body_kinds: Readonly<BodyKind>,
    pub bodies: Mutable<Body>,
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

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Physics {
    BodyPositionChanged { id: usize, position: [f32; 2] },
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
        }
    }

    pub fn update(&mut self, time: f32) -> Vec<Physics> {
        let mut events = vec![];
        for body in self.bodies.iter_mut() {
            let kind = self.body_kinds.get_unchecked(body.kind);
            let delta = vec_scale([1.0, 0.0], time * kind.speed);
            body.position = vec_add(body.position, delta);
            events.push(Physics::BodyPositionChanged {
                id: body.id,
                position: body.position,
            })
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
