use crate::engine::base::Base;
use crate::engine::rendering::{Scene, SceneMetrics};
use crate::engine::{AppConfig, Input};
use ash::vk;
use std::ffi::CString;

use libfmod::ffi::{FMOD_INIT_NORMAL, FMOD_STUDIO_INIT_NORMAL, FMOD_VERSION};
use libfmod::{SpeakerMode, Studio};
use log::info;

use std::time::Instant;

use crate::assets::Assets;
use crate::monitoring::{spawn_prometheus_metrics_pusher, spawn_prometheus_metrics_server};
use crate::translation::Translator;
use prometheus::Registry;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;

pub struct Frame<'c> {
    pub config: &'c AppConfig,
    pub input: Input,
    pub scene: &'c mut Scene,
    pub assets: &'c mut Assets,
    pub studio: &'c Studio,
    pub translator: &'c Translator,
    pub metrics_registry: &'c Registry,
}

pub trait App {
    fn start(assets: &mut Assets) -> Self;
    fn update(&mut self, frame: &mut Frame);
}

pub fn startup<A: App>(title: String) {
    let config = AppConfig::load();
    let system = sdl2::init().unwrap();
    let video = system.video().unwrap();
    sdl2::image::init(InitFlag::PNG).unwrap();

    let _windowed = true;
    let mut window = video.window(&title, config.resolution[0], config.resolution[1]);
    let window = if config.windowed {
        window
            .borderless()
            .position(config.position[0], config.position[1])
    } else {
        window.fullscreen()
    };
    let window = window.vulkan().build().unwrap();
    info!(
        "SDL display: {:?}, dpi {:?}",
        video.display_bounds(0).unwrap(),
        video.display_dpi(0).unwrap(),
    );
    info!("SDL Vulkan drawable: {:?}", window.vulkan_drawable_size());
    let mut event_pump = system.event_pump().unwrap();
    let instance_extensions: Vec<&'static str> = window.vulkan_instance_extensions().unwrap();

    let mut base = Base::new(
        &window,
        instance_extensions
            .iter()
            .map(|value| CString::new(*value).expect("Unable to create CString"))
            .collect(),
    );
    info!(
        "Logical drawable: ({}, {})",
        base.screen.width(),
        base.screen.height()
    );

    let path = "./assets/text/general.ru.po";
    info!("Loads translations from {path}");
    let translator = Translator::new(path);

    info!("FMOD expected version: {:#08x}", FMOD_VERSION);
    let studio = Studio::create().unwrap();
    let system = studio.get_core_system().unwrap();
    system
        .set_software_format(None, Some(SpeakerMode::Quad), None)
        .unwrap();
    studio
        .initialize(1024, FMOD_STUDIO_INIT_NORMAL, FMOD_INIT_NORMAL, None)
        .unwrap();

    info!("Spawns monitoring agents");
    spawn_prometheus_metrics_server();
    let metrics_registry = Registry::new();
    if let Some(gateway) = config.metrics_gateway.clone() {
        spawn_prometheus_metrics_pusher(gateway, metrics_registry.clone());
    }

    unsafe {
        let renderpass = Base::create_render_pass(&base.device, &base.screen);
        base.recreate_frame_buffers(renderpass);

        let mut assets = Assets::new(base.device.clone(), base.pool, base.queue.clone());

        let mut scene = Scene::create(
            &base.device,
            &base.queue.device_memory,
            base.pool,
            base.queue.clone(),
            base.screen.clone(),
            base.present_image_views.len(),
            renderpass,
            &mut assets,
            2160.0 / base.screen.height() as f32, // reference resolution 4K
            SceneMetrics::new(&metrics_registry).unwrap(),
        );

        let mut app = A::start(&mut assets);

        let mut time = Instant::now();
        let mut input = Input::new(base.screen.size());

        loop {
            assets.process_assets_loading();
            studio.update().unwrap();

            input.reset();
            input.zoom = scene.zoom;
            input.window = base.screen.size().map(|value| value as f32);
            input.time = time.elapsed().as_secs_f32();
            time = Instant::now();
            // info!("et: {}", input.time);
            for event in event_pump.poll_iter() {
                input.handle(event, &video);
            }

            if input.terminating {
                break;
            }

            // if input.pressed(Keycode::Return) {
            //     let [w, h] = config.resolution.clone();
            //     if windowed {
            //         window.set_size(w * 2, h * 2).unwrap();
            //         window.set_fullscreen(FullscreenType::Desktop).unwrap();
            //         windowed = false;
            //     } else {
            //         window.set_size(w, h).unwrap();
            //         window.set_fullscreen(FullscreenType::Off).unwrap();
            //         windowed = true;
            //     }
            // }

            if input.pressed(Keycode::Z) {
                scene.zoom += 0.1;
                info!("ZOOM: {}", scene.zoom);
            }
            if input.pressed(Keycode::X) {
                scene.zoom -= 0.1;
                info!("ZOOM: {}", scene.zoom);
            }

            scene.update();

            let present_index = match base.swapchain_loader.acquire_next_image(
                base.swapchain,
                std::u64::MAX,
                base.present_complete_semaphore,
                vk::Fence::null(),
            ) {
                Ok((present_index, _)) => present_index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    base.recreate_swapchain(renderpass);
                    // TODO: do not recreate, update
                    scene = Scene::create(
                        &base.device,
                        &base.queue.device_memory,
                        base.pool,
                        base.queue.clone(),
                        base.screen.clone(),
                        base.present_image_views.len(),
                        renderpass,
                        &mut assets,
                        2160.0 / base.screen.height() as f32, // reference resolution 4K
                        SceneMetrics::new(&metrics_registry).unwrap(),
                    );
                    continue;
                }
                Err(error) => {
                    panic!("Presenation error {:?}", error);
                }
            };

            scene.present_index = present_index as usize;

            let t1 = Instant::now();
            app.update(&mut Frame {
                config: &config,
                input: input.clone(),
                scene: &mut scene,
                assets: &mut assets,
                studio: &studio,
                translator: &translator,
                metrics_registry: &metrics_registry,
            });
            let _t1 = t1.elapsed().as_secs_f32();
            // if t1 > 0.0002 {
            //     println!("t1: {}", t1);
            // }

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

            let present_queue = base.queue.handle.lock().unwrap();
            let frame_command_buffer = base.command_buffers[present_index as usize];
            let present = {
                let render_begin = vk::RenderPassBeginInfo::builder()
                    .render_pass(renderpass)
                    .framebuffer(base.framebuffers[present_index as usize])
                    .render_area(base.screen.resolution().into())
                    .clear_values(&clear_values);
                base.begin_commands(frame_command_buffer).unwrap();
                scene.draw2(&base.device, frame_command_buffer, &render_begin);
                base.end_commands(frame_command_buffer, *present_queue)
                    .unwrap();

                let wait_semaphores = [base.rendering_complete_semaphore];
                let swapchains = [base.swapchain];
                let image_indices = [present_index];
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&wait_semaphores)
                    .swapchains(&swapchains)
                    .image_indices(&image_indices);
                base.swapchain_loader
                    .queue_present(*present_queue, &present_info)
            };
            match present {
                Ok(_suboptimal) => {}
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // TODO: recreate swapchain
                }
                Err(error) => panic!("present error {:?}", error),
            }

            // info!("Begins render");
            // let render_begin = vk::RenderPassBeginInfo::builder()
            //     .render_pass(renderpass)
            //     .framebuffer(base.framebuffers[present_index as usize])
            //     .render_area(base.screen.resolution().into())
            //     .clear_values(&clear_values);
            // let present = {
            //     let present_queue = base.queue.handle.lock().unwrap();
            //     // info!("Begins submit commands");
            //     submit_commands(
            //         &base.device,
            //         base.command_buffers[present_index as usize],
            //         base.draw_commands_reuse_fence,
            //         *present_queue,
            //         &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            //         &[base.present_complete_semaphore],
            //         &[base.rendering_complete_semaphore],
            //         |device, buffer| {
            //             // info!("Begins render pass");
            //             device.cmd_begin_render_pass(
            //                 buffer,
            //                 &render_begin,
            //                 vk::SubpassContents::INLINE,
            //             );
            //
            //             //scene_renderer.render(device, buffer);
            //             sprites_renderer.render(device, buffer);
            //
            //             // info!("End render pass");
            //             device.cmd_end_render_pass(buffer);
            //         },
            //     );
            //
            //     //let mut present_info_err = mem::zeroed();
            //     let wait_semaphors = [base.rendering_complete_semaphore];
            //     let swapchains = [base.swapchain];
            //     let image_indices = [present_index];
            //     let present_info = vk::PresentInfoKHR::builder()
            //         .wait_semaphores(&wait_semaphors) // &base.rendering_complete_semaphore)
            //         .swapchains(&swapchains)
            //         .image_indices(&image_indices);
            //     // info!("Presents");
            //     base.swapchain_loader
            //         .queue_present(*present_queue, &present_info)
            // };
        }
    }
}
