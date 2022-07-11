use crate::engine::base::{submit_commands, Base};
use crate::engine::my::MyRenderer;
use crate::engine::{Assets, Input};
use ash::vk;
use datamap::Storage;
use log::info;
use std::ffi::CString;
use std::sync::Arc;
use std::time::Instant;

pub trait App {
    fn start(assets: &mut Assets) -> Self;
    fn update(&mut self, input: Input, renderer: &mut MyRenderer, assets: &mut Assets);
}

pub fn startup<A: App>(title: String) {
    #[cfg(windows)]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(1);
    }
    let system = sdl2::init().unwrap();
    let video = system.video().unwrap();
    #[cfg(windows)]
    let window_size = [1920, 1080];
    #[cfg(unix)]
    let window_size = [960, 540];
    let window = video
        .window(&title, window_size[0], window_size[1])
        .allow_highdpi()
        .position(1920, 0)
        .vulkan()
        .build()
        .unwrap();
    info!(
        "SDL display: {:?}, dpi {:?}",
        video.display_bounds(0).unwrap(),
        video.display_dpi(0).unwrap(),
    );
    info!("SDL Vulkan drawable: {:?}", window.vulkan_drawable_size());
    let window = Arc::new(window);
    let mut event_pump = system.event_pump().unwrap();
    let instance_extensions: Vec<CString> = window
        .vulkan_instance_extensions()
        .unwrap()
        .iter()
        .map(|&name| CString::new(name.to_string()).unwrap())
        .collect();
    info!("SDL Vulkan extensions: {:?}", instance_extensions);

    let base = Base::new(
        window_size[0],
        window_size[1],
        window.clone(),
        instance_extensions,
    );

    unsafe {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: base.surface_format.format,
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
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let renderpass = base
            .device
            .create_render_pass(&renderpass_create_info, None)
            .unwrap();

        let framebuffers: Vec<vk::Framebuffer> = base
            .present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, base.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(base.surface_resolution.width)
                    .height(base.surface_resolution.height)
                    .layers(1);

                base.device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            })
            .collect();

        let viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: base.surface_resolution.width as f32,
            height: base.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = vec![base.surface_resolution.into()];

        let storage = Storage::open("./assets/assets.sqlite").unwrap();
        let mut assets = Assets::new(base.device.clone(), base.pool, base.queue.clone());

        let mut my_renderer = MyRenderer::create(
            &base.device,
            &base.queue.device_memory,
            base.queue.clone(),
            scissors,
            viewports,
            base.present_images.len(),
            renderpass,
            &mut assets,
        );

        let mut app = A::start(&mut assets);

        let mut time = Instant::now();
        let mut input = Input::new(window_size);
        loop {
            assets.update(&storage);

            input.reset();
            input.time = time.elapsed().as_secs_f32();
            time = Instant::now();
            // info!("et: {}", input.time);
            for event in event_pump.poll_iter() {
                input.handle(event);
            }

            if input.terminating {
                break;
            }

            my_renderer.update();
            let (present_index, _) = base
                .swapchain_loader
                .acquire_next_image(
                    base.swapchain,
                    std::u64::MAX,
                    base.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();

            my_renderer.present_index = present_index;
            app.update(input.clone(), &mut my_renderer, &mut assets);

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.2, 0.2, 0.2, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];
            let render_begin = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(framebuffers[present_index as usize])
                .render_area(base.surface_resolution.into())
                .clear_values(&clear_values);

            {
                let present_queue = base.queue.handle.lock().unwrap();

                submit_commands(
                    &base.device,
                    base.draw_command_buffer,
                    base.draw_commands_reuse_fence,
                    *present_queue,
                    &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                    &[base.present_complete_semaphore],
                    &[base.rendering_complete_semaphore],
                    |device, buffer| {
                        device.cmd_begin_render_pass(
                            buffer,
                            &render_begin,
                            vk::SubpassContents::INLINE,
                        );

                        my_renderer.render(device, buffer);

                        device.cmd_end_render_pass(buffer);
                    },
                );
                //let mut present_info_err = mem::zeroed();
                let wait_semaphors = [base.rendering_complete_semaphore];
                let swapchains = [base.swapchain];
                let image_indices = [present_index];
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&wait_semaphors) // &base.rendering_complete_semaphore)
                    .swapchains(&swapchains)
                    .image_indices(&image_indices);

                base.swapchain_loader
                    .queue_present(*present_queue, &present_info)
                    .unwrap();

                // let frame_time = time.elapsed();
                // if frame_time.as_millis() > 0 {
                //     warn!("Frame time: {:?}s", frame_time)
                // }
            }
        }
    }
}
