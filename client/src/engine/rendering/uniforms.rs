use crate::engine::rendering::{VISIBLE_MAP_X, VISIBLE_MAP_Y};

#[derive(Clone, Copy)]
pub struct GroundUniform {
    pub map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y],
}

#[derive(Clone, Copy)]
pub struct TilemapUniform {
    pub map: [[[u32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y],
}

#[derive(Clone, Copy)]
pub struct SpineUniform {
    pub color: [[f32; 4]; 16],
    pub position: [[f32; 4]; 16],
}

pub struct Light {
    pub color: [f32; 4],
    pub position: [f32; 4],
}
