use crate::{
    instance::VInstance, queue_family::VQueueFamilyIndices, surface::VSurface, RendererResult,
};
use ash::vk::{
    ColorSpaceKHR, Format, PhysicalDevice, PhysicalDeviceProperties, PhysicalDeviceType,
    PresentModeKHR, QueueFamilyProperties, QueueFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
    SurfaceKHR,
};
use ash::{extensions::khr::Surface, Instance};
use thiserror::Error;

#[derive(Error, Debug)]
enum PhysicalDeviceError {
    #[error("The system has no suitable physical devices.")]
    SuitableDeviceNotFound,
    #[error("Failed to find compatible queue families")]
    IncompatibleQueueFamilies,
}

pub struct VPhysicalDeviceInformation {
    pub properties: PhysicalDeviceProperties,
    // pub features: PhysicalDeviceFeatures,
    pub queue_family_properties: Vec<QueueFamilyProperties>,
    pub surface_capabilities: SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<SurfaceFormatKHR>,
    pub surface_present_modes: Vec<PresentModeKHR>,
}

impl VPhysicalDeviceInformation {
    const OPTIMAL_FORMAT: Format = Format::B8G8R8A8_SRGB;
    const OPTIMAL_COLOR_SPACE: ColorSpaceKHR = ColorSpaceKHR::SRGB_NONLINEAR;

    pub fn generate(
        instance: &Instance,
        surface: &Surface,
        surface_khr: SurfaceKHR,
        physical_device: PhysicalDevice,
    ) -> RendererResult<Self> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let _features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let surface_capabilities = unsafe {
            surface.get_physical_device_surface_capabilities(physical_device, surface_khr)?
        };
        let surface_formats =
            unsafe { surface.get_physical_device_surface_formats(physical_device, surface_khr)? };
        let surface_present_modes = unsafe {
            surface.get_physical_device_surface_present_modes(physical_device, surface_khr)?
        };
        Ok(Self {
            properties,
            // features,
            queue_family_properties,
            surface_capabilities,
            surface_formats,
            surface_present_modes,
        })
    }

    fn rate_device(&self) -> usize {
        let type_score = match self.properties.device_type {
            PhysicalDeviceType::DISCRETE_GPU => 100,
            PhysicalDeviceType::INTEGRATED_GPU => 25,
            _ => 0,
        };

        let format_score = match self.choose_surface_format() {
            SurfaceFormatKHR {
                format: Self::OPTIMAL_FORMAT,
                color_space: Self::OPTIMAL_COLOR_SPACE,
            } => 100,
            _ => 25,
        };

        let present_mode_score = match self.choose_present_mode() {
            PresentModeKHR::MAILBOX => 100,
            PresentModeKHR::FIFO => 25,
            _ => 0,
        };

        type_score + format_score + present_mode_score
    }

    pub fn choose_surface_format(&self) -> SurfaceFormatKHR {
        if let Some(&format) = self.surface_formats.iter().find(|surface_format| {
            surface_format.format == Format::B8G8R8A8_SRGB
                && surface_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        }) {
            return format;
        }

        self.surface_formats[0]
    }

    pub fn choose_present_mode(&self) -> PresentModeKHR {
        if let Some(&mode) = self
            .surface_present_modes
            .iter()
            .find(|&&mode| mode == PresentModeKHR::MAILBOX)
        {
            return mode;
        }

        // This is required to be supported
        PresentModeKHR::FIFO
    }
}

pub struct VPhysicalDevice<'a> {
    instance: &'a VInstance,
    surface: &'a VSurface,
    physical_device: PhysicalDevice,
    queue_family_indices: VQueueFamilyIndices,
    physical_device_information: VPhysicalDeviceInformation,
}

impl<'a> VPhysicalDevice<'a> {
    pub fn new(instance: &'a VInstance, surface: &'a VSurface) -> RendererResult<Self> {
        let physical_device = Self::find_optimal_device(
            instance.instance(),
            surface.surface(),
            surface.surface_khr(),
        )?;

        let physical_device_information = VPhysicalDeviceInformation::generate(
            instance.instance(),
            surface.surface(),
            surface.surface_khr(),
            physical_device,
        )?;

        let queue_family_indices =
            Self::get_queue_family_indices(physical_device, surface, &physical_device_information)?;

        Ok(Self {
            instance,
            surface,
            physical_device,
            queue_family_indices,
            physical_device_information,
        })
    }

    pub fn instance(&self) -> &Instance {
        self.instance.instance()
    }

    pub fn surface(&self) -> &VSurface {
        self.surface
    }

    pub fn surface_khr(&self) -> SurfaceKHR {
        self.surface.surface_khr()
    }

    pub fn physical_device(&self) -> PhysicalDevice {
        self.physical_device
    }

    pub fn queue_family_indices(&self) -> VQueueFamilyIndices {
        self.queue_family_indices
    }

    pub fn physical_device_information(&self) -> &VPhysicalDeviceInformation {
        &self.physical_device_information
    }

    pub fn surface_format(&self) -> SurfaceFormatKHR {
        self.physical_device_information.choose_surface_format()
    }

    pub fn present_mode(&self) -> PresentModeKHR {
        self.physical_device_information.choose_present_mode()
    }

    fn get_queue_family_indices(
        physical_device: PhysicalDevice,
        surface: &VSurface,
        physical_device_information: &VPhysicalDeviceInformation,
    ) -> RendererResult<VQueueFamilyIndices> {
        let present = if let Some(index) = physical_device_information
            .queue_family_properties
            .iter()
            .enumerate()
            .position(|(index, _family)| unsafe {
                match surface.surface().get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    surface.surface_khr(),
                ) {
                    Ok(res) => res,
                    Err(err) => panic!("{}", err),
                }
            }) {
            index as u32
        } else {
            return Err(Box::new(PhysicalDeviceError::IncompatibleQueueFamilies));
        };

        let compute = match physical_device_information
            .queue_family_properties
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::COMPUTE))
        {
            Some(index) => index as u32,
            None => return Err(Box::new(PhysicalDeviceError::IncompatibleQueueFamilies)),
        };

        let graphics = match physical_device_information
            .queue_family_properties
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::GRAPHICS))
        {
            Some(index) => index as u32,
            None => return Err(Box::new(PhysicalDeviceError::IncompatibleQueueFamilies)),
        };

        Ok(VQueueFamilyIndices {
            present,
            compute,
            graphics,
        })
    }

    fn find_optimal_device(
        instance: &Instance,
        surface: &Surface,
        surface_khr: SurfaceKHR,
    ) -> RendererResult<PhysicalDevice> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        if devices.is_empty() {
            return Err(Box::new(PhysicalDeviceError::SuitableDeviceNotFound));
        }

        // TODO Needs testing
        match devices.iter().max_by_key(|&&device| {
            match VPhysicalDeviceInformation::generate(instance, surface, surface_khr, device) {
                Ok(info) => info.rate_device(),
                Err(_) => 0,
            }
        }) {
            Some(&device) => Ok(device),
            None => Err(Box::new(PhysicalDeviceError::SuitableDeviceNotFound)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VPhysicalDevice;
    use crate::{instance::VInstance, surface::VSurface, RendererResult};
    use winit::platform::windows::EventLoopExtWindows;

    #[test]
    fn creates_physical_device() -> RendererResult<()> {
        let instance = VInstance::create("Test", 0)?;

        #[cfg(target_os = "windows")]
        {
            let surface = VSurface::create_surface(
                instance.instance(),
                &EventLoopExtWindows::new_any_thread(),
            )?;
            VPhysicalDevice::new(&instance, &surface)?;
        }

        Ok(())
    }
}
