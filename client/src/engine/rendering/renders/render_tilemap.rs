use crate::assets::{SamplerAsset, TextureAsset};
use crate::engine::rendering::{
    Scene, TilemapPushConstants, TilemapRenderObject, TilemapUniform, VISIBLE_MAP_X, VISIBLE_MAP_Y,
};
use crate::engine::UniformBuffer;
use crate::gameplay::TILE_SIZE;

pub struct TilemapController {
    pub texture: TextureAsset,
    pub sampler: SamplerAsset,
    pub data: UniformBuffer<TilemapUniform>,
}

impl Scene {
    pub fn instantiate_tilemap(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
    ) -> TilemapController {
        let data = UniformBuffer::create(self.device.clone(), &self.device_memory, self.swapchain);
        TilemapController {
            texture,
            sampler,
            data,
        }
    }

    pub fn render_tilemap(
        &mut self,
        tilemap: &TilemapController,
        offset: [f32; 2],
        layer: usize,
        data: TilemapUniform,
    ) {
        tilemap.data.update(self.present_index, data);
        self.tilemaps.push(TilemapRenderObject {
            texture: tilemap.texture.view,
            sampler: tilemap.sampler.handle,
            constants: TilemapPushConstants {
                offset,
                size: [VISIBLE_MAP_X as f32, VISIBLE_MAP_Y as f32],
                tile: [TILE_SIZE, TILE_SIZE],
                layer: 0.0,
            },
            data: tilemap.data.info(self.present_index),
            layer,
        })
    }
}
