use crate::{device::VDevice, RendererResult};
use ash::vk::{Extent2D, Framebuffer, FramebufferCreateInfo, ImageView, RenderPass};
use std::ops::Index;

#[derive(Default, Debug)]
pub struct VFramebuffers {
    framebuffers: Vec<Framebuffer>,
}

impl VFramebuffers {
    pub fn new(
        device: &VDevice,
        image_views: &[ImageView],
        depth_image_view: ImageView,
        render_pass: RenderPass,
        extent: Extent2D,
    ) -> RendererResult<Self> {
        let framebuffers_result: Result<Vec<Framebuffer>, ash::vk::Result> = image_views
            .iter()
            .map(|&image_view| {
                let attachments = vec![image_view, depth_image_view];
                let create_info = Self::framebuffer_create_info(&attachments, render_pass, extent);
                unsafe { device.get().create_framebuffer(&create_info, None) }
            })
            .collect();

        let framebuffers = match framebuffers_result {
            Ok(framebuffers) => Ok(framebuffers),
            Err(err) => Err(Box::new(err)),
        }?;

        Ok(Self { framebuffers })
    }

    pub fn get(&self, framebuffer_ind: usize) -> Option<Framebuffer> {
        self.framebuffers.get(framebuffer_ind).copied()
    }

    pub fn get_framebuffers(&self) -> Vec<Framebuffer> {
        self.framebuffers.clone()
    }

    fn framebuffer_create_info(
        attachments: &[ImageView],
        render_pass: RenderPass,
        extent: Extent2D,
    ) -> FramebufferCreateInfo {
        FramebufferCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            render_pass,
            width: extent.width,
            height: extent.height,
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
