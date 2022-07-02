use crate::engine::base::{submit_commands, Base};
use crate::engine::mesh::{IndexBuffer, Vertex, VertexBuffer};
use crate::engine::texture::Texture;
use crate::engine::uniform::{CameraUniform, UniformBuffer};
use crate::engine::Input;
use ash::util::read_spv;
use ash::vk;
use glam::{vec3, Mat4};
use log::info;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub trait App {
    fn start() -> Self;
    fn update(&mut self, input: Input);
}

pub fn startup<A: App>(title: String) {
    #[cfg(windows)]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(1);
    }
    let system = sdl2::init().unwrap();
    let video = system.video().unwrap();
    let window_size = [960, 540];
    let window = video
        .window(&title, window_size[0], window_size[1])
        .allow_highdpi()
        .position(0, 0)
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

        let vertices = vec![
            Vertex {
                pos: [-1.0, 1.0, 0.0, 1.0],
                color: [0.0, 1.0, 0.0, 1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 0.0, 1.0],
                color: [0.0, 0.0, 1.0, 1.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                pos: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
                uv: [0.5, 1.0],
            },
        ];

        let indices = vec![0, 1, 2];

        let camera = CameraUniform {
            model: Mat4::IDENTITY,
            view: Mat4::look_at_rh(
                vec3(0.0, 0.0, 3.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
            ),
            proj: Mat4::perspective_rh(
                45.0_f32.to_radians(),
                window_size[0] as f32 / window_size[1] as f32,
                0.1,
                100.0,
            ),
        };

        let index_buffer =
            IndexBuffer::create(&base.device, &base.device_memory_properties, indices);
        let vertex_buffer =
            VertexBuffer::create(&base.device, &base.device_memory_properties, vertices);
        let camera_buffer = UniformBuffer::create::<CameraUniform>(
            base.device.clone(),
            &base.device_memory_properties,
            base.present_images.len(),
        );
        let texture = Texture::create_and_read_image(
            &base.device,
            base.pool,
            base.present_queue,
            &base.device_memory_properties,
            "./assets/mylama.png",
        );

        let mut vertex_spv_file =
            Cursor::new(&include_bytes!("../../../assets/shaders/triangle.vert.spv")[..]);
        let mut frag_spv_file =
            Cursor::new(&include_bytes!("../../../assets/shaders/triangle.frag.spv")[..]);

        let vertex_code =
            read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
        let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

        let frag_code =
            read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

        let vertex_shader_module = base
            .device
            .create_shader_module(&vertex_shader_info, None)
            .expect("Vertex shader module error");

        let fragment_shader_module = base
            .device
            .create_shader_module(&frag_shader_info, None)
            .expect("Fragment shader module error");

        let descriptor_set_layout = UniformBuffer::create_descriptor_set_layout(&base.device);
        let set_layouts = [descriptor_set_layout];
        let descriptor_pool =
            UniformBuffer::create_descriptor_pool(&base.device, base.present_images.len() as u32);
        let descriptor_sets = UniformBuffer::create_descriptor_sets::<CameraUniform>(
            &base.device,
            descriptor_pool,
            descriptor_set_layout,
            &camera_buffer.buffers,
            &texture,
            base.present_images.len(),
        );

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts);

        let pipeline_layout = base
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap();

        let shader_entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_input_state_info = Vertex::describe();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: base.surface_resolution.width as f32,
            height: base.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [base.surface_resolution.into()];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .build();

        let graphics_pipelines = base
            .device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None)
            .expect("Unable to create graphics pipeline");

        let graphic_pipeline = graphics_pipelines[0];

        let mut app = A::start();

        let mut time = Instant::now();
        let mut input = Input::new();
        loop {
            input.reset();
            input.time = time.elapsed().as_secs_f32();
            time = Instant::now();
            for event in event_pump.poll_iter() {
                input.handle(event);
            }

            if input.terminating {
                break;
            }

            app.update(input.clone());

            let (present_index, _) = base
                .swapchain_loader
                .acquire_next_image(
                    base.swapchain,
                    std::u64::MAX,
                    base.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [1.0, 0.0, 1.0, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(framebuffers[present_index as usize])
                .render_area(base.surface_resolution.into())
                .clear_values(&clear_values);

            camera_buffer.update(present_index as usize, camera.clone());

            submit_commands(
                &base.device,
                base.draw_command_buffer,
                base.draw_commands_reuse_fence,
                base.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[base.present_complete_semaphore],
                &[base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        graphic_pipeline,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_vertex_buffers(
                        draw_command_buffer,
                        0,
                        &[vertex_buffer.bind()],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        draw_command_buffer,
                        index_buffer.bind(),
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_bind_descriptor_sets(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_layout,
                        0,
                        &[descriptor_sets[0]],
                        &[],
                    );
                    device.cmd_draw_indexed(draw_command_buffer, index_buffer.count(), 1, 0, 0, 1);
                    // Or draw without the index buffer
                    // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
                    device.cmd_end_render_pass(draw_command_buffer);
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
                .queue_present(base.present_queue, &present_info)
                .unwrap();

            thread::sleep(Duration::from_millis(16));
        }
    }
}
