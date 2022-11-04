use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::base::{Pipeline, Screen, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{MeshAsset, ShaderAsset, TextureAsset, Vertex};
use crate::Assets;
use ash::vk::Handle;
use ash::{vk, Device};
use glam::Mat4;
use log::{error, info};

use std::time::Instant;

pub struct Material {}

pub struct SceneObject {
    transform: Mat4,
    mesh: MeshAsset,
    texture: TextureAsset,
    texture_bind: vk::DescriptorSet,
    pose_data: Option<vk::DescriptorSet>,
}

pub struct SpriteUniform {}

pub struct SpritesRenderer {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    swapchain: usize,
    objects: Vec<SceneObject>,
    bounds: Vec<SceneObject>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    gizmos_pipeline: vk::Pipeline,
    // pub scene_data: ShaderData,
    pub material_data: ShaderDataSet<1>,
    pub object_data: ShaderDataSet<1>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub static_pose: vk::DescriptorSet,
    camera_buffer: UniformBuffer,
    pub present_index: u32,
    pub viewport: [f32; 2],
    pass: vk::RenderPass,
    vertex_shader: ShaderAsset,
    vertex_last_id: u64,
    fragment_shader: ShaderAsset,
    fragment_last_id: u64,
    screen: Screen,
    wireframe_tex: TextureAsset,
}

impl SpritesRenderer {
    pub fn look_at(&mut self, uniform: CameraUniform) {
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.bounds.clear();
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

    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        screen: Screen,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
    ) -> Self {
        let fragment_shader = assets.shader("./assets/shaders/animated.frag.spv");
        let vertex_shader = assets.shader("./assets/shaders/animated.vert.spv");
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
            4,
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
            swapchain,
            objects: vec![],
            bounds: vec![],
            layout: pipeline_layout,
            pipeline: vk::Pipeline::null(), // will be immediately overridden
            gizmos_pipeline: vk::Pipeline::null(),
            material_data,
            object_data,
            descriptor_sets,
            static_pose,
            camera_buffer,
            present_index: 0,
            viewport: [screen.width() as f32, screen.height() as f32],
            pass,
            vertex_shader: vertex_shader.clone(),
            vertex_last_id: 0,
            fragment_shader,
            fragment_last_id: 0,
            screen,
            wireframe_tex: assets.texture_white(),
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
        let vertex_changed = self.vertex_last_id != self.vertex_shader.module().as_raw();
        let fragment_changed = self.fragment_last_id != self.fragment_shader.module().as_raw();
        if vertex_changed || fragment_changed {
            self.vertex_last_id = self.vertex_shader.module().as_raw();
            self.fragment_last_id = self.fragment_shader.module().as_raw();
            self.rebuild_pipeline();
        }
    }

    pub fn rebuild_pipeline(&mut self) {
        let time = Instant::now();
        let building = Pipeline::new()
            .layout(self.layout)
            .vertex(self.vertex_shader.module())
            .fragment(self.fragment_shader.module())
            .pass(self.pass)
            .build(
                &self.device,
                &self.screen.scissors,
                &self.screen.viewports,
                &Vertex::ATTRIBUTES,
                &Vertex::BINDINGS,
            );
        match building {
            Ok(pipeline) => {
                info!("Create pipeline in {:?}", time.elapsed());
                self.pipeline = pipeline;
            }
            Err(error) => {
                error!("Unable to rebuild pipeline, {:?}", error);
            }
        }
    }
}
