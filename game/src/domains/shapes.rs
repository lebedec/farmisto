#[derive(Debug)]
pub struct TriangleKind {
    pub name: String,
}

#[derive(Debug)]
pub struct QuadKind {
    pub name: String,
}

#[derive(Debug)]
pub struct Triangle {
    pub id: usize,
    pub kind: usize,
    pub position: [f32; 2],
}

#[derive(Debug)]
pub struct Quad {
    pub id: usize,
    pub kind: usize,
    pub position: [f32; 2],
}
