use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::base::{Pipeline, Screen, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{ShaderAsset, TextureAsset, VertexBuffer};
use crate::Assets;

use ash::{vk, Device};
use glam::{vec3, Mat4};
use log::{debug, error, info};

use std::time::Instant;

pub struct SpriteRenderer {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    sprites: Vec<Sprite>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    camera_buffer: UniformBuffer,
    vertex_buffer: VertexBuffer,
    pub present_index: u32,
    pass: vk::RenderPass,

    pub material_slot: ShaderDataSet<1>,
    vertex_shader: ShaderAsset,
    fragment_shader: ShaderAsset,

    screen: Screen,
}

impl SpriteRenderer {
    pub fn look_at(&mut self) {
        let width = self.screen.width() as f32;
        let height = self.screen.height() as f32;
        // GLM was originally designed for OpenGL,
        // where the Y coordinate of the clip coordinates is inverted
        // let inverted = Mat4::from_scale(vec3(1.0, -1.0, 1.0));
        //
        // let uniform = CameraUniform {
        //     model: Mat4::IDENTITY,
        //     view: Mat4::IDENTITY,
        //     proj: Mat4::orthographic_rh(
        //         0.0,
        //         self.screen.width() as f32,
        //         self.screen.height() as f32,
        //         0.0,
        //         0.1,
        //         1000.0,
        //     ) * inverted,
        // };

        let inverted = Mat4::from_scale(vec3(1.0, -1.0, 1.0));

        let uniform = CameraUniform {
            model: Mat4::IDENTITY,
            view: Mat4::look_at_rh(
                vec3(0.0, -5.0, -5.0), // Vulkan Z: inside screen
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, -1.0, 0.0), // Vulkan Y: bottom screen
            ),
            proj: Mat4::perspective_rh(45.0_f32.to_radians(), width / height, 0.1, 1000.0)
                * inverted,
        };

        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    pub fn update(&self) {}

    pub fn draw(&mut self, texture: &TextureAsset) {
        self.sprites.push(Sprite {
            texture: self
                .material_slot
                .describe(texture.id(), vec![[ShaderData::from(texture)]])[0],
            sprite: SpriteUniform {
                position: [0.0, 0.0],
                size: [1.0, 1.0],
                coords: [0.0, 0.0, 1.0, 1.0],
            },
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
        let fragment_shader = assets.shader("./assets/shaders/sprite.frag.spv");
        let vertex_shader = assets.shader("./assets/shaders/sprite.vert.spv");
        //
        let camera_buffer =
            UniformBuffer::create::<CameraUniform>(device.clone(), device_memory, swapchain);

        let vertex_buffer = VertexBuffer::create(device, device_memory, SPRITE_VERTICES.to_vec());

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
            device_memory: device_memory.clone(),
            sprites: vec![],
            layout: pipeline_layout,
            pipeline: vk::Pipeline::null(),
            descriptor_sets,
            camera_buffer,
            vertex_buffer,
            present_index: 0,
            pass,
            material_slot: material_data,
            vertex_shader: vertex_shader.clone(),
            fragment_shader,
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
        device.cmd_bind_vertex_buffers(buffer, 0, &[self.vertex_buffer.bind()], &[0]);

        let mut previous_texture = Default::default();
        for object in self.sprites.iter() {
            if previous_texture != object.texture {
                previous_texture = object.texture;
                device.cmd_bind_descriptor_sets(
                    buffer,
                    point,
                    self.layout,
                    1,
                    &[object.texture],
                    &[],
                );
            }
            device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::bytes_of(&object.sprite),
            );
            device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        }
    }

    pub fn rebuild_pipeline(&mut self) {
        let time = Instant::now();
        debug!(
            "Prepare pipeline layout={:?} pass={:?}",
            self.layout, self.pass
        );
        let building = Pipeline::new()
            .layout(self.layout)
            .vertex(self.vertex_shader.module())
            .fragment(self.fragment_shader.module())
            .pass(self.pass)
            .build(
                &self.device,
                &self.screen.scissors,
                &self.screen.viewports,
                &SpriteVertex::ATTRIBUTES,
                &SpriteVertex::BINDINGS,
            );
        match building {
            Ok(pipeline) => {
                info!("Build pipeline in {:?}", time.elapsed());
                self.pipeline = pipeline;
            }
            Err(error) => {
                error!("Unable to build pipeline, {:?}", error);
            }
        }
    }
}

pub struct Sprite {
    texture: vk::DescriptorSet,
    sprite: SpriteUniform,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpriteUniform {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub coords: [f32; 4],
}

#[derive(Default, Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Deserialize)]
#[repr(C)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl SpriteVertex {
    pub const BINDINGS: [vk::VertexInputBindingDescription; 1] =
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<SpriteVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 2] = [
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
    ];
}

const SPRITE_VERTICES: [SpriteVertex; 6] = [
    SpriteVertex {
        position: [-0.5, -0.5],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, -0.5],
        uv: [1.0, 0.0],
    },
    SpriteVertex {
        position: [0.5, -0.5],
        uv: [1.0, 0.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, 0.5],
        uv: [1.0, 1.0],
    },
];
