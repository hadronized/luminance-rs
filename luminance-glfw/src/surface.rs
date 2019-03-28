//! Surface implementation.
//!
//! This module exports the `GlfwSurface` as an implementation of `Surface`.

use gl;
use glfw::{self, Context, CursorMode, SwapInterval, Window, WindowMode};
use luminance::context::GraphicsContext;
use luminance::state::GraphicsState;
pub use luminance_windowing::{Surface, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

pub use error::{GlfwSurfaceError, InitError, StateQueryError};
pub use event::{Action, Key, MouseButton, WindowEvent};

/// GLFW surface.
///
/// This type implements `GraphicsContext` so that you can use it to perform render with
/// **luminance**.
pub struct GlfwSurface {
  window: Window,
  events_rx: Receiver<(f64, WindowEvent)>,
  gfx_state: Rc<RefCell<GraphicsState>>,
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

    if win_opt.is_cursor_hidden() {
      window.set_cursor_mode(CursorMode::Disabled);
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
    };

    Ok(surface)
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
