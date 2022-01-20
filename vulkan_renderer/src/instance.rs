use crate::RendererResult;
use ash::{
    extensions::ext::DebugUtils,
    vk::{self, DebugUtilsMessengerEXT},
    Entry, Instance,
};
use colored::*;
use std::{
    borrow::Cow,
    ffi::{c_void, CStr, CString},
};

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;

    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]".white(),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]".green(),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]".yellow(),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]".red(),
        _ => "[Unknown]".white(),
    };

    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]".bright_blue(),
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]".red(),
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]".yellow(),
        _ => "[Unknown]".white(),
    };

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    }
    .cyan();

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    }
    .bright_black();

    println!("{}{}: [{}] : {}", severity, types, message_id_name, message,);

    vk::FALSE
}

#[cfg(debug_assertions)]
const IS_VALIDATION_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
const IS_VALIDATION_ENABLED: bool = false;

pub struct VInstance {
    instance: Instance,
    _debug_utils: Option<DebugUtils>,
    _debug_callback: Option<vk::DebugUtilsMessengerEXT>,
}

impl VInstance {
    pub fn new(name: &str, version: u32) -> RendererResult<Self> {
        let entry = Entry::linked();

        let application_info = Self::application_info(name, version);
        let layers = Self::layers();
        let extensions = Self::extensions();
        let create_info = Self::create_info(&application_info, &layers, &extensions);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        let (debug_utils, debug_callback) =
            Self::create_debug_utils_and_callback(&entry, &instance)?;

        Ok(Self {
            instance,
            _debug_utils: debug_utils,
            _debug_callback: debug_callback,
        })
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

    fn debug_utils_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build()
    }

    fn create_info(
        application_info: &vk::ApplicationInfo,
        layers: &[*const i8],
        extensions: &[*const i8],
    ) -> vk::InstanceCreateInfo {
        let mut p_next = std::ptr::null();
        if IS_VALIDATION_ENABLED {
            p_next = &Self::debug_utils_create_info() as *const vk::DebugUtilsMessengerCreateInfoEXT
                as *const c_void;
        }
        vk::InstanceCreateInfo {
            p_next,
            p_application_info: application_info,
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
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
        let mut extensions = vec![
            ash::extensions::khr::Surface::name(),
            #[cfg(target_os = "windows")]
            ash::extensions::khr::Win32Surface::name(),
        ];
        if IS_VALIDATION_ENABLED {
            extensions.push(ash::vk::ExtDebugUtilsFn::name());
        }

        extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect()
    }

    fn debug_callback(
        debug_utils: &DebugUtils,
    ) -> RendererResult<Option<vk::DebugUtilsMessengerEXT>> {
        let debug_info = Self::debug_utils_create_info();
        unsafe {
            Ok(Some(
                debug_utils.create_debug_utils_messenger(&debug_info, None)?,
            ))
        }
    }

    fn create_debug_utils_and_callback(
        entry: &Entry,
        instance: &Instance,
    ) -> RendererResult<(Option<DebugUtils>, Option<DebugUtilsMessengerEXT>)> {
        let mut debug_utils = None;
        let mut debug_callback = None;
        if IS_VALIDATION_ENABLED {
            debug_utils = Some(DebugUtils::new(entry, instance));
            debug_callback = Self::debug_callback(debug_utils.as_ref().unwrap())?;
        }
        Ok((debug_utils, debug_callback))
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
        let entry = Entry::linked();
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
        let (debug_utils, debug_callback) =
            VInstance::create_debug_utils_and_callback(&entry, &instance)?;

        Ok(VInstance {
            instance,
            _debug_utils: debug_utils,
            _debug_callback: debug_callback,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_instance() -> RendererResult<()> {
        VInstance::new("Test", 1)?;
        Ok(())
    }

    #[test]
    fn builder_creates_instance() -> RendererResult<()> {
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

        builder.create_instance()?;
        Ok(())
    }
}
