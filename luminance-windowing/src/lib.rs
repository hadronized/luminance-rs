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

#![deny(missing_docs)]

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::texture::{Dim2, Flat};

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

/// Cursor mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorMode {
  /// The cursor is always visible.
  Visible,
  /// The cursor exists yet has been disabled.
  Invisible,
  /// The cursor is disabled.
  Disabled,
}

/// Different window options.
///
/// Feel free to look at the different methods available to tweak the options. You may want to start
/// with `default()` though.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowOpt {
  cursor_mode: CursorMode,
  num_samples: Option<u32>,
}

impl Default for WindowOpt {
  /// Defaults:
  ///
  /// - `cursor_mode` set to `CursorMode::Visible`.
  /// - `num_samples` set to `None`.
  fn default() -> Self {
    WindowOpt {
      cursor_mode: CursorMode::Visible,
      num_samples: None,
    }
  }
}

impl WindowOpt {
  /// Hide, unhide or disable the cursor. Default to `CursorMode::Visible`.
  #[inline]
  pub fn set_cursor_mode(self, mode: CursorMode) -> Self {
    WindowOpt {
      cursor_mode: mode,
      ..self
    }
  }

  /// Get the cursor mode.
  #[inline]
  pub fn cursor_mode(&self) -> CursorMode {
    self.cursor_mode
  }

  /// Set the number of samples to use for multisampling.
  ///
  /// Pass `None` to disable multisampling.
  #[inline]
  pub fn set_num_samples<S>(self, samples: S) -> Self
  where
    S: Into<Option<u32>>,
  {
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
  /// that lives as long as the surface lives.
  fn new(dim: WindowDim, title: &str, win_opt: WindowOpt) -> Result<Self, Self::Error>;

  /// Retrieve opitions and allow editing them.
  fn opts(&self) -> &WindowOpt;

  /// Change the cursor mode.
  fn set_cursor_mode(&mut self, mode: CursorMode) -> &mut Self;

  /// Change the multisampling state.
  fn set_num_samples<S>(&mut self, samples: S) -> &mut Self
  where
    S: Into<Option<u32>>;

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

  /// Get access to the back buffer.
  fn back_buffer(&mut self) -> Result<Framebuffer<Flat, Dim2, (), ()>, Self::Error> {
    Ok(Framebuffer::back_buffer(self, self.size()))
  }
}
