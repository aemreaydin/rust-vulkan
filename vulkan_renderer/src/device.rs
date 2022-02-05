use crate::{
    enums::EOperationType,
    instance::VInstance,
    queue_family::{VQueueFamilyIndices, VQueues},
    RendererResult,
};
use ash::{
    extensions::khr::{Surface, Swapchain},
    vk::{
        CommandBuffer, DeviceCreateInfo, DeviceQueueCreateInfo, Fence, PhysicalDevice,
        PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, PipelineStageFlags, Queue,
        QueueFlags, Semaphore, SubmitInfo, SurfaceCapabilitiesKHR, SurfaceKHR,
    },
    Device, Instance,
};
use winit::window::Window;

/// Keeps tracks of the logical device, queues, command_pools and the render_pass
pub struct VDevice {
    device: Device,

    // Surface
    surface_khr: SurfaceKHR,
    surface_capabilities: SurfaceCapabilitiesKHR,

    // Physical Device
    physical_device: PhysicalDevice,
    memory_properties: PhysicalDeviceMemoryProperties,
    device_properties: PhysicalDeviceProperties,

    // Queue
    queues: VQueues,
    queue_family_indices: VQueueFamilyIndices,
}

impl VDevice {
    pub fn new(instance: &VInstance, window: &Window) -> RendererResult<Self> {
        // Physical Device
        let physical_device = instance.select_physical_device()?;
        let memory_properties = unsafe {
            instance
                .get()
                .get_physical_device_memory_properties(physical_device)
        };
        let device_properties = unsafe {
            instance
                .get()
                .get_physical_device_properties(physical_device)
        };

        // Surface
        let entry = ash::Entry::linked();
        let surface = Surface::new(&entry, instance.get());
        let surface_khr =
            unsafe { ash_window::create_surface(&entry, instance.get(), &window, None)? };
        let surface_capabilities = unsafe {
            surface.get_physical_device_surface_capabilities(physical_device, surface_khr)?
        };

        // Queue
        let queue_family_indices = Self::select_queue_family_indices(
            instance.get(),
            physical_device,
            &surface,
            surface_khr,
        );

        let queue_create_infos = Self::device_queue_create_infos(queue_family_indices);
        let extensions = [Swapchain::name().as_ptr()];
        let device_create_info = Self::device_create_info(&queue_create_infos, &extensions);
        let device = unsafe {
            instance
                .get()
                .create_device(physical_device, &device_create_info, None)?
        };

        let queues = VQueues::new(&device, queue_family_indices);

        Ok(Self {
            device,
            physical_device,
            memory_properties,
            device_properties,
            queue_family_indices,
            queues,
            surface_khr,
            surface_capabilities,
        })
    }

    pub fn get(&self) -> &Device {
        &self.device
    }

    pub fn get_physical_device(&self) -> PhysicalDevice {
        self.physical_device
    }

    pub fn get_surface_khr(&self) -> SurfaceKHR {
        self.surface_khr
    }

    pub fn get_queue(&self, operation_type: EOperationType) -> Queue {
        self.queues.get(operation_type)
    }

    pub fn get_queue_family_index(&self, operation_type: EOperationType) -> u32 {
        self.queue_family_indices.get(operation_type)
    }

    pub fn get_memory_properties(&self) -> PhysicalDeviceMemoryProperties {
        self.memory_properties
    }

    pub fn get_device_properties(&self) -> PhysicalDeviceProperties {
        self.device_properties
    }

    pub fn get_surface_capabilities(&self) -> SurfaceCapabilitiesKHR {
        self.surface_capabilities
    }

    fn select_queue_family_indices(
        instance: &Instance,
        physical_device: PhysicalDevice,
        surface: &Surface,
        surface_khr: SurfaceKHR,
    ) -> VQueueFamilyIndices {
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = VQueueFamilyIndices::default();
        for (ind, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                queue_family_indices.graphics = ind as u32;

                if let Ok(supports_present) = unsafe {
                    surface.get_physical_device_surface_support(
                        physical_device,
                        ind as u32,
                        surface_khr,
                    )
                } {
                    if supports_present {
                        queue_family_indices.present = ind as u32;
                        break;
                    }
                };
            }
        }

        if queue_family_indices.present == u32::MAX {
            for (ind, _) in queue_family_properties.iter().enumerate() {
                if let Ok(supports_present) = unsafe {
                    surface.get_physical_device_surface_support(
                        physical_device,
                        ind as u32,
                        surface_khr,
                    )
                } {
                    if supports_present {
                        queue_family_indices.present = ind as u32;
                        break;
                    }
                };
            }
        }

        for (ind, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE) {
                if queue_family_indices.compute == u32::MAX {
                    queue_family_indices.compute = ind as u32;
                }
                if ind as u32 != queue_family_indices.graphics {
                    queue_family_indices.compute = ind as u32;
                    break;
                }
            }
        }
        queue_family_indices
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
        let unique_indices =
            Vec::from_iter([queue_family_indices.compute, queue_family_indices.graphics]);
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
        physical_device: PhysicalDevice,
    ) -> RendererResult<()> {
        let extension_props = unsafe {
            instance
                .get()
                .enumerate_device_extension_properties(physical_device)?
        };
        println!("{:#?}", extension_props);
        Ok(())
    }
}
