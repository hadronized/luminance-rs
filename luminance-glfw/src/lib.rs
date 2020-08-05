//! [GLFW](https://crates.io/crates/glfw) backend for [luminance](https://crates.io/crates/luminance)
//! and [luminance-windowing](https://crates.io/crates/luminance-windowing).

#![deny(missing_docs)]

use glfw::{self, Context, CursorMode as GlfwCursorMode, SwapInterval, Window, WindowMode};
use glfw::{InitError, WindowEvent};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::framebuffer::FramebufferError;
use luminance::texture::Dim2;
pub use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
use luminance_windowing::{CursorMode, WindowDim, WindowOpt};
use std::error;
use std::fmt;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;

/// Error that can be risen while creating a surface.
#[non_exhaustive]
#[derive(Debug)]
pub enum GlfwSurfaceError {
  /// Initialization of the surface went wrong.
  ///
  /// This variant exposes a **glfw** error for further information about what went wrong.
  InitError(InitError),
  /// Window creation failed.
  WindowCreationFailed,
  /// No primary monitor detected.
  NoPrimaryMonitor,
  /// No available video mode.
  NoVideoMode,
  /// The graphics state is not available.
  ///
  /// This error is generated when the initialization code is called on a thread on which the
  /// graphics state has already been acquired.
  GraphicsStateError(StateQueryError),
}

impl fmt::Display for GlfwSurfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      GlfwSurfaceError::InitError(ref e) => write!(f, "initialization error: {}", e),
      GlfwSurfaceError::WindowCreationFailed => f.write_str("failed to create window"),
      GlfwSurfaceError::NoPrimaryMonitor => f.write_str("no primary monitor"),
      GlfwSurfaceError::NoVideoMode => f.write_str("no video mode"),
      GlfwSurfaceError::GraphicsStateError(ref e) => {
        write!(f, "failed to get graphics state: {}", e)
      }
    }
  }
}

impl error::Error for GlfwSurfaceError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      GlfwSurfaceError::InitError(e) => Some(e),
      GlfwSurfaceError::GraphicsStateError(e) => Some(e),
      _ => None,
    }
  }
}

/// GLFW surface.
///
/// This type implements `GraphicsContext` so that you can use it to perform render with
/// **luminance**.
pub struct GlfwSurface {
  /// Wrapped GLFW window.
  pub window: Window,
  /// Wrapped GLFW events queue.
  pub events_rx: Receiver<(f64, WindowEvent)>,
  /// OpenGL 3.3 state.
  gl: GL33,
}

impl GlfwSurface {
  /// Create a [`GlfwSurface`].
  pub fn new_gl33<S>(title: S, win_opt: WindowOpt) -> Result<Self, GlfwSurfaceError>
  where
    S: AsRef<str>,
  {
    #[cfg(feature = "log-errors")]
    let error_cbk = glfw::LOG_ERRORS;
    #[cfg(not(feature = "log-errors"))]
    let error_cbk = glfw::FAIL_ON_ERRORS;

    let mut glfw = glfw::init(error_cbk).map_err(GlfwSurfaceError::InitError)?;

    // OpenGL hints
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
      glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::Samples(*win_opt.num_samples()));

    // open a window in windowed or fullscreen mode
    let title = title.as_ref();
    let dim = win_opt.dim;
    let (mut window, events_rx) = match dim {
      WindowDim::Windowed { width, height } => glfw
        .create_window(width, height, title, WindowMode::Windowed)
        .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
      WindowDim::Fullscreen => glfw.with_primary_monitor(|glfw, monitor| {
        let monitor = monitor.ok_or(GlfwSurfaceError::NoPrimaryMonitor)?;
        let vmode = monitor
          .get_video_mode()
          .ok_or(GlfwSurfaceError::NoVideoMode)?;
        let (w, h) = (vmode.width, vmode.height);

        Ok(
          glfw
            .create_window(w, h, title, WindowMode::FullScreen(monitor))
            .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
        )
      })?,
      WindowDim::FullscreenRestricted { width, height } => {
        glfw.with_primary_monitor(|glfw, monitor| {
          let monitor = monitor.ok_or(GlfwSurfaceError::NoPrimaryMonitor)?;

          Ok(
            glfw
              .create_window(width, height, title, WindowMode::FullScreen(monitor))
              .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
          )
        })?
      }
    };

    window.make_current();

    match win_opt.cursor_mode() {
      CursorMode::Visible => window.set_cursor_mode(GlfwCursorMode::Normal),
      CursorMode::Invisible => window.set_cursor_mode(GlfwCursorMode::Hidden),
      CursorMode::Disabled => window.set_cursor_mode(GlfwCursorMode::Disabled),
    }

    window.set_all_polling(true);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    // init OpenGL
    gl::load_with(|s| window.get_proc_address(s) as *const c_void);

    let gl = GL33::new().map_err(GlfwSurfaceError::GraphicsStateError)?;
    let surface = GlfwSurface {
      window,
      events_rx,
      gl,
    };

    Ok(surface)
  }

  /// Get the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<GL33, Dim2, (), ()>, FramebufferError> {
    let (w, h) = self.window.get_framebuffer_size();
    Framebuffer::back_buffer(self, [w as u32, h as u32])
  }
}

unsafe impl GraphicsContext for GlfwSurface {
  type Backend = GL33;

  fn backend(&mut self) -> &mut Self::Backend {
    &mut self.gl
  }
}
