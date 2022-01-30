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
        Buffer, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
        CommandBufferLevel, CommandBufferUsageFlags, CommandPool, DescriptorSet, DeviceCreateInfo,
        DeviceQueueCreateInfo, DeviceSize, Extent2D, Fence, Framebuffer, IndexType, Offset2D,
        PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineStageFlags, Queue, Rect2D, RenderPass, RenderPassBeginInfo, Semaphore,
        ShaderStageFlags, SubmitInfo, SubpassContents,
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

    pub fn allocate_command_buffers(
        &self,
        command_pool: CommandPool,
        command_buffer_count: u32,
    ) -> RendererResult<Vec<CommandBuffer>> {
        let command_buffer_allocate_info = CommandBufferAllocateInfo {
            command_buffer_count,
            level: CommandBufferLevel::PRIMARY,
            command_pool,
            ..Default::default()
        };

        unsafe {
            Ok(self
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)?)
        }
    }

    pub fn begin_command_buffer(&self, command_buffer: CommandBuffer) -> RendererResult<()> {
        let begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)?
        }
        Ok(())
    }

    pub fn end_command_buffer(&self, command_buffer: CommandBuffer) -> RendererResult<()> {
        unsafe { self.device.end_command_buffer(command_buffer)? };
        Ok(())
    }

    pub fn begin_render_pass(
        &self,
        command_buffer: CommandBuffer,
        framebuffer: Framebuffer,
        clear_values: &[ClearValue],
        extent: Extent2D,
    ) {
        let render_pass_begin_info = RenderPassBeginInfo {
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            render_pass: self.render_pass(),
            framebuffer,
            render_area: Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent,
            },
            ..Default::default()
        };
        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            );
        }
    }

    pub fn bind_pipeline(
        &self,
        command_buffer: CommandBuffer,
        bind_point: PipelineBindPoint,
        pipeline: Pipeline,
    ) {
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, bind_point, pipeline);
        };
    }

    pub fn bind_vertex_buffer(
        &self,
        command_buffer: CommandBuffer,
        buffers: &[Buffer],
        offsets: &[DeviceSize],
    ) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(command_buffer, 0, buffers, offsets);
        }
    }

    pub fn bind_index_buffer(
        &self,
        command_buffer: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
    ) {
        unsafe {
            self.device
                .cmd_bind_index_buffer(command_buffer, buffer, offset, IndexType::UINT32);
        }
    }

    pub fn push_constants(
        &self,
        command_buffer: CommandBuffer,
        layout: PipelineLayout,
        stage_flags: ShaderStageFlags,
        constants: &[u8],
    ) {
        unsafe {
            self.device
                .cmd_push_constants(command_buffer, layout, stage_flags, 0, constants);
        }
    }

    pub fn descriptor_sets(
        &self,
        command_buffer: CommandBuffer,
        pipeline_bind_point: PipelineBindPoint,
        layout: PipelineLayout,
        descriptor_sets: &[DescriptorSet],
    ) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                pipeline_bind_point,
                layout,
                0,
                descriptor_sets,
                &[],
            );
        }
    }

    pub fn draw(&self, command_buffer: CommandBuffer, vertex_count: u32, instance_count: u32) {
        unsafe {
            self.device
                .cmd_draw(command_buffer, vertex_count, instance_count, 0, 0);
        }
    }

    pub fn draw_indexed(
        &self,
        command_buffer: CommandBuffer,
        index_count: u32,
        instance_count: u32,
    ) {
        unsafe {
            self.device
                .cmd_draw_indexed(command_buffer, index_count, instance_count, 0, 0, 0);
        }
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

#[cfg(test)]
mod tests {
    use super::{VDevice, VPhysicalDevice};
    use crate::{
        command_pool::VCommandPool, instance::VInstance, surface::VSurface, RendererResult,
    };
    use ash::vk::{CommandPoolCreateFlags, Handle};
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_device() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;

            // Queues
            assert_ne!(device.queues.compute.as_raw(), 0);
            assert_ne!(device.queues.graphics.as_raw(), 0);
            assert_ne!(device.queues.present.as_raw(), 0);
        }
        Ok(())
    }

    #[test]
    fn creates_command_buffers() -> RendererResult<()> {
        let instance = VInstance::new("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::new(&instance, &EventLoopExtWindows::new_any_thread())?;
            let physical_device = VPhysicalDevice::new(&instance, &surface)?;
            let device = VDevice::new(&instance, &physical_device)?;
            let command_pool = VCommandPool::new(
                &device,
                physical_device.queue_family_indices().graphics,
                CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )?;
            let num_buffers = 3;
            let command_buffers =
                VDevice::allocate_command_buffers(&device, command_pool.get(), num_buffers)?;

            assert_eq!(command_buffers.len(), num_buffers as usize);
            for buffer in command_buffers {
                assert_ne!(buffer.as_raw(), 0);
            }
        }

        Ok(())
    }
}
