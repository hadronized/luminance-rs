use luminance::context::GraphicsContext;
use luminance::framebuffer::{Framebuffer, FramebufferError};
use luminance::texture::Dim2;
use luminance_webgl::webgl2::{StateQueryError, WebGL2};
use std::fmt;
use wasm_bindgen::{JsCast as _, JsValue};
use web_sys::{Document, HtmlCanvasElement, Window};

/// web-sys errors that might occur while initializing and using the platform.
#[non_exhaustive]
#[derive(Debug)]
pub enum WebSysWebGL2SurfaceError {
  CannotGrabWindow,
  CannotGrabDocument,
  NotSuchCanvasElement(String),
  CannotGrabWebGL2Context,
  NoAvailableWebGL2Context,
  StateQueryError(StateQueryError),
}

impl WebSysWebGL2SurfaceError {
  fn cannot_grab_window() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabWindow
  }

  fn cannot_grab_document() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabDocument
  }

  fn not_such_canvas_element(name: impl Into<String>) -> Self {
    WebSysWebGL2SurfaceError::NotSuchCanvasElement(name.into())
  }

  fn cannot_grab_webgl2_context() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabWebGL2Context
  }

  fn no_available_webgl2_context() -> Self {
    WebSysWebGL2SurfaceError::NoAvailableWebGL2Context
  }
}

impl fmt::Display for WebSysWebGL2SurfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      WebSysWebGL2SurfaceError::CannotGrabWindow => f.write_str("cannot grab the window node"),
      WebSysWebGL2SurfaceError::CannotGrabDocument => f.write_str("cannot grab the document node"),
      WebSysWebGL2SurfaceError::NotSuchCanvasElement(ref name) => {
        write!(f, "cannot grab canvas named {}", name)
      }
      WebSysWebGL2SurfaceError::CannotGrabWebGL2Context => {
        f.write_str("cannot grab WebGL2 context")
      }
      WebSysWebGL2SurfaceError::NoAvailableWebGL2Context => {
        f.write_str("no available WebGL2 context")
      }
      WebSysWebGL2SurfaceError::StateQueryError(ref e) => {
        write!(f, "WebGL2 state query error: {}", e)
      }
    }
  }
}

impl std::error::Error for WebSysWebGL2SurfaceError {}

impl From<StateQueryError> for WebSysWebGL2SurfaceError {
  fn from(e: StateQueryError) -> Self {
    WebSysWebGL2SurfaceError::StateQueryError(e)
  }
}

/// web-sys surface for WebGL2.
pub struct WebSysWebGL2Surface {
  pub window: Window,
  pub document: Document,
  pub canvas: HtmlCanvasElement,
  backend: WebGL2,
}

impl WebSysWebGL2Surface {
  /// Create a new [`WebSysWebGL2Surface`] based on the name of the DOM canvas element named by
  /// `canvas_name`.
  pub fn new(canvas_name: impl AsRef<str>) -> Result<Self, WebSysWebGL2SurfaceError> {
    let (window, document, canvas) = Self::get_canvas(canvas_name)?;
    Self::from_canvas(window, document, canvas)
  }

  /// Create a new [`WebSysWebGL2Surface`] based on the name of the DOM canvas element named by `canvas_name` and pass
  /// along a list of parameters when creating the WebGL context.
  pub fn new_with_params(
    canvas_name: impl AsRef<str>,
    params: impl AsRef<JsValue>,
  ) -> Result<Self, WebSysWebGL2SurfaceError> {
    let (window, document, canvas) = Self::get_canvas(canvas_name)?;
    Self::from_canvas_with_params(window, document, canvas, params)
  }

  /// Create a new [`WebSysWebGL2Surface`] based on a given [`HtmlCanvasElement`].
  pub fn from_canvas(
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
  ) -> Result<Self, WebSysWebGL2SurfaceError> {
    let webgl2 = canvas
      .get_context("webgl2")
      .map_err(|_| WebSysWebGL2SurfaceError::cannot_grab_webgl2_context())?
      .ok_or_else(|| WebSysWebGL2SurfaceError::no_available_webgl2_context())?;
    let ctx = webgl2
      .dyn_into()
      .map_err(|_| WebSysWebGL2SurfaceError::no_available_webgl2_context())?;

    // create the backend object and return the whole object
    let backend = WebGL2::new(ctx)?;

    Ok(Self {
      window,
      document,
      canvas,
      backend,
    })
  }

  /// Obtain a canvas from its name, as well as the document it is attached in.
  fn get_canvas(
    canvas_name: impl AsRef<str>,
  ) -> Result<(Window, Document, HtmlCanvasElement), WebSysWebGL2SurfaceError> {
    let window = web_sys::window().ok_or_else(|| WebSysWebGL2SurfaceError::cannot_grab_window())?;

    let document = window
      .document()
      .ok_or_else(|| WebSysWebGL2SurfaceError::cannot_grab_document())?;

    let canvas_name = canvas_name.as_ref();
    let canvas = document
      .get_element_by_id(canvas_name)
      .ok_or_else(|| WebSysWebGL2SurfaceError::not_such_canvas_element(canvas_name))?;

    let canvas = canvas
      .dyn_into::<HtmlCanvasElement>()
      .map_err(|_| WebSysWebGL2SurfaceError::not_such_canvas_element(canvas_name))?;

    Ok((window, document, canvas))
  }

  /// Create a new [`WebSysWebGL2Surface`] based on a given [`HtmlCanvasElement`] and pass along a list of parameters
  /// when creating the WebGL context.
  pub fn from_canvas_with_params(
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
    params: impl AsRef<JsValue>,
  ) -> Result<Self, WebSysWebGL2SurfaceError> {
    let webgl2 = canvas
      .get_context_with_context_options("webgl2", params.as_ref())
      .map_err(|_| WebSysWebGL2SurfaceError::cannot_grab_webgl2_context())?
      .ok_or_else(|| WebSysWebGL2SurfaceError::no_available_webgl2_context())?;
    let ctx = webgl2
      .dyn_into()
      .map_err(|_| WebSysWebGL2SurfaceError::no_available_webgl2_context())?;

    // create the backend object and return the whole object
    let backend = WebGL2::new(ctx)?;

    Ok(Self {
      window,
      document,
      canvas,
      backend,
    })
  }

  /// Get the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<WebGL2, Dim2, (), ()>, FramebufferError> {
    let dim = [self.canvas.width(), self.canvas.height()];
    Framebuffer::back_buffer(self, dim)
  }
}

unsafe impl GraphicsContext for WebSysWebGL2Surface {
  type Backend = WebGL2;

  fn backend(&mut self) -> &mut Self::Backend {
    &mut self.backend
  }
}
