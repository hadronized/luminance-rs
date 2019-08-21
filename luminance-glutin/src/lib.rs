//! The [glutin] windowing implementation for [luminance-windowing].
//!
//! [glutin]: https://crates.io/crates/glutin
//! [luminance-windowing]: https://crates.io/crates/luminance-windowing

#![deny(missing_docs)]

use gl;
pub use glutin::{
  ContextError, CreationError, DeviceEvent, DeviceId, ElementState, Event, KeyboardInput,
  ModifiersState, MouseScrollDelta, Touch, TouchPhase, VirtualKeyCode, WindowEvent, WindowId
};
pub use glutin::dpi::{LogicalPosition, LogicalSize};
pub use luminance_windowing::{CursorMode, Surface, WindowDim, WindowOpt};

use glutin::{
  Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, PossiblyCurrent,
  WindowBuilder, WindowedContext
};
use glutin::dpi::PhysicalSize;
use luminance::context::GraphicsContext;
use luminance::state::{GraphicsState, StateQueryError};
use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;

/// Error that might occur when creating a Glutin surface.
#[derive(Debug)]
pub enum Error {
  /// Something went wrong when creating the Glutin surface. The carried [`CreationError`] provides
  /// more information.
  CreationError(CreationError),
  /// OpenGL context error.
  ContextError(ContextError),
  /// Graphics state error that might occur when querying the initial state.
  GraphicsStateError(StateQueryError)
}

impl From<CreationError> for Error {
  fn from(e: CreationError) -> Self {
    Error::CreationError(e)
  }
}

impl From<ContextError> for Error {
  fn from(e: ContextError) -> Self {
    Error::ContextError(e)
  }
}

/// The Glutin surface.
///
/// You want to create such an object in order to use any [luminance] construct.
///
/// [luminance]: https://crates.io/crates/luminance
pub struct GlutinSurface {
  ctx: WindowedContext<PossiblyCurrent>,
  event_loop: EventsLoop,
  gfx_state: Rc<RefCell<GraphicsState>>,
  opts: WindowOpt,
  // a list of event that has happened
  event_queue: Vec<Event>
}

unsafe impl GraphicsContext for GlutinSurface {
  fn state(&self) -> &Rc<RefCell<GraphicsState>> {
    &self.gfx_state
  }
}

impl Surface for GlutinSurface {
  type Error = Error;
  type Event = Event;

  fn new(dim: WindowDim, title: &str, win_opt: WindowOpt) -> Result<Self, Self::Error> {
    let event_loop = EventsLoop::new();

    let window_builder = WindowBuilder::new().with_title(title);
    let window_builder = match dim {
      WindowDim::Windowed(w, h) => window_builder.with_dimensions((w, h).into()),
      WindowDim::Fullscreen => window_builder.with_fullscreen(Some(event_loop.get_primary_monitor())),
      WindowDim::FullscreenRestricted(w, h) =>
        window_builder
          .with_dimensions((w, h).into())
          .with_fullscreen(Some(event_loop.get_primary_monitor()))
    };

    let windowed_ctx = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .with_multisampling(win_opt.num_samples().unwrap_or(0) as u16)
      .with_double_buffer(Some(true))
      .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

    // init OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

    match win_opt.cursor_mode() {
      CursorMode::Visible => ctx.window().hide_cursor(false),
      // glutin doesn’t support disabled cursors; default to invisible
      CursorMode::Invisible | CursorMode::Disabled => ctx.window().hide_cursor(true),
    }

    ctx.window().show();

    let gfx_state = GraphicsState::new().map_err(Error::GraphicsStateError)?;
    let surface = GlutinSurface {
      ctx,
      event_loop,
      gfx_state: Rc::new(RefCell::new(gfx_state)),
      opts: win_opt,
      event_queue: Vec::new()
    };

    Ok(surface)
  }

  fn opts(&self) -> &WindowOpt {
    &self.opts
  }

  fn set_cursor_mode(&mut self, mode: CursorMode) -> &mut Self {
    match mode {
      CursorMode::Visible => self.ctx.window().hide_cursor(false),
      CursorMode::Invisible | CursorMode::Disabled => self.ctx.window().hide_cursor(true)
    }

    self.opts = self.opts.set_cursor_mode(mode);
    self
  }

  fn set_num_samples<S>(&mut self, _samples: S) -> &mut Self where S: Into<Option<u32>> {
    panic!("not supported")
  }

  fn size(&self) -> [u32; 2] {
    let logical = self.ctx.window().get_inner_size().unwrap();
    let (w, h) = PhysicalSize::from_logical(logical, self.ctx.window().get_hidpi_factor()).into();
    [w, h]
  }

  fn wait_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a> {
    panic!("not implemented yet")
  }

  fn poll_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = Self::Event> + 'a> {
    self.event_queue.clear();

    let queue = &mut self.event_queue;
    self.event_loop.poll_events(|event| {
      queue.push(event);
    });

    Box::new(self.event_queue.iter().cloned())
  }

  fn swap_buffers(&mut self) {
    self.ctx.swap_buffers().unwrap();
  }
}
