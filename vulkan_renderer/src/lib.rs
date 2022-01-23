pub mod command_pool;
pub mod device;
pub mod enums;
pub mod framebuffer;
pub mod instance;
pub mod physical_device;
pub mod queue_family;
pub mod render_pass;
pub mod surface;
pub mod swapchain;
pub mod sync;

pub(crate) type RendererError = Box<dyn std::error::Error>;
pub(crate) type RendererResult<T> = Result<T, RendererError>;
