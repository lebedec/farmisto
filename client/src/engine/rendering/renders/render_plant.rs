use ash::vk;
use rusty_spine::controller::SkeletonController;

use crate::assets::{SpineAsset, TextureAsset};
use crate::engine::base::ShaderData;
use crate::engine::rendering::{
    PlantPushConstants, PlantRenderObject, RenderingLine, Scene, SpineRenderController,
    SpineUniform, SpineVertex, SpriteVertex,
};
use crate::engine::{IndexBuffer, UniformBuffer, VertexBuffer};

impl Scene {
    pub fn instantiate_plant(
        &mut self,
        spine: &SpineAsset,
        colors: [[f32; 4]; 4],
    ) -> SpineRenderController {
        let skeleton = SkeletonController::new(spine.skeleton.clone(), spine.animation.clone());
        let mut vertices: Vec<SpineVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut meshes: Vec<usize> = vec![];
        let mut index_offset = 0;
        for skin in skeleton.skeleton.data().skins() {
            for attachment in skin.attachments() {
                // info!(
                //     "skin {} {} {} {:?}",
                //     skin.name(),
                //     attachment.slot_index, // group by slot index (max)
                //     attachment.attachment.name(),
                //     attachment.attachment.attachment_type()
                // );
                // TODO: slot can have multiple attachments, need to reserve max size
                let slot = skeleton
                    .skeleton
                    .slot_at_index(attachment.slot_index)
                    .unwrap();
                SpineRenderController::fill_attachment_buffers(
                    &slot,
                    &attachment.attachment,
                    &mut index_offset,
                    &mut meshes,
                    &mut indices,
                    &mut vertices,
                )
            }
        }
        let vertex_buffer = VertexBuffer::create(&self.device, &self.device_memory, vertices);
        let index_buffer = IndexBuffer::create(&self.device, &self.device_memory, indices);
        let lights_buffer =
            UniformBuffer::create(self.device.clone(), &self.device_memory, self.swapchain);
        let controller = SpineRenderController {
            skeleton,
            vertex_buffer,
            index_buffer,
            atlas: spine.atlas.share(),
            colors,
            lights_buffer,
        };
        controller
    }

    pub fn render_plant(
        &mut self,
        spine: &SpineRenderController,
        coloration: TextureAsset,
        position: [f32; 2],
        health: f32,
        thirst: f32,
        colors: [[f32; 4]; 4],
    ) {
        let meshes = spine.update_spine_buffers(&self.device);
        spine.lights_buffer.update(
            self.present_index,
            SpineUniform {
                color: [
                    [1.0, 0.0, 0.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ],
                position: [
                    [0.0, 0.0, 512.0, 0.0],
                    [512.0, 0.0, 512.0, 0.0],
                    [1024.0, 0.0, 512.0, 0.0],
                    [0.0, 512.0, 512.0, 0.0],
                    [0.0, 1024.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                    [0.0, 0.0, 512.0, 0.0],
                ],
            },
        );
        let lights_descriptor =
            self.spine_pipeline
                .data
                .as_mut()
                .unwrap()
                .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: spine.lights_buffer.buffers[self.present_index],
                    offset: 0,
                    range: std::mem::size_of::<SpineUniform>() as u64,
                })]])[0];
        let object = PlantRenderObject {
            vertex_buffer: spine.vertex_buffer.clone(),
            index_buffer: spine.index_buffer.clone(),
            texture: spine.atlas.share(),
            coloration,
            position,
            colors: spine.colors,
            meshes,
            constants: PlantPushConstants {
                colors,
                position,
                attributes: [health, thirst, 0.0, 0.0],
            },
            lights_descriptor,
        };
        let objects = self
            .sorted_render_objects
            .entry(position[1] as isize)
            .or_default();
        objects.plants.push(object);
    }
}
