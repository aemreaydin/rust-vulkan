use ash::{extensions::khr::Surface, vk::SurfaceKHR, Instance};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::RendererResult;

pub struct VSurface {
    surface: Surface,
    surface_khr: SurfaceKHR,
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
            surface_khr,
            window,
        })
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_khr(&self) -> SurfaceKHR {
        self.surface_khr
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn dimensions(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }
}

#[cfg(test)]
mod tests {
    use super::VSurface;
    use crate::{instance::VInstance, RendererResult};
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_surface() -> RendererResult<()> {
        let instance = VInstance::create("Test", 0)?;
        let instance = instance.instance();
        #[cfg(target_os = "windows")]
        let surface = VSurface::create_surface(instance, &EventLoopExtWindows::new_any_thread())?;

        assert_ne!(surface.surface_khr.as_raw(), 0);

        Ok(())
    }
}
