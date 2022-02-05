use crate::{device::VDevice, RendererResult};
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

#[derive(Default, Debug, Clone, Copy)]
pub struct VFence {
    fence: Fence,
}

impl VFence {
    pub fn new(device: &VDevice, is_signaled: bool) -> RendererResult<Self> {
        let create_info = Self::fence_create_info(is_signaled);
        let fence = unsafe { device.get().create_fence(&create_info, None)? };
        Ok(Self { fence })
    }

    pub fn get(&self) -> Fence {
        self.fence
    }

    fn fence_create_info(is_signaled: bool) -> FenceCreateInfo {
        let flags = match is_signaled {
            true => FenceCreateFlags::SIGNALED,
            false => FenceCreateFlags::empty(),
        };
        FenceCreateInfo {
            flags,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VSemaphore {
    semaphore: Semaphore,
}

impl VSemaphore {
    pub fn new(device: &VDevice) -> RendererResult<Self> {
        let create_info = Self::semaphore_create_info();
        let semaphore = unsafe { device.get().create_semaphore(&create_info, None)? };
        Ok(Self { semaphore })
    }

    pub fn get(&self) -> Semaphore {
        self.semaphore
    }

    fn semaphore_create_info() -> SemaphoreCreateInfo {
        SemaphoreCreateInfo {
            ..Default::default()
        }
    }
}
