use ash::vk::{CommandBuffer, CommandPoolCreateFlags, Extent2D, Format};
use vulkan_renderer::{
    command_pool::VCommandPool, device::VDevice, enums::EOperationType, instance::VInstance,
    pipeline::VGraphicsPipeline, swapchain::VSwapchain,
};

pub struct App {
    pub instance: VInstance,
    pub device: VDevice,
    pub swapchain: VSwapchain,
    pub command_pool: VCommandPool,
    pub pipeline: VGraphicsPipeline,
    pub commandbuffers: Vec<CommandBuffer>,

    pub extent: Extent2D,
    pub color_format: Format,
}

impl App {
    pub fn init(
        instance: VInstance,
        device: VDevice,
        swapchain: VSwapchain,
        extent: Extent2D,
    ) -> Self {
        Self {
            instance,
            device,
            swapchain,
            pipeline: VGraphicsPipeline::default(),
            command_pool: VCommandPool::default(),
            commandbuffers: Vec::default(),

            extent,
            color_format: Format::B8G8R8A8_SRGB,
        }
    }

    pub fn create_command_pool(&mut self, flags: CommandPoolCreateFlags) {
        self.command_pool = VCommandPool::new(
            &self.device,
            self.device.get_queue_family_index(EOperationType::Graphics),
            flags,
        )
        .expect("Failed to create command pool.");
    }

    pub fn create_graphics_pipeline(&mut self, pipeline: VGraphicsPipeline) {
        self.pipeline = pipeline;
    }

    #[allow(dead_code)]
    pub fn find_optimal_surface_format(&mut self) {
        // self.device.get_surface_capabilities().
    }
}
