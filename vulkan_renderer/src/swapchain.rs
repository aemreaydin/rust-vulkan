use crate::{device::VDevice, instance::VInstance, render_pass::VRenderPass, RendererResult};
use ash::{
    extensions::khr::Swapchain,
    vk::{
        ColorSpaceKHR, ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D, Fence,
        Format, Handle, Image, ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView,
        ImageViewCreateInfo, ImageViewType, PresentInfoKHR, PresentModeKHR, Queue, RenderPass,
        Semaphore, SharingMode, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
pub struct VSwapchain {
    swapchain: Swapchain,
    swapchain_khr: SwapchainKHR,
    images: Vec<Image>,
    image_views: Vec<ImageView>,
    render_pass: VRenderPass,
}

impl VSwapchain {
    pub fn new(instance: &VInstance, device: &VDevice, extent: Extent2D) -> RendererResult<Self> {
        let format = Format::B8G8R8A8_SRGB;
        let color_space = ColorSpaceKHR::SRGB_NONLINEAR;
        let present_mode = PresentModeKHR::FIFO;
        let swapchain = Swapchain::new(instance.get(), device.get());
        let create_info =
            Self::swapchain_create_info(device, format, color_space, extent, present_mode);
        let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None) }?;
        let images = unsafe { swapchain.get_swapchain_images(swapchain_khr)? };
        let image_views = Self::create_image_views(device, &images, format)?;

        let render_pass = VRenderPass::new(device.get(), format)?;
        Ok(Self {
            swapchain,
            swapchain_khr,
            images,
            image_views,
            render_pass,
        })
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn get_swapchain_khr(&self) -> SwapchainKHR {
        self.swapchain_khr
    }

    pub fn get_image(&self, image_ind: usize) -> Option<Image> {
        self.images.get(image_ind).copied()
    }

    pub fn get_image_view(&self, image_ind: usize) -> Option<ImageView> {
        self.image_views.get(image_ind).copied()
    }

    pub fn get_image_views(&self) -> &[ImageView] {
        &self.image_views
    }

    pub fn get_renderpass(&self) -> RenderPass {
        self.render_pass.get()
    }

    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: Option<Semaphore>,
        fence: Option<Fence>,
    ) -> RendererResult<(u32, bool)> {
        let fence = fence.unwrap_or_else(|| Fence::from_raw(0));
        let semaphore = semaphore.unwrap_or_else(|| Semaphore::from_raw(0));
        let next_image = unsafe {
            self.swapchain
                .acquire_next_image(self.swapchain_khr, timeout, semaphore, fence)?
        };
        Ok(next_image)
    }

    pub fn create_present_info(
        image_indices: &[u32],
        swapchains: &[SwapchainKHR],
        wait_semaphores: &[Semaphore],
    ) -> PresentInfoKHR {
        PresentInfoKHR {
            p_image_indices: image_indices.as_ptr(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            p_swapchains: swapchains.as_ptr(),
            ..Default::default()
        }
    }

    pub fn queue_present(&self, queue: Queue, present_info: PresentInfoKHR) -> RendererResult<()> {
        unsafe { self.swapchain.queue_present(queue, &present_info)? };
        Ok(())
    }

    fn create_image_views(
        device: &VDevice,
        images: &[Image],
        format: Format,
    ) -> RendererResult<Vec<ImageView>> {
        let image_views_result: Result<Vec<ImageView>, ash::vk::Result> = images
            .iter()
            .map(|&image| {
                let create_info = Self::image_view_create_info(image, format);
                unsafe { device.get().create_image_view(&create_info, None) }
            })
            .collect();
        match image_views_result {
            Ok(image_views) => Ok(image_views),
            Err(err) => Err(Box::new(err)),
        }
    }

    fn swapchain_create_info(
        device: &VDevice,
        image_format: Format,
        image_color_space: ColorSpaceKHR,
        image_extent: Extent2D,
        present_mode: PresentModeKHR,
    ) -> SwapchainCreateInfoKHR {
        let surface_capabilities = device.get_surface_capabilities();
        let min_image_count = surface_capabilities.min_image_count;
        let max_image_count = surface_capabilities.max_image_count;
        let mut desired_image_count = min_image_count + 1;
        if max_image_count > 0 && desired_image_count > max_image_count {
            desired_image_count = max_image_count;
        }

        let image_usage = ImageUsageFlags::COLOR_ATTACHMENT;
        let sharing_mode = SharingMode::EXCLUSIVE;
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(SurfaceTransformFlagsKHR::IDENTITY)
        {
            SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let composite_alpha = CompositeAlphaFlagsKHR::OPAQUE;
        let clipped = true;
        let image_array_layers = 1;
        SwapchainCreateInfoKHR {
            surface: device.get_surface_khr(),
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
