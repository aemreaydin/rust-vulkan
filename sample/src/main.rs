use vulkan_renderer::{
    device::VDevice,
    enums::EOperationType,
    framebuffer::VFramebuffers,
    instance::VInstance,
    physical_device::VPhysicalDevice,
    surface::VSurface,
    swapchain::VSwapchain,
    sync::{VFence, VSemaphore},
    ClearColorValue, ClearValue, PipelineStageFlags,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
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

    let fence = VFence::new(&device, true).expect("Failed to create fence.");
    let graphics_semaphore =
        VSemaphore::new(&device).expect("Failed to create graphics semaphore.");
    let present_semaphore = VSemaphore::new(&device).expect("Failed to create present semaphore.");

    let mut frame_count = 0;
    event_loop.run(move |event, _, control_flow| {
        let fences = &[fence.fence()];
        device
            .wait_for_fences(fences, 1_000_000_000)
            .expect("Failed to wait for fences.");
        device
            .reset_fences(fences)
            .expect("Failed to reset fences.");

        let (image_ind, _is_suboptimal) = swapchain
            .acquire_next_image(1_000_000_000, Some(present_semaphore.semaphore()), None)
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
        device.end_render_pass(command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer.");

        let command_buffers = &[command_buffer];
        let wait_semaphores = &[present_semaphore.semaphore()];
        let dst_semaphores = &[graphics_semaphore.semaphore()];
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
                fence.fence(),
            )
            .expect("Failed to submit queue.");

        let image_indices = &[image_ind];
        let swapchains = &[swapchain.swapchain_khr()];
        let wait_semaphores = &[graphics_semaphore.semaphore()];
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
