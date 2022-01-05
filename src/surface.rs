use ash::{extensions::khr::Surface, vk::SurfaceKHR, Instance};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::RendererResult;

pub struct VSurface {
    surface: Surface,
    surface_khr: Arc<SurfaceKHR>,
    window: Window,
}

impl VSurface {
    pub fn create_surface(instance: &Instance, event_loop: &EventLoop<()>) -> RendererResult<Self> {
        let entry = ash::Entry::linked();

        // TODO Use JSON to get these information
        let window = WindowBuilder::new()
            .with_title("Vulkan Renderer")
            .with_inner_size(PhysicalSize::new(1920, 1080))
            .build(event_loop)?;

        let surface = Surface::new(&entry, instance);
        let surface_khr = unsafe { ash_window::create_surface(&entry, instance, &window, None)? };

        Ok(Self {
            surface,
            surface_khr: Arc::new(surface_khr),
            window,
        })
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_khr(&self) -> Arc<SurfaceKHR> {
        self.surface_khr.clone()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

#[cfg(test)]
mod tests {
    use super::VSurface;
    use crate::{instance::VInstance, RendererResult};
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_surface() -> RendererResult<()> {
        let instance = VInstance::create("Test", 0)?.instance();
        #[cfg(target_os = "windows")]
        VSurface::create_surface(&instance, &EventLoopExtWindows::new_any_thread())?;

        Ok(())
    }
}
