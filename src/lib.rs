pub mod device;
pub mod instance;
pub mod physical_device;
pub mod queue_family;
pub mod surface;

pub(crate) type RendererError = Box<dyn std::error::Error>;
pub(crate) type RendererResult<T> = Result<T, RendererError>;
