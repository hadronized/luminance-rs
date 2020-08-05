//! Graphics state.

use js_sys::{Float32Array, Int32Array};
use luminance::blending::{Equation, Factor};
use luminance::depth_test::{DepthComparison, DepthWrite};
use luminance::face_culling::{FaceCullingMode, FaceCullingOrder};
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use web_sys::{
  WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlTexture,
  WebGlVertexArrayObject,
};

// TLS synchronization barrier for `GLState`.
thread_local!(static TLS_ACQUIRE_GFX_STATE: RefCell<Option<()>> = RefCell::new(Some(())));

#[derive(Debug)]
pub(crate) struct BindingStack {
  pub(crate) next_texture_unit: u32,
  pub(crate) free_texture_units: Vec<u32>,
  pub(crate) next_buffer_binding: u32,
  pub(crate) free_buffer_bindings: Vec<u32>,
}

impl BindingStack {
  // Create a new, empty binding stack.
  fn new() -> Self {
    BindingStack {
      next_texture_unit: 0,
      free_texture_units: Vec::new(),
      next_buffer_binding: 0,
      free_buffer_bindings: Vec::new(),
    }
  }
}

/// The graphics state.
///
/// This type represents the current state of a given graphics context. It acts
/// as a forward-gate to all the exposed features from the low-level API but
/// adds a small cache layer over it to prevent from issuing the same API call (with
/// the same parameters).
#[derive(Debug)]
pub struct WebGL2State {
  _phantom: PhantomData<*const ()>, // !Send and !Sync

  // WebGL context
  pub(crate) ctx: WebGl2RenderingContext,

  // binding stack
  binding_stack: BindingStack,

  // viewport
  viewport: [i32; 4],

  // clear buffers
  clear_color: [f32; 4],

  // blending
  blending_state: BlendingState,
  blending_equations: BlendingEquations,
  blending_funcs: BlendingFactors,

  // depth test
  depth_test: DepthTest,
  depth_test_comparison: DepthComparison,

  // depth write
  depth_write: DepthWrite,

  // face culling
  face_culling_state: FaceCullingState,
  face_culling_order: FaceCullingOrder,
  face_culling_mode: FaceCullingMode,

  // texture
  current_texture_unit: u32,
  bound_textures: Vec<(u32, Option<WebGlTexture>)>,

  // texture buffer used to optimize texture creation; regular textures typically will never ask
  // for fetching from this set but framebuffers, who often generate several textures, might use
  // this opportunity to get N textures (color, depth and stencil) at once, in a single CPU / GPU
  // roundtrip
  //
  // fishy fishy
  texture_swimming_pool: Vec<Option<WebGlTexture>>,

  // uniform buffer
  bound_uniform_buffers: Vec<Option<WebGlBuffer>>,

  // array buffer
  bound_array_buffer: Option<WebGlBuffer>,
  // element buffer
  bound_element_array_buffer: Option<WebGlBuffer>,

  // framebuffer
  bound_draw_framebuffer: Option<WebGlFramebuffer>,
  bound_read_framebuffer: Option<WebGlFramebuffer>,

  // A special framebuffer used to read textures (workaround the fact WebGL2 doesn’t have
  // support of glGetTexImage). That object will never be created until trying to read a
  // texture’s image.
  readback_framebuffer: Option<WebGlFramebuffer>,

  // vertex array
  bound_vertex_array: Option<WebGlVertexArrayObject>,
  // shader program
  current_program: Option<WebGlProgram>,
}

impl WebGL2State {
  /// Create a new `GLState`.
  ///
  /// > Note: keep in mind you can create only one per thread. However, if you’re building without
  /// > standard library, this function will always return successfully. You have to take extra care
  /// > in this case.
  pub(crate) fn new(ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    TLS_ACQUIRE_GFX_STATE.with(|rc| {
      let mut inner = rc.borrow_mut();

      match *inner {
        Some(_) => {
          inner.take();
          Self::get_from_context(ctx)
        }

        None => Err(StateQueryError::UnavailableWebGL2State),
      }
    })
  }

  /// Get a `GraphicsContext` from the current OpenGL context.
  fn get_from_context(mut ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    load_webgl2_extensions(&mut ctx)?;

    let binding_stack = BindingStack::new();
    let viewport = get_ctx_viewport(&mut ctx)?;
    let clear_color = get_ctx_clear_color(&mut ctx)?;
    let blending_state = get_ctx_blending_state(&mut ctx);
    let blending_equations = get_ctx_blending_equations(&mut ctx)?;
    let blending_funcs = get_ctx_blending_factors(&mut ctx)?;
    let depth_test = get_ctx_depth_test(&mut ctx);
    let depth_test_comparison = DepthComparison::Less;
    let depth_write = get_ctx_depth_write(&mut ctx);
    let face_culling_state = get_ctx_face_culling_state(&mut ctx);
    let face_culling_order = get_ctx_face_culling_order(&mut ctx)?;
    let face_culling_mode = get_ctx_face_culling_mode(&mut ctx)?;

    let current_texture_unit = 0;
    let bound_textures = vec![(WebGl2RenderingContext::TEXTURE0, None); 48]; // 48 is the platform minimal requirement
    let texture_swimming_pool = Vec::new();
    let bound_uniform_buffers = vec![None; 36]; // 36 is the platform minimal requirement
    let bound_array_buffer = None;
    let bound_element_array_buffer = None;
    let bound_draw_framebuffer = None;
    let bound_read_framebuffer = None;
    let readback_framebuffer = None;
    let bound_vertex_array = None;
    let current_program = None;

    Ok(WebGL2State {
      _phantom: PhantomData,
      ctx,
      binding_stack,
      viewport,
      clear_color,
      blending_state,
      blending_equations,
      blending_funcs,
      depth_test,
      depth_test_comparison,
      depth_write,
      face_culling_state,
      face_culling_order,
      face_culling_mode,
      current_texture_unit,
      bound_textures,
      texture_swimming_pool,
      bound_uniform_buffers,
      bound_array_buffer,
      bound_element_array_buffer,
      bound_draw_framebuffer,
      bound_read_framebuffer,
      readback_framebuffer,
      bound_vertex_array,
      current_program,
    })
  }

  pub(crate) fn binding_stack_mut(&mut self) -> &mut BindingStack {
    &mut self.binding_stack
  }

  pub(crate) fn create_buffer(&mut self) -> Option<WebGlBuffer> {
    self.ctx.create_buffer()
  }

  pub(crate) fn bind_buffer_base(&mut self, handle: &WebGlBuffer, binding: u32) {
    match self.bound_uniform_buffers.get(binding as usize) {
      Some(ref handle_) if Some(handle) != handle_.as_ref() => {
        self.ctx.bind_buffer_base(
          WebGl2RenderingContext::UNIFORM_BUFFER,
          binding,
          Some(handle),
        );
        self.bound_uniform_buffers[binding as usize] = Some(handle.clone());
      }

      None => {
        self.ctx.bind_buffer_base(
          WebGl2RenderingContext::UNIFORM_BUFFER,
          binding,
          Some(handle),
        );

        // not enough registered buffer bindings; let’s grow a bit more
        self
          .bound_uniform_buffers
          .resize(binding as usize + 1, None);
        self.bound_uniform_buffers[binding as usize] = Some(handle.clone());
      }

      _ => (), // cached
    }
  }

  pub(crate) fn bind_array_buffer(&mut self, buffer: Option<&WebGlBuffer>, bind: Bind) {
    if bind == Bind::Forced || self.bound_array_buffer.as_ref() != buffer {
      self
        .ctx
        .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, buffer);
      self.bound_array_buffer = buffer.cloned();
    }
  }

  pub(crate) fn bind_element_array_buffer(&mut self, buffer: Option<&WebGlBuffer>, bind: Bind) {
    if bind == Bind::Forced || self.bound_element_array_buffer.as_ref() != buffer {
      self
        .ctx
        .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, buffer);
      self.bound_element_array_buffer = buffer.cloned();
    }
  }

  pub(crate) fn unbind_buffer(&mut self, buffer: &WebGlBuffer) {
    if self.bound_array_buffer.as_ref() == Some(buffer) {
      self.bind_array_buffer(None, Bind::Cached);
    } else if self.bound_element_array_buffer.as_ref() == Some(buffer) {
      self.bind_element_array_buffer(None, Bind::Cached);
    } else if let Some(handle_) = self
      .bound_uniform_buffers
      .iter_mut()
      .find(|h| h.as_ref() == Some(buffer))
    {
      *handle_ = None;
    }
  }

  pub(crate) fn create_vertex_array(&mut self) -> Option<WebGlVertexArrayObject> {
    self.ctx.create_vertex_array()
  }

  pub(crate) fn bind_vertex_array(&mut self, vao: Option<&WebGlVertexArrayObject>, bind: Bind) {
    if bind == Bind::Forced || self.bound_vertex_array.as_ref() != vao {
      self.ctx.bind_vertex_array(vao);
      self.bound_vertex_array = vao.cloned();
    }
  }

  pub(crate) fn create_texture(&mut self) -> Option<WebGlTexture> {
    self
      .texture_swimming_pool
      .pop()
      .flatten()
      .or_else(|| self.ctx.create_texture())
  }

  /// Reserve at least a given number of textures.
  pub(crate) fn reserve_textures(&mut self, nb: usize) {
    let available = self.texture_swimming_pool.len();
    let needed = nb.max(available) - available;

    if needed > 0 {
      // resize the internal buffer to hold all the new textures and create a slice starting from
      // the previous end to the new end
      self.texture_swimming_pool.resize(available + needed, None);

      for _ in 0..needed {
        match self.ctx.create_texture() {
          Some(texture) => self.texture_swimming_pool.push(Some(texture)),
          None => break,
        }
      }
    }
  }

  pub(crate) fn set_texture_unit(&mut self, unit: u32) {
    if self.current_texture_unit != unit {
      self
        .ctx
        .active_texture(WebGl2RenderingContext::TEXTURE0 + unit);
      self.current_texture_unit = unit;
    }
  }

  pub(crate) fn bind_texture(&mut self, target: u32, handle: Option<&WebGlTexture>) {
    let unit = self.current_texture_unit as usize;

    match self.bound_textures.get(unit) {
      Some((t, ref h)) if target != *t || handle != h.as_ref() => {
        self.ctx.bind_texture(target, handle);
        self.bound_textures[unit] = (target, handle.cloned());
      }

      None => {
        self.ctx.bind_texture(target, handle);

        // not enough available texture units; let’s grow a bit more
        self
          .bound_textures
          .resize(unit + 1, (WebGl2RenderingContext::TEXTURE_2D, None));
        self.bound_textures[unit] = (target, handle.cloned());
      }

      _ => (), // cached
    }
  }

  pub(crate) fn create_framebuffer(&mut self) -> Option<WebGlFramebuffer> {
    self.ctx.create_framebuffer()
  }

  pub(crate) fn create_or_get_readback_framebuffer(&mut self) -> Option<WebGlFramebuffer> {
    self.readback_framebuffer.clone().or_else(|| {
      // create the readback framebuffer if not already created
      self.readback_framebuffer = self.create_framebuffer();
      self.readback_framebuffer.clone()
    })
  }

  pub(crate) fn bind_draw_framebuffer(&mut self, handle: Option<&WebGlFramebuffer>) {
    if self.bound_draw_framebuffer.as_ref() != handle {
      self
        .ctx
        .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, handle);
      self.bound_draw_framebuffer = handle.cloned();
    }
  }

  pub(crate) fn bind_read_framebuffer(&mut self, handle: Option<&WebGlFramebuffer>) {
    if self.bound_read_framebuffer.as_ref() != handle {
      self
        .ctx
        .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, handle);
      self.bound_read_framebuffer = handle.cloned();
    }
  }

  pub(crate) fn use_program(&mut self, handle: Option<&WebGlProgram>) {
    if self.current_program.as_ref() != handle {
      self.ctx.use_program(handle);
      self.current_program = handle.cloned();
    }
  }

  pub(crate) fn set_viewport(&mut self, viewport: [i32; 4]) {
    if self.viewport != viewport {
      self
        .ctx
        .viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
      self.viewport = viewport;
    }
  }

  pub(crate) fn set_clear_color(&mut self, clear_color: [f32; 4]) {
    let new_color_different = self
      .clear_color
      .iter()
      .zip(&clear_color)
      .any(|(&x, &y)| (x - y) > std::f32::EPSILON);

    if new_color_different {
      self.ctx.clear_color(
        clear_color[0],
        clear_color[1],
        clear_color[2],
        clear_color[3],
      );
      self.clear_color = clear_color;
    }
  }

  pub(crate) fn set_blending_state(&mut self, state: BlendingState) {
    if self.blending_state != state {
      match state {
        BlendingState::On => self.ctx.enable(WebGl2RenderingContext::BLEND),
        BlendingState::Off => self.ctx.disable(WebGl2RenderingContext::BLEND),
      }

      self.blending_state = state;
    }
  }

  pub(crate) fn set_blending_equation(&mut self, equation: Equation) {
    let equations = BlendingEquations {
      rgb: equation,
      alpha: equation,
    };

    if self.blending_equations != equations {
      self
        .ctx
        .blend_equation(blending_equation_to_webgl(equation));
      self.blending_equations = equations;
    }
  }

  pub(crate) fn set_blending_equation_separate(
    &mut self,
    equation_rgb: Equation,
    equation_alpha: Equation,
  ) {
    let equations = BlendingEquations {
      rgb: equation_rgb,
      alpha: equation_alpha,
    };

    if self.blending_equations != equations {
      self.ctx.blend_equation_separate(
        blending_equation_to_webgl(equation_rgb),
        blending_equation_to_webgl(equation_alpha),
      );

      self.blending_equations = equations;
    }
  }

  pub(crate) fn set_blending_func(&mut self, src: Factor, dst: Factor) {
    let funcs = BlendingFactors {
      src_rgb: src,
      dst_rgb: dst,
      src_alpha: src,
      dst_alpha: dst,
    };

    if self.blending_funcs != funcs {
      self
        .ctx
        .blend_func(blending_factor_to_webgl(src), blending_factor_to_webgl(dst));

      self.blending_funcs = funcs;
    }
  }

  pub(crate) fn set_blending_func_separate(
    &mut self,
    src_rgb: Factor,
    dst_rgb: Factor,
    src_alpha: Factor,
    dst_alpha: Factor,
  ) {
    let funcs = BlendingFactors {
      src_rgb,
      dst_rgb,
      src_alpha,
      dst_alpha,
    };
    if self.blending_funcs != funcs {
      self.ctx.blend_func_separate(
        blending_factor_to_webgl(src_rgb),
        blending_factor_to_webgl(dst_rgb),
        blending_factor_to_webgl(src_alpha),
        blending_factor_to_webgl(dst_alpha),
      );

      self.blending_funcs = funcs;
    }
  }

  pub(crate) fn set_depth_test(&mut self, depth_test: DepthTest) {
    if self.depth_test != depth_test {
      match depth_test {
        DepthTest::On => self.ctx.enable(WebGl2RenderingContext::DEPTH_TEST),
        DepthTest::Off => self.ctx.disable(WebGl2RenderingContext::DEPTH_TEST),
      }

      self.depth_test = depth_test;
    }
  }

  pub(crate) fn set_depth_test_comparison(&mut self, depth_test_comparison: DepthComparison) {
    if self.depth_test_comparison != depth_test_comparison {
      self
        .ctx
        .depth_func(depth_comparison_to_webgl(depth_test_comparison));

      self.depth_test_comparison = depth_test_comparison;
    }
  }

  pub(crate) fn set_depth_write(&mut self, depth_write: DepthWrite) {
    if self.depth_write != depth_write {
      let enabled = match depth_write {
        DepthWrite::On => true,
        DepthWrite::Off => false,
      };

      self.ctx.depth_mask(enabled);

      self.depth_write = depth_write;
    }
  }

  pub(crate) fn set_face_culling_state(&mut self, state: FaceCullingState) {
    if self.face_culling_state != state {
      match state {
        FaceCullingState::On => self.ctx.enable(WebGl2RenderingContext::CULL_FACE),
        FaceCullingState::Off => self.ctx.disable(WebGl2RenderingContext::CULL_FACE),
      }

      self.face_culling_state = state;
    }
  }

  pub(crate) fn set_face_culling_order(&mut self, order: FaceCullingOrder) {
    if self.face_culling_order != order {
      match order {
        FaceCullingOrder::CW => self.ctx.front_face(WebGl2RenderingContext::CW),
        FaceCullingOrder::CCW => self.ctx.front_face(WebGl2RenderingContext::CCW),
      }

      self.face_culling_order = order;
    }
  }

  pub(crate) fn set_face_culling_mode(&mut self, mode: FaceCullingMode) {
    if self.face_culling_mode != mode {
      match mode {
        FaceCullingMode::Front => self.ctx.cull_face(WebGl2RenderingContext::FRONT),
        FaceCullingMode::Back => self.ctx.cull_face(WebGl2RenderingContext::BACK),
        FaceCullingMode::Both => self.ctx.cull_face(WebGl2RenderingContext::FRONT_AND_BACK),
      }

      self.face_culling_mode = mode;
    }
  }
}

impl Drop for WebGL2State {
  fn drop(&mut self) {
    // drop the readback framebuffer if it was allocated
    self
      .ctx
      .delete_framebuffer(self.readback_framebuffer.as_ref());
  }
}

/// An error that might happen when the context is queried.
#[non_exhaustive]
#[derive(Debug)]
pub enum StateQueryError {
  /// The [`WebGL2State`] object is unavailable.
  ///
  /// That might occur if the current thread doesn’t support allocating a new graphics state. It
  /// might happen if you try to have more than one state on the same thread, for instance.
  ///
  /// [`WebGL2State`]: crate::webgl2::state::WebGL2State
  UnavailableWebGL2State,
  /// Unknown array buffer initial state.
  UnknownArrayBufferInitialState,
  /// Unknown viewport initial state.
  UnknownViewportInitialState,
  /// Unknown clear color initial state.
  UnknownClearColorInitialState,
  /// Corrupted blending equation.
  UnknownBlendingEquation(u32),
  /// RGB blending equation couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingEquationRGB,
  /// Alpha blending equation couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingEquationAlpha,
  /// Source RGB factor couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingSrcFactorRGB,
  /// Source alpha factor couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingSrcFactorAlpha,
  /// Destination RGB factor couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingDstFactorRGB,
  /// Destination alpha factor couldn’t be retrieved when initializing the WebGL2 state.
  CannotRetrieveBlendingDstFactorAlpha,
  /// Required WebGL extensions cannot be enabled
  CannotRetrieveRequiredWebGL2Extensions(Vec<String>),
  /// Corrupted blending source factor (RGB).
  UnknownBlendingSrcFactorRGB(u32),
  /// Corrupted blending source factor (alpha).
  UnknownBlendingSrcFactorAlpha(u32),
  /// Corrupted blending destination factor (RGB).
  UnknownBlendingDstFactorRGB(u32),
  /// Corrupted blending destination factor (alpha).
  UnknownBlendingDstFactorAlpha(u32),
  /// Corrupted face culling order.
  UnknownFaceCullingOrder,
  /// Corrupted face culling mode.
  UnknownFaceCullingMode,
}

impl fmt::Display for StateQueryError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StateQueryError::UnavailableWebGL2State => write!(f, "unavailable graphics state"),

      StateQueryError::UnknownArrayBufferInitialState => {
        write!(f, "unknown array buffer initial state")
      }

      StateQueryError::UnknownViewportInitialState => write!(f, "unknown viewport initial state"),

      StateQueryError::UnknownClearColorInitialState => {
        write!(f, "unknown clear color initial state")
      }

      StateQueryError::UnknownBlendingEquation(ref e) => {
        write!(f, "unknown blending equation: {}", e)
      }

      StateQueryError::CannotRetrieveBlendingEquationRGB => {
        f.write_str("cannot retrieve blending equation (RGB)")
      }

      StateQueryError::CannotRetrieveBlendingEquationAlpha => {
        f.write_str("cannot retrieve blending equation (alpha)")
      }

      StateQueryError::CannotRetrieveBlendingSrcFactorRGB => {
        f.write_str("cannot retrieve blending source factor (RGB)")
      }

      StateQueryError::CannotRetrieveBlendingSrcFactorAlpha => {
        f.write_str("cannot retrieve blending source factor (alpha)")
      }

      StateQueryError::CannotRetrieveBlendingDstFactorRGB => {
        f.write_str("cannot retrieve blending destination factor (RGB)")
      }

      StateQueryError::CannotRetrieveBlendingDstFactorAlpha => {
        f.write_str("cannot retrieve blending destination factor (alpha)")
      }

      StateQueryError::CannotRetrieveRequiredWebGL2Extensions(ref extensions) => write!(
        f,
        "missing WebGL2 extensions: [{}]",
        extensions.join(", ").as_str()
      ),

      StateQueryError::UnknownBlendingSrcFactorRGB(ref k) => {
        write!(f, "unknown blending source factor (RGB): {}", k)
      }

      StateQueryError::UnknownBlendingSrcFactorAlpha(ref k) => {
        write!(f, "unknown blending source factor (alpha): {}", k)
      }

      StateQueryError::UnknownBlendingDstFactorRGB(ref k) => {
        write!(f, "unknown blending destination factor (RGB): {}", k)
      }

      StateQueryError::UnknownBlendingDstFactorAlpha(ref k) => {
        write!(f, "unknown blending destination factor (alpha): {}", k)
      }

      StateQueryError::UnknownFaceCullingOrder => f.write_str("unknown face culling order"),

      StateQueryError::UnknownFaceCullingMode => f.write_str("unknown face culling mode"),
    }
  }
}

impl std::error::Error for StateQueryError {}

fn get_ctx_viewport(ctx: &mut WebGl2RenderingContext) -> Result<[i32; 4], StateQueryError> {
  let parameter = ctx
    .get_parameter(WebGl2RenderingContext::VIEWPORT)
    .map_err(|_| StateQueryError::UnknownViewportInitialState)?;
  let array: Int32Array = parameter.into();

  if array.length() != 4 {
    return Err(StateQueryError::UnknownViewportInitialState);
  }

  let mut viewport = [0; 4];
  array.copy_to(&mut viewport); // safe thanks to the test above on array.length() above

  Ok(viewport)
}

fn get_ctx_clear_color(ctx: &mut WebGl2RenderingContext) -> Result<[f32; 4], StateQueryError> {
  let parameter = ctx
    .get_parameter(WebGl2RenderingContext::COLOR_CLEAR_VALUE)
    .map_err(|_| StateQueryError::UnknownClearColorInitialState)?;
  let array: Float32Array = parameter.into();

  if array.length() != 4 {
    return Err(StateQueryError::UnknownClearColorInitialState);
  }

  let mut color = [0.0; 4];
  array.copy_to(&mut color); // safe thanks to the test above on array.length() above

  Ok(color)
}

fn get_ctx_blending_state(ctx: &mut WebGl2RenderingContext) -> BlendingState {
  if ctx.is_enabled(WebGl2RenderingContext::BLEND) {
    BlendingState::On
  } else {
    BlendingState::Off
  }
}

fn get_ctx_blending_equations(
  ctx: &mut WebGl2RenderingContext,
) -> Result<BlendingEquations, StateQueryError> {
  let rgb = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_EQUATION_RGB)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingEquationRGB)
    .and_then(map_enum_to_blending_equation)?;
  let alpha = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_EQUATION_ALPHA)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingEquationAlpha)
    .and_then(map_enum_to_blending_equation)?;

  Ok(BlendingEquations { rgb, alpha })
}

#[inline]
fn map_enum_to_blending_equation(data: u32) -> Result<Equation, StateQueryError> {
  match data {
    WebGl2RenderingContext::FUNC_ADD => Ok(Equation::Additive),
    WebGl2RenderingContext::FUNC_SUBTRACT => Ok(Equation::Subtract),
    WebGl2RenderingContext::FUNC_REVERSE_SUBTRACT => Ok(Equation::ReverseSubtract),
    WebGl2RenderingContext::MIN => Ok(Equation::Min),
    WebGl2RenderingContext::MAX => Ok(Equation::Max),
    _ => Err(StateQueryError::UnknownBlendingEquation(data)),
  }
}

fn get_ctx_blending_factors(
  ctx: &mut WebGl2RenderingContext,
) -> Result<BlendingFactors, StateQueryError> {
  let src_rgb = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_SRC_RGB)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingSrcFactorRGB)?;
  let src_rgb =
    from_gl_blending_factor(src_rgb).map_err(StateQueryError::UnknownBlendingSrcFactorRGB)?;

  let src_alpha = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_SRC_ALPHA)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingSrcFactorAlpha)?;
  let src_alpha =
    from_gl_blending_factor(src_alpha).map_err(StateQueryError::UnknownBlendingSrcFactorAlpha)?;

  let dst_rgb = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_DST_RGB)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingDstFactorRGB)?;
  let dst_rgb =
    from_gl_blending_factor(dst_rgb).map_err(StateQueryError::UnknownBlendingDstFactorRGB)?;

  let dst_alpha = get_webgl_param(ctx, WebGl2RenderingContext::BLEND_DST_ALPHA)
    .ok_or_else(|| StateQueryError::CannotRetrieveBlendingDstFactorAlpha)?;
  let dst_alpha =
    from_gl_blending_factor(dst_alpha).map_err(StateQueryError::UnknownBlendingDstFactorAlpha)?;

  Ok(BlendingFactors {
    src_rgb,
    dst_rgb,
    src_alpha,
    dst_alpha,
  })
}

#[inline]
fn from_gl_blending_factor(factor: u32) -> Result<Factor, u32> {
  match factor {
    WebGl2RenderingContext::ONE => Ok(Factor::One),
    WebGl2RenderingContext::ZERO => Ok(Factor::Zero),
    WebGl2RenderingContext::SRC_COLOR => Ok(Factor::SrcColor),
    WebGl2RenderingContext::ONE_MINUS_SRC_COLOR => Ok(Factor::SrcColorComplement),
    WebGl2RenderingContext::DST_COLOR => Ok(Factor::DestColor),
    WebGl2RenderingContext::ONE_MINUS_DST_COLOR => Ok(Factor::DestColorComplement),
    WebGl2RenderingContext::SRC_ALPHA => Ok(Factor::SrcAlpha),
    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA => Ok(Factor::SrcAlphaComplement),
    WebGl2RenderingContext::DST_ALPHA => Ok(Factor::DstAlpha),
    WebGl2RenderingContext::ONE_MINUS_DST_ALPHA => Ok(Factor::DstAlphaComplement),
    WebGl2RenderingContext::SRC_ALPHA_SATURATE => Ok(Factor::SrcAlphaSaturate),
    _ => Err(factor),
  }
}

fn get_ctx_depth_test(ctx: &mut WebGl2RenderingContext) -> DepthTest {
  let enabled = ctx.is_enabled(WebGl2RenderingContext::DEPTH_TEST);

  if enabled {
    DepthTest::On
  } else {
    DepthTest::Off
  }
}

fn get_ctx_depth_write(ctx: &mut WebGl2RenderingContext) -> DepthWrite {
  let enabled = ctx.is_enabled(WebGl2RenderingContext::DEPTH_WRITEMASK);

  if enabled {
    DepthWrite::On
  } else {
    DepthWrite::Off
  }
}

fn get_ctx_face_culling_state(ctx: &mut WebGl2RenderingContext) -> FaceCullingState {
  let enabled = ctx.is_enabled(WebGl2RenderingContext::CULL_FACE);

  if enabled {
    FaceCullingState::On
  } else {
    FaceCullingState::Off
  }
}

fn get_ctx_face_culling_order(
  ctx: &mut WebGl2RenderingContext,
) -> Result<FaceCullingOrder, StateQueryError> {
  let order = get_webgl_param(ctx, WebGl2RenderingContext::FRONT_FACE)
    .ok_or_else(|| StateQueryError::UnknownFaceCullingOrder)?;

  match order {
    WebGl2RenderingContext::CCW => Ok(FaceCullingOrder::CCW),
    WebGl2RenderingContext::CW => Ok(FaceCullingOrder::CW),
    _ => Err(StateQueryError::UnknownFaceCullingOrder),
  }
}

fn get_ctx_face_culling_mode(
  ctx: &mut WebGl2RenderingContext,
) -> Result<FaceCullingMode, StateQueryError> {
  let mode = get_webgl_param(ctx, WebGl2RenderingContext::CULL_FACE_MODE)
    .ok_or_else(|| StateQueryError::UnknownFaceCullingMode)?;

  match mode {
    WebGl2RenderingContext::FRONT => Ok(FaceCullingMode::Front),
    WebGl2RenderingContext::BACK => Ok(FaceCullingMode::Back),
    WebGl2RenderingContext::FRONT_AND_BACK => Ok(FaceCullingMode::Both),
    _ => Err(StateQueryError::UnknownFaceCullingMode),
  }
}

fn load_webgl2_extensions(ctx: &mut WebGl2RenderingContext) -> Result<(), StateQueryError> {
  let required_extensions = ["OES_texture_float_linear", "EXT_color_buffer_float"];

  let available_extensions: Vec<&str> = required_extensions
    .iter()
    .map(|ext| (*ext, ctx.get_extension(ext)))
    .flat_map(|(ext, result)| result.ok().flatten().map(|_| ext))
    .collect();

  if available_extensions.len() < required_extensions.len() {
    let missing_extensions: Vec<String> = required_extensions
      .iter()
      .filter(|e| !available_extensions.contains(e))
      .map(|e| e.to_string())
      .collect();

    return Err(StateQueryError::CannotRetrieveRequiredWebGL2Extensions(
      missing_extensions,
    ));
  }

  Ok(())
}

/// Should the binding be cached or forced to the provided value?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Bind {
  Forced,
  Cached,
}

/// Whether or not enable blending.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum BlendingState {
  /// Enable blending.
  On,
  /// Disable blending.
  Off,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BlendingFactors {
  src_rgb: Factor,
  dst_rgb: Factor,
  src_alpha: Factor,
  dst_alpha: Factor,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BlendingEquations {
  rgb: Equation,
  alpha: Equation,
}

/// Whether or not depth test should be enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum DepthTest {
  /// The depth test is enabled.
  On,
  /// The depth test is disabled.
  Off,
}

/// Should face culling be enabled?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FaceCullingState {
  /// Enable face culling.
  On,
  /// Disable face culling.
  Off,
}

#[inline]
fn depth_comparison_to_webgl(dc: DepthComparison) -> u32 {
  match dc {
    DepthComparison::Never => WebGl2RenderingContext::NEVER,
    DepthComparison::Always => WebGl2RenderingContext::ALWAYS,
    DepthComparison::Equal => WebGl2RenderingContext::EQUAL,
    DepthComparison::NotEqual => WebGl2RenderingContext::NOTEQUAL,
    DepthComparison::Less => WebGl2RenderingContext::LESS,
    DepthComparison::LessOrEqual => WebGl2RenderingContext::LEQUAL,
    DepthComparison::Greater => WebGl2RenderingContext::GREATER,
    DepthComparison::GreaterOrEqual => WebGl2RenderingContext::GEQUAL,
  }
}

#[inline]
fn blending_equation_to_webgl(equation: Equation) -> u32 {
  match equation {
    Equation::Additive => WebGl2RenderingContext::FUNC_ADD,
    Equation::Subtract => WebGl2RenderingContext::FUNC_SUBTRACT,
    Equation::ReverseSubtract => WebGl2RenderingContext::FUNC_REVERSE_SUBTRACT,
    Equation::Min => WebGl2RenderingContext::MIN,
    Equation::Max => WebGl2RenderingContext::MAX,
  }
}

#[inline]
fn blending_factor_to_webgl(factor: Factor) -> u32 {
  match factor {
    Factor::One => WebGl2RenderingContext::ONE,
    Factor::Zero => WebGl2RenderingContext::ZERO,
    Factor::SrcColor => WebGl2RenderingContext::SRC_COLOR,
    Factor::SrcColorComplement => WebGl2RenderingContext::ONE_MINUS_SRC_COLOR,
    Factor::DestColor => WebGl2RenderingContext::DST_COLOR,
    Factor::DestColorComplement => WebGl2RenderingContext::ONE_MINUS_DST_COLOR,
    Factor::SrcAlpha => WebGl2RenderingContext::SRC_ALPHA,
    Factor::SrcAlphaComplement => WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    Factor::DstAlpha => WebGl2RenderingContext::DST_ALPHA,
    Factor::DstAlphaComplement => WebGl2RenderingContext::ONE_MINUS_DST_ALPHA,
    Factor::SrcAlphaSaturate => WebGl2RenderingContext::SRC_ALPHA_SATURATE,
  }
}

/// Get a typed WebGL parameter.
fn get_webgl_param(ctx: &mut WebGl2RenderingContext, param: u32) -> Option<u32> {
  ctx
    .get_parameter(param)
    .ok()
    .and_then(|x| x.as_f64())
    .map(|x| x as u32)
}
