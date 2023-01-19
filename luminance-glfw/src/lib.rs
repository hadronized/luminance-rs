//! [GLFW](https://crates.io/crates/glfw) backend for [luminance](https://crates.io/crates/luminance).

use glfw::{self, Glfw, InitError, Window, WindowEvent};
use luminance::context::Context;
use luminance_gl2::GL33;
use std::{error, fmt, os::raw::c_void, sync::mpsc::Receiver};

/// Error that can be risen while creating a surface.
#[non_exhaustive]
#[derive(Debug)]
pub enum GlfwSurfaceError<E> {
  /// Initialization of the surface went wrong.
  ///
  /// This variant exposes a **glfw** error for further information about what went wrong.
  InitError(InitError),

  /// Error with the backend.
  BackendError(String),

  /// User error.
  UserError(E),
}

impl<E> fmt::Display for GlfwSurfaceError<E>
where
  E: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      GlfwSurfaceError::InitError(e) => write!(f, "initialization error: {}", e),
      GlfwSurfaceError::BackendError(reason) => write!(f, "backend error: {}", reason),
      GlfwSurfaceError::UserError(e) => write!(f, "user error: {}", e),
    }
  }
}

impl<E> From<InitError> for GlfwSurfaceError<E> {
  fn from(e: InitError) -> Self {
    GlfwSurfaceError::InitError(e)
  }
}

impl<E> error::Error for GlfwSurfaceError<E>
where
  E: 'static + error::Error,
{
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      GlfwSurfaceError::InitError(e) => Some(e),
      GlfwSurfaceError::UserError(e) => Some(e),
      _ => None,
    }
  }
}

/// GLFW surface.
///
/// This type is a helper that exposes two important concepts: the GLFW event receiver that you can use it with to
/// poll events and the [`GL33Context`], which allows you to perform the rendering part.
#[derive(Debug)]
pub struct GlfwSurface {
  /// Wrapped GLFW events queue.
  pub events_rx: Receiver<(f64, WindowEvent)>,
  ///
  /// Wrapped GLFW window.
  pub window: Window,

  /// Wrapped luminance context.
  pub ctx: Context<GL33>,
}

impl GlfwSurface {
  /// Initialize GLFW to provide a luminance context.
  pub fn new_gl33<E>(
    create_window: impl FnOnce(
      &mut Glfw,
    )
      -> Result<(Window, Receiver<(f64, WindowEvent)>), GlfwSurfaceError<E>>,
  ) -> Result<Self, GlfwSurfaceError<E>> {
    #[cfg(feature = "log-errors")]
    let error_cbk = glfw::LOG_ERRORS;
    #[cfg(not(feature = "log-errors"))]
    let error_cbk = glfw::FAIL_ON_ERRORS;

    let mut glfw = glfw::init(error_cbk)?;

    // OpenGL hints
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
      glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

    let (mut window, events_rx) = create_window(&mut glfw)?;

    // init OpenGL
    gl::load_with(|s| window.get_proc_address(s) as *const c_void);

    let ctx = Context::new(GL33::new)
      .ok_or_else(|| GlfwSurfaceError::BackendError("unavailable OpenGL 3.3 state".to_owned()))?;
    let surface = GlfwSurface {
      events_rx,
      window,
      ctx,
    };

    Ok(surface)
  }
}
