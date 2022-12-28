use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::base::{Pipeline, Screen, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{MeshAsset, MeshBounds, PipelineAsset, ShaderAsset, TextureAsset, Vertex};
use crate::Assets;
use ash::vk::Handle;
use ash::{vk, Device};
use glam::{Mat4, Vec3};
use log::info;

use std::time::Instant;

pub struct Material {}

pub struct SceneObject {
    transform: Mat4,
    mesh: MeshAsset,
    texture: TextureAsset,
    texture_bind: vk::DescriptorSet,
    pose_data: Option<vk::DescriptorSet>,
}

pub struct SceneRenderer {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    objects: Vec<SceneObject>,
    layout: vk::PipelineLayout,

    pipeline_asset: PipelineAsset,
    pipeline: vk::Pipeline,

    // pub scene_data: ShaderData,
    pub material_data: ShaderDataSet<1>,
    pub object_data: ShaderDataSet<1>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub static_pose: vk::DescriptorSet,
    camera_buffer: UniformBuffer,
    pub present_index: u32,
    pass: vk::RenderPass,
    pub screen: Screen,
}

impl SceneRenderer {
    pub fn look_at(&mut self, uniform: CameraUniform) {
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn draw(&mut self, transform: Mat4, mesh: &MeshAsset, texture: &TextureAsset) {
        self.objects.push(SceneObject {
            transform,
            mesh: mesh.clone(),
            texture: texture.clone(),
            texture_bind: self.material_data.describe(
                texture.id(),
                vec![[ShaderData::Texture([vk::DescriptorImageInfo {
                    sampler: texture.sampler(),
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }])]],
            )[0],
            pose_data: None,
        })
    }

    pub fn animate(
        &mut self,
        transform: Mat4,
        mesh: &MeshAsset,
        texture: &TextureAsset,
        pose_id: u64,
        pose_buffer: &PoseBuffer,
    ) {
        self.objects.push(SceneObject {
            transform,
            mesh: mesh.clone(),
            texture: texture.clone(),
            texture_bind: self.material_data.describe(
                texture.id(),
                vec![[ShaderData::Texture([vk::DescriptorImageInfo {
                    sampler: texture.sampler(),
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }])]],
            )[0],
            pose_data: Some(
                self.object_data.describe(
                    pose_id,
                    vec![[ShaderData::Uniform([vk::DescriptorBufferInfo {
                        buffer: pose_buffer.buffers[0],
                        offset: 0,
                        range: std::mem::size_of::<PoseUniform>() as u64,
                    }])]],
                )[0],
            ),
        })
    }

    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        screen: Screen,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
    ) -> Self {
        let pipeline_asset = assets.pipeline("scene");
        //
        let camera_buffer =
            UniformBuffer::create::<CameraUniform>(device.clone(), device_memory, swapchain);

        // LAYOUT //

        let mut scene_data = ShaderDataSet::create(
            device.clone(),
            swapchain as u32,
            vk::ShaderStageFlags::VERTEX,
            [vk::DescriptorType::UNIFORM_BUFFER],
        );
        let descriptor_sets = scene_data.describe(
            0,
            (0..swapchain)
                .map(|index| {
                    [ShaderData::Uniform([vk::DescriptorBufferInfo {
                        buffer: camera_buffer.buffers[index],
                        offset: 0,
                        range: std::mem::size_of::<CameraUniform>() as u64,
                    }])]
                })
                .collect(),
        );

        let mut object_data = ShaderDataSet::create(
            device.clone(),
            swapchain as u32,
            vk::ShaderStageFlags::VERTEX,
            [vk::DescriptorType::UNIFORM_BUFFER],
        );

        // no animator, static mesh
        let pose_buffer = PoseBuffer::create::<PoseUniform>(device.clone(), device_memory, 1);
        pose_buffer.update(
            0,
            PoseUniform {
                bones: [Mat4::IDENTITY; 64],
            },
        );
        let static_pose = object_data.describe(
            0,
            vec![[ShaderData::Uniform([vk::DescriptorBufferInfo {
                buffer: pose_buffer.buffers[0],
                offset: 0,
                range: std::mem::size_of::<PoseUniform>() as u64,
            }])]],
        )[0];

        let material_data = ShaderDataSet::create(
            device.clone(),
            4, // TODO:  Unable to allocate 2 descriptors of type VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SA
            vk::ShaderStageFlags::FRAGMENT,
            [vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
        );

        let set_layouts = [scene_data.layout, material_data.layout, object_data.layout];

        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<Mat4>() as u32,
        }];

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        };

        let mut renderer = Self {
            device: device.clone(),
            device_memory: device_memory.clone(),
            objects: vec![],
            layout: pipeline_layout,
            pipeline_asset,
            pipeline: vk::Pipeline::null(), // will be immediately overridden
            material_data,
            object_data,
            descriptor_sets,
            static_pose,
            camera_buffer,
            present_index: 0,
            pass,
            screen,
        };
        renderer.rebuild_pipeline();
        renderer
    }

    pub unsafe fn render(&self, device: &Device, buffer: vk::CommandBuffer) {
        device.cmd_set_viewport(buffer, 0, &self.screen.viewports);
        device.cmd_set_scissor(buffer, 0, &self.screen.scissors);

        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        let point = vk::PipelineBindPoint::GRAPHICS;
        let descriptor_set = &[self.descriptor_sets[0]];
        device.cmd_bind_descriptor_sets(buffer, point, self.layout, 0, descriptor_set, &[]);

        let mut previous_mesh = 0;
        let mut previous_texture = 0;

        for object in self.objects.iter() {
            if previous_mesh != object.mesh.id() {
                previous_mesh = object.mesh.id();
                device.cmd_bind_vertex_buffers(buffer, 0, &[object.mesh.vertex()], &[0]);
                device.cmd_bind_index_buffer(buffer, object.mesh.index(), 0, vk::IndexType::UINT32);
            }

            if previous_texture != object.texture.id() {
                previous_texture = object.texture.id();
                device.cmd_bind_descriptor_sets(
                    buffer,
                    point,
                    self.layout,
                    1,
                    &[object.texture_bind],
                    &[],
                );
            }

            let pose = match object.pose_data {
                None => self.static_pose,
                Some(pose) => pose,
            };
            device.cmd_bind_descriptor_sets(buffer, point, self.layout, 2, &[pose], &[]);

            device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(&object.transform.to_cols_array()),
            );
            device.cmd_draw_indexed(buffer, object.mesh.vertices(), 1, 0, 0, 1);
        }
    }

    pub fn update(&mut self) {
        if self.pipeline_asset.changed {
            self.rebuild_pipeline();
            self.pipeline_asset.changed = false;
        }
    }

    pub fn rebuild_pipeline(&mut self) {
        self.pipeline = Pipeline::new()
            .layout(self.layout)
            .vertex(self.pipeline_asset.vertex.module)
            .fragment(self.pipeline_asset.fragment.module)
            .pass(self.pass)
            .build(
                &self.device,
                &self.screen.scissors,
                &self.screen.viewports,
                &Vertex::ATTRIBUTES,
                &Vertex::BINDINGS,
            )
            .unwrap();
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}
