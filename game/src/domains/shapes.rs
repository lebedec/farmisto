use crate::persistence::{Domain, Mutable, Persisted, Readonly};
use log::info;

#[derive(Domain)]
pub struct Shapes {
    pub triangle_kinds: Readonly<TriangleKind>,
    pub triangles: Mutable<Triangle>,
    pub quad_kinds: Readonly<QuadKind>,
}

#[derive(Debug, Persisted)]
pub struct TriangleKind {
    pub name: String,
}

#[derive(Debug, Persisted)]
pub struct QuadKind {
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

impl Shapes {
    pub fn new() -> Self {
        Self {
            triangle_kinds: Readonly::new(),
            triangles: Mutable::new(),
            quad_kinds: Readonly::new(),
        }
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
