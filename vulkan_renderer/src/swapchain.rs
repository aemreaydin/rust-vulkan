use ash::{
    extensions::khr::Swapchain,
    vk::{
        ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D, Format, Image,
        ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo,
        ImageViewType, SharingMode, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};

use crate::{device::VDevice, physical_device::VPhysicalDevice, RendererResult};

pub struct VSwapchain {
    swapchain: Swapchain,
    swapchain_khr: SwapchainKHR,
    images: Vec<Image>,
    image_views: Vec<ImageView>,
}

impl VSwapchain {
    pub fn new(device: &VDevice) -> RendererResult<Self> {
        let swapchain = Swapchain::new(device.instance(), device.device());
        let create_info = Self::swapchain_create_info(device.physical_device());
        let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None) }?;
        let images = unsafe { swapchain.get_swapchain_images(swapchain_khr)? };
        let image_views = Self::create_image_views(device, &images)?;

        Ok(Self {
            swapchain,
            swapchain_khr,
            images,
            image_views,
        })
    }

    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn swapchain_khr(&self) -> SwapchainKHR {
        self.swapchain_khr
    }

    pub fn get_image(&self, image_ind: usize) -> Option<Image> {
        self.images.get(image_ind).copied()
    }

    pub fn get_image_view(&self, image_ind: usize) -> Option<ImageView> {
        self.image_views.get(image_ind).copied()
    }

    fn create_image_views(device: &VDevice, images: &[Image]) -> RendererResult<Vec<ImageView>> {
        let format = device
            .physical_device()
            .physical_device_information()
            .choose_surface_format()
            .format;

        let image_views_result: Result<Vec<ImageView>, ash::vk::Result> = images
            .iter()
            .map(|&image| {
                let create_info = Self::image_view_create_info(image, format);
                let image_view = unsafe { device.device().create_image_view(&create_info, None) };
                image_view
            })
            .collect();
        match image_views_result {
            Ok(image_views) => Ok(image_views),
            Err(err) => Err(Box::new(err)),
        }
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

    fn image_view_create_info(image: Image, format: Format) -> ImageViewCreateInfo {
        ImageViewCreateInfo {
            format,
            image,
            view_type: ImageViewType::TYPE_2D,
            components: ComponentMapping {
                r: ComponentSwizzle::R,
                g: ComponentSwizzle::G,
                b: ComponentSwizzle::B,
                a: ComponentSwizzle::A,
            },
            subresource_range: ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                base_array_layer: 0,
                base_mip_level: 0,
            },
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

            assert_eq!(swapchain.image_views.len(), swapchain.images.len());
            for image in swapchain.images.iter() {
                assert_ne!(image.as_raw(), 0);
            }
            for image_view in swapchain.image_views.iter() {
                assert_ne!(image_view.as_raw(), 0);
            }
        }
        Ok(())
    }
}
