use crate::engine::mesh::{IndexBuffer, Transform, Vertex, VertexBuffer};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::TextureAsset;
use ash::vk::{CommandBuffer, DescriptorSetLayout};
use ash::{vk, Device};
use log::info;
use std::ffi::CStr;
use std::ptr;

pub struct Material {}

pub struct MyRenderObject {
    vertex: VertexBuffer,
    index: IndexBuffer,
    transform: Transform,
    texture: TextureAsset,
}

pub struct MyRenderer {
    device: Device,
    swapchain: usize,
    objects: Vec<MyRenderObject>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub texture_set_layout: DescriptorSetLayout,
}

impl MyRenderer {
    pub fn draw(
        &mut self,
        vertex: VertexBuffer,
        index: IndexBuffer,
        transform: Transform,
        texture: TextureAsset,
    ) {
        self.objects.push(MyRenderObject {
            vertex,
            index,
            transform,
            texture,
        })
    }

    pub unsafe fn render(&self, device: &Device, buffer: CommandBuffer) {
        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        for (index, object) in self.objects.iter().enumerate() {
            // todo: if last != object.mesh
            {
                device.cmd_bind_vertex_buffers(buffer, 0, &[object.vertex.bind()], &[0]);
                device.cmd_bind_index_buffer(buffer, object.index.bind(), 0, vk::IndexType::UINT32);
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
                    &[object.texture.descriptor()],
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
            device.cmd_draw_indexed(buffer, object.index.count(), 1, 0, 0, 1);
        }
    }

    pub fn create<'a>(
        device: &Device,
        scissors: &'a [vk::Rect2D],
        viewports: &'a [vk::Viewport],
        swapchain: usize,
        fragment_shader_module: vk::ShaderModule,
        vertex_shader_module: vk::ShaderModule,
        camera_buffer: &UniformBuffer,
        renderpass: vk::RenderPass,
    ) -> Self {
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
        //

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

        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader_module,
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
            .scissors(scissors)
            .viewports(viewports);

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
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .build();

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        let pipeline = graphics_pipelines[0];
        Self {
            device: device.clone(),
            swapchain,
            objects: vec![],
            layout: pipeline_layout,
            pipeline,
            descriptor_sets,
            texture_set_layout,
        }
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}
