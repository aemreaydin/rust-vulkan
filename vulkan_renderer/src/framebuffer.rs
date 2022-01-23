use crate::{device::VDevice, RendererResult};
use ash::vk::{Framebuffer, FramebufferCreateInfo, ImageView, RenderPass};
use std::ops::Index;
use winit::dpi::PhysicalSize;

pub struct VFramebuffers {
    framebuffers: Vec<Framebuffer>,
}

impl VFramebuffers {
    pub fn new(
        device: &VDevice,
        image_views: &[ImageView],
        dimensions: PhysicalSize<u32>,
    ) -> RendererResult<Self> {
        let framebuffers_result: Result<Vec<Framebuffer>, ash::vk::Result> = image_views
            .iter()
            .map(|&image_view| {
                let attachments = vec![image_view];
                let create_info =
                    Self::framebuffer_create_info(&attachments, device.render_pass(), dimensions);
                unsafe { device.device().create_framebuffer(&create_info, None) }
            })
            .collect();

        let framebuffers = match framebuffers_result {
            Ok(framebuffers) => Ok(framebuffers),
            Err(err) => Err(Box::new(err)),
        }?;

        Ok(Self { framebuffers })
    }

    pub fn get_framebuffer(&self, framebuffer_ind: usize) -> Option<Framebuffer> {
        self.framebuffers.get(framebuffer_ind).copied()
    }

    fn framebuffer_create_info(
        attachments: &[ImageView],
        render_pass: RenderPass,
        dimensions: PhysicalSize<u32>,
    ) -> FramebufferCreateInfo {
        FramebufferCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            render_pass,
            width: dimensions.width,
            height: dimensions.height,
            layers: 1,
            ..Default::default()
        }
    }
}

macro_rules! impl_index_for_vframebuffers {
    ($ty: ident) => {
        impl Index<$ty> for VFramebuffers {
            type Output = Framebuffer;
            fn index(&self, index: $ty) -> &Self::Output {
                &self.framebuffers[index as usize]
            }
        }
    };
}
impl_index_for_vframebuffers!(usize);
impl_index_for_vframebuffers!(u32);

#[cfg(test)]
mod tests {
    use crate::{
        device::VDevice, framebuffer::VFramebuffers, instance::VInstance,
        physical_device::VPhysicalDevice, surface::VSurface, swapchain::VSwapchain, RendererResult,
    };
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_swapchain() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;
            let swapchain = VSwapchain::new(&instance, &physical_device, &device, &surface)?;
            let framebuffers =
                VFramebuffers::new(&device, swapchain.get_image_views(), surface.dimensions())?;

            for framebuffer in framebuffers.framebuffers {
                assert_ne!(framebuffer.as_raw(), 0);
            }
        }
        Ok(())
    }
}
