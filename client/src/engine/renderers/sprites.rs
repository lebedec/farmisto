use ash::prelude::VkResult;
use ash::{vk, Device};
use game::building::{Platform, Shape};
use game::math::VectorMath;
use game::planting::Cell;
use glam::{vec3, Mat4, Vec3};
use lazy_static::lazy_static;
use log::{error, info};
use rusty_spine::controller::SkeletonController;
use rusty_spine::{AttachmentType, Skin};

use crate::engine::base::{index_memory_type, MyPipeline, Screen, ShaderData, ShaderDataSet};
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
    pub present_index: u32,
    pub screen: Screen,
    pub zoom: f32,

    camera_position: [f32; 2],
    camera_buffer: UniformBuffer,

    spine_pipeline: MyPipeline<2, SpinePushConstants, 0>,
    spine_sprites: Vec<SpineSprite>,
    coloration_texture: TextureAsset,
    coloration_sampler: SamplerAsset,

    ground_pipeline: MyPipeline<1, GroundPushConstants, 1>,
    ground_sprites: Vec<GroundSprite>,
    ground_vertex_buffer: VertexBuffer,
    ground_buffer: UniformBuffer,

    floor_buffer: UniformBuffer,

    roof_pipeline: MyPipeline<1, GroundPushConstants, 1>,
    roof_sprites: Vec<GroundSprite>,
    roof_vertex_buffer: VertexBuffer,
    roof_buffer: UniformBuffer,

    sprite_pipeline: MyPipeline<1, SpritePushConstants, 1>,
    sprites: Vec<Sprite>,
    sprite_vertex_buffer: VertexBuffer,

    light_map_pipeline: MyPipeline<1, SpritePushConstants, 1>,
    light_map_framebuffer: vk::Framebuffer,
    light_map_render_pass: vk::RenderPass,
    light_map_sampler: SamplerAsset,
    light_map_view: vk::ImageView,
    lights: Vec<Sprite>,
    lights_texture: SpriteAsset,
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

        let sprite_vertex_buffer =
            VertexBuffer::create(device, device_memory, SPRITE_VERTICES.to_vec());

        let spine_pipeline = MyPipeline::build(assets.pipeline("spines"), pass)
            .material([
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ])
            .build(device, &screen);

        let ground_buffer =
            UniformBuffer::create::<GroundUniform>(device.clone(), device_memory, swapchain);
        let ground_pipeline = MyPipeline::build(assets.pipeline("ground"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);
        let ground_vertex_buffer =
            VertexBuffer::create(device, device_memory, GROUND_VERTICES.to_vec());

        let floor_buffer =
            UniformBuffer::create::<GroundUniform>(device.clone(), device_memory, swapchain);

        let roof_buffer =
            UniformBuffer::create::<GroundUniform>(device.clone(), device_memory, swapchain);
        let roof_pipeline = MyPipeline::build(assets.pipeline("roof"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);
        let roof_vertex_buffer =
            VertexBuffer::create(device, device_memory, GROUND_VERTICES.to_vec());

        let sprite_pipeline = MyPipeline::build(assets.pipeline("sprites"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .build(device, &screen);

        let (light_map_view, light_map_render_pass, light_map_framebuffer) =
            Self::create_light_map(device, device_memory).unwrap();

        let light_map_pipeline =
            MyPipeline::build(assets.pipeline("light-map"), light_map_render_pass)
                .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
                .build(device, &screen);

        let mut renderer = Self {
            device: device.clone(),
            device_memory: device_memory.clone(),
            sprites: vec![],
            spine_sprites: vec![],
            spine_pipeline,
            ground_pipeline,
            camera_buffer,
            ground_buffer,
            floor_buffer,
            roof_pipeline,
            roof_buffer,
            roof_sprites: vec![],
            roof_vertex_buffer,
            sprite_vertex_buffer,
            present_index: 0,
            light_map_pipeline,
            light_map_framebuffer,
            light_map_render_pass,
            coloration_texture,
            coloration_sampler,
            ground_sprites: vec![],
            screen,
            zoom,
            sprite_pipeline,
            ground_vertex_buffer,
            camera_position: [0.0, 0.0],
            light_map_sampler: assets.sampler("light-map"),
            light_map_view,
            lights: vec![],
            lights_texture: assets.sprite("light-test"),
        };

        renderer
    }

    pub fn create_light_map(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
    ) -> VkResult<(vk::ImageView, vk::RenderPass, vk::Framebuffer)> {
        let light_color = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D {
                width: 960,
                height: 540,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED);
        let light_color = unsafe { device.create_image(&light_color, None)? };

        let memory = unsafe { device.get_image_memory_requirements(light_color) };
        let memory_type_index = index_memory_type(
            &memory,
            device_memory,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .unwrap();
        let memory_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&memory_allocate_info, None)? };
        unsafe {
            device.bind_image_memory(light_color, memory, 0)?;
        }

        let light_color_view = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(light_color);
        let light_color_view = unsafe { device.create_image_view(&light_color_view, None)? };

        let renderpass_attachments = [vk::AttachmentDescription {
            flags: Default::default(),
            format: vk::Format::R8G8B8A8_UNORM,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::SHADER_READ,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            },
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            },
        ];
        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);
        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let attachments = [light_color_view];
        let framebuffer = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(960)
            .height(540)
            .layers(1);
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer, None)? };

        Ok((light_color_view, render_pass, framebuffer))
    }

    pub fn look_at(&mut self, target: Vec3) {
        let width = self.screen.width() as f32;
        let height = self.screen.height() as f32;
        let uniform = CameraUniform {
            model: Mat4::from_translation(vec3(-target.x, target.y, 0.0)),
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
        self.camera_position = [target.x, -target.y];
        self.camera_buffer
            .update(self.present_index as usize, uniform);
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
        self.spine_sprites.clear();
        self.ground_sprites.clear();
        self.roof_sprites.clear();
    }

    pub fn update(&mut self) {
        self.spine_pipeline.update(&self.device, &self.screen);
        self.ground_pipeline.update(&self.device, &self.screen);
        self.sprite_pipeline.update(&self.device, &self.screen);
        self.roof_pipeline.update(&self.device, &self.screen);
    }

    pub fn draw_sprite(&mut self, asset: &SpriteAsset, position: [f32; 2], highlight: f32) {
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
                pivot: asset.pivot,
                highlight,
            },
            texture_descriptor: self.sprite_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: asset.sampler.handle,
                    image_view: texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
        })
    }

    pub fn draw_spine(&mut self, sprite: &SpineSpriteController, position: [f32; 2]) {
        self.update_spine_buffers(sprite);
        self.spine_sprites.push(SpineSprite {
            buffer: sprite.mega_buffer.clone(),
            index_buffer: sprite.mega_index_buffer.clone(),
            texture: sprite.mega_texture.clone(),
            position,
            colors: sprite.colors,
            counters: sprite.counters.clone(),
            constants: SpinePushConstants {
                colors: sprite.colors,
                position,
            },
        })
    }

    pub fn draw_ground(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        input: &Vec<Vec<Cell>>,
        shapes: &Vec<Shape>,
    ) {
        let mut global_interior_map = [0u128; Platform::SIZE_Y];
        for shape in shapes {
            if shape.id == Shape::EXTERIOR_ID {
                continue;
            }
            for (i, row) in shape.rows.iter().enumerate() {
                global_interior_map[shape.rows_y + i] = global_interior_map[shape.rows_y + i] | row;
            }
        }

        const CELL_SIZE: f32 = 128.0;
        let input_size = [input[0].len(), input.len()];
        let [input_size_x, input_size_y] = input_size;
        let offset_step = self.camera_position.div(CELL_SIZE).floor();
        let offset_step = offset_step.clamp(
            [0.0, 0.0],
            [
                (input_size_x - VISIBLE_MAP_X) as f32,
                (input_size_y - VISIBLE_MAP_Y) as f32,
            ],
        );
        let offset = offset_step.mul(CELL_SIZE);
        let mut map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y] = Default::default();
        for y in 0..VISIBLE_MAP_Y {
            for x in 0..VISIBLE_MAP_X {
                let [step_x, step_y] = offset_step;
                let iy = y + step_y as usize;
                let ix = x + step_x as usize;
                let [capacity, moisture] = input[iy][ix];
                let pos = 1 << (Platform::SIZE_X - ix - 1);
                let visible = if global_interior_map[iy] & pos == pos {
                    1.0
                } else {
                    0.0
                };
                map[y][x] = [capacity, moisture, 1.0, 0.0];
            }
        }
        self.ground_sprites.push(GroundSprite {
            texture,
            sampler,
            uniform: GroundUniform { map },
            constants: GroundPushConstants {
                offset,
                map_size: [VISIBLE_MAP_X as f32, VISIBLE_MAP_Y as f32],
                cell_size: [CELL_SIZE as f32, CELL_SIZE as f32],
                layer: 0.2,
            },
        })
    }

    pub fn draw_floor(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        input: &Vec<Vec<Cell>>,
        shapes: &Vec<Shape>,
    ) {
        let mut global_interior_map = [0u128; Platform::SIZE_Y];
        for shape in shapes {
            if shape.id == Shape::EXTERIOR_ID {
                continue;
            }
            for (i, row) in shape.rows.iter().enumerate() {
                global_interior_map[shape.rows_y + i] = global_interior_map[shape.rows_y + i] | row;
            }
        }

        const CELL_SIZE: f32 = 128.0;
        let input_size = [input[0].len(), input.len()];
        let [input_size_x, input_size_y] = input_size;
        let offset_step = self.camera_position.div(CELL_SIZE).floor();
        let offset_step = offset_step.clamp(
            [0.0, 0.0],
            [
                (input_size_x - VISIBLE_MAP_X) as f32,
                (input_size_y - VISIBLE_MAP_Y) as f32,
            ],
        );
        let offset = offset_step.mul(CELL_SIZE);
        let mut map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y] = Default::default();
        for y in 0..VISIBLE_MAP_Y {
            for x in 0..VISIBLE_MAP_X {
                let [step_x, step_y] = offset_step;
                let iy = y + step_y as usize;
                let ix = x + step_x as usize;
                let [capacity, moisture] = input[iy][ix];
                let pos = 1 << (Platform::SIZE_X - ix - 1);
                let visible = if global_interior_map[iy] & pos == pos {
                    1.0
                } else {
                    0.0
                };
                map[y][x] = [1.0, 1.0, visible, 0.0];
            }
        }
        self.ground_sprites.push(GroundSprite {
            texture,
            sampler,
            uniform: GroundUniform { map },
            constants: GroundPushConstants {
                offset,
                map_size: [VISIBLE_MAP_X as f32, VISIBLE_MAP_Y as f32],
                cell_size: [CELL_SIZE as f32, CELL_SIZE as f32],
                layer: 0.1,
            },
        })
    }

    pub fn draw_roof(
        &mut self,
        texture: TextureAsset,
        sampler: SamplerAsset,
        input: &Vec<Vec<Cell>>,
        shapes: &Vec<Shape>,
        exclude_shape: usize,
    ) {
        let mut global_interior_map = [0u128; Platform::SIZE_Y];
        for shape in shapes {
            if shape.id == Shape::EXTERIOR_ID || shape.id == exclude_shape {
                continue;
            }
            for (i, row) in shape.rows.iter().enumerate() {
                global_interior_map[shape.rows_y + i] = global_interior_map[shape.rows_y + i] | row;
            }
        }

        const CELL_SIZE: f32 = 128.0;
        let input_size = [input[0].len(), input.len()];
        let [input_size_x, input_size_y] = input_size;
        let offset_step = self.camera_position.div(CELL_SIZE).floor();
        let offset_step = offset_step.clamp(
            [0.0, 0.0],
            [
                (input_size_x - VISIBLE_MAP_X) as f32,
                (input_size_y - VISIBLE_MAP_Y) as f32,
            ],
        );
        let roof_offset = [0.0, -2.0].mul(CELL_SIZE);
        let offset = offset_step.mul(CELL_SIZE).add(roof_offset);
        let mut map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y] = Default::default();
        for y in 0..VISIBLE_MAP_Y {
            for x in 0..VISIBLE_MAP_X {
                let [step_x, step_y] = offset_step;
                let iy = y + step_y as usize;
                let ix = x + step_x as usize;
                let [capacity, moisture] = input[iy][ix];
                let pos = 1 << (Platform::SIZE_X - ix - 1);
                let visible = if global_interior_map[iy] & pos == pos {
                    1.0
                } else {
                    0.0
                };
                map[y][x] = [0.0, 1.0, visible, 0.0];
            }
        }
        self.roof_sprites.push(GroundSprite {
            texture,
            sampler,
            uniform: GroundUniform { map },
            constants: GroundPushConstants {
                offset,
                map_size: [VISIBLE_MAP_X as f32, VISIBLE_MAP_Y as f32],
                cell_size: [CELL_SIZE as f32, CELL_SIZE as f32],
                layer: 0.0,
            },
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

    pub unsafe fn render2(
        &mut self,
        device: &Device,
        buffer: vk::CommandBuffer,
        render_begin: &vk::RenderPassBeginInfo,
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.43, 0.51, 86.0, 0.2],
                // float32: [0.43, 0.0, 0.0, 0.2],
            },
        }];
        let render_begin2 = vk::RenderPassBeginInfo::builder()
            .render_pass(self.light_map_render_pass)
            .framebuffer(self.light_map_framebuffer)
            .render_area(
                vk::Extent2D {
                    width: 960,
                    height: 540,
                }
                .into(),
            )
            .clear_values(&clear_values);
        device.cmd_begin_render_pass(buffer, &render_begin2, vk::SubpassContents::INLINE);
        device.cmd_set_viewport(
            buffer,
            0,
            &vec![vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: 960.0 as f32,
                height: 540.0 as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
        );
        device.cmd_set_scissor(
            buffer,
            0,
            &[vk::Extent2D {
                width: 960,
                height: 540,
            }
            .into()],
        );
        let camera_descriptor = self
            .spine_pipeline
            .camera
            .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                buffer: self.camera_buffer.buffers[self.present_index as usize],
                offset: 0,
                range: std::mem::size_of::<CameraUniform>() as u64,
            })]])[0];
        device.cmd_bind_pipeline(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.light_map_pipeline.handle,
        );
        device.cmd_bind_descriptor_sets(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.light_map_pipeline.layout,
            0,
            &[camera_descriptor],
            &[],
        );
        device.cmd_bind_vertex_buffers(buffer, 0, &[self.sprite_vertex_buffer.bind()], &[0]);
        let mut previous_texture = Default::default();
        self.lights = vec![Sprite {
            uniform: SpritePushConstants {
                position: [256.0, 256.0],
                size: [256.0, 256.0],
                coords: [0.0, 0.0, 1.0, 1.0],
                pivot: [0.5, 0.5],
                highlight: 1.0,
            },
            texture_descriptor: self.light_map_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: self.lights_texture.sampler.handle,
                    image_view: self.lights_texture.texture.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
        }];
        for light in self.lights.iter() {
            if previous_texture != light.texture_descriptor {
                previous_texture = light.texture_descriptor;
                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.light_map_pipeline.layout,
                    1,
                    &[light.texture_descriptor],
                    &[],
                );
            }
            self.light_map_pipeline
                .push_constants(light.uniform, buffer);
            device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        }
        device.cmd_end_render_pass(buffer);

        device.cmd_begin_render_pass(buffer, &render_begin, vk::SubpassContents::INLINE);
        self.render(device, buffer);
        device.cmd_end_render_pass(buffer);
    }

    pub unsafe fn render(&mut self, device: &Device, buffer: vk::CommandBuffer) {
        let mut timer = Timer::now();

        device.cmd_set_viewport(buffer, 0, &self.screen.viewports);
        device.cmd_set_scissor(buffer, 0, &self.screen.scissors);

        // TODO: SHARED descriptor
        let camera_descriptor = self
            .spine_pipeline
            .camera
            .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                buffer: self.camera_buffer.buffers[self.present_index as usize],
                offset: 0,
                range: std::mem::size_of::<CameraUniform>() as u64,
            })]])[0];

        // GROUND

        for ground in &self.ground_sprites {
            device.cmd_bind_pipeline(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.ground_pipeline.handle,
            );
            device.cmd_bind_vertex_buffers(buffer, 0, &[self.ground_vertex_buffer.bind()], &[0]);
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.ground_pipeline.layout,
                0,
                &[camera_descriptor],
                &[],
            );

            if ground.constants.layer == 0.2 {
                self.ground_buffer
                    .update(self.present_index as usize, ground.uniform);
                let ground_descriptor = self.ground_pipeline.data.as_mut().unwrap().describe(vec![
                    [ShaderData::Uniform(vk::DescriptorBufferInfo {
                        buffer: self.ground_buffer.buffers[self.present_index as usize],
                        offset: 0,
                        range: std::mem::size_of::<GroundUniform>() as u64,
                    })],
                ])[0];

                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ground_pipeline.layout,
                    2,
                    &[ground_descriptor],
                    &[],
                );
            } else {
                self.floor_buffer
                    .update(self.present_index as usize, ground.uniform);
                let ground_descriptor = self.ground_pipeline.data.as_mut().unwrap().describe(vec![
                    [ShaderData::Uniform(vk::DescriptorBufferInfo {
                        buffer: self.floor_buffer.buffers[self.present_index as usize],
                        offset: 0,
                        range: std::mem::size_of::<GroundUniform>() as u64,
                    })],
                ])[0];

                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ground_pipeline.layout,
                    2,
                    &[ground_descriptor],
                    &[],
                );
            }

            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.ground_pipeline.layout,
                1,
                &[self
                    .ground_pipeline
                    .material
                    .describe(vec![[ShaderData::Texture(vk::DescriptorImageInfo {
                        sampler: ground.sampler.handle,
                        image_view: ground.texture.view(),
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    })]])[0]],
                &[],
            );
            self.ground_pipeline
                .push_constants(ground.constants, buffer);
            device.cmd_draw(buffer, GROUND_VERTICES.len() as u32, 1, 0, 0);
        }
        timer.record("ground", &METRIC_RENDER_SECONDS);

        /*
        LIGHTS
        self.sprites.push(Sprite {
            uniform: SpritePushConstants {
                position: [0.0, 0.0],
                size: self.screen.size_f32().mul(self.zoom),
                coords: [0.0, 0.0, 1.0, 1.0],
                pivot: [0.0, 0.0],
                highlight: 1.0,
            },
            texture_descriptor: self.sprite_pipeline.material.describe(vec![[
                ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: self.light_map_sampler.handle,
                    image_view: self.light_map_view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }),
            ]])[0],
        });*/

        // STATIC
        device.cmd_bind_pipeline(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.sprite_pipeline.handle,
        );
        device.cmd_bind_descriptor_sets(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.sprite_pipeline.layout,
            0,
            &[camera_descriptor],
            &[],
        );
        device.cmd_bind_vertex_buffers(buffer, 0, &[self.sprite_vertex_buffer.bind()], &[0]);
        let mut previous_texture = Default::default();
        for sprite in self.sprites.iter() {
            if previous_texture != sprite.texture_descriptor {
                previous_texture = sprite.texture_descriptor;
                device.cmd_bind_descriptor_sets(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.sprite_pipeline.layout,
                    1,
                    &[sprite.texture_descriptor],
                    &[],
                );
            }
            self.sprite_pipeline.push_constants(sprite.uniform, buffer);
            device.cmd_draw(buffer, SPRITE_VERTICES.len() as u32, 1, 0, 0);
        }
        timer.record("static", &METRIC_RENDER_SECONDS);

        // SPINE
        device.cmd_bind_pipeline(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.spine_pipeline.handle,
        );
        self.spine_pipeline
            .bind_camera(camera_descriptor, device, buffer);
        for sprite in self.spine_sprites.iter() {
            self.spine_pipeline.bind_material(
                [
                    (sprite.texture.sampler(), sprite.texture.view()),
                    (
                        self.coloration_sampler.handle,
                        self.coloration_texture.view(),
                    ),
                ],
                device,
                buffer,
            );
            device.cmd_bind_vertex_buffers(buffer, 0, &[sprite.buffer.bind()], &[0]);
            device.cmd_bind_index_buffer(
                buffer,
                sprite.index_buffer.bind(),
                0,
                vk::IndexType::UINT32,
            );
            self.spine_pipeline.push_constants(sprite.constants, buffer);
            //device.cmd_draw_indexed(buffer, (sprite.counters.len() * 6) as u32, 1, 0, 0, 1);
        }
        timer.record("spine", &METRIC_RENDER_SECONDS);

        // ROOF
        for roof in &self.roof_sprites {
            self.roof_buffer
                .update(self.present_index as usize, roof.uniform);
            let roof_descriptor =
                self.roof_pipeline
                    .data
                    .as_mut()
                    .unwrap()
                    .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                        buffer: self.roof_buffer.buffers[self.present_index as usize],
                        offset: 0,
                        range: std::mem::size_of::<GroundUniform>() as u64,
                    })]])[0];

            device.cmd_bind_pipeline(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.roof_pipeline.handle,
            );
            device.cmd_bind_vertex_buffers(buffer, 0, &[self.roof_vertex_buffer.bind()], &[0]);
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.roof_pipeline.layout,
                0,
                &[camera_descriptor],
                &[],
            );
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.roof_pipeline.layout,
                1,
                &[self
                    .roof_pipeline
                    .material
                    .describe(vec![[ShaderData::Texture(vk::DescriptorImageInfo {
                        sampler: roof.sampler.handle,
                        image_view: roof.texture.view(),
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    })]])[0]],
                &[],
            );
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.roof_pipeline.layout,
                2,
                &[roof_descriptor],
                &[],
            );
            self.roof_pipeline.push_constants(roof.constants, buffer);
            device.cmd_draw(buffer, GROUND_VERTICES.len() as u32, 1, 0, 0);
        }
        timer.record("roof", &METRIC_RENDER_SECONDS);

        // thread::sleep_ms(10);
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
    constants: GroundPushConstants,
}

pub struct SpineSprite {
    buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture: TextureAsset,
    position: [f32; 2],
    colors: [[f32; 4]; 4],
    pub counters: Vec<(u32, u32)>,
    constants: SpinePushConstants,
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
    pub pivot: [f32; 2],
    pub highlight: f32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GroundPushConstants {
    pub offset: [f32; 2],
    pub map_size: [f32; 2],
    pub cell_size: [f32; 2],
    pub layer: f32,
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

/*
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
];*/

const SPRITE_VERTICES: [SpriteVertex; 6] = [
    SpriteVertex {
        position: [0.0, 0.0],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    SpriteVertex {
        position: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    SpriteVertex {
        position: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
];

const GROUND_VERTICES: [SpriteVertex; 6] = [
    SpriteVertex {
        position: [0.0, 0.0],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [0.0, 1.0],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [1.0, 0.0],
        uv: [1.0, 1.0],
    },
    SpriteVertex {
        position: [1.0, 0.0],
        uv: [1.0, 1.0],
    },
    SpriteVertex {
        position: [0.0, 1.0],
        uv: [0.0, 0.0],
    },
    SpriteVertex {
        position: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
];

const VISIBLE_MAP_Y: usize = 18;
const VISIBLE_MAP_X: usize = 31;

#[derive(Clone, Copy)]
pub struct GroundUniform {
    pub map: [[[f32; 4]; VISIBLE_MAP_X]; VISIBLE_MAP_Y],
}
