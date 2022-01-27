use crate::{
    device::VDevice, enums::EOperationType, queue_family::VQueueFamilyIndices, RendererResult,
};
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

// TODO I should at least have a way to create a single command pool
pub struct VCommandPools {
    compute_pool: CommandPool,
    graphics_pool: CommandPool,
    present_pool: CommandPool,
}

impl VCommandPools {
    pub fn new(
        device: &VDevice,
        queue_family_indices: VQueueFamilyIndices,
    ) -> RendererResult<Self> {
        let compute_pool_create_info = Self::command_pool_create_info(queue_family_indices.compute);
        let graphics_pool_create_info =
            Self::command_pool_create_info(queue_family_indices.graphics);
        let present_pool_create_info = Self::command_pool_create_info(queue_family_indices.present);

        let compute_pool = unsafe {
            device
                .get()
                .create_command_pool(&compute_pool_create_info, None)?
        };
        let graphics_pool = unsafe {
            device
                .get()
                .create_command_pool(&graphics_pool_create_info, None)?
        };
        let present_pool = unsafe {
            device
                .get()
                .create_command_pool(&present_pool_create_info, None)?
        };
        Ok(Self {
            compute_pool,
            graphics_pool,
            present_pool,
        })
    }

    pub fn get(&self, operation_type: EOperationType) -> CommandPool {
        match operation_type {
            EOperationType::Compute => self.compute_pool,
            EOperationType::Graphics => self.graphics_pool,
            EOperationType::Present => self.present_pool,
        }
    }

    fn command_pool_create_info(queue_family_index: u32) -> CommandPoolCreateInfo {
        CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        command_pool::VCommandPools, device::VDevice, enums::EOperationType, instance::VInstance,
        physical_device::VPhysicalDevice, surface::VSurface, RendererResult,
    };
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_commandpool() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            let command_pools =
                VCommandPools::new(&device, physical_device.queue_family_indices())?;
            assert_ne!(command_pools.get(EOperationType::Compute).as_raw(), 0);
            assert_ne!(command_pools.get(EOperationType::Graphics).as_raw(), 0);
            assert_ne!(command_pools.get(EOperationType::Present).as_raw(), 0);
        }
        Ok(())
    }
}
