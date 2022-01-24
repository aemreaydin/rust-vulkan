pub mod buffer;
pub mod command_pool;
pub mod device;
pub mod enums;
pub mod framebuffer;
pub mod instance;
pub mod physical_device;
pub mod pipeline;
pub mod primitives;
pub mod queue_family;
pub mod render_pass;
pub mod shader_utils;
pub mod surface;
pub mod swapchain;
pub mod sync;

pub use nalgebra_glm as glm;
pub(crate) type RendererError = Box<dyn std::error::Error>;
pub(crate) type RendererResult<T> = Result<T, RendererError>;
