//! Shader-related modules.
//!
//! *Shaders* are a common shortcut to *shader programs*. Those are piece of code running on your
//! GPU. Everything you need to know can be found in the `program` module. The `stage` modules
//! contains types and functions that you’ll be linked from the `program` module.

pub mod program;
pub mod stage;
