use crate::{
    cmd::*, command_pool::VCommandPool, device::VDevice, enums::EOperationType, impl_get,
    RendererResult,
};
use ash::vk::{
    Buffer, BufferCopy, BufferCreateInfo, BufferUsageFlags, CommandBufferBeginInfo,
    CommandBufferUsageFlags, CommandPoolCreateFlags, DeviceMemory, Fence, MemoryAllocateInfo,
    MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties,
    SharingMode, SubmitInfo,
};
use std::mem::size_of;

#[derive(Default, Debug, Clone, Copy)]
pub struct VBuffer {
    buffer: Buffer,
    memory: DeviceMemory,
    allocation: u64,
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
        let buffer = Self::create_buffer(device, (data.len() * size_of::<T>()) as u64, usage)?;
        let memory_requirements = Self::memory_requirements(device, buffer);
        let memory = Self::create_memory(device, memory_requirements, flags)?;
        unsafe { device.get().bind_buffer_memory(buffer, memory, 0)? };

        let vbuffer = Self {
            buffer,
            memory,
            allocation: memory_requirements.size,
        };
        vbuffer.map_memory(device, data)?;

        Ok(vbuffer)
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
        let buffer = Self::create_buffer(device, (data.len() * size_of::<T>()) as u64, usage)?;
        let memory_requirements = Self::memory_requirements(device, buffer);
        let memory = Self::create_memory(device, memory_requirements, flags)?;
        unsafe { device.get().bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self {
            buffer,
            memory,
            allocation: memory_requirements.size,
        })
    }

    pub fn new_uniform_buffer(
        device: &VDevice,
        size: u64,
        flags: MemoryPropertyFlags,
    ) -> RendererResult<Self> {
        let buffer = Self::create_buffer(device, size, BufferUsageFlags::UNIFORM_BUFFER)?;
        let memory_requirements = Self::memory_requirements(device, buffer);
        let memory = Self::create_memory(device, memory_requirements, flags)?;
        unsafe { device.get().bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self {
            buffer,
            memory,
            allocation: memory_requirements.size,
        })
    }

    pub fn new_device_local_buffer<T: Copy>(
        device: &VDevice,
        data: &[T],
        dst_usage: BufferUsageFlags,
    ) -> RendererResult<Self> {
        let staging_buffer = Self::new_mapped(
            device,
            data,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let vertex_buffer = Self::new_unmapped(
            device,
            data,
            BufferUsageFlags::TRANSFER_DST | dst_usage,
            MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(device, data, staging_buffer.buffer, vertex_buffer.buffer)?;

        Ok(vertex_buffer)
    }

    pub fn create_buffer(
        device: &VDevice,
        size: u64,
        usage: BufferUsageFlags,
    ) -> RendererResult<Buffer> {
        let create_info = Self::buffer_create_info(size, usage);
        unsafe { Ok(device.get().create_buffer(&create_info, None)?) }
    }

    pub fn create_memory(
        device: &VDevice,
        memory_requirements: MemoryRequirements,
        flags: MemoryPropertyFlags,
    ) -> RendererResult<DeviceMemory> {
        let mem_type_ind = Self::find_memory_type_index(
            memory_requirements,
            device.get_memory_properties(),
            flags,
        );
        let allocate_info = Self::memory_allocate_info(mem_type_ind, memory_requirements.size);
        Ok(unsafe { device.get().allocate_memory(&allocate_info, None)? })
    }

    pub fn copy_buffer<T>(
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
        let command_buffer = allocate_command_buffers(device, command_pool.get(), 1)?[0];

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

    pub fn map_memory<T: Copy>(&self, device: &VDevice, data: &[T]) -> RendererResult<()> {
        unsafe {
            let ptr = device.get().map_memory(
                self.memory,
                0,
                self.allocation,
                MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
            device.get().unmap_memory(self.memory);
        };
        Ok(())
    }

    pub fn map_padded_memory<T: Copy>(
        &self,
        device: &VDevice,
        data: &[T],
        pad_offset: isize,
    ) -> RendererResult<()> {
        unsafe {
            let ptr = device.get().map_memory(
                self.memory,
                0,
                self.allocation,
                MemoryMapFlags::empty(),
            )?;
            let ptr = ptr.offset(pad_offset);
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
            device.get().unmap_memory(self.memory);
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
impl_get!(VBuffer, allocation, u64);
