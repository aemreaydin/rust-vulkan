use ash::{
    extensions::khr::Swapchain,
    vk::{
        CompositeAlphaFlagsKHR, Extent2D, ImageUsageFlags, SharingMode, SurfaceTransformFlagsKHR,
        SwapchainCreateInfoKHR, SwapchainKHR,
    },
};

use crate::{device::VDevice, physical_device::VPhysicalDevice, RendererResult};

pub struct VSwapchain {
    swapchain: Swapchain,
    swapchain_khr: SwapchainKHR,
}

impl VSwapchain {
    pub fn new(device: &VDevice) -> RendererResult<Self> {
        let swapchain = Swapchain::new(device.instance(), device.device());
        let create_info = Self::swapchain_create_info(device.physical_device());
        let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None) }?;

        Ok(Self {
            swapchain,
            swapchain_khr,
        })
    }

    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn swapchain_khr(&self) -> SwapchainKHR {
        self.swapchain_khr
    }

    fn swapchain_create_info(physical_device: &VPhysicalDevice) -> SwapchainCreateInfoKHR {
        let phys_dev_info = physical_device.physical_device_information();
        let min_image_count = phys_dev_info.surface_capabilities.min_image_count;
        let max_image_count = phys_dev_info.surface_capabilities.max_image_count;
        let mut desired_image_count = min_image_count + 1;
        if max_image_count > 0 && desired_image_count > max_image_count {
            desired_image_count = max_image_count;
        }

        let image_format = phys_dev_info.choose_surface_format().format;
        let image_color_space = phys_dev_info.choose_surface_format().color_space;
        let present_mode = phys_dev_info.choose_present_mode();
        let image_extent = Extent2D {
            width: physical_device.surface().dimensions().width,
            height: physical_device.surface().dimensions().height,
        };
        let image_usage = ImageUsageFlags::COLOR_ATTACHMENT;
        let sharing_mode = SharingMode::EXCLUSIVE;
        let pre_transform = if phys_dev_info
            .surface_capabilities
            .supported_transforms
            .contains(SurfaceTransformFlagsKHR::IDENTITY)
        {
            SurfaceTransformFlagsKHR::IDENTITY
        } else {
            phys_dev_info.surface_capabilities.current_transform
        };
        let composite_alpha = CompositeAlphaFlagsKHR::OPAQUE;
        let clipped = true;
        let image_array_layers = 1;
        SwapchainCreateInfoKHR {
            surface: physical_device.surface_khr(),
            min_image_count: desired_image_count,
            image_format,
            image_color_space,
            image_extent,
            image_usage,
            image_sharing_mode: sharing_mode,
            present_mode,
            pre_transform,
            composite_alpha,
            clipped: clipped.into(),
            image_array_layers,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VDevice, VPhysicalDevice, VSwapchain};
    use crate::{instance::VInstance, surface::VSurface, RendererResult};
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_swapchain() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface =
                VSurface::new(instance.instance(), &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&physical_device)?;

            let swapchain = VSwapchain::new(&device)?;
            assert_ne!(swapchain.swapchain_khr.as_raw(), 0);
        }
        Ok(())
    }
}
