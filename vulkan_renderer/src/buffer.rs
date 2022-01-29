use crate::{
    command_pool::VCommandPool,
    device::VDevice,
    enums::EOperationType,
    impl_get,
    primitives::{mesh::Index, vertex::Vertex},
    RendererResult,
};
use ash::vk::{
    Buffer, BufferCopy, BufferCreateInfo, BufferUsageFlags, CommandBufferBeginInfo,
    CommandBufferUsageFlags, CommandPoolCreateFlags, DeviceMemory, Fence, MemoryAllocateInfo,
    MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties,
    SharingMode, SubmitInfo,
};
use std::mem::size_of;

#[derive(Clone, Copy, Default)]
pub struct VBuffer {
    buffer: Buffer,
    memory: DeviceMemory,
}
// Create a staging buffer
// Create a transient command buffer
// Copy staging buffer into vertex buffer
impl VBuffer {
    /// Creates a [`Buffer`] and a [`DeviceMemory`]
    ///
    /// Maps the buffer to the memory and binds it
    pub fn new_mapped<T: Copy>(
        device: &VDevice,
        data: &[T],
        usage: BufferUsageFlags,
        flags: MemoryPropertyFlags,
    ) -> RendererResult<Self> {
        let buffer = Self::create_buffer(device, data, usage)?;
        let memory_requirements = Self::memory_requirements(device, buffer);
        let memory = Self::create_memory(device, memory_requirements, flags)?;
        unsafe { device.get().bind_buffer_memory(buffer, memory, 0)? };

        Self::map_memory(device, data, memory, memory_requirements)?;

        Ok(Self { buffer, memory })
    }

    /// Creates a [`Buffer`] and a [`DeviceMemory`] without mapping
    ///
    /// Useful for creating staging buffers
    pub fn new_unmapped<T: Copy>(
        device: &VDevice,
        data: &[T],
        usage: BufferUsageFlags,
        flags: MemoryPropertyFlags,
    ) -> RendererResult<Self> {
        let buffer = Self::create_buffer(device, data, usage)?;
        let memory_requirements = Self::memory_requirements(device, buffer);
        let memory = Self::create_memory(device, memory_requirements, flags)?;
        unsafe { device.get().bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self { buffer, memory })
    }

    pub fn new_vertex_buffer(device: &VDevice, vertices: &[Vertex]) -> RendererResult<Self> {
        let staging_buffer = Self::new_mapped(
            device,
            vertices,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let vertex_buffer = Self::new_unmapped(
            device,
            vertices,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(
            device,
            vertices,
            staging_buffer.buffer,
            vertex_buffer.buffer,
        )?;

        Ok(vertex_buffer)
    }

    pub fn new_index_buffer(device: &VDevice, indices: &[Index]) -> RendererResult<Self> {
        let staging_buffer = Self::new_mapped(
            device,
            indices,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let index_buffer = Self::new_unmapped(
            device,
            indices,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(device, indices, staging_buffer.buffer, index_buffer.buffer)?;

        Ok(index_buffer)
    }

    fn copy_buffer<T>(
        device: &VDevice,
        data: &[T],
        src: Buffer,
        dst: Buffer,
    ) -> RendererResult<()> {
        let command_pool = VCommandPool::new(
            device,
            device.get_queue_family_index(EOperationType::Graphics),
            CommandPoolCreateFlags::TRANSIENT,
        )?;
        let command_buffer = device.allocate_command_buffers(command_pool.get(), 1)?[0];

        unsafe {
            device.get().begin_command_buffer(
                command_buffer,
                &CommandBufferBeginInfo::builder().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;

            let region = *BufferCopy::builder().size((data.len() * size_of::<T>()) as u64);
            device
                .get()
                .cmd_copy_buffer(command_buffer, src, dst, &[region]);

            device.get().end_command_buffer(command_buffer)?;

            let command_buffers = &[command_buffer];
            let submit_info = *SubmitInfo::builder().command_buffers(command_buffers);
            device.get().queue_submit(
                device.get_queue(EOperationType::Graphics),
                &[submit_info],
                Fence::null(),
            )?;
            device
                .get()
                .queue_wait_idle(device.get_queue(EOperationType::Graphics))?;
        };

        Ok(())
    }
    fn create_buffer<T: Copy>(
        device: &VDevice,
        data: &[T],
        usage: BufferUsageFlags,
    ) -> RendererResult<Buffer> {
        let create_info = Self::buffer_create_info((data.len() * size_of::<T>()) as u64, usage);
        unsafe { Ok(device.get().create_buffer(&create_info, None)?) }
    }

    fn create_memory(
        device: &VDevice,
        memory_requirements: MemoryRequirements,
        flags: MemoryPropertyFlags,
    ) -> RendererResult<DeviceMemory> {
        let mem_type_ind =
            Self::find_memory_type_index(memory_requirements, device.memory_properties(), flags);
        let allocate_info = Self::memory_allocate_info(mem_type_ind, memory_requirements.size);
        Ok(unsafe { device.get().allocate_memory(&allocate_info, None)? })
    }

    fn map_memory<T: Copy>(
        device: &VDevice,
        data: &[T],
        memory: DeviceMemory,
        memory_requirements: MemoryRequirements,
    ) -> RendererResult<()> {
        unsafe {
            let ptr = device.get().map_memory(
                memory,
                0,
                memory_requirements.size,
                MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
            device.get().unmap_memory(memory);
        };
        Ok(())
    }

    fn buffer_create_info(size: u64, usage: BufferUsageFlags) -> BufferCreateInfo {
        BufferCreateInfo {
            size,
            usage,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        }
    }

    fn memory_allocate_info(memory_type_index: u32, size: u64) -> MemoryAllocateInfo {
        MemoryAllocateInfo {
            memory_type_index,
            allocation_size: size,
            ..Default::default()
        }
    }

    fn memory_requirements(device: &VDevice, buffer: Buffer) -> MemoryRequirements {
        unsafe { device.get().get_buffer_memory_requirements(buffer) }
    }

    fn find_memory_type_index(
        memory_requirements: MemoryRequirements,
        memory_properties: PhysicalDeviceMemoryProperties,
        flags: MemoryPropertyFlags,
    ) -> u32 {
        for (ind, mem_type) in memory_properties.memory_types.iter().enumerate() {
            if mem_type.property_flags & flags == flags
                && (1 << ind) & memory_requirements.memory_type_bits != 0
            {
                return ind as u32;
            }
        }

        panic!("Failed to find a suitable memory type.");
    }
}

impl_get!(VBuffer, buffer, Buffer);
impl_get!(VBuffer, memory, DeviceMemory);

#[cfg(test)]
mod tests {
    use crate::{
        device::VDevice, instance::VInstance, physical_device::VPhysicalDevice,
        primitives::vertex::Vertex, surface::VSurface, RendererResult,
    };
    use ash::vk::Handle;
    use winit::platform::windows::EventLoopExtWindows;

    use super::VBuffer;

    #[test]
    fn creates_buffers() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            let vertices = vec![Vertex::default(), Vertex::default(), Vertex::default()];
            let indices = vec![1, 2, 3];

            let vertex_buffer = VBuffer::new_vertex_buffer(&device, &vertices)?;
            let index_buffer = VBuffer::new_index_buffer(&device, &indices)?;

            assert_ne!(vertex_buffer.buffer.as_raw(), 0);
            assert_ne!(vertex_buffer.memory.as_raw(), 0);
            assert_ne!(index_buffer.buffer.as_raw(), 0);
            assert_ne!(index_buffer.memory.as_raw(), 0);
        }
        Ok(())
    }
}
