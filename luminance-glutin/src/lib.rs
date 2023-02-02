//! The [glutin](https://crates.io/crates/glutin) platform crate for [luminance](https://crates.io/crates/luminance).

#![deny(missing_docs)]

use gl;
use glutin::{
  event_loop::EventLoop, window::WindowBuilder, Api, ContextBuilder, ContextError, CreationError,
  GlProfile, GlRequest, NotCurrent, PossiblyCurrent, WindowedContext,
};
use luminance::context::GraphicsContext;
use luminance::framebuffer::{Framebuffer, FramebufferError};
use luminance::texture::Dim2;
pub use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
use std::error;
use std::fmt;
use std::os::raw::c_void;

/// Error that might occur when creating a Glutin surface.
#[derive(Debug)]
pub enum GlutinError {
  /// Something went wrong when creating the Glutin surface. The carried [`CreationError`] provides
  /// more information.
  CreationError(CreationError),
  /// OpenGL context error.
  ContextError(ContextError),
  /// Graphics state error that might occur when querying the initial state.
  GraphicsStateError(StateQueryError),
}

impl fmt::Display for GlutinError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      GlutinError::CreationError(ref e) => write!(f, "Glutin surface creation error: {}", e),
      GlutinError::ContextError(ref e) => write!(f, "Glutin OpenGL context creation error: {}", e),
      GlutinError::GraphicsStateError(ref e) => {
        write!(f, "OpenGL graphics state initialization error: {}", e)
      }
    }
  }
}

impl error::Error for GlutinError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      GlutinError::CreationError(e) => Some(e),
      GlutinError::ContextError(e) => Some(e),
      GlutinError::GraphicsStateError(e) => Some(e),
    }
  }
}

impl From<CreationError> for GlutinError {
  fn from(e: CreationError) -> Self {
    GlutinError::CreationError(e)
  }
}

impl From<ContextError> for GlutinError {
  fn from(e: ContextError) -> Self {
    GlutinError::ContextError(e)
  }
}

impl From<StateQueryError> for GlutinError {
  fn from(e: StateQueryError) -> Self {
    GlutinError::GraphicsStateError(e)
  }
}

/// The Glutin surface.
///
/// You want to create such an object in order to use any [luminance] construct.
///
/// [luminance]: https://crates.io/crates/luminance
pub struct GlutinSurface {
  /// The windowed context.
  pub ctx: WindowedContext<PossiblyCurrent>,
  /// OpenGL 3.3 state.
  gl: GL33,
}

unsafe impl GraphicsContext for GlutinSurface {
  type Backend = GL33;

  fn backend(&mut self) -> &mut Self::Backend {
    &mut self.gl
  }
}

impl GlutinSurface {
  /// Create a new [`GlutinSurface`] by consuming a [`WindowBuilder`].
  ///
  /// This is an alternative method to [`new_gl33`] that is more flexible as you have access to the
  /// whole `glutin` types.
  ///
  /// `window_builder` is the default object when passed to your closure and `ctx_builder` is
  /// already initialized for the OpenGL context (you’re not supposed to change it!).
  ///
  /// [`new_gl33`]: crate::GlutinSurface::new_gl33
  pub fn new_gl33_from_builders<'a, WB, CB>(
    window_builder: WB,
    ctx_builder: CB,
  ) -> Result<(Self, EventLoop<()>), GlutinError>
  where
    WB: FnOnce(&mut EventLoop<()>, WindowBuilder) -> WindowBuilder,
    CB:
      FnOnce(&mut EventLoop<()>, ContextBuilder<'a, NotCurrent>) -> ContextBuilder<'a, NotCurrent>,
  {
    let mut event_loop = EventLoop::new();

    let window_builder = window_builder(&mut event_loop, WindowBuilder::new());

    let windowed_ctx = ctx_builder(
      &mut event_loop,
      ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core),
    );

    let surface = Self::new_gl33_with_builders(&event_loop, window_builder, windowed_ctx)?;

    Ok((surface, event_loop))
  }

  /// Create a new [`GlutinSurface`] from scratch.
  pub fn new_gl33(
    window_builder: WindowBuilder,
    samples: u16,
  ) -> Result<(Self, EventLoop<()>), GlutinError> {
    let event_loop = EventLoop::new();
    let windowed_ctx = ContextBuilder::new()
      .with_multisampling(samples)
      .with_double_buffer(Some(true));
    let surface = Self::new_gl33_with_builders(&event_loop, window_builder, windowed_ctx)?;

    Ok((surface, event_loop))
  }

  /// Create a new [`GlutinSurface`] by passing in the builders.
  ///
  /// This is the most flexible but least hand-holding way of creating a [`GlutinSurface`].
  ///
  /// The OpenGL context should be version 3.3, core profile, and double-buffered.
  pub fn new_gl33_with_builders<T>(
    event_loop: &EventLoop<T>,
    window_builder: WindowBuilder,
    context_builder: ContextBuilder<NotCurrent>,
  ) -> Result<Self, GlutinError> {
    let windowed_ctx = context_builder
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

    // init OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

    ctx.window().set_visible(true);

    let gl = GL33::new().map_err(GlutinError::GraphicsStateError)?;
    let surface = GlutinSurface { ctx, gl };
    Ok(surface)
  }

  /// Get the underlying size (in physical pixels) of the surface.
  ///
  /// This is equivalent to getting the inner size of the windowed context and converting it to
  /// a physical size by using the HiDPI factor of the windowed context.
  pub fn size(&self) -> [u32; 2] {
    let size = self.ctx.window().inner_size();
    [size.width, size.height]
  }

  /// Get access to the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<GL33, Dim2, (), ()>, FramebufferError> {
    Framebuffer::back_buffer(self, self.size())
  }

  /// Swap the back and front buffers.
  pub fn swap_buffers(&mut self) {
    let _ = self.ctx.swap_buffers();
  }
}
