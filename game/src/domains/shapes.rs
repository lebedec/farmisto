use crate::persistence::{Mutable, Persisted, Readonly};
use log::info;

pub struct ShapesDomain {
    pub triangle_kinds: Readonly<TriangleKind>,
    pub triangles: Mutable<Triangle>,
    pub quad_kinds: Readonly<QuadKind>,
}

#[derive(Debug, Persisted)]
pub struct TriangleKind {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Persisted)]
pub struct QuadKind {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Persisted)]
pub struct Triangle {
    pub id: usize,
    pub kind: usize,
    pub position: [f32; 2],
}

#[derive(Debug, Persisted)]
pub struct Quad {
    pub id: usize,
    pub kind: usize,
    pub position: [f32; 2],
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Shapes {
    TriangleAppeared {
        id: usize,
        kind: usize,
        position: [f32; 2],
    },
    TriangleVanished {
        id: usize,
    },
    QuadAppeared {
        id: usize,
        kind: usize,
        position: [f32; 2],
    },
    QuadVanished {
        id: usize,
    },
}

impl Shapes {
    fn insert_triangle(triangle: &Triangle) -> Shapes {
        Shapes::TriangleAppeared {
            id: triangle.id,
            kind: triangle.kind,
            position: triangle.position,
        }
    }

    fn remove_triangle(id: usize) -> Shapes {
        Shapes::TriangleVanished { id }
    }
}

impl ShapesDomain {
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
    }
}

impl ShapesDomain {
    pub fn new() -> Self {
        Self {
            triangle_kinds: Readonly::new(),
            triangles: Mutable::new(),
            quad_kinds: Readonly::new(),
        }
    }

    pub fn look_around(&self) -> Vec<Shapes> {
        self.triangles.iter().map(Shapes::insert_triangle).collect()
    }

    pub fn create_triangle(&mut self) -> Vec<Shapes> {
        let mut events = vec![];
        let id = self.triangles.next_id();
        let triangle = Triangle {
            id,
            kind: 1,
            position: [1.0, 1.0],
        };
        events.push(Shapes::insert_triangle(&triangle));
        self.triangles.insert(id, triangle);
        events
    }

    pub fn update(&mut self, time: f32) {
        let mut sum = 0;
        for triangle in self.triangles.iter_mut() {
            sum += triangle.kind;
            triangle.kind *= 1;
        }
        info!("Sum: {}", sum as f32 * time);
    }
}
