use crate::{
    queue_family::{VQueueFamilyIndices, VQueueType},
    RendererResult,
};
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};
use ash::Device;

pub struct VCommandPools {
    compute_pool: CommandPool,
    graphics_pool: CommandPool,
    present_pool: CommandPool,
}

impl VCommandPools {
    pub fn new(device: &Device, queue_family_indices: VQueueFamilyIndices) -> RendererResult<Self> {
        let compute_pool_create_info = Self::command_pool_create_info(queue_family_indices.compute);
        let graphics_pool_create_info =
            Self::command_pool_create_info(queue_family_indices.graphics);
        let present_pool_create_info = Self::command_pool_create_info(queue_family_indices.present);

        let compute_pool = unsafe { device.create_command_pool(&compute_pool_create_info, None)? };
        let graphics_pool =
            unsafe { device.create_command_pool(&graphics_pool_create_info, None)? };
        let present_pool = unsafe { device.create_command_pool(&present_pool_create_info, None)? };
        Ok(Self {
            compute_pool,
            graphics_pool,
            present_pool,
        })
    }

    pub fn get_command_pool(&self, queue_type: VQueueType) -> CommandPool {
        match queue_type {
            VQueueType::Compute => self.compute_pool,
            VQueueType::Graphics => self.graphics_pool,
            VQueueType::Present => self.present_pool,
        }
    }

    fn command_pool_create_info(queue_family_index: u32) -> CommandPoolCreateInfo {
        CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
            ..Default::default()
        }
    }
}
