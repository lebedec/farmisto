use crate::engine::base::{submit_commands, Base};
use crate::engine::scene::SceneRenderer;
use crate::engine::sprites::SpriteRenderer;
use crate::engine::{Assets, Input};
use ash::vk;
use std::io::{BufReader, Write};
use std::net::TcpListener;

use libfmod::ffi::{FMOD_INIT_NORMAL, FMOD_STUDIO_INIT_NORMAL, FMOD_VERSION};
use libfmod::{SpeakerMode, Studio};
use log::info;

use serde::de::Unexpected::Str;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use prometheus::{IntCounter, IntGauge};

use lazy_static::lazy_static;
use prometheus::{register_int_counter, register_int_gauge};
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Keycode::K;
use sdl2::video::FullscreenType;

lazy_static! {
    static ref METRIC_FRAME: IntCounter = register_int_counter!("app_frame", "frame").unwrap();
    static ref METRIC_COUNTER: IntCounter =
        register_int_counter!("app_counter", "counter").unwrap();
}

pub struct Frame<'c> {
    pub input: Input,
    pub scene: &'c mut SceneRenderer,
    pub sprites: &'c mut SpriteRenderer,
    pub assets: &'c mut Assets,
    pub studio: &'c Studio,
}

pub trait App {
    fn start(assets: &mut Assets) -> Self;
    fn update(&mut self, frame: Frame);
}

pub fn startup<A: App>(title: String) {
    #[cfg(windows)]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(1);
    }
    let system = sdl2::init().unwrap();
    let video = system.video().unwrap();
    #[cfg(windows)]
    let mut windowed = true;
    let mut window = video
        .window(&title, 1920, 1080)
        .allow_highdpi()
        //.fullscreen()
        //.position(1920, 0)
        .borderless()
        .vulkan()
        .build()
        .unwrap();
    info!(
        "SDL display: {:?}, dpi {:?}",
        video.display_bounds(0).unwrap(),
        video.display_dpi(0).unwrap(),
    );
    info!("SDL Vulkan drawable: {:?}", window.vulkan_drawable_size());
    // let mut window = Arc::new(window);
    let mut event_pump = system.event_pump().unwrap();
    let instance_extensions: Vec<&'static str> = window.vulkan_instance_extensions().unwrap();
    info!("SDL Vulkan extensions: {:?}", instance_extensions);

    let mut base = Base::new(&window, instance_extensions);

    info!("FMOD expected version: {:#08x}", FMOD_VERSION);
    let studio = Studio::create().unwrap();
    let system = studio.get_core_system().unwrap();
    system
        .set_software_format(None, Some(SpeakerMode::Quad), None)
        .unwrap();
    studio
        .initialize(1024, FMOD_STUDIO_INIT_NORMAL, FMOD_INIT_NORMAL, None)
        .unwrap();

    unsafe {
        let mut renderpass = Base::create_render_pass(&base.device, &base.screen);
        base.recreate_frame_buffers(renderpass);

        let mut assets = Assets::new(base.device.clone(), base.pool, base.queue.clone());

        let mut scene_renderer = SceneRenderer::create(
            &base.device,
            &base.queue.device_memory,
            base.screen.clone(),
            base.present_image_views.len(),
            renderpass,
            &mut assets,
        );
        let mut sprites_renderer = SpriteRenderer::create(
            &base.device,
            &base.queue.device_memory,
            base.screen.clone(),
            base.present_image_views.len(),
            renderpass,
            &mut assets,
            2160.0 / base.screen.height() as f32, // reference resolution 4K
        );

        let mut app = A::start(&mut assets);

        let mut time = Instant::now();
        let mut input = Input::new(base.screen.size());

        thread::spawn(|| {
            let encoder = prometheus::TextEncoder::new();
            let listener = TcpListener::bind("127.0.0.1:9091").unwrap();
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let status_line = "HTTP/1.1 200 OK";
                let mut contents = String::new();
                let metric_families = prometheus::gather();
                encoder
                    .encode_utf8(&metric_families, &mut contents)
                    .unwrap();
                let length = contents.len();
                let response =
                    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
                stream.write_all(response.as_bytes()).unwrap();
            }
        });

        loop {
            METRIC_FRAME.inc();
            METRIC_COUNTER.inc();
            assets.update();
            studio.update().unwrap();

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

            if input.pressed(Keycode::Return) {
                if windowed {
                    window.set_size(1920 * 2, 1080 * 2).unwrap();
                    window.set_fullscreen(FullscreenType::Desktop).unwrap();
                    windowed = false;
                } else {
                    window.set_size(1920, 1080).unwrap();
                    window.set_fullscreen(FullscreenType::Off).unwrap();
                    windowed = true;
                }
            }

            if input.pressed(Keycode::Z) {
                sprites_renderer.zoom += 0.1;
                info!("ZOOM: {}", sprites_renderer.zoom);
            }
            if input.pressed(Keycode::X) {
                sprites_renderer.zoom -= 0.1;
                info!("ZOOM: {}", sprites_renderer.zoom);
            }

            //scene_renderer.update();
            sprites_renderer.update();

            let present_index = match base.swapchain_loader.acquire_next_image(
                base.swapchain,
                std::u64::MAX,
                base.present_complete_semaphore,
                vk::Fence::null(),
            ) {
                Ok((present_index, _)) => present_index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    base.recreate_swapchain(renderpass);

                    scene_renderer = SceneRenderer::create(
                        &base.device,
                        &base.queue.device_memory,
                        base.screen.clone(),
                        base.present_image_views.len(),
                        renderpass,
                        &mut assets,
                    );
                    sprites_renderer = SpriteRenderer::create(
                        &base.device,
                        &base.queue.device_memory,
                        base.screen.clone(),
                        base.present_image_views.len(),
                        renderpass,
                        &mut assets,
                        2160.0 / base.screen.height() as f32, // reference resolution 4K
                    );

                    continue;
                }
                Err(error) => {
                    panic!("Presenation error {:?}", error);
                }
            };

            sprites_renderer.present_index = present_index;

            app.update(Frame {
                input: input.clone(),
                scene: &mut scene_renderer,
                sprites: &mut sprites_renderer,
                assets: &mut assets,
                studio: &studio,
            });

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
            // info!("Begins render");
            let render_begin = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(base.framebuffers[present_index as usize])
                .render_area(base.screen.resolution().into())
                .clear_values(&clear_values);

            let present = {
                let present_queue = base.queue.handle.lock().unwrap();
                // info!("Begins submit commands");
                submit_commands(
                    &base.device,
                    base.draw_command_buffer,
                    base.draw_commands_reuse_fence,
                    *present_queue,
                    &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                    &[base.present_complete_semaphore],
                    &[base.rendering_complete_semaphore],
                    |device, buffer| {
                        // info!("Begins render pass");
                        device.cmd_begin_render_pass(
                            buffer,
                            &render_begin,
                            vk::SubpassContents::INLINE,
                        );

                        //scene_renderer.render(device, buffer);
                        sprites_renderer.render(device, buffer);

                        // info!("End render pass");
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
                // info!("Presents");
                base.swapchain_loader
                    .queue_present(*present_queue, &present_info)
            };

            match present {
                Ok(suboptimal) => {}
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // recreate KHR
                }
                Err(error) => panic!("present error {:?}", error),
            }
        }
    }
}
