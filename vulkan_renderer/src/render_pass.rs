use crate::RendererResult;
use ash::{
    vk::{
        AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Format,
        ImageLayout, PipelineBindPoint, RenderPass, RenderPassCreateInfo, SampleCountFlags,
        SubpassDescription,
    },
    Device,
};

pub struct VRenderPass {
    render_pass: RenderPass,
}

impl VRenderPass {
    pub fn new(device: &Device, format: Format) -> RendererResult<Self> {
        let attachments = Self::attachment_descriptions(format);
        let attachment_refs = Self::attachment_refs();
        let subpass_descriptions = Self::subpass_descriptions(&attachment_refs);
        let create_info = Self::render_pass_create_info(&attachments, &subpass_descriptions);

        let render_pass = unsafe { device.create_render_pass(&create_info, None)? };
        Ok(Self { render_pass })
    }

    pub fn render_pass(&self) -> RenderPass {
        self.render_pass
    }

    fn render_pass_create_info(
        attachments: &[AttachmentDescription],
        subpass_descriptions: &[SubpassDescription],
    ) -> RenderPassCreateInfo {
        RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpass_descriptions.len() as u32,
            p_subpasses: subpass_descriptions.as_ptr(),
            ..Default::default()
        }
    }

    fn subpass_descriptions(
        color_attachment_refs: &[AttachmentReference],
    ) -> Vec<SubpassDescription> {
        let subpass_description = SubpassDescription {
            pipeline_bind_point: PipelineBindPoint::GRAPHICS,
            color_attachment_count: color_attachment_refs.len() as u32,
            p_color_attachments: color_attachment_refs.as_ptr(),
            ..Default::default()
        };
        vec![subpass_description]
    }

    fn attachment_refs() -> Vec<AttachmentReference> {
        let attachment_reference = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        vec![attachment_reference]
    }

    fn attachment_descriptions(format: Format) -> Vec<AttachmentDescription> {
        // Just color attachment for now
        let color_attachment = AttachmentDescription {
            format,
            initial_layout: ImageLayout::UNDEFINED,
            load_op: AttachmentLoadOp::CLEAR,
            samples: SampleCountFlags::TYPE_1,
            store_op: AttachmentStoreOp::STORE,
            stencil_load_op: AttachmentLoadOp::DONT_CARE,
            stencil_store_op: AttachmentStoreOp::DONT_CARE,
            final_layout: ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };
        vec![color_attachment]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        device::VDevice, instance::VInstance, physical_device::VPhysicalDevice, surface::VSurface,
        RendererResult,
    };
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    use super::VRenderPass;

    #[test]
    fn creates_swapchain() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;
            let render_pass = VRenderPass::new(
                &device.device(),
                physical_device
                    .physical_device_information()
                    .choose_surface_format()
                    .format,
            )?;

            assert_ne!(render_pass.render_pass.as_raw(), 0);
        }
        Ok(())
    }
}
