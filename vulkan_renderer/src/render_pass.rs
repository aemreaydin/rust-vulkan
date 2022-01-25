use crate::RendererResult;
use ash::{
    vk::{
        AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference,
        AttachmentStoreOp, Format, ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPass,
        RenderPassCreateInfo, SampleCountFlags, SubpassDependency, SubpassDescription,
        SUBPASS_EXTERNAL,
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
        let depth_attachment_ref = Self::depth_attachment_ref();
        let subpass_descriptions =
            Self::subpass_descriptions(&attachment_refs, &depth_attachment_ref);
        let subpass_dependencies = Self::subpass_dependencies();
        let create_info = Self::render_pass_create_info(
            &attachments,
            &subpass_descriptions,
            &subpass_dependencies,
        );

        let render_pass = unsafe { device.create_render_pass(&create_info, None)? };
        Ok(Self { render_pass })
    }

    pub fn get(&self) -> RenderPass {
        self.render_pass
    }

    fn render_pass_create_info(
        attachments: &[AttachmentDescription],
        subpass_descriptions: &[SubpassDescription],
        subpass_dependencies: &[SubpassDependency],
    ) -> RenderPassCreateInfo {
        RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpass_descriptions.len() as u32,
            p_subpasses: subpass_descriptions.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
            ..Default::default()
        }
    }

    fn subpass_descriptions(
        attachment_refs: &[AttachmentReference],
        depth_attachment_ref: &AttachmentReference,
    ) -> Vec<SubpassDescription> {
        let subpass_description = SubpassDescription {
            pipeline_bind_point: PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            p_depth_stencil_attachment: depth_attachment_ref,
            ..Default::default()
        };
        vec![subpass_description]
    }

    fn subpass_dependencies() -> Vec<SubpassDependency> {
        let color_dependency = SubpassDependency {
            src_subpass: SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: AccessFlags::empty(),
            dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };
        let depth_dependency = SubpassDependency {
            src_subpass: SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            src_access_mask: AccessFlags::empty(),
            dst_stage_mask: PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            dst_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        };
        vec![color_dependency, depth_dependency]
    }

    fn attachment_refs() -> Vec<AttachmentReference> {
        let color_attachment_reference = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        vec![color_attachment_reference]
    }

    fn depth_attachment_ref() -> AttachmentReference {
        AttachmentReference {
            attachment: 1,
            layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }
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
        let depth_attachment = AttachmentDescription {
            format: Format::D32_SFLOAT,
            initial_layout: ImageLayout::UNDEFINED,
            load_op: AttachmentLoadOp::CLEAR,
            samples: SampleCountFlags::TYPE_1,
            store_op: AttachmentStoreOp::STORE,
            stencil_load_op: AttachmentLoadOp::DONT_CARE,
            stencil_store_op: AttachmentStoreOp::DONT_CARE,
            final_layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };
        vec![color_attachment, depth_attachment]
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
    fn creates_renderpass() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;
            let render_pass = VRenderPass::new(
                device.get(),
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
