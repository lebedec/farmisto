use crate::assets::TextureAsset;
use crate::engine::rendering::{LinePushConstants, LineRenderObject, Scene};

impl Scene {
    pub fn render_line(&mut self, start: [f32; 2], end: [f32; 2], texture: TextureAsset) {
        self.lines.push(LineRenderObject {
            texture,
            constants: LinePushConstants {
                start,
                end,
                coords: [0.0, 0.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                pivot: [0.5, 0.5],
            },
        });
    }
}
