use crate::RendererResult;
use ash::{vk, Instance};
use std::ffi::{CStr, CString};

pub struct VInstance {
    instance: Instance,
}

impl VInstance {
    pub fn create(name: &str, version: u32) -> RendererResult<Self> {
        let entry = ash::Entry::linked();

        let p_application_info = &Self::application_info(name, version);
        let enabled_layers = Self::layers();
        let enabled_extensions = Self::extensions();
        let create_info = vk::InstanceCreateInfo {
            p_application_info,
            enabled_layer_count: enabled_layers.len() as u32,
            pp_enabled_layer_names: enabled_layers.as_ptr(),
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(Self { instance })
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    fn application_info(name: &str, application_version: u32) -> vk::ApplicationInfo {
        let p_application_name = CString::new(name).expect("ApplicationInfo CString Error.");
        let p_application_name = p_application_name.as_ptr();
        vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_2,
            p_application_name,
            application_version,
            ..Default::default()
        }
    }

    fn layers() -> Vec<*const i8> {
        let layers = vec!["VK_LAYER_LUNARG_monitor\0", "VK_LAYER_KHRONOS_validation\0"];

        layers
            .iter()
            .filter_map(|ext| CStr::from_bytes_with_nul(ext.as_bytes()).ok())
            .map(|s| s.as_ptr())
            .collect()
    }

    fn extensions() -> Vec<*const i8> {
        #[cfg(target_os = "windows")]
        let extensions = vec![
            ash::extensions::khr::Surface::name(),
            ash::extensions::khr::Win32Surface::name(),
            ash::vk::ExtDebugUtilsFn::name(),
        ];

        extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect()
    }
}

#[derive(Default, Debug)]
pub struct VInstanceBuilder {
    layers: Vec<*const i8>,
    extensions: Vec<*const i8>,
    application_info: vk::ApplicationInfo,
    allocation_callbacks: Option<vk::AllocationCallbacks>,
}

impl VInstanceBuilder {
    pub fn start() -> Self {
        Self::default()
    }

    pub fn layers(mut self, layers: Vec<&str>) -> Self {
        self.layers = layers
            .iter()
            .filter_map(|ext| CStr::from_bytes_with_nul(ext.as_bytes()).ok())
            .map(|layer| layer.as_ptr())
            .collect::<Vec<_>>();
        self
    }

    pub fn extensions(mut self, extensions: Vec<&str>) -> Self {
        self.extensions = extensions
            .iter()
            .filter_map(|ext| CStr::from_bytes_with_nul(ext.as_bytes()).ok())
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        self
    }

    pub fn application_info(mut self, application_info: vk::ApplicationInfo) -> Self {
        self.application_info = application_info;
        self
    }

    pub fn allocation_callbacks(
        mut self,
        allocation_callbacks: Option<vk::AllocationCallbacks>,
    ) -> Self {
        self.allocation_callbacks = allocation_callbacks;
        self
    }

    pub fn create_instance(self) -> RendererResult<VInstance> {
        let entry = ash::Entry::linked();
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &self.application_info,
            enabled_extension_count: self.extensions.len() as u32,
            enabled_layer_count: self.layers.len() as u32,
            pp_enabled_extension_names: self.extensions.as_ptr(),
            pp_enabled_layer_names: self.layers.as_ptr(),
            ..Default::default()
        };

        let instance =
            unsafe { entry.create_instance(&create_info, self.allocation_callbacks.as_ref())? };
        Ok(VInstance { instance })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_instance() -> RendererResult<()> {
        VInstance::create("Test", 0)?;
        Ok(())
    }

    #[test]
    fn creates_instance_with_builder() -> RendererResult<()> {
        let application_info = VInstance::application_info("Test", 0);
        let layers = vec!["VK_LAYER_LUNARG_monitor\0", "VK_LAYER_KHRONOS_validation\0"];
        let extensions = vec![
            "VK_KHR_surface\0",
            "VK_KHR_win32_surface\0",
            "VK_EXT_debug_utils\0",
        ];

        let builder = VInstanceBuilder::start()
            .application_info(application_info)
            .layers(layers)
            .extensions(extensions);

        assert!(builder.layers.len() == 2);
        assert!(builder.extensions.len() == 3);
        builder.create_instance()?;
        Ok(())
    }
}
