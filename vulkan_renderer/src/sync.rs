use crate::{command_pool::VCommandPool, device::VDevice, RendererResult};
use ash::vk::{
    CommandBuffer, CommandPool, CommandPoolCreateFlags, Fence, FenceCreateFlags, FenceCreateInfo,
    Semaphore, SemaphoreCreateInfo,
};

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

pub struct VFrameData {
    pub fence: VFence,
    pub present_semaphore: VSemaphore,
    pub render_semaphore: VSemaphore,
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
}

impl VFrameData {
    pub fn new(device: &VDevice, queue_family_index: u32) -> RendererResult<Self> {
        let fence = VFence::new(device, true)?;
        let present_semaphore = VSemaphore::new(device)?;
        let render_semaphore = VSemaphore::new(device)?;
        let command_pool = VCommandPool::new(
            device,
            queue_family_index,
            CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?
        .get();
        let command_buffer = device.allocate_command_buffers(command_pool, 1)?[0];

        Ok(Self {
            fence,
            present_semaphore,
            render_semaphore,
            command_buffer,
            command_pool,
        })
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
    fn creates_syncs() -> RendererResult<()> {
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
