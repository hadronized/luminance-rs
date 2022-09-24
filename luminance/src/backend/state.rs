use crate::{
  blending::{Equation, Factor},
  depth_stencil::{Comparison, StencilOp},
  face_culling::{FaceCullingFace, FaceCullingOrder},
};
use std::{
  cell::RefCell,
  marker::PhantomData,
  ops::{Deref, DerefMut},
  rc::Rc,
};

/// Cached value.
///
/// A cached value is used to prevent issuing costy GPU commands if we know the target value is
/// already set to what the command tries to set. For instance, if you ask to use a texture ID
/// `34` once, that value will be set on the GPU and cached on our side. Later, if no other texture
/// setting has occurred, if you ask to use the texture ID `34` again, because the value is cached,
/// we know the GPU is already using it, so we don’t have to perform anything GPU-wise.
///
/// This optimization has limits and sometimes, because of side-effects, it is not possible to cache
/// something correctly.
#[derive(Debug)]
pub struct Cached<T>(Option<T>)
where
  T: PartialEq;

impl<T> Cached<T>
where
  T: PartialEq,
{
  /// Start with no value.
  fn empty() -> Self {
    Cached(None)
  }

  /// Explicitly invalidate a value.
  ///
  /// This is necessary when we want to be able to force a GPU command to run.
  pub fn invalidate(&mut self) {
    self.0 = None;
  }

  pub fn set(&mut self, value: T) -> Option<T> {
    self.0.replace(value)
  }

  /// Check whether the cached value is invalid regarding a value.
  ///
  /// A non-cached value (i.e. empty) is always invalid whatever compared value. If a value is already cached, then it’s
  /// invalid if it’s not equal ([`PartialEq`]) to the input value.
  pub fn is_invalid(&self, new_val: &T) -> bool {
    match &self.0 {
      Some(ref t) => t != new_val,
      _ => true,
    }
  }
}

/// Cached state.
///
/// This is a cache representation of the GPU global state.
#[derive(Debug)]
pub struct State<T> {
  _phantom: PhantomData<*const ()>, // !Send and !Sync

  // whether the associated context is still active
  pub context_active: bool,

  // backend-specific resources
  pub spec: T,

  // binding points
  pub next_texture_unit: u32,
  pub free_texture_units: Vec<u32>,
  pub next_uni_buffer: u32,
  pub free_uni_buffers: Vec<u32>,

  // viewport
  pub viewport: Cached<[i32; 4]>,

  // clear buffers
  pub clear_color: Cached<[f32; 4]>,
  pub clear_depth: Cached<f32>,
  pub clear_stencil: Cached<i32>,

  // blending
  pub blending_state: Cached<bool>,
  pub blending_rgb_equation: Cached<Equation>,
  pub blending_alpha_equation: Cached<Equation>,
  pub blending_rgb_src: Cached<Factor>,
  pub blending_rgb_dst: Cached<Factor>,
  pub blending_alpha_src: Cached<Factor>,
  pub blending_alpha_dst: Cached<Factor>,

  // depth test
  pub depth_test: Cached<bool>,
  pub depth_test_comparison: Cached<Comparison>,
  pub depth_write: Cached<bool>,

  // stencil test
  pub stencil_test: Cached<bool>,
  pub stencil_test_comparison: Cached<Comparison>,
  pub stencil_test_reference: Cached<u8>,
  pub stencil_test_mask: Cached<u8>,
  pub stencil_test_depth_passes_stencil_fails: Cached<StencilOp>,
  pub stencil_test_depth_fails_stencil_passes: Cached<StencilOp>,
  pub stencil_test_depth_pass: Cached<StencilOp>,

  // face culling
  pub face_culling: Cached<bool>,
  pub face_culling_order: Cached<FaceCullingOrder>,
  pub face_culling_face: Cached<FaceCullingFace>,

  // scissor
  pub scissor: Cached<bool>,
  pub scissor_x: Cached<u32>,
  pub scissor_y: Cached<u32>,
  pub scissor_width: Cached<u32>,
  pub scissor_height: Cached<u32>,

  // vertex restart
  pub vertex_restart: Cached<bool>,

  // patch primitive vertex number
  pub patch_vertex_nb: Cached<u32>,

  // texture
  pub current_texture_unit: Cached<i32>,
  pub bound_textures: Vec<u32>,
  // texture pool used to optimize texture creation; regular textures typically will never ask
  // for fetching from this set but framebuffers, who often generate several textures, might use
  // this opportunity to get N textures (color, depth and stencil) at once, in a single CPU / GPU
  // roundtrip
  //
  // fishy fishy
  pub texture_swimming_pool: Vec<u32>,

  // uniform buffer
  pub bound_uniform_buffers: Vec<u32>,

  // array buffer
  pub bound_array_buffer: Cached<u32>,

  // element buffer
  pub bound_element_array_buffer: Cached<u32>,

  // framebuffer
  pub bound_draw_framebuffer: Cached<u32>,

  // vertex array
  pub bound_vertex_array: Cached<u32>,

  // shader program
  pub current_program: Cached<u32>,

  // framebuffer sRGB
  pub srgb_framebuffer_enabled: Cached<bool>,

  // vendor name
  pub vendor_name: Cached<String>,

  // renderer name
  pub renderer_name: Cached<String>,

  // OpenGL version
  pub gl_version: Cached<String>,

  // GLSL version;
  pub glsl_version: Cached<String>,

  /// maximum number of elements a texture array can hold.
  pub max_texture_array_elements: Cached<usize>,
}

// TLS synchronization barrier for `GLState`.
thread_local!(static TLS_ACQUIRE_GFX_STATE: RefCell<Option<()>> = RefCell::new(Some(())));

impl<T> State<T> {
  /// Create a new [`State`]
  pub(crate) fn new(spec: T) -> Option<Self> {
    TLS_ACQUIRE_GFX_STATE.with(|rc| {
      let mut inner = rc.borrow_mut();

      inner.map(|_| {
        inner.take();
        Self::build(spec)
      })
    })
  }

  fn build(spec: T) -> Self {
    let context_active = true;
    let next_texture_unit = 0;
    let free_texture_units = Vec::new();
    let next_uni_buffer = 0;
    let free_uni_buffers = Vec::new();
    let viewport = Cached::empty();
    let clear_color = Cached::empty();
    let clear_depth = Cached::empty();
    let clear_stencil = Cached::empty();
    let blending_state = Cached::empty();
    let blending_rgb_equation = Cached::empty();
    let blending_alpha_equation = Cached::empty();
    let blending_rgb_src = Cached::empty();
    let blending_rgb_dst = Cached::empty();
    let blending_alpha_src = Cached::empty();
    let blending_alpha_dst = Cached::empty();
    let depth_test = Cached::empty();
    let depth_test_comparison = Cached::empty();
    let depth_write = Cached::empty();
    let stencil_test = Cached::empty();
    let stencil_test_comparison = Cached::empty();
    let stencil_test_reference = Cached::empty();
    let stencil_test_mask = Cached::empty();
    let stencil_test_depth_passes_stencil_fails = Cached::empty();
    let stencil_test_depth_fails_stencil_passes = Cached::empty();
    let stencil_test_depth_pass = Cached::empty();
    let face_culling = Cached::empty();
    let face_culling_order = Cached::empty();
    let face_culling_face = Cached::empty();
    let scissor = Cached::empty();
    let scissor_x = Cached::empty();
    let scissor_y = Cached::empty();
    let scissor_width = Cached::empty();
    let scissor_height = Cached::empty();
    let vertex_restart = Cached::empty();
    let patch_vertex_nb = Cached::empty();
    let current_texture_unit = Cached::empty();
    let bound_textures = vec![0; 48]; // 48 is the platform minimal requirement
    let texture_swimming_pool = Vec::new();
    let bound_uniform_buffers = vec![0; 36]; // 36 is the platform minimal requirement
    let bound_array_buffer = Cached::empty();
    let bound_element_array_buffer = Cached::empty();
    let bound_draw_framebuffer = Cached::empty();
    let bound_vertex_array = Cached::empty();
    let current_program = Cached::empty();
    let srgb_framebuffer_enabled = Cached::empty();
    let vendor_name = Cached::empty();
    let renderer_name = Cached::empty();
    let gl_version = Cached::empty();
    let glsl_version = Cached::empty();
    let max_texture_array_elements = Cached::empty();

    State {
      _phantom: PhantomData,

      context_active,
      spec,
      next_texture_unit,
      free_texture_units,
      next_uni_buffer,
      free_uni_buffers,
      viewport,
      clear_color,
      clear_depth,
      clear_stencil,
      blending_state,
      blending_rgb_equation,
      blending_alpha_equation,
      blending_rgb_src,
      blending_rgb_dst,
      blending_alpha_src,
      blending_alpha_dst,
      depth_test,
      depth_test_comparison,
      depth_write,
      stencil_test,
      stencil_test_comparison,
      stencil_test_reference,
      stencil_test_mask,
      stencil_test_depth_passes_stencil_fails,
      stencil_test_depth_fails_stencil_passes,
      stencil_test_depth_pass,
      face_culling,
      face_culling_order,
      face_culling_face,
      scissor,
      scissor_x,
      scissor_y,
      scissor_width,
      scissor_height,
      vertex_restart,
      patch_vertex_nb,
      current_texture_unit,
      bound_textures,
      texture_swimming_pool,
      bound_uniform_buffers,
      bound_array_buffer,
      bound_element_array_buffer,
      bound_draw_framebuffer,
      bound_vertex_array,
      current_program,
      srgb_framebuffer_enabled,
      vendor_name,
      renderer_name,
      gl_version,
      glsl_version,
      max_texture_array_elements,
    }
  }
}

#[derive(Debug)]
pub struct StateRef<T>(Rc<RefCell<State<T>>>);

impl<T> Clone for StateRef<T> {
  fn clone(&self) -> Self {
    StateRef(self.0.clone())
  }
}

impl<T> StateRef<T> {
  pub fn new(spec: T) -> Option<Self> {
    Some(StateRef(Rc::new(RefCell::new(State::new(spec)?))))
  }
}

impl<T> Deref for StateRef<T> {
  type Target = Rc<RefCell<State<T>>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for StateRef<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
