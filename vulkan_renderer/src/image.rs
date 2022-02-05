use crate::{device::VDevice, impl_get, RendererResult};
use ash::vk::{
    DeviceMemory, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo,
    ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryPropertyFlags, MemoryRequirements,
    PhysicalDeviceMemoryProperties, SampleCountFlags, SharingMode,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct VImage {
    image: Image,
    image_view: ImageView,
    memory: DeviceMemory,
}

impl VImage {
    pub fn new(
        device: &VDevice,
        usage: ImageUsageFlags,
        format: Format,
        extent: Extent3D,
        aspect_mask: ImageAspectFlags,
    ) -> RendererResult<Self> {
        let create_info = Self::image_create_info(usage, ImageType::TYPE_2D, format, extent);
        let image = unsafe { device.get().create_image(&create_info, None)? };

        // Device Memory
        let mem_req = Self::memory_requirements(device, image);
        let mem_type_ind = Self::find_memory_type_index(
            mem_req,
            device.get_memory_properties(),
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let allocate_info = Self::memory_allocate_info(mem_type_ind, mem_req.size);
        let memory = unsafe { device.get().allocate_memory(&allocate_info, None)? };

        unsafe {
            device
                .get()
                .bind_image_memory(image, memory, 0)
                .expect("Failed to bind buffer memory.")
        }

        // ImageView
        let create_info =
            Self::image_view_create_info(image, ImageViewType::TYPE_2D, format, aspect_mask);
        let image_view = unsafe { device.get().create_image_view(&create_info, None)? };

        Ok(Self {
            image,
            image_view,
            memory,
        })
    }

    fn image_create_info(
        usage: ImageUsageFlags,
        image_type: ImageType,
        format: Format,
        extent: Extent3D,
    ) -> ImageCreateInfo {
        ImageCreateInfo {
            usage,
            sharing_mode: SharingMode::EXCLUSIVE,
            image_type,
            format,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            ..Default::default()
        }
    }

    fn image_view_create_info(
        image: Image,
        view_type: ImageViewType,
        format: Format,
        aspect_mask: ImageAspectFlags,
    ) -> ImageViewCreateInfo {
        ImageViewCreateInfo {
            image,
            format,
            view_type,
            subresource_range: ImageSubresourceRange {
                base_array_layer: 0,
                base_mip_level: 0,
                layer_count: 1,
                level_count: 1,
                aspect_mask,
            },
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

    fn memory_requirements(device: &VDevice, image: Image) -> MemoryRequirements {
        unsafe { device.get().get_image_memory_requirements(image) }
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

impl_get!(VImage, image, Image);
impl_get!(VImage, image_view, ImageView);
impl_get!(VImage, memory, DeviceMemory);
