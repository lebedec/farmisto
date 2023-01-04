use std::thread;
use std::time::Instant;

use ash::{vk, Device};
use game::planting::Cell;
use glam::{vec3, Mat4};
use lazy_static::lazy_static;
use log::{error, info};
use rusty_spine::controller::SkeletonController;
use rusty_spine::{AttachmentType, Skin};
use sdl2::render::Texture;

use crate::engine::armature::PoseUniform;
use crate::engine::base::{MyPipeline, Screen, ShaderData, ShaderDataSet};
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::{
    IndexBuffer, SamplerAsset, ShaderAsset, SpineAsset, SpriteAsset, TextureAsset, VertexBuffer,
};
use crate::monitoring::Timer;
use crate::Assets;

lazy_static! {
    static ref METRIC_RENDER_SECONDS: prometheus::HistogramVec =
        prometheus::register_histogram_vec!(
            "sprites_render_seconds",
            "sprites_render_seconds",
            &["pipeline"]
        )
        .unwrap();
}

pub struct SpriteRenderer {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    sprites: Vec<Sprite>,
    spine_sprites: Vec<SpineSprite>,
    my_spine_pipeline: MyPipeline<2, SpinePushConstants, 0>,
    my_ground_pipeline: MyPipeline<1, SpritePushConstants, 1>,
    camera_buffer: UniformBuffer,
    ground_buffer: UniformBuffer,
    vertex_buffer: VertexBuffer,
    pub present_index: u32,
    lut_texture: TextureAsset,
    coloration_texture: TextureAsset,
    coloration_sampler: SamplerAsset,
    ground_sprites: Vec<GroundSprite>,
    pub screen: Screen,
    pub zoom: f32,
}

impl SpriteRenderer {
    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        screen: Screen,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
        zoom: f32,
    ) -> Self {
        let lut_texture = assets.texture("./assets/texture/lut-night.png");
        let coloration_texture = assets.texture("./assets/spine/lama384/coloration.png");
        let coloration_sampler = assets.sampler("coloration");
        //
        let camera_buffer =
            UniformBuffer::create::<CameraUniform>(device.clone(), device_memory, swapchain);

        let ground_buffer =
            UniformBuffer::create::<GroundUniform>(device.clone(), device_memory, swapchain);

        let vertex_buffer = VertexBuffer::create(device, device_memory, SPRITE_VERTICES.to_vec());

        let my_spine_pipeline = MyPipeline::build(assets.pipeline("spines"), pass)
            .material([
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ])
            .build(device, &screen);

        let my_ground_pipeline = MyPipeline::build(assets.pipeline("ground"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);

        Self {
            device: device.clone(),
            device_memory: device_memory.clone(),
            sprites: vec![],
            spine_sprites: vec![],
            my_spine_pipeline,
            my_ground_pipeline,
            camera_buffer,
            ground_buffer,
            vertex_buffer,
            present_index: 0,
            lut_texture,
            coloration_texture,
            coloration_sampler,
            ground_sprites: vec![],
            screen,
            zoom,
        }
    }

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
            proj: Mat4::orthographic_rh_gl(
                0.0,
                width * self.zoom,
                height * self.zoom,
                0.0,
                0.0,
                100.0,
            ),
        };
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
        self.spine_sprites.clear();
        self.ground_sprites.clear();
    }

    pub fn update(&mut self) {
        self.my_spine_pipeline.update(&self.device, &self.screen);
        self.my_ground_pipeline.update(&self.device, &self.screen);
    }

    pub fn draw_sprite(&mut self, asset: &SpriteAsset, position: [f32; 2]) {
        let texture = &asset.texture;
        let image_w = asset.texture.width() as f32;
        let image_h = asset.texture.height() as f32;
        let [sprite_x, sprite_y] = asset.position;
        let [sprite_w, sprite_h] = asset.size;
        let x = sprite_x / image_w;
        let y = sprite_y / image_h;
        let w = sprite_w / image_w;
        let h = sprite_h / image_h;
        self.sprites.push(Sprite {
            uniform: SpritePushConstants {
                position,
                size: asset.size,
                coords: [x, y, w, h],
            },
            texture_descriptor: self.my_spine_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: asset.sampler.handle,
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
                ShaderData::from(&self.lut_texture),
            ]])[0],
        })
    }

    pub fn draw_texture(&mut self, texture: &TextureAsset, position: [f32; 2]) {
        self.sprites.push(Sprite {
            uniform: SpritePushConstants {
                position: [
                    position[0] + texture.widthf() / 2.0,
                    position[1] + texture.heightf() / 2.0,
                ],
                size: [texture.width() as f32, texture.height() as f32],
                coords: [0.0, 0.0, 1.0, 1.0],
            },
            texture_descriptor: self.my_spine_pipeline.material.describe(vec![[
                ShaderData::from(texture),
                ShaderData::from(&self.lut_texture),
            ]])[0],
        })
    }

    pub fn draw_spine(&mut self, sprite: &SpineSpriteController, position: [f32; 2]) {
        self.update_spine_buffers(sprite);
        let texture = &sprite.mega_texture;
        self.spine_sprites.push(SpineSprite {
            buffer: sprite.mega_buffer.clone(),
            index_buffer: sprite.mega_index_buffer.clone(),
            texture_descriptor: self.my_spine_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: texture.sampler(),
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: self.coloration_sampler.handle,
                    image_view: self.coloration_texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
            position,
            colors: sprite.colors,
            counters: sprite.counters.clone(),
        })
    }

    pub fn draw_ground(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        input: &Vec<Vec<Cell>>,
    ) {
        let mut map: [[[f32; 4]; 28]; 16] = Default::default();
        for y in 0..16 {
            for x in 0..28 {
                let [capacity, moisture] = input[y][x];
                map[y][x] = [capacity, moisture, 0.0, 0.0];
            }
        }
        self.ground_sprites.push(GroundSprite {
            texture,
            sampler,
            uniform: GroundUniform { map },
        })
    }

    pub fn instantiate(
        &mut self,
        spine: &SpineAsset,
        features: [String; 2],
        colors: [[f32; 4]; 4],
    ) -> SpineSpriteController {
        let mut skeleton = SkeletonController::new(spine.skeleton.clone(), spine.animation.clone());

        let [head, tail] = features;
        let mut skin = Skin::new("lama-dynamic-848");
        let head = spine.skeleton.find_skin(&head).unwrap();
        let tail = spine.skeleton.find_skin(&tail).unwrap();
        skin.add_skin(&head);
        skin.add_skin(&tail);
        skeleton.skeleton.set_skin(&skin);

        skeleton
            .animation_state
            .set_animation_by_name(0, "default", true)
            .unwrap();

        let mut mega_vertices: Vec<SpriteVertex> = vec![];
        let mut mega_indices: Vec<u32> = vec![];
        let mut mega_counters: Vec<(u32, u32)> = vec![];

        let mut index_offset = 0;
        for index in 0..skeleton.skeleton.slots_count() {
            let slot = skeleton.skeleton.draw_order_at_index(index).unwrap();
            if let Some(attachment) = slot.attachment() {
                match attachment.attachment_type() {
                    AttachmentType::Region => {
                        let region = attachment.as_region().unwrap();
                        // info!(
                        //     "{}: spine REGION {} {}x{}",
                        //     slot.data().name(),
                        //     attachment.name(),
                        //     region.width(),
                        //     region.height()
                        // );

                        let mut spine_vertices = vec![0.0; 8];
                        unsafe {
                            region.compute_world_vertices(&slot, &mut spine_vertices, 0, 2);
                        }
                        let spine_uvs = region.uvs();
                        for i in 0..4 {
                            mega_vertices.push(SpriteVertex {
                                position: [spine_vertices[i * 2], -spine_vertices[i * 2 + 1]],
                                uv: [spine_uvs[i * 2], 1.0 - spine_uvs[i * 2 + 1]], // inverse
                            })
                        }
                        let indices = [0, 1, 2, 2, 3, 0].map(|index| index + index_offset);
                        mega_indices.extend_from_slice(&indices);
                        mega_counters.push((4, 6)); // 4 vertex, 6 indices

                        index_offset += 4;
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

        let mega_buffer = VertexBuffer::create(&self.device, &self.device_memory, mega_vertices);

        let mega_index_buffer =
            IndexBuffer::create(&self.device, &self.device_memory, mega_indices);

        SpineSpriteController {
            skeleton,
            mega_buffer,
            mega_index_buffer,
            mega_texture: spine.atlas.clone(),
            counters: mega_counters,
            colors,
        }
    }

    pub fn update_spine_buffers(&mut self, controller: &SpineSpriteController) {
        let mut mega_vertices = vec![];
        for index in 0..controller.skeleton.skeleton.slots_count() {
            let slot = controller
                .skeleton
                .skeleton
                .draw_order_at_index(index)
                .unwrap();
            if let Some(attachment) = slot.attachment() {
                match attachment.attachment_type() {
                    AttachmentType::Region => {
                        let region = attachment.as_region().unwrap();
                        let mut spine_vertices = vec![0.0; 8];
                        unsafe {
                            region.compute_world_vertices(&slot, &mut spine_vertices, 0, 2);
                        }
                        let spine_uvs = region.uvs();
                        for i in 0..4 {
                            mega_vertices.push(SpriteVertex {
                                position: [spine_vertices[i * 2], -spine_vertices[i * 2 + 1]],
                                uv: [spine_uvs[i * 2], 1.0 - spine_uvs[i * 2 + 1]], // inverse
                            });
                        }
                    }
                    AttachmentType::Mesh => {
                        let mesh = attachment.as_mesh().unwrap();
                        info!(
                            "{}: UPDATE MESH {} {}x{}",
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
        controller.mega_buffer.update(mega_vertices, &self.device);
    }

    pub unsafe fn render(&mut self, device: &Device, buffer: vk::CommandBuffer) {
        device.cmd_set_viewport(buffer, 0, &self.screen.viewports);
        device.cmd_set_scissor(buffer, 0, &self.screen.scissors);
        // device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        // let camera_descriptor =
        //     self.camera_data_set
        //         .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
        //             buffer: self.camera_buffer.buffers[0],
        //             offset: 0,
        //             range: std::mem::size_of::<CameraUniform>() as u64,
        //         })]])[0];
        // device.cmd_bind_descriptor_sets(
        //     buffer,
        //     vk::PipelineBindPoint::GRAPHICS,
        //     self.layout,
        //     0,
        //     &[camera_descriptor],
        //     &[],
        // );
        // device.cmd_bind_vertex_buffers(buffer, 0, &[self.vertex_buffer.bind()], &[0]);
        // let mut previous_texture = Default::default();
        // for sprite in self.sprites.iter() {
        //     if previous_texture != sprite.texture_descriptor {
        //         previous_texture = sprite.texture_descriptor;
        //         device.cmd_bind_descriptor_sets(
        //             buffer,
        //             vk::PipelineBindPoint::GRAPHICS,
        //             self.layout,
        //             1,
        //             &[sprite.texture_descriptor],
        //             &[],
        //         );
        //     }
        //     // device.cmd_push_constants(
        //     //     buffer,
        //     //     self.layout,
        //     //     vk::ShaderStageFlags::VERTEX,
        //     //     0,
        //     //     bytemuck::bytes_of(&sprite.uniform),
        //     // );
        //     // device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        // }

        // TODO: SHARED descriptor
        let camera_descriptor =
            self.my_spine_pipeline
                .camera
                .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: self.camera_buffer.buffers[0],
                    offset: 0,
                    range: std::mem::size_of::<CameraUniform>() as u64,
                })]])[0];

        let mut timer = Timer::now();
        // GROUND
        for ground in &self.ground_sprites {
            self.ground_buffer
                .update(self.present_index as usize, ground.uniform);
            let ground_descriptor = self
                .my_ground_pipeline
                .data
                .as_mut()
                .unwrap()
                .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: self.ground_buffer.buffers[0],
                    offset: 0,
                    range: std::mem::size_of::<GroundUniform>() as u64,
                })]])[0];

            device.cmd_bind_pipeline(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.my_ground_pipeline.handle,
            );
            device.cmd_bind_vertex_buffers(buffer, 0, &[self.vertex_buffer.bind()], &[0]);
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.my_ground_pipeline.layout,
                0,
                &[camera_descriptor],
                &[],
            );
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.my_ground_pipeline.layout,
                1,
                &[self
                    .my_ground_pipeline
                    .material
                    .describe(vec![[ShaderData::Texture(vk::DescriptorImageInfo {
                        sampler: ground.sampler.handle,
                        image_view: ground.texture.view(),
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    })]])[0]],
                &[],
            );
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.my_ground_pipeline.layout,
                2,
                &[ground_descriptor],
                &[],
            );
            let constants = SpritePushConstants {
                position: [512.0, 512.0],
                size: [1024.0, 1024.0],
                coords: [0.0, 0.0, 1.0, 1.0],
            };
            self.my_ground_pipeline.push_constants(constants, buffer);
            device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        }
        timer.record("ground", &METRIC_RENDER_SECONDS);

        // SPINE
        device.cmd_bind_pipeline(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.my_spine_pipeline.handle,
        );
        device.cmd_bind_descriptor_sets(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.my_spine_pipeline.layout,
            0,
            &[camera_descriptor],
            &[],
        );
        for (index, sprite) in self.spine_sprites.iter().enumerate() {
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.my_spine_pipeline.layout,
                1,
                &[sprite.texture_descriptor],
                &[],
            );
            device.cmd_bind_vertex_buffers(buffer, 0, &[sprite.buffer.bind()], &[0]);
            device.cmd_bind_index_buffer(
                buffer,
                sprite.index_buffer.bind(),
                0,
                vk::IndexType::UINT32,
            );
            let constants = SpinePushConstants {
                position: sprite.position,
                colors: sprite.colors,
            };
            self.my_spine_pipeline.push_constants(constants, buffer);
            // device.cmd_draw_indexed(buffer, (sprite.counters.len() * 6) as u32, 1, 0, 0, 1);
        }
        timer.record("spine", &METRIC_RENDER_SECONDS);

        thread::sleep_ms(10);
    }
}

pub struct SpineSpriteController {
    pub skeleton: SkeletonController,
    pub mega_buffer: VertexBuffer,
    pub mega_index_buffer: IndexBuffer,
    pub mega_texture: TextureAsset,
    pub counters: Vec<(u32, u32)>,
    colors: [[f32; 4]; 4],
}

pub struct GroundSprite {
    texture: TextureAsset,
    sampler: SamplerAsset,
    uniform: GroundUniform,
}

pub struct SpineSprite {
    buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture_descriptor: vk::DescriptorSet,
    position: [f32; 2],
    colors: [[f32; 4]; 4],
    pub counters: Vec<(u32, u32)>,
}

pub struct Sprite {
    uniform: SpritePushConstants,
    texture_descriptor: vk::DescriptorSet,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpritePushConstants {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub coords: [f32; 4],
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpinePushConstants {
    pub colors: [[f32; 4]; 4],
    pub position: [f32; 2],
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

#[derive(Clone, Copy)]
pub struct GroundUniform {
    pub map: [[[f32; 4]; 28]; 16],
}

fn g(value: f32) -> [f32; 4] {
    [value, value, value, value]
}
