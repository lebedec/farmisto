use crate::engine::armature::{ArmatureBuffer, ArmatureUniform};
use crate::engine::base::{Queue, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{MeshAsset, MeshBounds, ShaderAsset, TextureAsset, Vertex};
use crate::Assets;
use ash::vk::Handle;
use ash::{vk, Device};
use glam::{Mat4, Vec3};
use log::info;
use std::ffi::CStr;
use std::sync::Arc;
use std::time::Instant;

pub struct Material {}

pub struct SceneObject {
    transform: Mat4,
    mesh: MeshAsset,
    texture: TextureAsset,
    texture_bind: vk::DescriptorSet,
}

pub struct SceneRenderer {
    device: Device,
    swapchain: usize,
    objects: Vec<SceneObject>,
    bounds: Vec<SceneObject>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    gizmos_pipeline: vk::Pipeline,
    // pub scene_data: ShaderData,
    pub material_data: ShaderDataSet<1>,
    // pub object_data: ShaderData,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    camera_buffer: UniformBuffer,
    pub armature_buffer: ArmatureBuffer,
    pub present_index: u32,
    pub viewport: [f32; 2],
    pass: vk::RenderPass,
    vertex_shader: ShaderAsset,
    vertex_last_id: u64,
    fragment_shader: ShaderAsset,
    fragment_last_id: u64,
    scissors: Vec<vk::Rect2D>,
    viewports: Vec<vk::Viewport>,
    bounds_mesh: MeshAsset,
    wireframe_tex: TextureAsset,
}

impl SceneRenderer {
    pub fn look_at(&mut self, uniform: CameraUniform) {
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.bounds.clear();
    }

    pub fn bounds(&mut self, transform: Mat4, bounds: MeshBounds) {
        let bounds_matrix = Mat4::from_scale(Vec3::from(bounds.length()))
            * Mat4::from_translation(Vec3::from(bounds.offset()));
        let texture = &self.wireframe_tex;
        self.bounds.push(SceneObject {
            transform: transform * bounds_matrix,
            mesh: self.bounds_mesh.clone(),
            texture: texture.clone(),
            texture_bind: self.material_data.describe(
                texture.id(),
                vec![[ShaderData::Texture([vk::DescriptorImageInfo {
                    sampler: texture.sampler(),
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }])]],
            )[0],
        })
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
        })
    }

    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        queue: Arc<Queue>,
        scissors: Vec<vk::Rect2D>,
        viewports: Vec<vk::Viewport>,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
    ) -> Self {
        let fragment_shader = assets.shader("./assets/shaders/animated.frag.spv");
        let wireframe_shader = assets.shader("./assets/shaders/wireframe.frag.spv");
        let vertex_shader = assets.shader("./assets/shaders/animated.vert.spv");
        //
        let camera_buffer =
            UniformBuffer::create::<CameraUniform>(device.clone(), device_memory, swapchain);
        let armature_buffer =
            ArmatureBuffer::create::<ArmatureUniform>(device.clone(), device_memory, swapchain);

        for present_index in 0..swapchain {
            armature_buffer.update(
                present_index,
                ArmatureUniform {
                    bones: [Mat4::IDENTITY; 64],
                },
            );
        }

        // LAYOUT //

        let mut scene_data = ShaderDataSet::create(
            device.clone(),
            swapchain as u32,
            vk::ShaderStageFlags::VERTEX,
            [
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::DescriptorType::UNIFORM_BUFFER,
            ],
        );

        let descriptor_sets = scene_data.describe(
            0,
            (0..swapchain)
                .map(|index| {
                    [
                        ShaderData::Uniform([vk::DescriptorBufferInfo {
                            buffer: camera_buffer.buffers[index],
                            offset: 0,
                            range: std::mem::size_of::<CameraUniform>() as u64,
                        }]),
                        ShaderData::Uniform([vk::DescriptorBufferInfo {
                            buffer: armature_buffer.buffers[index],
                            offset: 0,
                            range: std::mem::size_of::<ArmatureUniform>() as u64,
                        }]),
                    ]
                })
                .collect(),
        );

        let material_data = ShaderDataSet::create(
            device.clone(),
            4,
            vk::ShaderStageFlags::FRAGMENT,
            [vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
        );

        let set_layouts = [scene_data.layout, material_data.layout];

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
            swapchain,
            objects: vec![],
            bounds: vec![],
            layout: pipeline_layout,
            pipeline: vk::Pipeline::null(), // will be immediately overridden
            gizmos_pipeline: vk::Pipeline::null(),
            material_data,
            descriptor_sets,
            camera_buffer,
            armature_buffer,
            present_index: 0,
            viewport: [viewports[0].width, viewports[0].height],
            pass,
            vertex_shader: vertex_shader.clone(),
            vertex_last_id: 0,
            fragment_shader,
            fragment_last_id: 0,
            scissors,
            viewports,
            bounds_mesh: assets.cube(),
            wireframe_tex: assets.texture_white(),
        };
        renderer.build_pipeline();
        renderer.build_gizmos_pipeline(vertex_shader, wireframe_shader);
        renderer
    }

    pub unsafe fn render(&self, device: &Device, buffer: vk::CommandBuffer) {
        device.cmd_set_viewport(buffer, 0, &self.viewports);
        device.cmd_set_scissor(buffer, 0, &self.scissors);

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

            device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(&object.transform.to_cols_array()),
            );
            device.cmd_draw_indexed(buffer, object.mesh.vertices(), 1, 0, 0, 1);
        }

        // GIZMOS

        device.cmd_bind_pipeline(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.gizmos_pipeline,
        );

        let bind_point = vk::PipelineBindPoint::GRAPHICS;
        let descriptor_set = &[self.descriptor_sets[0]];
        device.cmd_bind_descriptor_sets(buffer, bind_point, self.layout, 0, descriptor_set, &[]);

        let mut previous_mesh = 0;
        let mut previous_texture = 0;

        for object in self.bounds.iter() {
            if previous_mesh != object.mesh.id() {
                previous_mesh = object.mesh.id();
                device.cmd_bind_vertex_buffers(buffer, 0, &[object.mesh.vertex()], &[0]);
                device.cmd_bind_index_buffer(buffer, object.mesh.index(), 0, vk::IndexType::UINT32);
            }

            if previous_texture != object.texture.id() {
                previous_texture = object.texture.id();
                device.cmd_bind_descriptor_sets(
                    buffer,
                    bind_point,
                    self.layout,
                    1,
                    &[object.texture_bind],
                    &[],
                );
            }

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
            self.build_pipeline();
        }
    }

    pub fn build_gizmos_pipeline(&mut self, vertex: ShaderAsset, fragment: ShaderAsset) {
        let t1 = Instant::now();

        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex.module(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment.module(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&Vertex::ATTRIBUTES)
            .vertex_binding_descriptions(&Vertex::BINDINGS);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&self.scissors)
            .viewports(&self.viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::LINE);

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.layout)
            .render_pass(self.pass)
            .build();

        let graphics_pipelines = unsafe {
            self.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        self.gizmos_pipeline = graphics_pipelines[0];
        info!("Rebuild gizmos pipeline in {:?}", t1.elapsed())
    }

    pub fn build_pipeline(&mut self) {
        let t1 = Instant::now();

        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: self.vertex_shader.module(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.fragment_shader.module(),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&Vertex::ATTRIBUTES)
            .vertex_binding_descriptions(&Vertex::BINDINGS);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&self.scissors)
            .viewports(&self.viewports);

        // let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        //     front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        //     line_width: 1.0,
        //     polygon_mode: vk::PolygonMode::FILL,
        //     ..Default::default()
        // };
        //

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.layout)
            .render_pass(self.pass)
            .build();

        let graphics_pipelines = unsafe {
            self.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        self.pipeline = graphics_pipelines[0];
        info!("Rebuild pipeline in {:?}", t1.elapsed())
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}
