use crate::engine::base::{Screen, ShaderData, ShaderDataSet};
use crate::engine::sprites::SpriteVertex;
use crate::engine::PipelineAsset;
use ash::vk::{ImageView, Sampler};
use ash::{vk, Device};
use bytemuck::NoUninit;
use log::{error, info};
use std::ffi::CStr;
use std::marker::PhantomData;
use std::time::Instant;

pub struct MyPipeline<const M: usize, C, const D: usize> {
    device: Device,
    asset: PipelineAsset,
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pass: vk::RenderPass,
    pub camera: ShaderDataSet<1>,
    pub material: ShaderDataSet<M>,
    pub data: Option<ShaderDataSet<D>>,
    _constants: PhantomData<C>,
}

impl<const M: usize, C, const D: usize> MyPipeline<M, C, D>
where
    C: NoUninit,
{
    pub fn build(asset: PipelineAsset, pass: vk::RenderPass) -> MyPipelineBuilder<M, C, D> {
        MyPipelineBuilder::new(asset, pass)
    }

    pub fn bind_camera(
        &mut self,
        camera_descriptor: vk::DescriptorSet,
        device: &Device,
        buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &[camera_descriptor],
                &[],
            );
        }
    }

    pub fn bind_material(
        &mut self,
        textures: [(Sampler, ImageView); M],
        device: &Device,
        buffer: vk::CommandBuffer,
    ) {
        let descriptor = self
            .material
            .describe(vec![textures.map(|(sampler, image_view)| {
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler,
                    image_view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                })
            })])[0];
        unsafe {
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                1,
                &[descriptor],
                &[],
            );
        }
    }

    pub fn push_constants(&self, constants: C, buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&constants),
            );
        }
    }

    pub fn update(&mut self, device: &Device, screen: &Screen) {
        if self.asset.changed {
            self.rebuild(device, screen);
            self.asset.changed = false;
        }
    }

    pub fn rebuild(&mut self, device: &Device, screen: &Screen) {
        let time = Instant::now();
        let building = Pipeline::new()
            .layout(self.layout)
            .vertex(self.asset.vertex.module)
            .fragment(self.asset.fragment.module)
            .pass(self.pass)
            .build(
                &device,
                &screen.scissors,
                &screen.viewports,
                &SpriteVertex::ATTRIBUTES,
                &SpriteVertex::BINDINGS,
            );
        match building {
            Ok(handle) => {
                info!("Build pipeline in {:?}", time.elapsed());
                self.handle = handle;
            }
            Err(error) => {
                error!("Unable to build pipeline, {:?}", error);
            }
        }
    }
}

pub struct MyPipelineBuilder<const M: usize, C, const D: usize> {
    asset: PipelineAsset,
    pass: vk::RenderPass,
    camera: [vk::DescriptorType; 1],
    material: Option<[vk::DescriptorType; M]>,
    data: Option<[vk::DescriptorType; D]>,
    _constants: PhantomData<C>,
}

impl<const M: usize, C: NoUninit, const D: usize> MyPipelineBuilder<M, C, D> {
    pub fn new(asset: PipelineAsset, pass: vk::RenderPass) -> Self {
        Self {
            asset,
            pass,
            camera: [vk::DescriptorType::UNIFORM_BUFFER],
            material: None,
            data: None,
            _constants: Default::default(),
        }
    }

    pub fn material(mut self, bindings: [vk::DescriptorType; M]) -> Self {
        self.material = Some(bindings);
        self
    }

    pub fn data(mut self, bindings: [vk::DescriptorType; D]) -> Self {
        self.data = Some(bindings);
        self
    }

    pub fn build(self, device: &Device, screen: &Screen) -> MyPipeline<M, C, D> {
        let swapchain = 0;
        let camera =
            ShaderDataSet::create(device.clone(), 3, vk::ShaderStageFlags::VERTEX, self.camera);
        let material = ShaderDataSet::create(
            device.clone(),
            8,
            vk::ShaderStageFlags::FRAGMENT,
            self.material.unwrap(),
        );

        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: std::mem::size_of::<C>() as u32,
        }];
        let mut sets = vec![camera.layout, material.layout];
        let data = match self.data {
            None => None,
            Some(bindings) => {
                let data = ShaderDataSet::create(
                    device.clone(),
                    6,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, // vertex ?
                    bindings,
                );
                sets.push(data.layout);
                Some(data)
            }
        };
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&sets)
            .push_constant_ranges(&push_constant_ranges);
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() };
        let mut pipeline = MyPipeline {
            device: device.clone(),
            asset: self.asset,
            handle: vk::Pipeline::null(),
            layout,
            pass: self.pass,
            camera,
            material,
            data,
            _constants: Default::default(),
        };
        pipeline.rebuild(device, screen);
        pipeline
    }
}

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
