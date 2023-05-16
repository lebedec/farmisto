use ash::vk;

use crate::assets::{SamplerAsset, TextureAsset};
use crate::engine::{IndexBuffer, VertexBuffer};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GroundPushConstants {
    pub offset: [f32; 2],
    pub map_size: [f32; 2],
    pub cell_size: [f32; 2],
    pub layer: f32,
}

pub struct GroundRenderObject {
    pub texture: TextureAsset,
    pub sampler: SamplerAsset,
    pub constants: GroundPushConstants,
    pub data_descriptor: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct TilemapPushConstants {
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub tile: [f32; 2],
    pub layer: f32,
}

pub struct TilemapRenderObject {
    pub texture: vk::ImageView,
    pub sampler: vk::Sampler,
    pub constants: TilemapPushConstants,
    pub data: vk::DescriptorBufferInfo,
    pub layer: usize,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PlantPushConstants {
    pub colors: [[f32; 4]; 4],
    pub attributes: [f32; 4],
    pub position: [f32; 2],
}

#[derive(Default)]
pub struct RenderingLine {
    pub plants: Vec<PlantRenderObject>,
    pub animals: Vec<AnimalRenderObject>,
    pub sprites: Vec<SpriteRenderObject>,
}

pub struct PlantRenderObject {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub texture: TextureAsset,
    pub coloration: TextureAsset,
    pub position: [f32; 2],
    pub colors: [[f32; 4]; 4],
    pub meshes: Vec<usize>,
    pub constants: PlantPushConstants,
    pub lights_descriptor: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct AnimalPushConstants {
    pub colors: [[f32; 4]; 4],
    pub position: [f32; 2],
    pub size: [f32; 2],
}

pub struct AnimalRenderObject {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub texture: TextureAsset,
    pub coloration: TextureAsset,
    pub position: [f32; 2],
    pub colors: [[f32; 4]; 4],
    pub meshes: Vec<usize>,
    pub constants: AnimalPushConstants,
    pub lights_descriptor: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpritePushConstants {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub coords: [f32; 4],
    pub color: [f32; 4],
    pub pivot: [f32; 2],
}

pub struct SpriteRenderObject {
    pub constants: SpritePushConstants,
    pub texture_descriptor: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ElementPushConstants {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub coords: [f32; 4],
    pub color: [f32; 4],
    pub pivot: [f32; 2],
}

pub struct ElementRenderObject {
    pub constants: ElementPushConstants,
    pub texture: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LinePushConstants {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub coords: [f32; 4],
    pub color: [f32; 4],
    pub pivot: [f32; 2],
}

pub struct LineRenderObject {
    pub texture: TextureAsset,
    pub constants: LinePushConstants,
}
