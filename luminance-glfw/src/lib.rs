//! [GLFW](https://crates.io/crates/glfw) backend for [luminance](https://crates.io/crates/luminance)
//! and [luminance-windowing](https://crates.io/crates/luminance-windowing).

#![deny(missing_docs)]

use gl;
use glfw::{self, Context, CursorMode as GlfwCursorMode, SwapInterval, Window, WindowMode};
use luminance::context::GraphicsContext;
use luminance::state::GraphicsState;
pub use luminance::state::StateQueryError;
pub use luminance_windowing::{CursorMode, Surface, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::fmt;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

pub use glfw::{Action, InitError, Key, MouseButton, WindowEvent};

/// Error that can be risen while creating a surface.
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

// TODO: better implementation
impl fmt::Display for GlfwSurfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      GlfwSurfaceError::InitError(ref e) => write!(f, "initialization error: {}", e),
      GlfwSurfaceError::WindowCreationFailed => f.write_str("failed to create window"),
      GlfwSurfaceError::NoPrimaryMonitor => f.write_str("no primary monitor"),
      GlfwSurfaceError::NoVideoMode => f.write_str("no video mode"),
      GlfwSurfaceError::GraphicsStateError(ref e) => write!(f, "failed to get graphics state: {}", e),
    }
  }
}

/// GLFW surface.
///
/// This type implements `GraphicsContext` so that you can use it to perform render with
/// **luminance**.
pub struct GlfwSurface {
  window: Window,
  events_rx: Receiver<(f64, WindowEvent)>,
  gfx_state: Rc<RefCell<GraphicsState>>,
  opts: WindowOpt
}

unsafe impl GraphicsContext for GlfwSurface {
  fn state(&self) -> &Rc<RefCell<GraphicsState>> {
    &self.gfx_state
  }
}

impl Surface for GlfwSurface {
  type Error = GlfwSurfaceError;
  type Event = WindowEvent;

  fn new(dim: WindowDim, title: &str, win_opt: WindowOpt) -> Result<Self, Self::Error> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).map_err(GlfwSurfaceError::InitError)?;

    // OpenGL hints
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::Samples(win_opt.num_samples()));

    // open a window in windowed or fullscreen mode
    let (mut window, events_rx) = match dim {
      WindowDim::Windowed(w, h) => glfw
        .create_window(w, h, title, WindowMode::Windowed)
        .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
      WindowDim::Fullscreen => glfw.with_primary_monitor(|glfw, monitor| {
        let monitor = monitor.ok_or(GlfwSurfaceError::NoPrimaryMonitor)?;
        let vmode = monitor.get_video_mode().ok_or(GlfwSurfaceError::NoVideoMode)?;
        let (w, h) = (vmode.width, vmode.height);

        Ok(
          glfw
            .create_window(w, h, title, WindowMode::FullScreen(monitor))
            .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
        )
      })?,
      WindowDim::FullscreenRestricted(w, h) => glfw.with_primary_monitor(|glfw, monitor| {
        let monitor = monitor.ok_or(GlfwSurfaceError::NoPrimaryMonitor)?;

        Ok(
          glfw
            .create_window(w, h, title, WindowMode::FullScreen(monitor))
            .ok_or(GlfwSurfaceError::WindowCreationFailed)?,
        )
      })?,
    };

    window.make_current();

    match win_opt.cursor_mode() {
      CursorMode::Visible => window.set_cursor_mode(GlfwCursorMode::Normal),
      CursorMode::Invisible =>  window.set_cursor_mode(GlfwCursorMode::Hidden),
      CursorMode::Disabled =>  window.set_cursor_mode(GlfwCursorMode::Disabled),
    }

    window.set_all_polling(true);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    // init OpenGL
    gl::load_with(|s| window.get_proc_address(s) as *const c_void);

    let gfx_state = GraphicsState::new().map_err(GlfwSurfaceError::GraphicsStateError)?;
    let surface = GlfwSurface {
      window,
      events_rx,
      gfx_state: Rc::new(RefCell::new(gfx_state)),
      opts: win_opt
    };

    Ok(surface)
  }

  fn opts(&self) -> &WindowOpt {
    &self.opts
  }

  fn set_cursor_mode(&mut self, mode: CursorMode) -> &mut Self {
    match mode {
      CursorMode::Visible => self.window.set_cursor_mode(GlfwCursorMode::Normal),
      CursorMode::Invisible =>  self.window.set_cursor_mode(GlfwCursorMode::Hidden),
      CursorMode::Disabled =>  self.window.set_cursor_mode(GlfwCursorMode::Disabled),
    }

    self.opts = self.opts.set_cursor_mode(mode);
    self
  }

  fn set_num_samples<S>(&mut self, samples: S) -> &mut Self where S: Into<Option<u32>> {
    let samples = samples.into();
    self.window.glfw.window_hint(glfw::WindowHint::Samples(samples));
    self.opts = self.opts.set_num_samples(samples);
    self
  }

  fn size(&self) -> [u32; 2] {
    let (x, y) = self.window.get_framebuffer_size();
    [x as u32, y as u32]
  }

  fn wait_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a> {
    self.window.glfw.wait_events();
    Box::new(self.events_rx.iter().map(|(_, e)| e))
  }

  fn poll_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a> {
    self.window.glfw.poll_events();
    Box::new(self.events_rx.try_iter().map(|(_, e)| e))
  }

  fn swap_buffers(&mut self) {
    self.window.swap_buffers();
  }
}
