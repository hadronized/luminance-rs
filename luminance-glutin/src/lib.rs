//! The [glutin] windowing implementation for [luminance-windowing].
//!
//! [glutin]: https://crates.io/crates/glutin
//! [luminance-windowing]: https://crates.io/crates/luminance-windowing

use glutin::{
  Api, ContextBuilder, ContextError, CreationError, Event, EventsLoop, GlProfile, GlRequest, PossiblyCurrent,
  WindowBuilder, WindowedContext
};
use glutin::dpi::LogicalSize;
use luminance::context::GraphicsContext;
use luminance::state::{GraphicsState, StateQueryError};
use luminance_windowing::{CursorMode, Surface, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
  CreationError(CreationError),
  ContextError(ContextError),
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

pub struct GlutinSurface {
  ctx: WindowedContext<PossiblyCurrent>,
  event_loop: EventsLoop,
  gfx_state: Rc<RefCell<GraphicsState>>,
  opts: WindowOpt,
  // this is unfortunate but glutin returns an Option<LogicalSize> and hence we will store the
  // dimensions here instead of guessing when we want to get the size
  dimensions: [u32; 2],
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
    let (window_builder, dimensions) = match dim {
      WindowDim::Windowed(w, h) => {
        let wb = window_builder.with_dimensions(LogicalSize::new(w as f64, h as f64));
        let dims = [w, h];
        (wb, dims)
      }

      WindowDim::Fullscreen => {
        let prim_mon = event_loop.get_primary_monitor();
        let (w, h) = prim_mon.get_dimensions().into();
        let dims = [w, h];
        let wb = window_builder.with_fullscreen(Some(prim_mon));

        (wb, dims)
      }

      WindowDim::FullscreenRestricted(w, h) => {
        let prim_mon = event_loop.get_primary_monitor();
        let dims = [w, h];
        let wb = window_builder
          .with_dimensions(LogicalSize::new(w as f64, h as f64))
          .with_fullscreen(Some(prim_mon));

        (wb, dims)
      }
    };

    let windowed_ctx = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .with_multisampling(win_opt.num_samples().unwrap_or(0) as u16)
      .with_double_buffer(Some(true))
      .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

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
      dimensions,
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
    self.dimensions
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
