//! # luminance windowing
//!
//! This is the base, abstract crate for windowing common types and functions in luminance. The
//! `luminance` crate provides you with abstracting over OpenGL, but it doesn’t give you a way to
//! create an OpenGL context. This is due to the fact that creating and managing OpenGL contexts is
//! tightly related to the type of application you target. Typical PC applications might need
//! something like **GLFW** or **glutin** whilst others will directly bind to **X11** or **Windows
//! API**. Several crates – `luminance-*` – exist to solve that problem. They all provide a
//! different implementation for a simple need: opening an OpenGL context, opening a window and
//! manage events. In theory, you could even have a `luminance-gtk` or `luminance-qt` to embed
//! `luminance` in surfaces in those libraries.
//!
//! # What’s included
//!
//! This crate exposes several important types that all backends must use. Among them, you’ll find:
//!
//! - `WindowDim`: abstraction over the dimension of a window and its mode (windowed, fullscreen, fullscreen
//!   restricted).
//! - `WindowOpt`: an opaque type giving access to hints to customize the window integration, such as whether
//!   the cursor should be hidden or not.
//!
//! The `Device` trait must be implemented by a backend so that an application is completely
//! agnostic of the backend. This trait defines several basic methods that will help you to:
//!
//! - Retrieve the dimension of the window / framebuffer.
//! - Iterate over the system events captured by your application.
//! - Draw and swap the buffer chain.

extern crate luminance;

use luminance::context::GraphicsContext;

/// Dimension metrics.
///
///   - `Windowed(width, height)` opens in windowed mode with the wished resolution.
///   - `Fullscreen` opens in fullscreen mode by using the primary monitor resolution.
///   - `FullscreenRestricted(width, height)` is a mix between `Windowed(width, height)` and `Fullscreen`. It
///     opens in fullscreen mode by using the wished resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowDim {
  /// Windowed mode.
  Windowed(u32, u32),
  /// Fullscreen mode (adapt to your screen).
  Fullscreen,
  /// Fullscreen mode with restricted viewport dimension.
  FullscreenRestricted(u32, u32),
}

/// Different window options.
///
/// Feel free to look at the different methods available to tweak the options. You may want to start
/// with `default()` though.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowOpt {
  hide_cursor: bool,
  num_samples: Option<u32>,
}

impl Default for WindowOpt {
  /// Defaults:
  ///
  /// - `hide_cursor` set to `false`.
  /// - `num_samples` set to `None`.
  fn default() -> Self {
    WindowOpt {
      hide_cursor: false,
      num_samples: None,
    }
  }
}

impl WindowOpt {
  /// Hide or unhide the cursor. Default to `false`.
  #[inline]
  pub fn hide_cursor(self, hide: bool) -> Self {
    WindowOpt {
      hide_cursor: hide,
      ..self
    }
  }

  /// Check whether the cursor is hidden.
  #[inline]
  pub fn is_cursor_hidden(&self) -> bool {
    self.hide_cursor
  }

  /// Set the number of samples to use for multisampling.
  ///
  /// Pass `None` to disable multisampling.
  #[inline]
  pub fn set_num_samples<S>(self, samples: S) -> Self where S: Into<Option<u32>> {
    WindowOpt {
      num_samples: samples.into(),
      ..self
    }
  }

  /// Get the number of samples to use in multisampling, if any.
  #[inline]
  pub fn num_samples(&self) -> Option<u32> {
    self.num_samples
  }
}

/// Rendering surface.
///
/// This type holds anything related to rendering. The interface is straight forward, so feel
/// free to have a look around.
pub trait Surface: GraphicsContext + Sized {
  /// Type of events.
  type Event;

  /// Type of surface errors.
  type Error;

  /// Create a surface along with its associated event stream and bootstrap a luminance environment
  /// that it lives as long as the surface lives.
  fn new(dim: WindowDim, title: &str, win_opt: WindowOpt) -> Result<Self, Self::Error>;

  /// Size of the surface’s framebuffer.
  fn size(&self) -> [u32; 2];

  /// Width of the surface’s framebuffer.
  ///
  /// # Defaults
  ///
  /// Defaults to `.size()[0]`.
  fn width(&self) -> u32 {
    self.size()[0]
  }

  /// Height of the surface’s framebuffer.
  ///
  /// # Defaults
  ///
  /// Defaults to `.size()[1]`.
  fn height(&self) -> u32 {
    self.size()[1]
  }

  // FIXME: existential impl trait
  /// Get an iterator over events by blocking until the first event happens.
  fn wait_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a>;

  // FIXME: existential impl trait
  /// Get an iterator over events without blocking if no event is there.
  fn poll_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a>;

  /// Swap the back and front buffers.
  fn swap_buffers(&mut self);
}
