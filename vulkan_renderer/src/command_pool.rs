use crate::{device::VDevice, RendererResult};
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

#[derive(Clone, Copy)]
pub struct VCommandPool {
    command_pool: CommandPool,
}

impl VCommandPool {
    pub fn new(
        device: &VDevice,
        queue_family_index: u32,
        flags: CommandPoolCreateFlags,
    ) -> RendererResult<Self> {
        let create_info = Self::command_pool_create_info(queue_family_index, flags);
        let command_pool = unsafe { device.get().create_command_pool(&create_info, None)? };
        Ok(Self { command_pool })
    }

    pub fn get(&self) -> CommandPool {
        self.command_pool
    }

    fn command_pool_create_info(
        queue_family_index: u32,
        flags: CommandPoolCreateFlags,
    ) -> CommandPoolCreateInfo {
        CommandPoolCreateInfo {
            flags,
            queue_family_index,
            ..Default::default()
        }
    }
}
// TODO I should at least have a way to create a single command pool
// pub struct VCommandPools {
//     compute_pool: VCommandPool,
//     graphics_pool: VCommandPool,
//     present_pool: VCommandPool,
// }

// impl VCommandPools {
//     pub fn new(
//         device: &VDevice,
//         queue_family_indices: VQueueFamilyIndices,
//     ) -> RendererResult<Self> {

//         Ok(Self {
//             compute_pool,
//             graphics_pool,
//             present_pool,
//         })
//     }

//     pub fn get(&self, operation_type: EOperationType) -> CommandPool {
//         match operation_type {
//             EOperationType::Compute => self.compute_pool,
//             EOperationType::Graphics => self.graphics_pool,
//             EOperationType::Present => self.present_pool,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{
        command_pool::VCommandPool, device::VDevice, instance::VInstance,
        physical_device::VPhysicalDevice, surface::VSurface, RendererResult,
    };
    use ash::vk::{CommandPoolCreateFlags, Handle};
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_commandpool() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            let command_pool = VCommandPool::new(
                &device,
                physical_device.queue_family_indices().graphics,
                CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )?;
            assert_ne!(command_pool.get().as_raw(), 0);
        }
        Ok(())
    }
}
