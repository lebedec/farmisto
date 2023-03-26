extern crate ash;

use std::borrow::Cow;
use std::default::Default;
use std::ffi::CStr;
use std::ops::Drop;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};
use ash::prelude::VkResult;
use ash::vk::Handle;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use ash::vk::KhrPortabilitySubsetFn;
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use log::{info, log, Level};
use sdl2::video::Window;

pub use pipeline::*;
pub use screen::*;
pub use shader::*;

mod pipeline;
mod screen;
mod shader;

pub struct Queue {
    pub device: Device,
    pub device_memory: vk::PhysicalDeviceMemoryProperties,
    pub handle: Mutex<vk::Queue>,
    pub family: u32,
}

pub struct Base {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,

    pub swapchain_loader: Swapchain,
    pub debug_utils_loader: DebugUtils,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub queue: Arc<Queue>,
    pub screen: Screen,
    pub swapchain: vk::SwapchainKHR,
    pub present_image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    // sync
    pub draw_commands_reuse_fence: vk::Fence,
    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,
}

impl Base {
    pub unsafe fn begin_commands(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> VkResult<vk::CommandBuffer> {
        let fences = &[self.draw_commands_reuse_fence];
        self.device.wait_for_fences(fences, true, u64::MAX)?;
        self.device.reset_fences(fences)?;
        self.device.reset_command_buffer(
            command_buffer,
            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )?;
        let begin = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device.begin_command_buffer(command_buffer, &begin)?;
        Ok(command_buffer)
    }

    pub unsafe fn end_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        submit_queue: vk::Queue,
    ) -> VkResult<()> {
        self.device.end_command_buffer(command_buffer)?;
        let wait_semaphores = [self.present_complete_semaphore];
        let signal_semaphores = [self.rendering_complete_semaphore];
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();
        self.device
            .queue_submit(submit_queue, &[submit_info], self.draw_commands_reuse_fence)
    }
}

impl Base {
    pub fn is_development() -> bool {
        std::env::var("DEV_MODE").is_ok()
    }

    pub fn new(window: &Window, sdl_extension_names: Vec<&str>) -> Self {
        unsafe {
            let mut extension_names = vec![DebugUtils::name().as_ptr()];
            extension_names.extend(
                sdl_extension_names
                    .into_iter()
                    .map(|ext| ext.as_ptr() as *const c_char),
            );

            let entry = Entry::load().unwrap();
            let app_name = CStr::from_bytes_with_nul_unchecked(b"Farmisto\0");

            let appinfo = vk::ApplicationInfo::builder()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            let layers_names_raw = if Self::is_development() {
                let layer_names = [CStr::from_bytes_with_nul_unchecked(
                    b"VK_LAYER_KHRONOS_validation\0",
                )];
                info!("Enables Vulkan layers: {:?}", layer_names);
                let layers_names_raw: Vec<*const c_char> = layer_names
                    .iter()
                    .map(|raw_name| raw_name.as_ptr())
                    .collect();
                layers_names_raw
            } else {
                info!("Enables no Vulkan layers");
                vec![]
            };

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names)
                .flags(create_flags);

            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let surface_handle = window
                .vulkan_create_surface(instance.handle().as_raw() as usize)
                .unwrap();

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_utils_loader = DebugUtils::new(&entry, &instance);
            let debug_call_back = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();

            let surface = vk::SurfaceKHR::from_raw(surface_handle);
            let surface_loader = Surface::new(&entry, &instance);

            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let (physical_device, queue_family_index) = pdevices
                .iter()
                .find_map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(
                                            *pdevice,
                                            index as u32,
                                            surface,
                                        )
                                        .unwrap();
                            if supports_graphic_and_surface {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                })
                .expect("Couldn't find suitable device.");

            let screen = Screen::new(surface, surface_loader, physical_device);

            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [
                Swapchain::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                KhrPortabilitySubsetFn::name().as_ptr(),
            ];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                sampler_anisotropy: 1,
                fill_mode_non_solid: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device: Device = instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index as u32, 0);

            let swapchain_loader = Swapchain::new(&instance, &device);
            let swapchain = Self::create_swapchain(&instance, &screen, &device);

            let present_image_views =
                Self::create_present_images(swapchain, &instance, &screen, &device);

            // command buffer
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);
            let pool = device.create_command_pool(&pool_create_info, None).unwrap();
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(present_image_views.len() as u32)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();

            // synchronization
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            let draw_commands_reuse_fence = device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            // depth
            let depth_image = Self::create_depth_image(&instance, &screen, &device);
            let (depth_image_view, depth_image_memory) =
                Self::create_depth_image_view(&instance, &screen, &device, depth_image);

            submit_commands(
                &device,
                command_buffers[0],
                draw_commands_reuse_fence,
                present_queue,
                &[],
                &[],
                &[],
                |device, draw_command_buffer| {
                    let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                        .image(depth_image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1)
                                .build(),
                        )
                        .build();

                    device.cmd_pipeline_barrier(
                        draw_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers],
                    );
                },
            );

            let device_memory_properties =
                instance.get_physical_device_memory_properties(physical_device);
            let queue = Arc::new(Queue {
                device: device.clone(),
                device_memory: device_memory_properties,
                handle: Mutex::new(present_queue),
                family: queue_family_index,
            });

            Base {
                entry,
                instance,
                device,
                physical_device,
                screen,
                swapchain_loader,
                swapchain,
                present_image_views,
                framebuffers: vec![],
                pool,
                depth_image,
                depth_image_view,
                depth_image_memory,
                present_complete_semaphore,
                rendering_complete_semaphore,
                draw_commands_reuse_fence,
                debug_call_back,
                debug_utils_loader,
                queue,
                command_buffers,
            }
        }
    }

    pub unsafe fn create_depth_image(
        instance: &Instance,
        screen: &Screen,
        device: &Device,
    ) -> vk::Image {
        let device_memory_properties =
            instance.get_physical_device_memory_properties(screen.physical_device());

        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(screen.resolution().into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        device.create_image(&depth_image_create_info, None).unwrap()
    }

    pub unsafe fn create_depth_image_view(
        instance: &Instance,
        screen: &Screen,
        device: &Device,
        depth_image: vk::Image,
    ) -> (vk::ImageView, vk::DeviceMemory) {
        let device_memory_properties =
            instance.get_physical_device_memory_properties(screen.physical_device());

        let depth_image_memory_req = device.get_image_memory_requirements(depth_image);
        let depth_image_memory_index = index_memory_type(
            &depth_image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory index for depth image.");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(depth_image_memory_req.size)
            .memory_type_index(depth_image_memory_index);

        let depth_image_memory = device
            .allocate_memory(&depth_image_allocate_info, None)
            .unwrap();

        device
            .bind_image_memory(depth_image, depth_image_memory, 0)
            .expect("Unable to bind depth image memory");

        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(vk::Format::D16_UNORM)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = device
            .create_image_view(&depth_image_view_info, None)
            .unwrap();

        (depth_image_view, depth_image_memory)
    }

    pub fn recreate_frame_buffers(&mut self, renderpass: vk::RenderPass) {
        info!("Recreates frame buffers {:?}", self.screen.resolution());
        unsafe {
            self.framebuffers = self
                .present_image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view, self.depth_image_view];
                    let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(self.screen.width())
                        .height(self.screen.height())
                        .layers(1);
                    self.device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                })
                .collect();
        }
    }

    fn create_present_images(
        swapchain: vk::SwapchainKHR,
        instance: &Instance,
        screen: &Screen,
        device: &Device,
    ) -> Vec<vk::ImageView> {
        let surface_format = screen.format();
        let swapchain_loader = Swapchain::new(&instance, &device);
        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        present_images
            .iter()
            .map(|&image| {
                let components = vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                };
                let subresource_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(components)
                    .subresource_range(subresource_range)
                    .image(image);
                unsafe { device.create_image_view(&create_view_info, None).unwrap() }
            })
            .collect()
    }

    fn create_swapchain(instance: &Instance, screen: &Screen, device: &Device) -> vk::SwapchainKHR {
        let surface_format = screen.format();
        let surface_caps = screen.get_capabilities();
        let mut min_image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && min_image_count > surface_caps.max_image_count {
            min_image_count = surface_caps.max_image_count;
        }
        let pre_transform = if surface_caps
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_caps.current_transform
        };
        let present_modes = screen.present_modes();
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::FIFO)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(screen.surface())
            .min_image_count(min_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(screen.resolution())
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);
        unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        }
    }

    pub fn create_render_pass(device: &Device, screen: &Screen) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: screen.format().format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: Default::default(),
            dependency_flags: Default::default(),
        }];
        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        }
    }

    pub unsafe fn recreate_swapchain(&mut self, renderpass: vk::RenderPass) {
        info!("Recreates swapchain");
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            // self.device.destroy_image(self.depth_image, None);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
        self.screen.resize();
        self.swapchain = Self::create_swapchain(&self.instance, &self.screen, &self.device);
        self.present_image_views =
            Self::create_present_images(self.swapchain, &self.instance, &self.screen, &self.device);

        // DEPTH IMAGE RECREATE
        // let depth_image =
        //     unsafe { Self::create_depth_image(&self.instance, &self.screen, &self.device) };
        // let (depth_image_view, depth_image_memory) = unsafe {
        //     Self::create_depth_image_view(&self.instance, &self.screen, &self.device, depth_image)
        // };
        // self.depth_image = depth_image;
        // self.depth_image_view = depth_image_view;
        // self.depth_image_memory = depth_image_memory;

        info!("Recreate depth image");
        let depth_image = Self::create_depth_image(&self.instance, &self.screen, &self.device);
        let (depth_image_view, depth_image_memory) =
            Self::create_depth_image_view(&self.instance, &self.screen, &self.device, depth_image);
        self.depth_image = depth_image;
        self.depth_image_view = depth_image_view;
        self.depth_image_memory = depth_image_memory;

        self.recreate_frame_buffers(renderpass);
        info!("Done");
    }

    /*
    fn recreate_swap_chain(&mut self) {
        let (swap_chain, images) = Self::create_swap_chain(
            &self.instance,
            &self.surface,
            self.physical_device_index,
            &self.device,
            &self.graphics_queue,
            &self.present_queue,
            Some(self.swap_chain.clone()),
        );
        self.swap_chain = swap_chain;
        self.swap_chain_images = images;
        self.render_pass = Self::create_render_pass(&self.device, self.swap_chain.format());
        self.graphics_pipeline = Self::create_graphics_pipeline(
            &self.device,
            self.swap_chain.dimensions(),
            &self.render_pass,
        );
        self.swap_chain_framebuffers =
            Self::create_framebuffers(&self.swap_chain_images, &self.render_pass);
        self.create_command_buffers();
    }*/
}

impl Drop for Base {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device
                .destroy_semaphore(self.present_complete_semaphore, None);
            self.device
                .destroy_semaphore(self.rendering_complete_semaphore, None);
            self.device
                .destroy_fence(self.draw_commands_reuse_fence, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.screen
                .surface_loader
                .destroy_surface(self.screen.surface(), None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub fn create_buffer(
    device: &Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_flags: vk::MemoryPropertyFlags,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
    let info = vk::BufferCreateInfo {
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let buffer = unsafe { device.create_buffer(&info, None).unwrap() };
    let memory = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = index_memory_type(&memory, memory_properties, memory_flags).unwrap();

    let allocation = vk::MemoryAllocateInfo {
        allocation_size: memory.size,
        memory_type_index,
        ..Default::default()
    };

    let device_memory = unsafe { device.allocate_memory(&allocation, None).unwrap() };

    unsafe {
        device.bind_buffer_memory(buffer, device_memory, 0).unwrap();
    }

    (buffer, device_memory, memory.size)
}

#[allow(clippy::too_many_arguments)]
pub fn submit_commands<F: FnOnce(&Device, vk::CommandBuffer)>(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        device
            .wait_for_fences(&[fence], true, u64::MAX)
            .expect("Wait for fence failed.");

        device.reset_fences(&[fence]).expect("Reset fences failed.");

        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin command buffer");

        f(device, command_buffer);

        device
            .end_command_buffer(command_buffer)
            .expect("End command buffer");

        let command_buffers = vec![command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        device
            .queue_submit(submit_queue, &[submit_info], fence)
            .expect("queue submit failed.");
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    // let message_id_number: i32 = callback_data.message_id_number as i32;
    //
    // let message_id_name = if callback_data.p_message_id_name.is_null() {
    //     Cow::from("")
    // } else {
    //     CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    // };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let level = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => Level::Debug,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => Level::Error,
        _ => Level::Error,
    };

    log!(level, "{}", message);

    vk::FALSE
}

pub fn index_memory_type(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
