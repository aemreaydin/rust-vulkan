use crate::{device::VDevice, RendererResult};
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

#[derive(Default, Debug, Clone, Copy)]
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
