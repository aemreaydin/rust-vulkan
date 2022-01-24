use ash::vk::{
    ClearColorValue, ClearValue, ColorComponentFlags, PipelineBindPoint,
    PipelineColorBlendAttachmentState, PipelineStageFlags, Rect2D, ShaderStageFlags, Viewport,
};
use vulkan_renderer::{
    device::VDevice,
    enums::EOperationType,
    framebuffer::VFramebuffers,
    glm::Vec4,
    instance::VInstance,
    physical_device::VPhysicalDevice,
    pipeline::VGraphicsPipelineBuilder,
    primitives::{mesh::Mesh, vertex::Vertex},
    shader_utils::VShaderUtils,
    surface::VSurface,
    swapchain::VSwapchain,
    sync::{VFence, VSemaphore},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    // Device Vars
    let event_loop = EventLoop::new();
    let instance = VInstance::new("Sample", 0).expect("Failed to create instance.");
    let surface = VSurface::new(&instance, &event_loop).expect("Failed to create surface.");
    let physical_device =
        VPhysicalDevice::new(&instance, &surface).expect("Failed to create physical device.");
    let device = VDevice::new(&instance, &physical_device).expect("Failed to create device.");
    let swapchain = VSwapchain::new(&instance, &physical_device, &device, &surface)
        .expect("Failed to create swapchain.");
    let framebuffers =
        VFramebuffers::new(&device, swapchain.get_image_views(), surface.dimensions())
            .expect("Failed to create framebuffers.");
    let command_buffer = device
        .allocate_command_buffers(1, EOperationType::Graphics)
        .expect("Failed to allocate command buffers.")[0];

    // Shader Vars
    let vertex_code = VShaderUtils::load_shader("./sample/shaders/base.vert.spv")
        .expect("Failed to load vertex shader code.");
    let vertex_shader_module = VShaderUtils::create_shader_module(&device, &vertex_code)
        .expect("Failed to create vertex shader module.");
    let fragment_code = VShaderUtils::load_shader("./sample/shaders/base.frag.spv")
        .expect("Failed to load fragment shader code.");
    let fragment_shader_module = VShaderUtils::create_shader_module(&device, &fragment_code)
        .expect("Failed to create fragment shader module.");

    // Graphics Pipeline
    let builder = VGraphicsPipelineBuilder::start();
    let shader_infos = &[
        (ShaderStageFlags::VERTEX, vertex_shader_module),
        (ShaderStageFlags::FRAGMENT, fragment_shader_module),
    ];
    let viewports = &[Viewport {
        x: 0.0,
        y: 0.0,
        max_depth: 1.0,
        min_depth: 0.0,
        height: surface.extent_2d().height as f32,
        width: surface.extent_2d().width as f32,
    }];
    let scissors = &[Rect2D {
        extent: surface.extent_2d(),
        ..Default::default()
    }];
    let color_blend_attachments = &[PipelineColorBlendAttachmentState {
        color_write_mask: ColorComponentFlags::RGBA,
        ..Default::default()
    }];
    let vertex_input_desc = Vertex::vertex_description();
    let builder = builder
        .shader_stages(shader_infos)
        .vertex_input(&vertex_input_desc.bindings, &vertex_input_desc.attributes)
        .viewport(viewports, scissors)
        .color_blend_state(color_blend_attachments);
    let pipeline = builder
        .build(&device)
        .expect("Failed to create graphics pipeline.");

    // Sync Vars
    let fence = VFence::new(&device, true).expect("Failed to create fence.");
    let graphics_semaphore =
        VSemaphore::new(&device).expect("Failed to create graphics semaphore.");
    let present_semaphore = VSemaphore::new(&device).expect("Failed to create present semaphore.");

    let triangle_mesh = Mesh::new(
        &device,
        vec![
            Vertex::new(
                Vec4::new(1.0, 1.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::default(),
            ),
            Vertex::new(
                Vec4::new(-1.0, 1.0, 0.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::default(),
            ),
            Vertex::new(
                Vec4::new(0.0, -1.0, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::default(),
            ),
        ],
        vec![0, 1, 2],
    );

    let mut frame_count = 0;
    event_loop.run(move |event, _, control_flow| {
        let fences = &[fence.get()];
        device
            .wait_for_fences(fences, 1_000_000_000)
            .expect("Failed to wait for fences.");
        device
            .reset_fences(fences)
            .expect("Failed to reset fences.");

        let (image_ind, _is_suboptimal) = swapchain
            .acquire_next_image(1_000_000_000, Some(present_semaphore.get()), None)
            .expect("Failed to acquire next image.");

        let flash = (frame_count as f32 / 1200.0).sin().abs();

        // Begin Rendering
        device
            .begin_command_buffer(command_buffer)
            .expect("Failed to begin command buffer.");

        let clear_values = &[ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, flash, 1.0],
            },
        }];
        device.begin_render_pass(
            command_buffer,
            framebuffers[image_ind],
            clear_values,
            surface.extent_2d(),
        );

        device.bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, pipeline);
        device.bind_vertex_buffer(
            command_buffer,
            &[triangle_mesh.vertex_buffer().buffer()],
            &[0],
        );
        device.bind_index_buffer(command_buffer, triangle_mesh.index_buffer().buffer(), 0);
        device.draw_indexed(command_buffer, triangle_mesh.indices().len() as u32, 1);

        device.end_render_pass(command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer.");

        let command_buffers = &[command_buffer];
        let wait_semaphores = &[present_semaphore.get()];
        let dst_semaphores = &[graphics_semaphore.get()];
        let pipeline_stage_flags = &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = VDevice::create_queue_submit_info(
            command_buffers,
            wait_semaphores,
            dst_semaphores,
            pipeline_stage_flags,
        );

        device
            .queue_submit(
                device.get_queue(EOperationType::Graphics),
                &[submit_info],
                fence.get(),
            )
            .expect("Failed to submit queue.");

        let image_indices = &[image_ind];
        let swapchains = &[swapchain.swapchain_khr()];
        let wait_semaphores = &[graphics_semaphore.get()];
        let present_info =
            VSwapchain::create_present_info(image_indices, swapchains, wait_semaphores);
        swapchain
            .queue_present(device.get_queue(EOperationType::Graphics), present_info)
            .expect("Failed to present queue.");

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {}
            _ => (),
        }
        frame_count += 1;
    });
}
