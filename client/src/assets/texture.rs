use std::sync::Arc;
use std::{fs, ptr};

use ash::prelude::VkResult;
use ash::vk::Handle;
use ash::{vk, Device};
use lazy_static::lazy_static;
use log::{debug, error, info};

use crate::assets::Asset;
use crate::engine::base::{create_buffer, index_memory_type, MyQueue};
use crate::engine::commands::Single;
use crate::monitoring::Timer;

lazy_static! {
    static ref METRIC_LOADING_SECONDS: prometheus::HistogramVec =
        prometheus::register_histogram_vec!(
            "texture_loading_seconds",
            "texture_loading_seconds",
            &["path", "stage"]
        )
        .unwrap();
    static ref VULKAN_IMAGES_TOTAL: prometheus::IntGauge =
        prometheus::register_int_gauge!("vulkan_images_total", "vulkan_images_total").unwrap();
}

#[repr(i64)]
enum _LoadingStage {
    FileRead = 1,
    Decode = 2,
    Buffering = 3,
    Transition = 4,
    Complete = 5,
}

pub type TextureAsset = Asset<TextureAssetData>;

#[derive(Clone)]
pub struct TextureAssetData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    image: vk::Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    device: Device,
}

impl TextureAssetData {
    fn create(
        name: String,
        device: &Device,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        let image = Self::create_image(
            device,
            width,
            height,
            format,
            tiling,
            usage,
            memory_flags,
            memory_properties,
        )
        .unwrap();
        let view = Self::create_image_view(device, image, format);
        let sampler = Self::create_texture_sampler(device);
        VULKAN_IMAGES_TOTAL.inc();
        let n = VULKAN_IMAGES_TOTAL.get();
        info!("Create image {} {}x{} N{n}", name, width, height);
        Self {
            name,
            width,
            height,
            image,
            view,
            sampler,
            device: device.clone(),
        }
    }
}

impl Drop for TextureAssetData {
    fn drop(&mut self) {
        if self.name == "./assets/fallback/texture.png" {
            // do not drop builtin texture
            // because TextureAsset shares one Arc<RefCell
            // TODO: add good fallback mechanism
            return;
        }
        VULKAN_IMAGES_TOTAL.dec();
        debug!(
            "Destroys image {} {}x{}",
            self.name, self.width, self.height
        );
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
    }
}

impl TextureAsset {
    #[inline]
    pub fn id(&self) -> u64 {
        self.view.as_raw()
    }
}

impl TextureAssetData {
    pub fn read_image_file(
        device: &Device,
        command_pool: vk::CommandPool,
        queue: Arc<MyQueue>,
        path: &str,
    ) -> Self {
        let data = fs::read(&path).unwrap();
        // timer.record2(path, "io", &METRIC_LOADING_SECONDS);

        // let png_decoder = png::Decoder::new(File::open(path).unwrap());
        // let mut png_reader = png_decoder.read_info().unwrap();
        // let mut buf = vec![0; png_reader.output_buffer_size()];
        // let info = png_reader.next_frame(&mut buf).unwrap();
        // let image_data_len = info.buffer_size();
        // let image_width = info.width;
        // let image_height = info.height;
        // let c = info.color_type;
        // let image_data = &buf[..info.buffer_size()];
        // info!("{path} L{image_data_len} {image_width}x{image_height} {c:?} PNG");

        let image_object = image::load_from_memory(&data).unwrap();
        let image_object = image_object.flipv();
        let c = image_object.color();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_data = image_object.to_rgba8();
        let image_data_len = image_data.len();
        let image_data = image_data.as_ptr();
        debug!("{path} L{image_data_len} {image_width}x{image_height} {c:?}");

        Self::read_image_data(
            String::from(path),
            device,
            command_pool,
            queue,
            image_width,
            image_height,
            image_data,
            image_data_len,
        )
    }

    pub fn write_image_data(
        &mut self,
        queue: Arc<MyQueue>,
        pool: vk::CommandPool,
        image_data: *const u8,
        image_data_len: usize,
    ) {
        let image_width = self.width;
        let image_height = self.height;
        let device = &self.device;

        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let (staging_buffer, staging_buffer_memory, _size) = create_buffer(
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &queue.device_memory,
        );
        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data, image_data_len);
            device.unmap_memory(staging_buffer_memory);
        }
        let queue = queue.handle.lock().unwrap();
        let transition = ImageLayoutTransition::new(device, pool, *queue, self.image);
        transition.specify_layout(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        transition.copy_buffer_to_image(staging_buffer, image_width, image_height);
        transition.specify_layout(
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }
    }

    pub fn read_image_data(
        name: String,
        device: &Device,
        pool: vk::CommandPool,
        queue: Arc<MyQueue>,
        image_width: u32,
        image_height: u32,
        image_data: *const u8,
        image_data_len: usize,
    ) -> Self {
        // timer.record2(path, "decode", &METRIC_LOADING_SECONDS);

        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let (staging_buffer, staging_buffer_memory, _size) = create_buffer(
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &queue.device_memory,
        );
        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data, image_data_len);
            device.unmap_memory(staging_buffer_memory);
        }
        // timer.record2(path, "buffering", &METRIC_LOADING_SECONDS);

        let format = vk::Format::R8G8B8A8_UNORM;
        let image = Self::create(
            name,
            device,
            image_width,
            image_height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &queue.device_memory,
        );
        let queue = queue.handle.lock().unwrap();
        let transition = ImageLayoutTransition::new(device, pool, *queue, image.image);
        transition.specify_layout(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        transition.copy_buffer_to_image(staging_buffer, image_width, image_height);
        transition.specify_layout(
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }
        // timer.record2(path, "transition", &METRIC_LOADING_SECONDS);
        image
    }

    fn create_image(
        device: &Device,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> VkResult<vk::Image> {
        let image_create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
        };
        let image = unsafe { device.create_image(&image_create_info, None)? };
        let memory = unsafe { device.get_image_memory_requirements(image) };
        let memory_type_index =
            index_memory_type(&memory, memory_properties, memory_flags).unwrap();

        let memory_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&memory_allocate_info, None)? };
        unsafe {
            device.bind_image_memory(image, memory, 0)?;
        }
        Ok(image)
    }

    fn create_image_view(device: &Device, image: vk::Image, format: vk::Format) -> vk::ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };

        unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        }
    }

    fn create_texture_sampler(device: &Device) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        unsafe {
            device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }
}

struct ImageLayoutTransition<'a> {
    device: &'a Device,
    pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
}

impl<'a> ImageLayoutTransition<'a> {
    fn new(device: &'a Device, pool: vk::CommandPool, queue: vk::Queue, image: vk::Image) -> Self {
        Self {
            device,
            pool,
            queue,
            image,
        }
    }

    fn specify_layout(&self, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
        let command_buffer = Single::begin(self.device, self.pool);

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer.buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        command_buffer.submit(self.device, self.queue);
    }

    fn copy_buffer_to_image(&self, buffer: vk::Buffer, width: u32, height: u32) {
        let command_buffer = Single::begin(self.device, self.pool);

        let buffer_image_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            buffer_offset: 0,
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];

        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer.buffer,
                buffer,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &buffer_image_regions,
            );
        }

        command_buffer.submit(self.device, self.queue);
    }
}
