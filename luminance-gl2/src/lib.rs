#[macro_use]
mod log;

#[cfg(feature = "GL33")]
pub mod gl33;

#[cfg(feature = "GL33")]
pub use gl33::GL33;
