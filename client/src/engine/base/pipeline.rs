use ash::{vk, Device};
use std::ffi::CStr;

pub struct Pipeline {
    layout: vk::PipelineLayout,
    vertex: vk::ShaderModule,
    fragment: vk::ShaderModule,
    render_pass: vk::RenderPass,
    entry_name: Vec<u8>,
    front_face: vk::FrontFace,
    polygon_mode: vk::PolygonMode,
}

#[derive(Debug)]
pub enum PipelineError {
    Vulkan((Vec<vk::Pipeline>, vk::Result)),
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            layout: Default::default(),
            vertex: Default::default(),
            fragment: Default::default(),
            render_pass: Default::default(),
            entry_name: b"main\0".to_vec(),
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            polygon_mode: vk::PolygonMode::FILL,
        }
    }

    pub fn layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn vertex(mut self, module: vk::ShaderModule) -> Self {
        self.vertex = module;
        self
    }

    pub fn fragment(mut self, module: vk::ShaderModule) -> Self {
        self.fragment = module;
        self
    }

    pub fn pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.render_pass = render_pass;
        self
    }

    pub fn polygon_mode(mut self, polygon_mode: vk::PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }

    pub fn build(
        self,
        device: &Device,
        scissors: &Vec<vk::Rect2D>,
        viewports: &Vec<vk::Viewport>,
        attributes: &[vk::VertexInputAttributeDescription],
        bindings: &[vk::VertexInputBindingDescription],
    ) -> Result<vk::Pipeline, PipelineError> {
        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(&self.entry_name) };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: self.vertex,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.fragment,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(attributes)
            .vertex_binding_descriptions(bindings);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(scissors)
            .viewports(viewports);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(self.front_face)
            .line_width(1.0)
            .polygon_mode(self.polygon_mode);
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
            blend_enable: 1,
            color_blend_op: vk::BlendOp::ADD,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            alpha_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .attachments(&color_blend_attachment_states);
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);
        let info = vk::GraphicsPipelineCreateInfo::builder()
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
            .render_pass(self.render_pass)
            .build();
        let cache = vk::PipelineCache::null();
        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(cache, &[info], None)
                .map_err(PipelineError::Vulkan)?
        };
        Ok(graphics_pipelines[0])
    }
}
