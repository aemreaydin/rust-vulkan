use crate::{instance::VInstance, surface::VSurface, RendererResult};
use ash::vk::{
    self, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceProperties, PhysicalDeviceType,
    SurfaceKHR,
};
use ash::{extensions::khr::Surface, Instance};
use thiserror::Error;

#[derive(Error, Debug)]
enum PhysicalDeviceError {
    #[error("The system has no suitable physical devices.")]
    SuitableDeviceNotFound,
}

pub struct VPhysicalDevice<'a> {
    instance: &'a VInstance,
    surface: &'a VSurface,
}

impl<'a> VPhysicalDevice<'a> {
    pub fn new(instance: &'a VInstance, surface: &'a VSurface) -> RendererResult<Self> {
        let best_device = Self::choose_best_device(
            &instance.instance(),
            surface.surface(),
            *surface.surface_khr(),
        )?;
        Ok(Self { instance, surface })
    }

    fn choose_best_device(
        instance: &Instance,
        surface: &Surface,
        surface_khr: SurfaceKHR,
    ) -> RendererResult<PhysicalDevice> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        if devices.is_empty() {
            return Err(Box::new(PhysicalDeviceError::SuitableDeviceNotFound));
        }

        let best_device = devices[0];
        let props = unsafe { instance.get_physical_device_properties(best_device) };
        let features = unsafe { instance.get_physical_device_features(best_device) };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(best_device) };
        let surface_capabilities =
            unsafe { surface.get_physical_device_surface_capabilities(best_device, surface_khr)? };
        let surface_formats =
            unsafe { surface.get_physical_device_surface_formats(best_device, surface_khr)? };
        let surface_present_modes =
            unsafe { surface.get_physical_device_surface_present_modes(best_device, surface_khr)? };
        // let surface_support = unsafe { surface.get_physical_device_surface_support(best_device, surface_khr)?};
        println!("{:#?}", props);
        println!("{:#?}", features);
        println!("{:#?}", queue_families);
        println!("{:#?}", surface_capabilities);
        println!("{:#?}", surface_formats);
        println!("{:#?}", surface_present_modes);
        Ok(devices[0])
    }

    fn rate_device(
        device_props: &PhysicalDeviceProperties,
        device_features: &PhysicalDeviceFeatures,
    ) -> f64 {
        let type_score = match device_props.device_type {
            PhysicalDeviceType::DISCRETE_GPU => 100.0,
            PhysicalDeviceType::INTEGRATED_GPU => 25.0,
            _ => 0.0,
        };
        0.0
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
        let surface =
            VSurface::create_surface(&instance.instance(), &EventLoopExtWindows::new_any_thread())?;
        VPhysicalDevice::new(&instance, &surface)?;

        Ok(())
    }
}
