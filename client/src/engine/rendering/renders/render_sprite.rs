use crate::assets::SpriteAsset;
use crate::engine::base::ShaderData;
use crate::engine::rendering::{Scene, SpritePushConstants, SpriteRenderObject};
use ash::vk;

impl Scene {
    pub fn render_sprite(
        &mut self,
        asset: &SpriteAsset,
        position: [f32; 2],
        line: usize,
        highlight: f32,
    ) {
        let texture = &asset.texture;
        let image_w = asset.texture.width as f32;
        let image_h = asset.texture.height as f32;
        let [sprite_x, sprite_y] = asset.position;
        let [sprite_w, sprite_h] = asset.size;
        let x = sprite_x / image_w;
        let y = sprite_y / image_h;
        let w = sprite_w / image_w;
        let h = sprite_h / image_h;
        self.sprites[line].push(SpriteRenderObject {
            constants: SpritePushConstants {
                position,
                size: asset.size,
                coords: [x, y, w, h],
                pivot: asset.pivot,
                highlight,
            },
            texture_descriptor: self.sprite_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: asset.sampler.handle,
                    image_view: texture.view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
        })
    }
}
