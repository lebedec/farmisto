use crate::engine::armature::{PoseBuffer, PoseUniform};
use crate::engine::base::{Pipeline, Screen, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{
    IndexBuffer, PipelineAsset, ShaderAsset, SpineAsset, SpriteAsset, TextureAsset, VertexBuffer,
};
use crate::Assets;
use std::hint::spin_loop;

use ash::{vk, Device};
use glam::{vec3, Mat4};
use log::{debug, error, info};

use game::physics::SpaceId;
use rusty_spine::controller::SkeletonController;
use rusty_spine::AttachmentType;
use std::time::Instant;

pub struct SpriteRenderer {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    sprites: Vec<Sprite>,
    spine_sprites: Vec<SpineSprite>,
    layout: vk::PipelineLayout,
    pipeline_asset: PipelineAsset,
    pipeline: vk::Pipeline,
    pipeline_spine_asset: PipelineAsset,
    pipeline_spine: vk::Pipeline,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    camera_buffer: UniformBuffer,
    vertex_buffer: VertexBuffer,
    pub present_index: u32,
    pass: vk::RenderPass,
    pub material_slot: ShaderDataSet<2>,
    lut_texture: TextureAsset,
    screen: Screen,
}

pub struct SpineSpriteController {
    pub skeleton: SkeletonController,
    pub buffers: Vec<VertexBuffer>,
    pub index_buffers: Vec<IndexBuffer>,
    pub textures: Vec<TextureAsset>,
}

pub struct SpineSprite {
    buffers: Vec<VertexBuffer>,
    index_buffers: Vec<IndexBuffer>,
    textures: Vec<vk::DescriptorSet>,
    position: [f32; 2],
}

impl SpriteRenderer {
    pub fn look_at(&mut self) {
        let width = self.screen.width() as f32;
        let height = self.screen.height() as f32;
        let uniform = CameraUniform {
            model: Mat4::IDENTITY,
            view: Mat4::look_at_rh(
                vec3(0.0, 0.0, -1.0), // Vulkan Z: inside screen
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, -1.0, 0.0), // Vulkan Y: bottom screen
            ),
            proj: Mat4::orthographic_rh_gl(0.0, width, height, 0.0, 0.0, 100.0),
        };
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
        self.spine_sprites.clear();
    }

    pub fn update(&mut self) {
        if self.pipeline_asset.changed {
            self.rebuild_pipeline();
            self.pipeline_asset.changed = false;
        }
        if self.pipeline_spine_asset.changed {
            self.rebuild_pipeline_spine();
            self.pipeline_spine_asset.changed = false;
        }
    }

    pub fn draw(&mut self, asset: &SpriteAsset, position: [f32; 2]) {
        let texture = &asset.texture;
        self.sprites.push(Sprite {
            asset: asset.share(),
            texture: self.material_slot.describe(
                texture.id(),
                vec![[
                    ShaderData::from(texture),
                    ShaderData::from(&self.lut_texture),
                ]],
            )[0],
            position,
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
        let lut_texture = assets.texture("./assets/texture/lut-night.png");
        let pipeline_asset = assets.pipeline("sprites");
        let pipeline_spine_asset = assets.pipeline("spines");
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
            [
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ],
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

        let layout = unsafe {
            device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        };

        let mut renderer = Self {
            device: device.clone(),
            device_memory: device_memory.clone(),
            sprites: vec![],
            spine_sprites: vec![],
            layout,
            pipeline_asset,
            pipeline: vk::Pipeline::null(),
            pipeline_spine_asset,
            pipeline_spine: vk::Pipeline::null(),
            descriptor_sets,
            camera_buffer,
            vertex_buffer,
            present_index: 0,
            pass,
            material_slot: material_data,
            lut_texture,
            screen,
        };
        renderer.rebuild_pipeline();
        renderer.rebuild_pipeline_spine();
        renderer
    }

    pub fn instantiate(&mut self, spine: &SpineAsset) -> SpineSpriteController {
        let skeleton = SkeletonController::new(spine.skeleton.clone(), spine.animation.clone());
        let mut buffers = vec![];
        let mut index_buffers = vec![];
        let mut textures = vec![];

        let vertices: Vec<SpriteVertex> = vec![];

        for index in 0..skeleton.skeleton.slots_count() {
            let slot = skeleton.skeleton.draw_order_at_index(index).unwrap();
            if let Some(attachment) = slot.attachment() {
                match attachment.attachment_type() {
                    AttachmentType::Region => {
                        let region = attachment.as_region().unwrap();
                        info!(
                            "{}: REGION {} {}x{}",
                            slot.data().name(),
                            attachment.name(),
                            region.width(),
                            region.height()
                        );

                        let mut spine_vertices = vec![0.0; 8];
                        unsafe {
                            region.compute_world_vertices(&slot, &mut spine_vertices, 0, 2);
                        }
                        let spine_uvs = region.uvs();
                        let mut vertices = vec![];
                        for i in 0..4 {
                            vertices.push(SpriteVertex {
                                position: [spine_vertices[i * 2], -spine_vertices[i * 2 + 1]],
                                uv: [spine_uvs[i * 2], 1.0 - spine_uvs[i * 2 + 1]], // inverse
                            });
                        }

                        unsafe {
                            let mut obj = region.renderer_object();
                            let mut obj2 = obj.get_atlas_region();
                            let page = obj2.unwrap();
                            let mut obj3 = page.page();
                            let mut obj4 = obj3.renderer_object();
                            let texture: &mut TextureAsset = obj4.get_unchecked();
                            textures.push(texture.clone());
                        }

                        //
                        //info!("VERTICES: {:?}", vertices);
                        let buffer =
                            VertexBuffer::create(&self.device, &self.device_memory, vertices);
                        buffers.push(buffer);

                        let indices = vec![0, 1, 2, 2, 3, 0];
                        let buffer =
                            IndexBuffer::create(&self.device, &self.device_memory, indices);
                        index_buffers.push(buffer);
                    }
                    AttachmentType::Mesh => {
                        let mesh = attachment.as_mesh().unwrap();
                        info!(
                            "{}: MESH {} {}x{}",
                            slot.data().name(),
                            attachment.name(),
                            mesh.width(),
                            mesh.height()
                        )
                    }
                    attachment_type => {
                        error!("Unknown attachment type {:?}", attachment_type)
                    }
                }
            }
        }

        SpineSpriteController {
            skeleton,
            buffers,
            index_buffers,
            textures,
        }
    }

    pub fn draw_spine(&mut self, sprite: &SpineSpriteController) {
        let mut textures = vec![];
        for texture in sprite.textures.iter() {
            let tex = self.material_slot.describe(
                texture.id(),
                vec![[
                    ShaderData::from(texture),
                    ShaderData::from(&self.lut_texture),
                ]],
            )[0];
            textures.push(tex);
        }
        self.spine_sprites.push(SpineSprite {
            buffers: sprite.buffers.clone(),
            index_buffers: sprite.index_buffers.clone(),
            textures: textures,
            position: [512.0, 512.0],
        })
    }

    pub unsafe fn render(&self, device: &Device, buffer: vk::CommandBuffer) {
        device.cmd_set_viewport(buffer, 0, &self.screen.viewports);
        device.cmd_set_scissor(buffer, 0, &self.screen.scissors);

        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        let descriptor_set = &[self.descriptor_sets[0]];
        device.cmd_bind_descriptor_sets(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.layout,
            0,
            descriptor_set,
            &[],
        );
        device.cmd_bind_vertex_buffers(buffer, 0, &[self.vertex_buffer.bind()], &[0]);
        let mut previous_texture = Default::default();
        for sprite in self.sprites.iter() {
            if previous_texture != sprite.texture {
                previous_texture = sprite.texture;
                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.layout,
                    1,
                    &[sprite.texture],
                    &[],
                );
            }
            device.cmd_push_constants(
                buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::bytes_of(&sprite.uniform()),
            );
            device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        }

        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_spine);
        for sprite in self.spine_sprites.iter() {
            for i in 0..sprite.buffers.len() {
                let b = sprite.buffers[i];
                let t = sprite.textures[i];
                let i = sprite.index_buffers[i];
                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.layout,
                    1,
                    &[t],
                    &[],
                );
                device.cmd_bind_vertex_buffers(buffer, 0, &[b.bind()], &[0]);
                device.cmd_bind_index_buffer(buffer, i.bind(), 0, vk::IndexType::UINT32);
                device.cmd_push_constants(
                    buffer,
                    self.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    bytemuck::bytes_of(&SpriteUniform {
                        position: sprite.position,
                        size: [1.0, 1.0],
                        coords: [0.0, 0.0, 0.0, 0.0],
                    }),
                );
                device.cmd_draw_indexed(buffer, 6, 1, 0, 0, 1);
            }
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
            .vertex(self.pipeline_asset.vertex.module)
            .fragment(self.pipeline_asset.fragment.module)
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

    pub fn rebuild_pipeline_spine(&mut self) {
        let time = Instant::now();
        debug!(
            "Prepare pipeline spine layout={:?} pass={:?}",
            self.layout, self.pass
        );
        let building = Pipeline::new()
            .layout(self.layout)
            .vertex(self.pipeline_spine_asset.vertex.module)
            .fragment(self.pipeline_spine_asset.fragment.module)
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
                info!("Build pipeline spine in {:?}", time.elapsed());
                self.pipeline_spine = pipeline;
            }
            Err(error) => {
                error!("Unable to build pipeline spine, {:?}", error);
            }
        }
    }
}

pub struct Sprite {
    asset: SpriteAsset,
    texture: vk::DescriptorSet,
    position: [f32; 2],
}

impl Sprite {
    #[inline]
    pub fn uniform(&self) -> SpriteUniform {
        let asset = &self.asset;
        let image_w = asset.texture.width() as f32;
        let image_h = asset.texture.height() as f32;
        let [sprite_x, sprite_y] = asset.position;
        let [sprite_w, sprite_h] = asset.size;
        let x = sprite_x / image_w;
        let y = sprite_y / image_h;
        let w = sprite_w / image_w;
        let h = sprite_h / image_h;
        SpriteUniform {
            position: self.position,
            size: asset.size,
            coords: [x, y, w, h],
        }
    }
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
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
];
