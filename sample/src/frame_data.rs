use crate::camera::CameraData;
use ash::vk::{
    CommandBuffer, CommandPool, CommandPoolCreateFlags, DescriptorBufferInfo, DescriptorPool,
    DescriptorSet, DescriptorSetLayout, DescriptorType, MemoryPropertyFlags, WriteDescriptorSet,
};
use std::mem::size_of;
use vulkan_renderer::{
    buffer::VBuffer,
    command_pool::VCommandPool,
    descriptorset::VDescriptorSet,
    device::VDevice,
    sync::{VFence, VSemaphore},
    RendererResult,
};

pub struct FrameData {
    pub fence: VFence,
    pub present_semaphore: VSemaphore,
    pub render_semaphore: VSemaphore,
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub camera_buffer: VBuffer,
    pub desc_set: DescriptorSet,
}

impl FrameData {
    pub fn new(
        device: &VDevice,
        queue_family_index: u32,
        descriptor_pool: DescriptorPool,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> RendererResult<Self> {
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

        let camera_buffer = VBuffer::new_uniform_buffer(
            device,
            size_of::<CameraData>() as u64,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let desc_set = VDescriptorSet::new(device, descriptor_pool, descriptor_set_layouts)?.get();

        let buffer_info = DescriptorBufferInfo {
            buffer: camera_buffer.buffer(),
            range: size_of::<CameraData>() as u64,
            offset: 0,
        };

        let write_set = WriteDescriptorSet {
            dst_binding: 0,
            dst_set: desc_set,
            descriptor_count: 1,
            descriptor_type: DescriptorType::UNIFORM_BUFFER,
            p_buffer_info: &buffer_info,
            ..Default::default()
        };

        unsafe {
            device.get().update_descriptor_sets(&[write_set], &[]);
        };

        Ok(Self {
            fence,
            present_semaphore,
            render_semaphore,
            command_buffer,
            command_pool,
            camera_buffer,
            desc_set,
        })
    }
}
