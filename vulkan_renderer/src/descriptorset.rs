use ash::vk::{
    DescriptorBufferInfo, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize,
    DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, ShaderStageFlags, WriteDescriptorSet,
};

use crate::{device::VDevice, RendererResult};

pub struct VDescriptorPool {
    descriptor_pool: DescriptorPool,
}

impl VDescriptorPool {
    pub fn new(device: &VDevice) -> RendererResult<Self> {
        let pool_sizes = &[
            DescriptorPoolSize {
                descriptor_count: 10,
                ty: DescriptorType::UNIFORM_BUFFER,
            },
            DescriptorPoolSize {
                descriptor_count: 10,
                ty: DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            },
        ];
        let create_info = Self::create_info(pool_sizes);
        let descriptor_pool = unsafe { device.get().create_descriptor_pool(&create_info, None)? };
        Ok(Self { descriptor_pool })
    }

    pub fn get(&self) -> DescriptorPool {
        self.descriptor_pool
    }

    fn create_info(pool_sizes: &[DescriptorPoolSize]) -> DescriptorPoolCreateInfo {
        DescriptorPoolCreateInfo {
            max_sets: 10,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        }
    }
}

pub struct VDescriptorSetLayout {
    descriptor_set_layout: DescriptorSetLayout,
}

impl VDescriptorSetLayout {
    pub fn new(device: &VDevice, bindings: &[DescriptorSetLayoutBinding]) -> RendererResult<Self> {
        let create_info = Self::create_info(bindings);
        let descriptor_set_layout = unsafe {
            device
                .get()
                .create_descriptor_set_layout(&create_info, None)?
        };
        Ok(Self {
            descriptor_set_layout,
        })
    }

    pub fn get(&self) -> DescriptorSetLayout {
        self.descriptor_set_layout
    }

    pub fn layout_binding(
        binding: u32,
        count: u32,
        ty: DescriptorType,
        stage: ShaderStageFlags,
    ) -> DescriptorSetLayoutBinding {
        DescriptorSetLayoutBinding {
            binding,
            descriptor_count: count,
            descriptor_type: ty,
            stage_flags: stage,
            ..Default::default()
        }
    }

    fn create_info(bindings: &[DescriptorSetLayoutBinding]) -> DescriptorSetLayoutCreateInfo {
        DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        }
    }
}

pub struct VDescriptorSet {
    descriptor_set: DescriptorSet,
}

impl VDescriptorSet {
    pub fn new(
        device: &VDevice,
        descriptor_pool: DescriptorPool,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> RendererResult<Self> {
        let create_info = Self::allocate_info(descriptor_pool, descriptor_set_layouts);
        let descriptor_set = unsafe { device.get().allocate_descriptor_sets(&create_info)?[0] };
        Ok(Self { descriptor_set })
    }

    pub fn get(&self) -> DescriptorSet {
        self.descriptor_set
    }

    fn allocate_info(
        descriptor_pool: DescriptorPool,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> DescriptorSetAllocateInfo {
        DescriptorSetAllocateInfo {
            descriptor_pool,
            descriptor_set_count: descriptor_set_layouts.len() as u32,
            p_set_layouts: descriptor_set_layouts.as_ptr(),
            ..Default::default()
        }
    }

    pub fn write_descriptor_set(
        dst_set: DescriptorSet,
        binding: u32,
        descriptor_type: DescriptorType,
        buffer_info: &DescriptorBufferInfo,
    ) -> WriteDescriptorSet {
        WriteDescriptorSet {
            p_buffer_info: buffer_info,
            dst_set,
            dst_binding: binding,
            descriptor_type,
            descriptor_count: 1,
            ..Default::default()
        }
    }
}
