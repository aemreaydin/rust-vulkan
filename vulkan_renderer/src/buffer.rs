use std::mem::{align_of, size_of};

use crate::{device::VDevice, impl_get, RendererResult};
use ash::{
    util::Align,
    vk::{
        Buffer, BufferCreateInfo, BufferUsageFlags, DeviceMemory, MemoryAllocateInfo,
        MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties,
        SharingMode,
    },
};

#[derive(Clone, Copy, Default)]
pub struct VBuffer {
    buffer: Buffer,
    memory: DeviceMemory,
}

impl VBuffer {
    pub fn new<T: Copy>(
        device: &VDevice,
        data: &[T],
        usage: BufferUsageFlags,
    ) -> RendererResult<Self> {
        let create_info = Self::buffer_create_info((data.len() * size_of::<T>()) as u64, usage);
        let buffer = unsafe { device.get().create_buffer(&create_info, None)? };

        let mem_req = Self::memory_requirements(device, buffer);
        let mem_type_ind = Self::find_memory_type_index(
            mem_req,
            device.memory_properties(),
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        );

        let allocate_info = Self::memory_allocate_info(mem_type_ind, mem_req.size);
        let memory = unsafe { device.get().allocate_memory(&allocate_info, None)? };

        let ptr = unsafe {
            device
                .get()
                .map_memory(memory, 0, mem_req.size, MemoryMapFlags::empty())
                .expect("Failed to map memory.")
        };

        let mut align = unsafe { Align::new(ptr, align_of::<T>() as u64, mem_req.size) };
        align.copy_from_slice(data);

        unsafe {
            device.get().unmap_memory(memory);
        }

        unsafe {
            device
                .get()
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind buffer memory.")
        }

        Ok(Self { buffer, memory })
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
