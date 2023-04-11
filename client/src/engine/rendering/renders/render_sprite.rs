use ash::vk;

use game::math::Position;

use crate::assets::SpriteAsset;
use crate::engine::base::ShaderData;
use crate::engine::rendering::{RenderingLine, Scene, SpritePushConstants, SpriteRenderObject};

#[derive(Debug, Clone, Copy)]
pub struct SpritePosition {
    pub xy: Position,
    pub sorting: isize,
    pub z: f32,
}

pub fn xy(xy: Position) -> SpritePosition {
    SpritePosition { xy, z: 0.0, sorting: xy[1] as isize }
}

impl SpritePosition {
    pub fn z(mut self, z: f32) -> SpritePosition {
        self.z = z;
        self
    }
    
    pub fn sorting(mut self, value: isize) -> SpritePosition {
        self.sorting = value;
        self
    }
}

impl Scene {
    pub fn render_sprite(&mut self, asset: &SpriteAsset, position: SpritePosition) {
        self.render_sprite_colored(asset, position, [1.0, 1.0, 1.0, 1.0]);
    }

    pub fn render_sprite_colored(
        &mut self,
        asset: &SpriteAsset,
        position: SpritePosition,
        color: [f32; 4],
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
        let object = SpriteRenderObject {
            constants: SpritePushConstants {
                position: position.xy,
                size: asset.size,
                coords: [x, y, w, h],
                pivot: asset.pivot,
                color,
            },
            texture_descriptor: self.sprite_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: asset.sampler.handle,
                    image_view: texture.view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
        };
        let objects = self
            .sorted_render_objects
            .entry(position.sorting)
            .or_default();
        objects.sprites.push(object)
    }
}
