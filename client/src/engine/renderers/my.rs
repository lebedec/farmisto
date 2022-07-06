use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{MeshAsset, ShaderAsset, TextureAsset, Transform, Vertex};
use crate::Assets;
use ash::vk::Handle;
use ash::{vk, Device};
use log::info;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;
use std::sync::Arc;
use std::time::Instant;

pub struct Material {}

pub struct MyRenderObject {
    transform: Transform,
    mesh: MeshAsset,
    texture: TextureAsset,
}

pub struct MyRenderer {
    device: Device,
    swapchain: usize,
    objects: Vec<MyRenderObject>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub texture_set_layout: vk::DescriptorSetLayout,
    pub texture_descriptors: Arc<RefCell<HashMap<u64, vk::DescriptorSet>>>,
    camera_buffer: UniformBuffer,
    pub present_index: u32,
    pub viewport: [f32; 2],
    pass: vk::RenderPass,
    vertex_shader: ShaderAsset,
    vertex_last_id: u64,
    fragment_shader: ShaderAsset,
    fragment_last_id: u64,
    scissors: Vec<vk::Rect2D>,
    viewports: Vec<vk::Viewport>,
}

impl MyRenderer {
    pub fn look_at(&mut self, uniform: CameraUniform) {
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn draw(&mut self, transform: Transform, mesh: MeshAsset, texture: TextureAsset) {
        self.objects.push(MyRenderObject {
            transform,
            mesh,
            texture,
        })
    }

    pub fn describe_texture(&self, texture: &TextureAsset) -> vk::DescriptorSet {
        let mut descriptors = self.texture_descriptors.borrow_mut();
        if let Some(descriptor) = descriptors.get(&texture.id()) {
            return *descriptor;
        }

        let descriptor_count = 1;
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count,
        }];
        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(descriptor_count)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe { self.device.create_descriptor_pool(&info, None).unwrap() };

        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..1 {
            layouts.push(self.texture_set_layout);
        }
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool,
            descriptor_set_count: 1,
            p_set_layouts: layouts.as_ptr(),
        };
        let texture_descriptor_sets = unsafe {
            self.device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };
        let tex_descriptor_set = texture_descriptor_sets[0];
        let descriptor_image_infos = [vk::DescriptorImageInfo {
            sampler: texture.sampler(),
            image_view: texture.view(),
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }];
        let descriptor_write_sets = [vk::WriteDescriptorSet {
            // sampler uniform
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: tex_descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: descriptor_image_infos.as_ptr(),
            p_buffer_info: ptr::null(),
            p_texel_buffer_view: ptr::null(),
        }];
        unsafe {
            self.device
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }

        descriptors.insert(texture.id(), tex_descriptor_set);

        tex_descriptor_set
    }

    pub unsafe fn render(&mut self, device: &Device, buffer: vk::CommandBuffer) {
        device.cmd_set_viewport(buffer, 0, &self.viewports);
        device.cmd_set_scissor(buffer, 0, &self.scissors);

        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        for object in self.objects.iter() {
            // todo: if last != object.mesh
            {
                device.cmd_bind_vertex_buffers(buffer, 0, &[object.mesh.vertex()], &[0]);
                device.cmd_bind_index_buffer(buffer, object.mesh.index(), 0, vk::IndexType::UINT32);
            }
            let bind_point = vk::PipelineBindPoint::GRAPHICS;
            let descriptor_set = &[self.descriptor_sets[0]];

            // todo: if last != object.texture
            {
                device.cmd_bind_descriptor_sets(
                    buffer,
                    bind_point,
                    self.layout,
                    1,
                    // todo: move &mut texture descriptor creation out of render function
                    &[self.describe_texture(&object.texture)],
                    &[],
                );
            }

            device.cmd_bind_descriptor_sets(
                buffer,
                bind_point,
                self.layout,
                0,
                descriptor_set,
                &[],
            );
            device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(&object.transform.matrix.to_cols_array()),
            );
            device.cmd_draw_indexed(buffer, object.mesh.vertices(), 1, 0, 0, 1);
        }
    }

    pub fn update(&mut self) {
        let vertex_changed = self.vertex_last_id != self.vertex_shader.module().as_raw();
        let fragment_changed = self.fragment_last_id != self.fragment_shader.module().as_raw();
        if vertex_changed || fragment_changed {
            self.build_pipeline();
        }
    }

    pub fn build_pipeline(&mut self) {
        let t1 = Instant::now();
        self.vertex_last_id = self.vertex_shader.module().as_raw();
        self.fragment_last_id = self.fragment_shader.module().as_raw();

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

        let vertex_input_state_info = Vertex::describe();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&self.scissors)
            .viewports(&self.viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };
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

    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        scissors: Vec<vk::Rect2D>,
        viewports: Vec<vk::Viewport>,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
    ) -> Self {
        let fragment_shader = assets.shader("./assets/shaders/triangle.frag.spv");
        let vertex_shader = assets.shader("./assets/shaders/triangle.vert.spv");
        //
        let camera_buffer =
            UniformBuffer::create::<CameraUniform>(device.clone(), device_memory, swapchain);
        //
        let descriptor_pool = UniformBuffer::create_descriptor_pool(device, swapchain as u32);
        let descriptor_set_layout = UniformBuffer::create_descriptor_set_layout(device);
        let descriptor_sets = UniformBuffer::create_descriptor_sets::<CameraUniform>(
            device,
            descriptor_pool,
            descriptor_set_layout,
            &camera_buffer.buffers,
            swapchain,
        );
        // ^
        let bindings = [vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        }];
        let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let texture_set_layout =
            unsafe { device.create_descriptor_set_layout(&info, None).unwrap() };

        let set_layouts = [descriptor_set_layout, texture_set_layout];

        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<Transform>() as u32,
        }];

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        };

        let texture_descriptors = Arc::new(RefCell::new(Default::default()));

        let mut renderer = Self {
            device: device.clone(),
            swapchain,
            objects: vec![],
            layout: pipeline_layout,
            pipeline: vk::Pipeline::null(), // will be immediately overridden
            descriptor_set_layout,
            descriptor_sets,
            texture_set_layout,
            texture_descriptors,
            camera_buffer,
            present_index: 0,
            viewport: [viewports[0].width, viewports[0].height],
            pass,
            vertex_shader,
            vertex_last_id: 0,
            fragment_shader,
            fragment_last_id: 0,
            scissors,
            viewports,
        };
        renderer.build_pipeline();
        renderer
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}
