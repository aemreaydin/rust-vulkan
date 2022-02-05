use crate::{
    enums::EOperationType,
    instance::VInstance,
    physical_device::VPhysicalDevice,
    queue_family::{VQueueFamilyIndices, VQueues},
    render_pass::VRenderPass,
    RendererResult,
};
use ash::{
    extensions::khr::Swapchain,
    vk::{
        CommandBuffer, DeviceCreateInfo, DeviceQueueCreateInfo, Fence,
        PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, PipelineStageFlags, Queue,
        RenderPass, Semaphore, SubmitInfo,
    },
    Device,
};
use std::collections::HashSet;

/// Keeps tracks of the logical device, queues, command_pools and the render_pass
pub struct VDevice {
    device: Device,
    queues: VQueues,
    queue_family_indices: VQueueFamilyIndices,
    render_pass: VRenderPass,
    memory_properties: PhysicalDeviceMemoryProperties,
    physical_device_properties: PhysicalDeviceProperties,
}

impl VDevice {
    pub fn new(instance: &VInstance, physical_device: &VPhysicalDevice) -> RendererResult<Self> {
        let queue_infos = Self::device_queue_create_infos(physical_device.queue_family_indices());
        let extensions = [Swapchain::name().as_ptr()];
        let device_create_info = Self::device_create_info(&queue_infos, &extensions);
        let device = unsafe {
            instance
                .get()
                .create_device(physical_device.get(), &device_create_info, None)?
        };

        let queues = VQueues::new(&device, physical_device.queue_family_indices());
        let render_pass = VRenderPass::new(
            &device,
            physical_device
                .physical_device_information()
                .choose_surface_format()
                .format,
        )?;

        Ok(Self {
            device,
            queues,
            queue_family_indices: physical_device.queue_family_indices(),
            render_pass,
            memory_properties: physical_device
                .physical_device_information()
                .memory_properties,
            physical_device_properties: physical_device.physical_device_information().properties,
        })
    }

    pub fn get(&self) -> &Device {
        &self.device
    }

    pub fn get_queue(&self, operation_type: EOperationType) -> Queue {
        self.queues.get(operation_type)
    }

    pub fn get_queue_family_index(&self, operation_type: EOperationType) -> u32 {
        self.queue_family_indices.get(operation_type)
    }

    pub fn render_pass(&self) -> RenderPass {
        self.render_pass.get()
    }

    pub fn memory_properties(&self) -> PhysicalDeviceMemoryProperties {
        self.memory_properties
    }

    pub fn physical_device_properties(&self) -> PhysicalDeviceProperties {
        self.physical_device_properties
    }

    pub fn create_queue_submit_info(
        command_buffers: &[CommandBuffer],
        wait_semaphores: &[Semaphore],
        dst_semaphores: &[Semaphore],
        pipeline_stage_flags: &[PipelineStageFlags],
    ) -> SubmitInfo {
        SubmitInfo {
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            signal_semaphore_count: dst_semaphores.len() as u32,
            p_signal_semaphores: dst_semaphores.as_ptr(),
            p_wait_dst_stage_mask: pipeline_stage_flags.as_ptr(),
            ..Default::default()
        }
    }

    pub fn queue_submit(
        &self,
        queue: Queue,
        submits: &[SubmitInfo],
        fence: Fence,
    ) -> RendererResult<()> {
        unsafe { self.device.queue_submit(queue, submits, fence)? }
        Ok(())
    }

    pub fn end_render_pass(&self, command_buffer: CommandBuffer) {
        unsafe { self.device.cmd_end_render_pass(command_buffer) }
    }

    pub fn wait_for_fences(&self, fences: &[Fence], timeout: u64) -> RendererResult<()> {
        unsafe { self.device.wait_for_fences(fences, true, timeout)? }
        Ok(())
    }

    pub fn reset_fences(&self, fences: &[Fence]) -> RendererResult<()> {
        unsafe { self.device.reset_fences(fences)? }
        Ok(())
    }

    fn device_create_info(
        queue_infos: &[DeviceQueueCreateInfo],
        extensions: &[*const i8],
    ) -> DeviceCreateInfo {
        DeviceCreateInfo {
            queue_create_info_count: queue_infos.len() as u32,
            p_queue_create_infos: queue_infos.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        }
    }

    // This makes no sense probably
    fn device_queue_create_infos(
        queue_family_indices: VQueueFamilyIndices,
    ) -> Vec<DeviceQueueCreateInfo> {
        let unique_indices = HashSet::<u32>::from_iter([
            queue_family_indices.compute,
            queue_family_indices.graphics,
            queue_family_indices.present,
        ]);
        unique_indices
            .iter()
            .map(|&queue_family_index| DeviceQueueCreateInfo {
                p_queue_priorities: [1.0].as_ptr(),
                queue_family_index,
                queue_count: 1,
                ..Default::default()
            })
            .collect()
    }

    #[allow(dead_code)]
    fn get_device_extensions(
        instance: &VInstance,
        physical_device: &VPhysicalDevice,
    ) -> RendererResult<()> {
        let extension_props = unsafe {
            instance
                .get()
                .enumerate_device_extension_properties(physical_device.get())?
        };
        println!("{:#?}", extension_props);
        Ok(())
    }
}
