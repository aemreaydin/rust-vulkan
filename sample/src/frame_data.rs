use crate::{camera::CameraData, scene::SceneData};
use ash::vk::{
    CommandBuffer, CommandPool, CommandPoolCreateFlags, DescriptorBufferInfo, DescriptorPool,
    DescriptorSet, DescriptorSetLayout, DescriptorType, MemoryPropertyFlags,
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
    pub frame_index: usize,
}

impl FrameData {
    pub fn new(
        device: &VDevice,
        queue_family_index: u32,
        descriptor_pool: DescriptorPool,
        descriptor_set_layouts: &[DescriptorSetLayout],
        scene_buffer: VBuffer,
        frame_index: usize,
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

        let camera_buffer_info = DescriptorBufferInfo {
            buffer: camera_buffer.buffer(),
            range: size_of::<CameraData>() as u64,
            offset: 0,
        };
        let scene_buffer_info = DescriptorBufferInfo {
            buffer: scene_buffer.buffer(),
            range: size_of::<SceneData>() as u64,
            offset: 0,
        };

        let camera_write_set = VDescriptorSet::write_descriptor_set(
            desc_set,
            0,
            DescriptorType::UNIFORM_BUFFER,
            &camera_buffer_info,
        );
        let scene_write_set = VDescriptorSet::write_descriptor_set(
            desc_set,
            1,
            DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            &scene_buffer_info,
        );

        unsafe {
            device
                .get()
                .update_descriptor_sets(&[camera_write_set, scene_write_set], &[]);
        };

        Ok(Self {
            fence,
            present_semaphore,
            render_semaphore,
            command_buffer,
            command_pool,
            camera_buffer,
            desc_set,
            frame_index,
        })
    }
}
