use crate::assets::TextureAsset;
use ash::{vk, Device};
use log::error;
use rusty_spine::controller::SkeletonController;
use rusty_spine::{AnimationState, Attachment, AttachmentType, Slot};

use crate::engine::rendering::SpineUniform;
use crate::engine::{IndexBuffer, UniformBuffer, VertexBuffer};

pub struct SpineRenderController {
    pub skeleton: SkeletonController,
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub atlas: TextureAsset,
    pub colors: [[f32; 4]; 4],
    pub lights_buffer: UniformBuffer<SpineUniform>,
}

impl SpineRenderController {
    pub fn animation(&self) -> &AnimationState {
        &self.skeleton.animation_state
    }

    pub fn update_spine_buffers(&self, device: &Device) -> Vec<usize> {
        let mut index_offset = 0;
        let mut meshes = vec![];
        let mut vertices = vec![];
        let mut indices: Vec<u32> = vec![];
        for index in 0..self.skeleton.skeleton.slots_count() {
            let slot = self.skeleton.skeleton.draw_order_at_index(index).unwrap();
            if !slot.bone().active() {
                continue;
            }
            if let Some(attachment) = slot.attachment() {
                SpineRenderController::fill_attachment_buffers(
                    &slot,
                    &attachment,
                    &mut index_offset,
                    &mut meshes,
                    &mut indices,
                    &mut vertices,
                );
            }
        }
        self.vertex_buffer.update(vertices, device);
        self.index_buffer.update(indices, device);
        meshes
    }

    pub fn fill_attachment_buffers(
        slot: &Slot,
        attachment: &Attachment,
        index_offset: &mut u32,
        meshes: &mut Vec<usize>,
        mega_indices: &mut Vec<u32>,
        mega_vertices: &mut Vec<SpineVertex>,
    ) {
        let mut mask = 0;
        if slot.data().name().contains("(damage)") {
            mask += 1;
        }
        if slot.data().name().contains("(color)") {
            mask += 10;
        }

        match attachment.attachment_type() {
            AttachmentType::Region => {
                let region = attachment.as_region().unwrap();
                let mut spine_vertices = vec![0.0; 8];
                unsafe {
                    region.compute_world_vertices(slot, &mut spine_vertices, 0, 2);
                }

                let spine_uvs = region.uvs();
                for i in 0..4 {
                    mega_vertices.push(SpineVertex {
                        position: [spine_vertices[i * 2], -spine_vertices[i * 2 + 1]],
                        uv: [spine_uvs[i * 2], 1.0 - spine_uvs[i * 2 + 1]], // inverse
                        mask,
                    })
                }
                let indices = [0, 1, 2, 2, 3, 0].map(|index| index + *index_offset);
                mega_indices.extend_from_slice(&indices);
                meshes.push(indices.len());
                *index_offset += 4;
            }
            AttachmentType::Mesh => {
                let mesh = attachment.as_mesh().unwrap();
                let stride = 2;
                let spine_vertices_count = mesh.world_vertices_length() as usize;
                let mut spine_vertices = vec![0.0; spine_vertices_count];
                unsafe {
                    mesh.compute_world_vertices(
                        slot,
                        0,
                        spine_vertices_count as i32,
                        &mut spine_vertices,
                        0,
                        stride as i32,
                    );
                }
                let uvs_slice =
                    unsafe { std::slice::from_raw_parts(mesh.uvs(), spine_vertices_count) };
                let spine_uvs: Vec<f32> = uvs_slice.to_vec();
                let mut vertices = vec![];
                for i in 0..(spine_vertices_count / stride) {
                    vertices.push(SpineVertex {
                        position: [spine_vertices[i * stride], -spine_vertices[i * stride + 1]],
                        uv: [spine_uvs[i * 2], 1.0 - spine_uvs[i * 2 + 1]], // inverse
                        mask,
                    })
                }
                let indices_count = mesh.triangles_count() as usize;
                let indices_slice =
                    unsafe { std::slice::from_raw_parts(mesh.triangles(), indices_count) };

                let indices: Vec<u32> = indices_slice
                    .iter()
                    .map(|index| (*index as u32) + *index_offset)
                    .collect();

                mega_indices.extend_from_slice(&indices);
                mega_vertices.extend_from_slice(&vertices);
                meshes.push(indices.len());

                *index_offset += vertices.len() as u32;
            }
            AttachmentType::Point => {}
            attachment_type => {
                error!("Unknown attachment type {:?}", attachment_type)
            }
        }
    }
}

#[derive(Default, Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Deserialize)]
#[repr(C)]
pub struct SpineVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub mask: u32,
}

impl SpineVertex {
    pub const BINDINGS: [vk::VertexInputBindingDescription; 1] =
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<SpineVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 3] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 8,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32_UINT,
            offset: 16,
        },
    ];
}
