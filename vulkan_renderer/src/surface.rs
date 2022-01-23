use crate::{instance::VInstance, RendererResult};
use ash::{
    extensions::khr::Surface,
    vk::{Extent2D, SurfaceKHR},
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct VSurface {
    surface: Arc<Surface>,
    surface_khr: SurfaceKHR,
    window: Arc<Window>,
}

impl VSurface {
    pub fn new(instance: &VInstance, event_loop: &EventLoop<()>) -> RendererResult<Self> {
        let entry = ash::Entry::linked();

        // TODO Use JSON to get these information
        let window = WindowBuilder::new()
            .with_title("Vulkan Renderer")
            .with_inner_size(PhysicalSize::new(1920, 1080))
            .build(event_loop)?;

        let surface = Surface::new(&entry, &instance.get());
        let surface_khr =
            unsafe { ash_window::create_surface(&entry, &instance.get(), &window, None)? };

        Ok(Self {
            surface: Arc::new(surface),
            surface_khr,
            window: Arc::new(window),
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

    pub fn extent_2d(&self) -> Extent2D {
        let PhysicalSize { width, height } = self.dimensions();
        Extent2D { width, height }
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
        let instance = VInstance::new("Test", 0)?;
        #[cfg(target_os = "windows")]
        let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;

        assert_ne!(surface.surface_khr.as_raw(), 0);

        Ok(())
    }
}
