use ash::{vk, Device};
use glam::{vec3, Mat4, Vec3};
use lazy_static::lazy_static;

use crate::assets::SamplerAsset;
use crate::engine::base::MyPipeline;
use crate::engine::base::Screen;
use crate::engine::base::ShaderData;
use crate::engine::buffers::{CameraUniform, LightUniform, UniformBuffer};
use crate::engine::rendering::PlantPushConstants;
use crate::engine::rendering::PlantRenderObject;
use crate::engine::rendering::SpritePushConstants;
use crate::engine::rendering::SpriteRenderObject;
use crate::engine::rendering::TilemapPushConstants;
use crate::engine::rendering::TilemapRenderObject;
use crate::engine::rendering::GROUND_VERTICES;
use crate::engine::rendering::SPRITE_VERTICES;
use crate::engine::rendering::{AnimalPushConstants, AnimalRenderObject, GroundRenderObject};
use crate::engine::rendering::{GroundPushConstants, GroundUniform, Light};
use crate::engine::VertexBuffer;
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

pub struct Scene {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    pub present_index: usize,
    pub screen: Screen,
    pub zoom: f32,

    pub camera_position: [f32; 2],
    pub camera_buffer: UniformBuffer<CameraUniform>,
    pub global_light_buffer: UniformBuffer<LightUniform>,
    pub global_lights: Vec<Light>,

    pub spine_pipeline: MyPipeline<2, PlantPushConstants, 1>,
    pub spines: Vec<Vec<PlantRenderObject>>,
    pub spine_coloration_sampler: SamplerAsset,

    pub animals_pipeline: MyPipeline<2, AnimalPushConstants, 1>,
    pub animals: Vec<Vec<AnimalRenderObject>>,

    pub ground_pipeline: MyPipeline<1, GroundPushConstants, 2>,
    pub grounds: Vec<GroundRenderObject>,
    pub ground_vertex_buffer: VertexBuffer,
    pub ground_buffer: UniformBuffer<GroundUniform>,

    pub sprite_pipeline: MyPipeline<1, SpritePushConstants, 1>,
    pub sprites: Vec<Vec<SpriteRenderObject>>,
    pub sprite_vertex_buffer: VertexBuffer,

    pub tilemap_pipeline: MyPipeline<1, TilemapPushConstants, 1>,
    pub tilemaps: Vec<Vec<TilemapRenderObject>>,
    pub tilemap_vertex_buffer: VertexBuffer,

    // light_map_pipeline: MyPipeline<1, SpritePushConstants, 1>,
    // light_map_framebuffer: vk::Framebuffer,
    // light_map_render_pass: vk::RenderPass,
    // light_map_sampler: SamplerAsset,
    // light_map_view: vk::ImageView,
    // lights: Vec<SpriteRenderObject>,
    // lights_texture: SpriteAsset,
    pub swapchain: usize,
}

impl Scene {
    pub fn create<'a>(
        device: &Device,
        device_memory: &vk::PhysicalDeviceMemoryProperties,
        screen: Screen,
        swapchain: usize,
        pass: vk::RenderPass,
        assets: &mut Assets,
        zoom: f32,
    ) -> Self {
        let spine_coloration_sampler = assets.sampler("coloration");
        let camera_buffer = UniformBuffer::create(device.clone(), device_memory, swapchain);
        let global_light_buffer = UniformBuffer::create(device.clone(), device_memory, swapchain);

        let sprite_vertex_buffer =
            VertexBuffer::create(device, device_memory, SPRITE_VERTICES.to_vec());

        let spine_pipeline = MyPipeline::build(assets.pipeline("spine:plants"), pass)
            .material([
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);

        let animals_pipeline = MyPipeline::build(assets.pipeline("spine:animals"), pass)
            .material([
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);

        let ground_buffer = UniformBuffer::create(device.clone(), device_memory, swapchain);
        let ground_pipeline = MyPipeline::build(assets.pipeline("ground"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::DescriptorType::UNIFORM_BUFFER,
            ])
            .build(device, &screen);
        let ground_vertex_buffer =
            VertexBuffer::create(device, device_memory, GROUND_VERTICES.to_vec());

        let sprite_pipeline = MyPipeline::build(assets.pipeline("sprites"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);

        let tilemap_pipeline = MyPipeline::build(assets.pipeline("tilemap"), pass)
            .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
            .data([vk::DescriptorType::UNIFORM_BUFFER])
            .build(device, &screen);

        // let (light_map_view, light_map_render_pass, light_map_framebuffer) =
        //     Self::create_light_map(device, device_memory).unwrap();
        //
        // let light_map_pipeline =
        //     MyPipeline::build(assets.pipeline("light-map"), light_map_render_pass)
        //         .material([vk::DescriptorType::COMBINED_IMAGE_SAMPLER])
        //         .build(device, &screen);

        let mut sprites = vec![];
        let mut spines = vec![];
        let mut tilemaps = vec![];
        let mut animals = vec![];
        for _ in 0..128 {
            sprites.push(vec![]);
            spines.push(vec![]);
            tilemaps.push(vec![]);
            animals.push(vec![]);
        }

        Self {
            device: device.clone(),
            device_memory: device_memory.clone(),
            sprites,
            spines,
            spine_pipeline,
            ground_pipeline,
            camera_buffer,
            ground_buffer,
            sprite_vertex_buffer,
            tilemap_pipeline,
            tilemaps,
            tilemap_vertex_buffer: ground_vertex_buffer,
            present_index: 0,
            //light_map_pipeline,
            //light_map_framebuffer,
            //light_map_render_pass,
            spine_coloration_sampler,
            animals_pipeline,
            grounds: vec![],
            screen,
            zoom,
            sprite_pipeline,
            ground_vertex_buffer,
            camera_position: [0.0, 0.0],
            //light_map_sampler: assets.sampler("light-map"),
            //light_map_view,
            // lights: vec![],
            // lights_texture: assets.sprite("light-test"),
            swapchain,
            global_light_buffer,
            global_lights: vec![],
            animals,
        }
    }

    pub fn look_at(&mut self, target: Vec3) {
        let width = self.screen.width() as f32;
        let height = self.screen.height() as f32;
        let uniform = CameraUniform {
            model: Mat4::from_translation(vec3(-target.x, -target.y, 0.0)),
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
        self.camera_position = [target.x, target.y];
        self.camera_buffer.update(self.present_index, uniform);
    }

    pub fn clear(&mut self) {
        for sprites in self.sprites.iter_mut() {
            sprites.clear();
        }
        for spines in self.spines.iter_mut() {
            spines.clear();
        }
        for animals in self.animals.iter_mut() {
            animals.clear();
        }
        for tilemaps in self.tilemaps.iter_mut() {
            tilemaps.clear();
        }
        // self.sprites.clear();
        // self.spines.clear();
        self.grounds.clear();
    }

    pub fn update(&mut self) {
        self.spine_pipeline.update(&self.device, &self.screen);
        self.animals_pipeline.update(&self.device, &self.screen);
        self.ground_pipeline.update(&self.device, &self.screen);
        self.sprite_pipeline.update(&self.device, &self.screen);
        self.tilemap_pipeline.update(&self.device, &self.screen);
    }

    pub fn set_point_light(&mut self, color: [f32; 4], radius: f32, position: [f32; 2]) {
        let [x, y] = position;
        self.global_lights.push(Light {
            color,
            position: [x, y, radius, 0.0],
        })
    }

    pub unsafe fn render2(
        &mut self,
        device: &Device,
        buffer: vk::CommandBuffer,
        render_begin: &vk::RenderPassBeginInfo,
    ) {
        device.cmd_begin_render_pass(buffer, &render_begin, vk::SubpassContents::INLINE);
        self.draw(device, buffer);
        device.cmd_end_render_pass(buffer);
    }

    pub unsafe fn draw(&mut self, device: &Device, buffer: vk::CommandBuffer) {
        let mut timer = Timer::now();
        let mut uniform = LightUniform {
            color: [[1.0; 4]; 512],
            position: [[0.0; 4]; 512],
        };
        let lights = self.global_lights.split_off(0);
        for (index, light) in lights.into_iter().enumerate() {
            uniform.color[index] = light.color;
            uniform.position[index] = light.position;
        }
        self.global_light_buffer.update(self.present_index, uniform);

        device.cmd_set_viewport(buffer, 0, &self.screen.viewports);
        device.cmd_set_scissor(buffer, 0, &self.screen.scissors);
        // TODO: SHARED descriptor
        let camera_descriptor = self
            .spine_pipeline
            .camera
            .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                buffer: self.camera_buffer.buffers[self.present_index],
                offset: 0,
                range: std::mem::size_of::<CameraUniform>() as u64,
            })]])[0];
        let lights_descriptor =
            self.sprite_pipeline
                .data
                .as_mut()
                .unwrap()
                .describe(vec![[ShaderData::Uniform(vk::DescriptorBufferInfo {
                    buffer: self.global_light_buffer.buffers[self.present_index],
                    offset: 0,
                    range: std::mem::size_of::<LightUniform>() as u64,
                })]])[0];

        let mut pipeline = self.ground_pipeline.perform(device, buffer);
        pipeline.bind_camera(camera_descriptor);
        for ground in &self.grounds {
            pipeline.bind_vertex_buffer(&self.ground_vertex_buffer);
            pipeline.bind_material([(ground.sampler.handle, ground.texture.view)]);
            pipeline.bind_data_by_descriptor(ground.data_descriptor);
            pipeline.push_constants(ground.constants);
            pipeline.draw_vertices(self.ground_vertex_buffer.vertices);
        }
        timer.record("ground", &METRIC_RENDER_SECONDS);

        for line in 0..128 {
            let mut pipeline = self.tilemap_pipeline.perform(device, buffer);
            pipeline.bind_camera(camera_descriptor);
            for tilemap in &self.tilemaps[line] {
                pipeline.bind_vertex_buffer(&self.tilemap_vertex_buffer);
                pipeline.bind_material([(tilemap.sampler, tilemap.texture)]);
                pipeline.bind_data([tilemap.data]);
                pipeline.push_constants(tilemap.constants);
                pipeline.draw_vertices(self.tilemap_vertex_buffer.vertices);
            }

            let mut pipeline = self.sprite_pipeline.perform(device, buffer);
            pipeline.bind_data_by_descriptor(lights_descriptor);
            pipeline.bind_camera(camera_descriptor);
            pipeline.bind_vertex_buffer(&self.sprite_vertex_buffer);
            let mut previous_texture = Default::default();
            for sprite in &self.sprites[line] {
                if previous_texture != sprite.texture_descriptor {
                    previous_texture = sprite.texture_descriptor;
                    pipeline.bind_texture(sprite.texture_descriptor);
                }
                pipeline.push_constants(sprite.constants);
                pipeline.draw_vertices(self.sprite_vertex_buffer.vertices);
            }

            let mut pipeline = self.spine_pipeline.perform(device, buffer);
            pipeline.bind_camera(camera_descriptor);
            for spine in &self.spines[line] {
                pipeline.bind_vertex_buffer(&spine.vertex_buffer);
                pipeline.bind_index_buffer(&spine.index_buffer);
                pipeline.bind_material([
                    (spine.texture.sampler, spine.texture.view),
                    (self.spine_coloration_sampler.handle, spine.coloration.view),
                ]);
                pipeline.bind_data_by_descriptor(spine.lights_descriptor);
                pipeline.push_constants(spine.constants);
                let indices: usize = spine.meshes.iter().map(|mesh| *mesh).sum();
                pipeline.draw(indices);
            }

            let mut pipeline = self.animals_pipeline.perform(device, buffer);
            pipeline.bind_camera(camera_descriptor);
            for animal in &self.animals[line] {
                pipeline.bind_vertex_buffer(&animal.vertex_buffer);
                pipeline.bind_index_buffer(&animal.index_buffer);
                pipeline.bind_material([
                    (animal.texture.sampler, animal.texture.view),
                    (self.spine_coloration_sampler.handle, animal.coloration.view),
                ]);
                pipeline.bind_data_by_descriptor(animal.lights_descriptor);
                pipeline.push_constants(animal.constants);
                let indices: usize = animal.meshes.iter().map(|mesh| *mesh).sum();
                pipeline.draw(indices);
            }
        }
        timer.record("static+spine+tilemap", &METRIC_RENDER_SECONDS);
    }
}
