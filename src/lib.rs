pub mod instance;

pub(crate) type RendererError = Box<dyn std::error::Error>;
pub(crate) type RendererResult<T> = Result<T, RendererError>;
