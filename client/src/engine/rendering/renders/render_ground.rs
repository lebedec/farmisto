use ash::vk;

use game::building::{Grid, Room};
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
        moisture: &[[u8; 128]; 128],
        moisture_capacity: &[[u8; 128]; 128],
        shapes: &Vec<Room>,
    ) {
        let mut global_interior_map = [0u128; Grid::ROWS];
        for shape in shapes {
            if shape.id == Room::EXTERIOR_ID {
                continue;
            }
            for (i, row) in shape.area.iter().enumerate() {
                global_interior_map[shape.area_y + i] = global_interior_map[shape.area_y + i] | row;
            }
        }

        const CELL_SIZE: f32 = 128.0;
        let [input_size_x, input_size_y] = [128, 128];
        let offset_step = self.camera_position.div(CELL_SIZE).floor();
        let offset_tile = offset_step.to_tile();
        let offset_step = offset_step.clamp(
            [0.0, 0.0],
            [
                (input_size_x - VISIBLE_MAP_X) as f32,
                (input_size_y - VISIBLE_MAP_Y) as f32,
            ],
        );
        let offset = offset_step.mul(CELL_SIZE);
        let mut map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y] = Default::default();
        for y in 0..VISIBLE_MAP_Y {
            for x in 0..VISIBLE_MAP_X {
                let [step_x, step_y] = offset_step;
                let iy = y + step_y as usize;
                let ix = x + step_x as usize;
                let moisture = moisture[iy][ix];
                let capacity = moisture_capacity[iy][ix];
                let capacity = capacity as f32 / 255.0;
                let moisture = moisture as f32 / 255.0;
                map[y][x] = [capacity, moisture, 1.0, 0.0];
            }
        }
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
                cell_size: [CELL_SIZE as f32, CELL_SIZE as f32],
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
