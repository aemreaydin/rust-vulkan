use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use crate::{device::VDevice, RendererResult};

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

#[cfg(test)]
mod tests {
    use crate::{
        device::VDevice, instance::VInstance, physical_device::VPhysicalDevice, surface::VSurface,
        RendererResult,
    };
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    use super::{VFence, VSemaphore};

    #[test]
    fn creates_swapchain() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            let fence = VFence::new(&device, true)?;
            let semaphore = VSemaphore::new(&device)?;

            assert_ne!(fence.fence.as_raw(), 0);
            assert_ne!(semaphore.semaphore.as_raw(), 0);
        }
        Ok(())
    }
}
