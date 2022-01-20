use vulkan_renderer::{
    device::VDevice, instance::VInstance, physical_device::VPhysicalDevice, surface::VSurface,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let instance = VInstance::new("Sample", 0).expect("Failed to create instance.");
    let surface =
        VSurface::new(instance.instance(), &event_loop).expect("Failed to create surface.");

    let physical_device =
        VPhysicalDevice::new(&instance, &surface).expect("Failed to create physical device.");
    let _device = VDevice::new(&physical_device).expect("Failed to create device.");

    event_loop.run(|event, _, control_flow| {
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
            _ => (),
        }
    });
}
