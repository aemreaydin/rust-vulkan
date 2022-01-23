use ash::{
    util::read_spv,
    vk::{ShaderModule, ShaderModuleCreateInfo},
};
use std::fs::File;

use crate::{device::VDevice, RendererResult};

pub struct VShaderUtils;
impl VShaderUtils {
    pub fn load_shader(path: &str) -> RendererResult<Vec<u32>> {
        let mut file = File::open(path)?;
        Ok(read_spv(&mut file)?)
    }

    pub fn create_shader_module(
        device: &VDevice,
        shader_code: &[u32],
    ) -> RendererResult<ShaderModule> {
        let create_info = ShaderModuleCreateInfo {
            code_size: shader_code.len() * 4,
            p_code: shader_code.as_ptr(),
            ..Default::default()
        };
        Ok(unsafe { device.get().create_shader_module(&create_info, None)? })
    }
}
