use ash::vk;

use crate::engine::{IndexBuffer, SamplerAsset, TextureAsset, VertexBuffer};

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
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpinePushConstants {
    pub colors: [[f32; 4]; 4],
    pub position: [f32; 2],
    pub size: [f32; 2],
}

pub struct SpineRenderObject {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub texture: TextureAsset,
    pub coloration: TextureAsset,
    pub position: [f32; 2],
    pub colors: [[f32; 4]; 4],
    pub meshes: Vec<usize>,
    pub constants: SpinePushConstants,
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
    pub pivot: [f32; 2],
    pub highlight: f32,
}

pub struct SpriteRenderObject {
    pub constants: SpritePushConstants,
    pub texture_descriptor: vk::DescriptorSet,
}
