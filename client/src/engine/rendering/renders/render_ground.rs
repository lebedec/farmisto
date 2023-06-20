use ash::vk;

use game::math::VectorMath;

use crate::assets::{SamplerAsset, TextureAsset};
use crate::engine::base::ShaderData;
use crate::engine::rendering::{
    GroundPushConstants, GroundRenderObject, GroundUniform, Scene, VISIBLE_MAP_X, VISIBLE_MAP_Y,
};

impl Scene {
    pub fn render_ground(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y],
        cell_size: f32,
        offset: [f32; 2],
    ) {
        let offset_step = self.camera_position.div(cell_size).floor();
        let offset_tile = offset_step.to_tile();
        let uniform = GroundUniform {
            map,
            offset: [offset_tile[0] as u32, offset_tile[1] as u32, 0, 0],
        };
        self.ground_buffer.update(self.present_index, uniform);
        self.grounds.push(GroundRenderObject {
            texture,
            sampler,
            constants: GroundPushConstants {
                offset,
                map_size: [VISIBLE_MAP_X as f32, VISIBLE_MAP_Y as f32],
                cell_size: [cell_size, cell_size],
                layer: 0.2,
            },
            data_descriptor: self.ground_pipeline.data.as_mut().unwrap().describe(vec![[
                ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: self.ground_buffer.buffers[self.present_index],
                    offset: 0,
                    range: std::mem::size_of::<GroundUniform>() as u64,
                }),
                ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: self.global_light_buffer.buffers[self.present_index],
                    offset: 0,
                    range: std::mem::size_of::<GroundUniform>() as u64,
                }),
            ]])[0],
        })
    }
}
