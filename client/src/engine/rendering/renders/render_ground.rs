use crate::engine::base::ShaderData;
use crate::engine::rendering::{
    GroundPushConstants, GroundRenderObject, GroundUniform, Scene, VISIBLE_MAP_X, VISIBLE_MAP_Y,
};
use crate::engine::{SamplerAsset, TextureAsset};
use ash::vk;
use game::building::{Grid, Room};
use game::math::VectorMath;

impl Scene {
    pub fn render_ground(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        input: &Vec<Vec<[f32; 2]>>,
        shapes: &Vec<Room>,
    ) {
        let mut global_interior_map = [0u128; Grid::ROWS];
        for shape in shapes {
            if shape.id == Room::EXTERIOR_ID {
                continue;
            }
            for (i, row) in shape.rows.iter().enumerate() {
                global_interior_map[shape.rows_y + i] = global_interior_map[shape.rows_y + i] | row;
            }
        }

        const CELL_SIZE: f32 = 128.0;
        let input_size = [input[0].len(), input.len()];
        let [input_size_x, input_size_y] = input_size;
        let offset_step = self.camera_position.div(CELL_SIZE).floor();
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
                let [capacity, moisture] = input[iy][ix];
                let pos = 1 << (Grid::COLUMNS - ix - 1);
                let visible = if global_interior_map[iy] & pos == pos {
                    1.0
                } else {
                    0.0
                };
                map[y][x] = [capacity, moisture, 1.0, 0.0];
            }
        }
        let uniform = GroundUniform { map };
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
