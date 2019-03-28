//! Graphics context.
//!
//! A graphics context is an object that abstracts all the low-level operations that happen on a
//! graphics device (it can be a GPU or a software implementation, for instance).
//!
//! This crate doesn’t provide you with creating such contexts. Instead, you must do it yourself
//! or rely on crates doing it for you.
//!
//! # On context and threads
//!
//! This crate is designed to work with the following principles:
//!
//!   - An object which type implements `GraphicsContext` must be `!Send` and `!Sync`. This enforces that it
//!     cannot be moved nor shared between threads. Because of `GraphicsState`, it’s very likely it’ll be
//!     `!Send` and `!Sync` automatically.
//!   - You can only create a single context per thread. Doing otherwise is undefined behavior.
//!   - You can create as many contexts as you want as long as they respectively live on a separate thread. In
//!     other terms, if you want `n` contexts, you need `n` threads.
//!
//! That last property might seem to be a drawback to you but is required to remove a lot of
//! dynamic branches in the implementation and reduce the number of required safety
//! checks – enforced at compile time instead.

#[cfg(feature = "std")]
use std::cell::RefCell;
#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(not(feature = "std"))]
use core::cell::RefCell;

use crate::pipeline::Builder;
use crate::state::GraphicsState;

/// Class of graphics context.
///
/// Such a context must not be Send nor Sync, which means that you cannot share it between
/// threads in any way (move / borrow).
pub unsafe trait GraphicsContext {
  /// Get access to the graphics state of this context.
  ///
  /// This must return a `Rc<RefCell<GraphicsState>>` because the state will be shared by OpenGL
  /// objects to ensure consistency with its state.
  fn state(&self) -> &Rc<RefCell<GraphicsState>>;

  /// Create a new pipeline builder.
  ///
  /// A pipeline builder is the only way to create new pipelines and issue draws. Feel free to dig
  /// in the documentation of `Builder` for further details.
  fn pipeline_builder(&self) -> Builder {
    Builder::new(self.state().clone())
  }
}
