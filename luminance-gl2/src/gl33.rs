use core::fmt;
use gl::types::{GLboolean, GLchar, GLenum, GLfloat, GLint, GLsizei, GLubyte, GLuint};
use luminance::{
  backend::{
    FramebufferBackend, FramebufferError, PipelineBackend, PipelineError, ShaderBackend,
    ShaderError, TextureError, VertexEntityBackend, VertexEntityError,
  },
  blending::{BlendingMode, Equation, Factor},
  context::ContextActive,
  depth_stencil::{Comparison, DepthTest, DepthWrite, StencilOp, StencilTest},
  dim::{Dim, Dimensionable},
  face_culling::{FaceCulling, FaceCullingFace, FaceCullingOrder},
  framebuffer::{Back, Framebuffer},
  pipeline::{PipelineState, Viewport, WithFramebuffer, WithProgram, WithRenderState},
  pixel::{Format, Pixel, PixelFormat, Size, Type},
  primitive::{Connector, Primitive},
  render_channel::{DepthChannel, RenderChannel},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  scissor::Scissor,
  shader::{Program, Uni, Uniform, Uniforms},
  texture::{MagFilter, MinFilter, Sampler, TexelUpload, Wrap},
  vertex::{
    Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType, VertexBufferDesc,
    VertexDesc, VertexInstancing,
  },
  vertex_entity::{VertexEntity, VertexEntityView},
  vertex_storage::{AsVertexStorage, Deinterleaved, Interleaved, VertexStorage},
};
use std::{
  cell::RefCell,
  collections::HashMap,
  ffi::{c_void, CString},
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
  ptr::{self, null, null_mut},
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
pub struct Cached<T>(Option<T>);

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

  // Set the value if invalid or never set (and call the function).
  //
  // If the value was still valid, returns `true`.
  pub fn set_if_invalid(&mut self, value: T, f: impl FnOnce()) -> bool {
    match self.0 {
      Some(ref x) if x == &value => false,

      _ => {
        self.0 = Some(value);
        f();
        true
      }
    }
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
pub struct State {
  _phantom: PhantomData<*const ()>, // !Send and !Sync

  // whether the associated context is still active
  context_active: ContextActive,

  // backend-specific resources
  vertex_entities: HashMap<usize, VertexEntityData>,
  framebuffers: HashMap<usize, FramebufferData>,
  textures: HashMap<usize, TextureData>,
  texture_units: TextureUnits,
  programs: HashMap<usize, ProgramData>,

  // viewport
  viewport: Cached<[GLint; 4]>,

  // clear buffers
  clear_color: Cached<[GLfloat; 4]>,
  clear_depth: Cached<GLfloat>,
  clear_stencil: Cached<GLint>,

  // blending
  blending_state: Cached<bool>,
  blending_equations: Cached<[Equation; 2]>,
  blending_factors: Cached<[Factor; 4]>,

  // depth test
  depth_test: Cached<bool>,
  depth_test_comparison: Cached<Comparison>,
  depth_write: Cached<DepthWrite>,

  // stencil test
  stencil_test: Cached<bool>,
  stencil_func: Cached<(Comparison, GLubyte, GLubyte)>,
  stencil_ops: Cached<[StencilOp; 3]>,

  // face culling
  face_culling: Cached<bool>,
  face_culling_order: Cached<FaceCullingOrder>,
  face_culling_face: Cached<FaceCullingFace>,

  // scissor
  scissor: Cached<bool>,
  scissor_region: Cached<[GLint; 4]>,

  // vertex restart
  primitive_restart: Cached<bool>,

  // patch primitive vertex number
  patch_vertex_nb: Cached<usize>,

  // array buffer
  bound_array_buffer: Cached<GLuint>,

  // element buffer
  bound_element_array_buffer: Cached<GLuint>,

  // framebuffer
  bound_draw_framebuffer: Cached<GLuint>,

  // vertex array
  bound_vertex_array: Cached<GLuint>,

  // shader program
  current_program: Cached<GLuint>,

  // framebuffer sRGB
  srgb_framebuffer_enabled: Cached<bool>,

  // vendor name
  vendor_name: Cached<String>,

  // renderer name
  renderer_name: Cached<String>,

  // OpenGL version
  gl_version: Cached<String>,

  // GLSL version;
  glsl_version: Cached<String>,

  /// maximum number of elements a texture array can hold.
  max_texture_array_elements: Cached<usize>,
}

// TLS synchronization barrier for `GLState`.
thread_local!(static TLS_ACQUIRE_GFX_STATE: RefCell<Option<()>> = RefCell::new(Some(())));

impl State {
  /// Create a new [`State`]
  pub(crate) fn new(context_active: ContextActive) -> Option<Self> {
    TLS_ACQUIRE_GFX_STATE.with(|rc| {
      let mut inner = rc.borrow_mut();

      inner.map(|_| {
        inner.take();
        Self::build(context_active)
      })
    })
  }

  fn build(context_active: ContextActive) -> Self {
    let vertex_entities = HashMap::new();
    let framebuffers = HashMap::new();
    let textures = HashMap::new();
    let texture_units = TextureUnits::new();
    let programs = HashMap::new();
    let viewport = Cached::empty();
    let clear_color = Cached::empty();
    let clear_depth = Cached::empty();
    let clear_stencil = Cached::empty();
    let blending_state = Cached::empty();
    let blending_equations = Cached::empty();
    let blending_factors = Cached::empty();
    let depth_test = Cached::empty();
    let depth_test_comparison = Cached::empty();
    let depth_write = Cached::empty();
    let stencil_test = Cached::empty();
    let stencil_func = Cached::empty();
    let stencil_ops = Cached::empty();
    let face_culling = Cached::empty();
    let face_culling_order = Cached::empty();
    let face_culling_face = Cached::empty();
    let scissor = Cached::empty();
    let scissor_region = Cached::empty();
    let primitive_restart = Cached::empty();
    let patch_vertex_nb = Cached::empty();
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

      vertex_entities,
      framebuffers,
      textures,
      texture_units,
      programs,
      context_active,
      viewport,
      clear_color,
      clear_depth,
      clear_stencil,
      blending_state,
      blending_equations,
      blending_factors,
      depth_test,
      depth_test_comparison,
      depth_write,
      stencil_test,
      stencil_func,
      stencil_ops,
      face_culling,
      face_culling_order,
      face_culling_face,
      scissor,
      scissor_region,
      primitive_restart,
      patch_vertex_nb,
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

  fn is_context_active(&self) -> bool {
    self.context_active.is_active()
  }

  fn bind_texture(&mut self, target: GLenum, handle: usize) -> Result<(), TextureError> {
    let texture_data = self
      .textures
      .get_mut(&handle)
      .ok_or_else(|| TextureError::NoData { handle })?;

    // check whether we are already bound to a texture unit
    if let Some(unit) = texture_data.unit {
      // remove the unit from the idling ones
      self.texture_units.mark_nonidle(unit);
      Ok(())
    } else {
      // if we don’t have any unit associated with, ask one
      let (unit, old_texture_handle) = self.texture_units.get_texture_unit()?;
      texture_data.unit = Some(unit);

      // if a texture was previously bound there, remove its unit
      if let Some(handle) = old_texture_handle {
        if let Some(old_texture_data) = self.textures.get_mut(&handle) {
          old_texture_data.unit = None;
        }
      }

      // do the bind
      self
        .texture_units
        .current_texture_unit
        .set_if_invalid(unit, || unsafe {
          gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
        });

      unsafe {
        gl::BindTexture(target, handle as GLuint);
      }

      Ok(())
    }
  }

  fn idle_texture(&mut self, handle: usize) -> Result<(), TextureError> {
    let texture_data = self
      .textures
      .get_mut(&handle)
      .ok_or_else(|| TextureError::NoData { handle })?;

    if let Some(unit) = texture_data.unit {
      self.texture_units.mark_idle(unit, handle);
    }

    Ok(())
  }

  fn drop_vertex_entity(&mut self, handle: usize) {
    if self.is_context_active() {
      self.vertex_entities.remove(&handle);
    }
  }

  fn drop_framebuffer(&mut self, handle: usize) {
    if self.is_context_active() {
      self.framebuffers.remove(&handle);
    }
  }

  fn drop_program(&mut self, handle: usize) {
    if self.is_context_active() {
      self.programs.remove(&handle);
    }
  }
}

#[derive(Debug)]
pub struct StateRef(Rc<RefCell<State>>);

impl Clone for StateRef {
  fn clone(&self) -> Self {
    StateRef(self.0.clone())
  }
}

impl StateRef {
  pub fn new(context_active: ContextActive) -> Option<Self> {
    Some(StateRef(Rc::new(RefCell::new(State::new(context_active)?))))
  }
}

impl Deref for StateRef {
  type Target = Rc<RefCell<State>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for StateRef {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug)]
struct Buffer {
  handle: GLuint,
  state: StateRef,
}

impl Drop for Buffer {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &self.handle);
    }
  }
}

impl Buffer {
  fn from_vec<T>(state: &StateRef, vec: &Vec<T>) -> Self {
    let mut st = state.borrow_mut();
    let mut handle: GLuint = 0;

    unsafe {
      gl::GenBuffers(1, &mut handle);
      gl::BindBuffer(gl::ARRAY_BUFFER, handle);
    }

    st.bound_array_buffer.set(handle);

    let len = vec.len();
    let bytes = mem::size_of::<T>() * len;

    unsafe {
      gl::BufferData(
        gl::ARRAY_BUFFER,
        bytes as isize,
        vec.as_ptr() as _,
        gl::STREAM_DRAW,
      );
    }

    Buffer {
      handle,
      state: state.clone(),
    }
  }

  fn update<T>(&self, values: &[T], start: usize, len: usize) -> Result<(), BufferError> {
    self
      .state
      .borrow_mut()
      .bound_array_buffer
      .set_if_invalid(self.handle, || unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      });

    let ptr = unsafe { gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T };

    if ptr.is_null() {
      return Err(BufferError::CannotUpdate {
        handle: self.handle as _,
      });
    }

    unsafe {
      let src = values.as_ptr();
      std::ptr::copy_nonoverlapping(src, ptr.add(start), len);
    }

    Ok(())
  }
}

#[derive(Debug)]
enum VertexEntityBuffers {
  Interleaved(Buffer),
  Deinterleaved(Vec<Buffer>),
}

#[derive(Debug)]
struct VertexEntityData {
  vao: GLuint,
  vertex_buffers: Option<VertexEntityBuffers>,
  index_buffer: Option<Buffer>,
}

impl Drop for VertexEntityData {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

struct BuiltVertexBuffers {
  buffers: Option<VertexEntityBuffers>,
  len: usize,
  primitive_restart: bool,
}

#[derive(Debug)]
pub enum BufferError {
  CannotUpdate { handle: usize },
}

impl std::error::Error for BufferError {}

impl fmt::Display for BufferError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      BufferError::CannotUpdate { handle } => write!(f, "cannot slice buffer {}", handle),
    }
  }
}

#[derive(Debug)]
struct FramebufferData {
  handle: GLuint,
  renderbuffer: Option<GLuint>,
}

impl FramebufferData {
  fn validate() -> Result<(), IncompleteReason> {
    let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };

    match status {
      gl::FRAMEBUFFER_COMPLETE => Ok(()),
      gl::FRAMEBUFFER_UNDEFINED => Err(IncompleteReason::Undefined),
      gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err(IncompleteReason::IncompleteAttachment),
      gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Err(IncompleteReason::MissingAttachment),
      gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => Err(IncompleteReason::IncompleteDrawBuffer),
      gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => Err(IncompleteReason::IncompleteReadBuffer),
      gl::FRAMEBUFFER_UNSUPPORTED => Err(IncompleteReason::Unsupported),
      gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => Err(IncompleteReason::IncompleteMultisample),
      gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => Err(IncompleteReason::IncompleteLayerTargets),
      _ => panic!(
        "unknown OpenGL framebuffer incomplete status! status={}",
        status
      ),
    }
  }
}

impl Drop for FramebufferData {
  fn drop(&mut self) {
    if let Some(renderbuffer) = self.renderbuffer {
      unsafe {
        gl::DeleteRenderbuffers(1, &renderbuffer);
      }
    }

    unsafe {
      gl::DeleteFramebuffers(1, &self.handle);
    }
  }
}

/// Reason a framebuffer is incomplete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
  /// Incomplete framebuffer.
  Undefined,

  /// Incomplete attachment (color / depth).
  IncompleteAttachment,

  /// An attachment was missing.
  MissingAttachment,

  /// Incomplete draw buffer.
  IncompleteDrawBuffer,

  /// Incomplete read buffer.
  IncompleteReadBuffer,

  /// Unsupported framebuffer.
  Unsupported,

  /// Incomplete multisample configuration.
  IncompleteMultisample,

  /// Incomplete layer targets.
  IncompleteLayerTargets,

  /// Unknown reason.
  Unknown(String),
}

impl fmt::Display for IncompleteReason {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      IncompleteReason::Undefined => write!(f, "incomplete reason"),
      IncompleteReason::IncompleteAttachment => write!(f, "incomplete attachment"),
      IncompleteReason::MissingAttachment => write!(f, "missing attachment"),
      IncompleteReason::IncompleteDrawBuffer => write!(f, "incomplete draw buffer"),
      IncompleteReason::IncompleteReadBuffer => write!(f, "incomplete read buffer"),
      IncompleteReason::Unsupported => write!(f, "unsupported"),
      IncompleteReason::IncompleteMultisample => write!(f, "incomplete multisample"),
      IncompleteReason::IncompleteLayerTargets => write!(f, "incomplete layer targets"),
      IncompleteReason::Unknown(reason) => write!(f, "unknown reason: {}", reason),
    }
  }
}

impl std::error::Error for IncompleteReason {}

impl From<IncompleteReason> for FramebufferError {
  fn from(e: IncompleteReason) -> Self {
    FramebufferError::Creation {
      cause: Some(Box::new(e)),
    }
  }
}

#[derive(Debug)]
struct TextureUnits {
  current_texture_unit: Cached<usize>,
  next_texture_unit: usize,
  max_texture_units: usize,
  idling_texture_units: HashMap<usize, usize>, // texture unit -> texture handle
}

impl TextureUnits {
  fn new() -> Self {
    Self {
      current_texture_unit: Cached::empty(),
      next_texture_unit: 0,
      max_texture_units: Self::get_max_texture_units(),
      idling_texture_units: HashMap::new(),
    }
  }

  fn get_max_texture_units() -> usize {
    let mut max: GLint = 0;
    unsafe {
      gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut max);
    }
    max as _
  }

  /// Get a texture unit for binding.
  ///
  /// We always try to get a fresh texture unit, and if we can’t, we will try to use an idling one.
  fn get_texture_unit(&mut self) -> Result<(usize, Option<usize>), TextureError> {
    if self.next_texture_unit < self.max_texture_units {
      // we still can use a fresh unit
      let unit = self.next_texture_unit;
      self.next_texture_unit += 1;

      Ok((unit, None))
    } else {
      // we have exhausted the hardware texture units; try to reuse an idling one and if we cannot, then it’s an error
      self
        .reuse_texture_unit()
        .ok_or_else(|| TextureError::NotEnoughTextureUnits {
          max: self.max_texture_units,
        })
    }
  }

  /// Try to reuse a texture unit. Return `None` if no texture unit is available, and `Some((unit, old_texture_handle))`
  /// otherwise.
  fn reuse_texture_unit(&mut self) -> Option<(usize, Option<usize>)> {
    let unit = self.idling_texture_units.keys().next().cloned()?;
    let old_texture_handle = self.idling_texture_units.remove(&unit)?;

    Some((unit, Some(old_texture_handle)))
  }

  /// Mark a unit as idle.
  fn mark_idle(&mut self, unit: usize, handle: usize) {
    self.idling_texture_units.insert(unit, handle);
  }

  /// Mark a unit as non-idle.
  fn mark_nonidle(&mut self, unit: usize) {
    self.idling_texture_units.remove(&unit);
  }
}

#[derive(Debug)]
struct TextureData {
  handle: GLuint,
  target: GLenum, // “type” of the texture; used for bindings
  mipmaps: usize,
  unit: Option<usize>, // texture unit the texture is bound to
  state: StateRef,
}

impl TextureData {
  fn new<D>(
    state: &StateRef,
    target: GLenum,
    size: D::Size,
    mipmaps: usize,
    pf: PixelFormat,
    sampler: Sampler,
  ) -> Result<usize, TextureError>
  where
    D: Dimensionable,
  {
    let mut texture: GLuint = 0;
    let handle = texture as usize;

    unsafe {
      gl::GenTextures(1, &mut texture);
    }

    let texture_data = TextureData {
      handle: texture,
      target,
      mipmaps,
      unit: None,
      state: state.clone(),
    };
    state.borrow_mut().textures.insert(handle, texture_data);

    state.borrow_mut().bind_texture(target, handle as _)?;

    Self::set_texture_levels(target, mipmaps);
    Self::apply_sampler_to_texture(target, sampler);
    Self::create_texture_storage::<D>(size, mipmaps + 1, pf)?;

    state.borrow_mut().idle_texture(handle as _)?;

    Ok(handle)
  }

  fn set_texture_levels(target: GLenum, mipmaps: usize) {
    unsafe {
      gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
      gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, mipmaps as GLint);
    }
  }

  fn apply_sampler_to_texture(target: GLenum, sampler: Sampler) {
    unsafe {
      gl::TexParameteri(
        target,
        gl::TEXTURE_WRAP_R,
        GL33::opengl_wrap(sampler.wrap_r) as GLint,
      );
      gl::TexParameteri(
        target,
        gl::TEXTURE_WRAP_S,
        GL33::opengl_wrap(sampler.wrap_s) as GLint,
      );
      gl::TexParameteri(
        target,
        gl::TEXTURE_WRAP_T,
        GL33::opengl_wrap(sampler.wrap_t) as GLint,
      );
      gl::TexParameteri(
        target,
        gl::TEXTURE_MIN_FILTER,
        GL33::opengl_min_filter(sampler.min_filter) as GLint,
      );
      gl::TexParameteri(
        target,
        gl::TEXTURE_MAG_FILTER,
        GL33::opengl_mag_filter(sampler.mag_filter) as GLint,
      );

      match sampler.depth_comparison {
        Some(fun) => {
          gl::TexParameteri(
            target,
            gl::TEXTURE_COMPARE_FUNC,
            GL33::opengl_comparison(fun) as GLint,
          );
          gl::TexParameteri(
            target,
            gl::TEXTURE_COMPARE_MODE,
            gl::COMPARE_REF_TO_TEXTURE as GLint,
          );
        }
        None => {
          gl::TexParameteri(target, gl::TEXTURE_COMPARE_MODE, gl::NONE as GLint);
        }
      }
    }
  }

  fn create_texture_storage<D>(
    size: D::Size,
    levels: usize,
    pf: PixelFormat,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
  {
    match GL33::opengl_pixel_format(pf) {
      Some(glf) => {
        let (format, iformat, encoding) = glf;

        match D::dim() {
          // 1D texture
          Dim::Dim1 => {
            Self::create_texture_1d_storage(format, iformat, encoding, D::width(size), levels);
            Ok(())
          }

          // 2D texture
          Dim::Dim2 => {
            Self::create_texture_2d_storage(
              gl::TEXTURE_2D,
              format,
              iformat,
              encoding,
              D::width(size),
              D::height(size),
              levels,
            );
            Ok(())
          }

          // 3D texture
          Dim::Dim3 => {
            Self::create_texture_3d_storage(
              gl::TEXTURE_3D,
              format,
              iformat,
              encoding,
              D::width(size),
              D::height(size),
              D::depth(size),
              levels,
            );
            Ok(())
          }

          // cubemap
          Dim::Cubemap => {
            Self::create_cubemap_storage(format, iformat, encoding, D::width(size), levels);
            Ok(())
          }

          // 1D array texture
          Dim::Dim1Array => {
            Self::create_texture_2d_storage(
              gl::TEXTURE_1D_ARRAY,
              format,
              iformat,
              encoding,
              D::width(size),
              D::height(size),
              levels,
            );
            Ok(())
          }

          // 2D array texture
          Dim::Dim2Array => {
            Self::create_texture_3d_storage(
              gl::TEXTURE_2D_ARRAY,
              format,
              iformat,
              encoding,
              D::width(size),
              D::height(size),
              D::depth(size),
              levels,
            );
            Ok(())
          }
        }
      }

      None => Err(TextureError::texture_storage_creation_failed(format!(
        "unsupported texture pixel format: {:?}",
        pf
      ))),
    }
  }

  fn create_texture_1d_storage(
    format: GLenum,
    iformat: GLenum,
    encoding: GLenum,
    w: u32,
    levels: usize,
  ) {
    for level in 0..levels {
      let w = w / (1 << level as u32);

      unsafe {
        gl::TexImage1D(
          gl::TEXTURE_1D,
          level as GLint,
          iformat as GLint,
          w as GLsizei,
          0,
          format,
          encoding,
          ptr::null(),
        )
      };
    }
  }

  fn create_texture_2d_storage(
    target: GLenum,
    format: GLenum,
    iformat: GLenum,
    encoding: GLenum,
    w: u32,
    h: u32,
    levels: usize,
  ) {
    for level in 0..levels {
      let div = 1 << level as u32;
      let w = w / div;
      let h = h / div;

      unsafe {
        gl::TexImage2D(
          target,
          level as GLint,
          iformat as GLint,
          w as GLsizei,
          h as GLsizei,
          0,
          format,
          encoding,
          ptr::null(),
        )
      };
    }
  }

  fn create_texture_3d_storage(
    target: GLenum,
    format: GLenum,
    iformat: GLenum,
    encoding: GLenum,
    w: u32,
    h: u32,
    d: u32,
    levels: usize,
  ) {
    for level in 0..levels {
      let div = 1 << level as u32;
      let w = w / div;
      let h = h / div;
      let d = d / div;

      unsafe {
        gl::TexImage3D(
          target,
          level as GLint,
          iformat as GLint,
          w as GLsizei,
          h as GLsizei,
          d as GLsizei,
          0,
          format,
          encoding,
          ptr::null(),
        )
      };
    }
  }

  fn create_cubemap_storage(
    format: GLenum,
    iformat: GLenum,
    encoding: GLenum,
    s: u32,
    levels: usize,
  ) {
    for level in 0..levels {
      let s = s / (1 << level as u32);

      for face in 0..6 {
        unsafe {
          gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
            level as GLint,
            iformat as GLint,
            s as GLsizei,
            s as GLsizei,
            0,
            format,
            encoding,
            ptr::null(),
          )
        };
      }
    }
  }

  // set the unpack alignment for uploading aligned texels
  fn set_unpack_alignment(skip_bytes: usize) {
    let unpack_alignment = match skip_bytes {
      0 => 8,
      2 => 2,
      4 => 4,
      _ => 1,
    };

    unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, unpack_alignment) };
  }

  // set the pack alignment for downloading aligned texels
  fn set_pack_alignment(skip_bytes: usize) {
    let pack_alignment = match skip_bytes {
      0 => 8,
      2 => 2,
      4 => 4,
      _ => 1,
    };

    unsafe { gl::PixelStorei(gl::PACK_ALIGNMENT, pack_alignment) };
  }

  // Upload texels into the texture’s memory.
  fn upload_texels<D, P, T>(
    target: GLenum,
    off: D::Offset,
    size: D::Size,
    texels: TexelUpload<[T]>,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    let pf = P::pixel_format();
    let pf_size = pf.format.bytes_len();
    let expected_bytes = D::count(size) * pf_size;

    if let Some(base_level_texels) = texels.get_base_level() {
      // number of bytes in the input texels argument
      let input_bytes = base_level_texels.len() * mem::size_of::<T>();

      if input_bytes < expected_bytes {
        // potential segfault / overflow; abort
        return Err(TextureError::not_enough_pixels(expected_bytes, input_bytes));
      }
    }

    // set the pixel row alignment to the required value for uploading data according to the width
    // of the texture and the size of a single pixel; here, skip_bytes represents the number of bytes
    // that will be skipped
    let skip_bytes = (D::width(size) as usize * pf_size) % 8;
    Self::set_unpack_alignment(skip_bytes);

    // handle mipmaps
    match texels {
      TexelUpload::BaseLevel { texels, mipmaps } => {
        Self::set_texels::<D, _>(target, pf, 0, size, off, texels)?;

        if mipmaps > 0 {
          unsafe { gl::GenerateMipmap(target) };
        }
      }

      TexelUpload::Levels(levels) => {
        for (i, &texels) in levels.into_iter().enumerate() {
          Self::set_texels::<D, _>(target, pf, i as _, size, off, texels)?;
        }
      }

      TexelUpload::Reserve { mipmaps } => {
        if mipmaps > 0 {
          unsafe { gl::GenerateMipmap(target) };
        }
      }
    }

    Ok(())
  }

  // Set texels for a texture.
  fn set_texels<D, T>(
    target: GLenum,
    pf: PixelFormat,
    level: GLint,
    size: D::Size,
    off: D::Offset,
    texels: &[T],
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
  {
    match GL33::opengl_pixel_format(pf) {
      Some((format, _, encoding)) => match D::dim() {
        Dim::Dim1 => unsafe {
          gl::TexSubImage1D(
            target,
            level,
            D::x_offset(off) as GLint,
            D::width(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },

        Dim::Dim2 => unsafe {
          gl::TexSubImage2D(
            target,
            level,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },

        Dim::Dim3 => unsafe {
          gl::TexSubImage3D(
            target,
            level,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::z_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            D::depth(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },

        Dim::Cubemap => unsafe {
          gl::TexSubImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + D::z_offset(off),
            level,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::width(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },

        Dim::Dim1Array => unsafe {
          gl::TexSubImage2D(
            target,
            level,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },

        Dim::Dim2Array => unsafe {
          gl::TexSubImage3D(
            target,
            level,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::z_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            D::depth(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          );
        },
      },

      None => return Err(TextureError::unsupported_pixel_format(pf)),
    }

    Ok(())
  }
}

impl Drop for TextureData {
  fn drop(&mut self) {
    // ensure we mark the texture unit (if any) idle before dying
    if let Some(unit) = self.unit {
      self
        .state
        .borrow_mut()
        .texture_units
        .mark_idle(unit, self.handle as _);
    }

    unsafe {
      gl::DeleteTextures(1, &self.handle);
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StageError {
  /// Occurs when a shader fails to compile.
  CompilationFailed { ty: GLenum, reason: String },

  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(GLenum),
}

impl fmt::Display for StageError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      StageError::CompilationFailed { ty, reason } => write!(
        f,
        "stage compilation failed: type={}, reason={}",
        ty, reason
      ),
      StageError::UnsupportedType(ty) => write!(f, "unsupported stage type: {}", ty),
    }
  }
}

impl std::error::Error for StageError {}

impl From<StageError> for ShaderError {
  fn from(e: StageError) -> Self {
    ShaderError::Creation {
      cause: Some(Box::new(e)),
    }
  }
}

#[derive(Debug)]
pub struct StageHandle {
  handle: GLuint,
}

impl StageHandle {
  fn new_stage(ty: GLenum, code: &str) -> Result<Self, StageError> {
    let handle = unsafe { gl::CreateShader(ty) };

    if handle == 0 {
      return Err(StageError::CompilationFailed {
        ty,
        reason: "unable to create shader stage".to_owned(),
      });
    }

    unsafe {
      let c_code = CString::new(Self::glsl_pragma_src(code).as_bytes()).unwrap();
      gl::ShaderSource(handle, 1, [c_code.as_ptr()].as_ptr(), null());
      gl::CompileShader(handle);

      let mut compiled: GLint = gl::FALSE.into();
      gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut compiled);

      // if compilation failed, retrieve the log to insert it into the error
      if compiled == gl::FALSE.into() {
        let mut log_len: GLint = 0;
        gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetShaderInfoLog(handle, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

        gl::DeleteShader(handle);

        log.set_len(log_len as usize);

        return Err(StageError::CompilationFailed {
          ty,
          reason: String::from_utf8(log).unwrap(),
        });
      }

      Ok(StageHandle { handle })
    }
  }

  #[cfg(feature = "GL_ARB_gpu_shader_fp64")]
  const GLSL_PRAGMA: &str = "#version 330 core\n\
                           #extension GL_ARB_separate_shader_objects : require\n
                           #extension GL_ARB_gpu_shader_fp64 : require\n\
                           layout(std140) uniform;\n";
  #[cfg(not(feature = "GL_ARB_gpu_shader_fp64"))]
  const GLSL_PRAGMA: &str = "#version 330 core\n\
                           #extension GL_ARB_separate_shader_objects : require\n\
                           layout(std140) uniform;\n";

  fn glsl_pragma_src(src: &str) -> String {
    let mut pragma = String::from(Self::GLSL_PRAGMA);
    pragma.push_str(src);
    pragma
  }
}

impl Drop for StageHandle {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteShader(self.handle);
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramError {
  LinkFailed { handle: usize, reason: String },
}

impl fmt::Display for ProgramError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProgramError::LinkFailed { handle, reason } => {
        write!(f, "program {} failed to link: {}", handle, reason)
      }
    }
  }
}

impl std::error::Error for ProgramError {}

impl From<ProgramError> for ShaderError {
  fn from(e: ProgramError) -> Self {
    ShaderError::Creation {
      cause: Some(Box::new(e)),
    }
  }
}

#[derive(Debug)]
struct ProgramData {
  handle: GLuint,
}

impl Drop for ProgramData {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.handle);
    }
  }
}

impl ProgramData {
  fn link(&self) -> Result<(), ProgramError> {
    unsafe {
      gl::LinkProgram(self.handle);

      let mut linked: GLint = gl::FALSE.into();
      gl::GetProgramiv(self.handle, gl::LINK_STATUS, &mut linked);

      if linked == gl::FALSE.into() {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(self.handle, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetProgramInfoLog(
          self.handle,
          log_len,
          null_mut(),
          log.as_mut_ptr() as *mut GLchar,
        );

        log.set_len(log_len as usize);

        return Err(ProgramError::LinkFailed {
          handle: self.handle as _,
          reason: String::from_utf8(log).unwrap(),
        });
      }
    }

    Ok(())
  }

  fn bind_vertex_attribs(&self, vertex_desc: VertexDesc) -> Result<(), ProgramError> {
    let mut warnings = Vec::new();

    for desc in vertex_desc {
      match self.get_vertex_attrib_location(&desc.name) {
        Some(_) => {
          let index = desc.index as GLuint;

          // we are not interested in the location as we’re about to change it to what we’ve
          // decided in the semantics
          let c_name = CString::new(desc.name.as_bytes()).unwrap();
          unsafe { gl::BindAttribLocation(self.handle, index, c_name.as_ptr() as *const GLchar) };
        }

        None => warnings.push(format!("{} vertex attribute has no location", desc.name)),
      }
    }

    // we must link again after binding attribute location (yeah it sucks)
    self.link()
  }

  fn get_vertex_attrib_location(&self, name: &str) -> Option<GLuint> {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetAttribLocation(self.handle, c_name.as_ptr() as *const GLchar) }
    };

    if location < 0 {
      return None;
    }
    Some(location as _)
  }

  fn ask_uniform<T>(handle: GLuint, name: &str) -> Option<Uni<T>>
  where
    T: Uniform,
  {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetUniformLocation(handle, c_name.as_ptr() as *const GLchar) }
    };

    // ensure the location smells good
    if location < 0 {
      return None;
    }

    let uni = unsafe { Uni::new(location as _) };

    Some(uni)
  }
}

#[derive(Debug)]
pub struct GL33 {
  state: StateRef,
}

impl GL33 {
  pub fn new(state: StateRef) -> Self {
    // some initialization things
    Self::init();
    Self { state }
  }

  fn init() {
    unsafe { gl::PrimitiveRestartIndex(u32::MAX) };
  }

  fn build_interleaved_buffer<V>(
    &self,
    storage: &Interleaved<V>,
  ) -> Result<BuiltVertexBuffers, VertexEntityError>
  where
    V: Vertex,
  {
    let vertices = storage.vertices();
    let len = vertices.len();

    if vertices.is_empty() {
      // no need do create a vertex buffer
      return Ok(BuiltVertexBuffers {
        buffers: None,
        len: 0,
        primitive_restart: false,
      });
    }

    let buffer = Buffer::from_vec(&self.state, storage.vertices());

    // force binding as it’s meaningful when a vao is bound
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, buffer.handle);
    }
    self
      .state
      .borrow_mut()
      .bound_array_buffer
      .set(buffer.handle);

    GL33::set_vertex_pointers(&V::vertex_desc());

    Ok(BuiltVertexBuffers {
      buffers: Some(VertexEntityBuffers::Interleaved(buffer)),
      len,
      primitive_restart: storage.primitive_restart(),
    })
  }

  fn build_deinterleaved_buffers<V>(
    &self,
    storage: &Deinterleaved<V>,
  ) -> Result<BuiltVertexBuffers, VertexEntityError>
  where
    V: Vertex,
  {
    let mut len = 0;
    let buffers = storage
      .components_list()
      .iter()
      .zip(V::vertex_desc())
      .map(|(vertices, fmt)| {
        let buffer = Buffer::from_vec(&self.state, vertices);

        if len == 0 {
          len = vertices.len();
        } else if vertices.len() != len {
          return Err(VertexEntityError::Creation { cause: None });
        }

        // force binding as it’s meaningful when a vao is bound
        unsafe {
          gl::BindBuffer(gl::ARRAY_BUFFER, buffer.handle);
        }

        self
          .state
          .borrow_mut()
          .bound_array_buffer
          .set(buffer.handle);

        GL33::set_vertex_pointers(&[fmt]);

        Ok(buffer)
      })
      .collect::<Result<_, _>>()?;

    Ok(BuiltVertexBuffers {
      buffers: Some(VertexEntityBuffers::Deinterleaved(buffers)),
      len,
      primitive_restart: storage.primitive_restart(),
    })
  }

  fn build_vertex_buffers<V>(
    &self,
    storage: &mut impl AsVertexStorage<V>,
  ) -> Result<BuiltVertexBuffers, VertexEntityError>
  where
    V: Vertex,
  {
    match storage.as_vertex_storage() {
      VertexStorage::Interleaved(storage) => self.build_interleaved_buffer(storage),
      VertexStorage::Deinterleaved(storage) => self.build_deinterleaved_buffers(storage),
    }
  }

  fn build_index_buffer(&mut self, indices: &Vec<u32>) -> Option<Buffer> {
    if indices.is_empty() {
      return None;
    }

    let buffer = Buffer::from_vec(&self.state, indices);

    // force binding as it’s meaningful when a vao is bound
    unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer.handle);
    }

    self
      .state
      .borrow_mut()
      .bound_element_array_buffer
      .set(buffer.handle);

    Some(buffer)
  }

  /// Give OpenGL types information on the content of the VBO by setting vertex descriptors and pointers
  /// to buffer memory.
  fn set_vertex_pointers(descriptors: &[VertexBufferDesc]) {
    // this function sets the vertex attribute pointer for the input list by computing:
    //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
    //     matters, for short).
    //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
    //   - The offsets: each attribute has a given offset in the buffer. This is computed by
    //     accumulating the size of all previously set attributes.
    let offsets = Self::aligned_offsets(descriptors);
    let vertex_weight = Self::offset_based_vertex_weight(descriptors, &offsets) as GLsizei;

    for (desc, off) in descriptors.iter().zip(offsets) {
      Self::set_component_format(vertex_weight, off, desc);
    }
  }

  /// Compute offsets for all the vertex components according to the alignments provided.
  fn aligned_offsets(descriptor: &[VertexBufferDesc]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(descriptor.len());
    let mut off = 0;

    // compute offsets
    for desc in descriptor {
      let desc = &desc.attrib_desc;
      off = Self::off_align(off, desc.align); // keep the current component descriptor aligned
      offsets.push(off);
      off += Self::component_weight(desc); // increment the offset by the pratical size of the component
    }

    offsets
  }

  /// Align an offset.
  #[inline]
  fn off_align(off: usize, align: usize) -> usize {
    let a = align - 1;
    (off + a) & !a
  }

  /// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
  /// aligned.
  fn offset_based_vertex_weight(descriptors: &[VertexBufferDesc], offsets: &[usize]) -> usize {
    if descriptors.is_empty() || offsets.is_empty() {
      return 0;
    }

    Self::off_align(
      offsets[offsets.len() - 1]
        + Self::component_weight(&descriptors[descriptors.len() - 1].attrib_desc),
      descriptors[0].attrib_desc.align,
    )
  }

  /// Set the vertex component OpenGL pointers regarding the index of the component and the vertex
  /// stride.
  fn set_component_format(stride: GLsizei, off: usize, desc: &VertexBufferDesc) {
    let attrib_desc = &desc.attrib_desc;
    let index = desc.index as GLuint;

    unsafe {
      match attrib_desc.ty {
        VertexAttribType::Floating => {
          gl::VertexAttribPointer(
            index,
            Self::dim_as_size(attrib_desc.dim),
            Self::opengl_sized_type(&attrib_desc),
            gl::FALSE,
            stride,
            ptr::null::<c_void>().add(off),
          );
        }

        VertexAttribType::Integral(Normalized::No)
        | VertexAttribType::Unsigned(Normalized::No)
        | VertexAttribType::Boolean => {
          // non-normalized integrals / booleans
          gl::VertexAttribIPointer(
            index,
            Self::dim_as_size(attrib_desc.dim),
            Self::opengl_sized_type(&attrib_desc),
            stride,
            ptr::null::<c_void>().add(off),
          );
        }

        _ => {
          // normalized integrals
          gl::VertexAttribPointer(
            index,
            Self::dim_as_size(attrib_desc.dim),
            Self::opengl_sized_type(&attrib_desc),
            gl::TRUE,
            stride,
            ptr::null::<c_void>().add(off),
          );
        }
      }

      // set vertex attribute divisor based on the vertex instancing configuration
      let divisor = match desc.instancing {
        VertexInstancing::On => 1,
        VertexInstancing::Off => 0,
      };
      gl::VertexAttribDivisor(index, divisor);

      gl::EnableVertexAttribArray(index);
    }
  }

  /// Weight in bytes of a vertex component.
  fn component_weight(f: &VertexAttribDesc) -> usize {
    Self::dim_as_size(f.dim) as usize * f.unit_size
  }

  fn dim_as_size(d: VertexAttribDim) -> GLint {
    match d {
      VertexAttribDim::Dim1 => 1,
      VertexAttribDim::Dim2 => 2,
      VertexAttribDim::Dim3 => 3,
      VertexAttribDim::Dim4 => 4,
    }
  }

  fn opengl_sized_type(f: &VertexAttribDesc) -> GLenum {
    match (f.ty, f.unit_size) {
      (VertexAttribType::Integral(_), 1) => gl::BYTE,
      (VertexAttribType::Integral(_), 2) => gl::SHORT,
      (VertexAttribType::Integral(_), 4) => gl::INT,
      (VertexAttribType::Unsigned(_), 1) | (VertexAttribType::Boolean, 1) => gl::UNSIGNED_BYTE,
      (VertexAttribType::Unsigned(_), 2) => gl::UNSIGNED_SHORT,
      (VertexAttribType::Unsigned(_), 4) => gl::UNSIGNED_INT,
      (VertexAttribType::Floating, 4) => gl::FLOAT,
      _ => panic!("unsupported vertex component format: {:?}", f),
    }
  }

  fn opengl_connector(connector: Connector) -> GLenum {
    match connector {
      Connector::Point => gl::POINTS,
      Connector::Line => gl::LINES,
      Connector::LineStrip => gl::LINE_STRIP,
      Connector::Triangle => gl::TRIANGLES,
      Connector::TriangleFan => gl::TRIANGLE_FAN,
      Connector::TriangleStrip => gl::TRIANGLE_STRIP,
      Connector::Patch(_) => gl::PATCHES,
    }
  }

  fn opengl_wrap(wrap: Wrap) -> GLenum {
    match wrap {
      Wrap::ClampToEdge => gl::CLAMP_TO_EDGE,
      Wrap::Repeat => gl::REPEAT,
      Wrap::MirroredRepeat => gl::MIRRORED_REPEAT,
    }
  }

  fn opengl_min_filter(filter: MinFilter) -> GLenum {
    match filter {
      MinFilter::Nearest => gl::NEAREST,
      MinFilter::Linear => gl::LINEAR,
      MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
      MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
      MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
      MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
    }
  }

  fn opengl_mag_filter(filter: MagFilter) -> GLenum {
    match filter {
      MagFilter::Nearest => gl::NEAREST,
      MagFilter::Linear => gl::LINEAR,
    }
  }

  fn opengl_target(d: Dim) -> GLenum {
    match d {
      Dim::Dim1 => gl::TEXTURE_1D,
      Dim::Dim2 => gl::TEXTURE_2D,
      Dim::Dim3 => gl::TEXTURE_3D,
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP,
      Dim::Dim1Array => gl::TEXTURE_1D_ARRAY,
      Dim::Dim2Array => gl::TEXTURE_2D_ARRAY,
    }
  }

  fn opengl_comparison(dc: Comparison) -> GLenum {
    match dc {
      Comparison::Never => gl::NEVER,
      Comparison::Always => gl::ALWAYS,
      Comparison::Equal => gl::EQUAL,
      Comparison::NotEqual => gl::NOTEQUAL,
      Comparison::Less => gl::LESS,
      Comparison::LessOrEqual => gl::LEQUAL,
      Comparison::Greater => gl::GREATER,
      Comparison::GreaterOrEqual => gl::GEQUAL,
    }
  }

  fn opengl_depth_write(dw: DepthWrite) -> GLboolean {
    match dw {
      DepthWrite::On => gl::TRUE,
      DepthWrite::Off => gl::FALSE,
    }
  }

  fn opengl_stencil_op(op: StencilOp) -> GLenum {
    match op {
      StencilOp::Keep => gl::KEEP,
      StencilOp::Zero => gl::ZERO,
      StencilOp::Replace => gl::REPLACE,
      StencilOp::Increment => gl::INCR,
      StencilOp::IncrementWrap => gl::INCR_WRAP,
      StencilOp::Decrement => gl::DECR,
      StencilOp::DecrementWrap => gl::DECR_WRAP,
      StencilOp::Invert => gl::INVERT,
    }
  }

  fn opengl_face_culling_order(order: FaceCullingOrder) -> GLenum {
    match order {
      FaceCullingOrder::CW => gl::CW,
      FaceCullingOrder::CCW => gl::CCW,
    }
  }

  fn opengl_face_culling_face(face: FaceCullingFace) -> GLenum {
    match face {
      FaceCullingFace::Front => gl::FRONT,
      FaceCullingFace::Back => gl::BACK,
      FaceCullingFace::Both => gl::FRONT_AND_BACK,
    }
  }

  // OpenGL format, internal sized-format and type.
  fn opengl_pixel_format(pf: PixelFormat) -> Option<(GLenum, GLenum, GLenum)> {
    match (pf.format, pf.encoding) {
      // red channel
      (Format::R(Size::Eight), Type::NormUnsigned) => Some((gl::RED, gl::R8, gl::UNSIGNED_BYTE)),
      (Format::R(Size::Eight), Type::NormIntegral) => Some((gl::RED, gl::R8_SNORM, gl::BYTE)),
      (Format::R(Size::Eight), Type::Integral) => Some((gl::RED_INTEGER, gl::R8I, gl::BYTE)),
      (Format::R(Size::Eight), Type::Unsigned) => {
        Some((gl::RED_INTEGER, gl::R8UI, gl::UNSIGNED_BYTE))
      }

      (Format::R(Size::Sixteen), Type::NormUnsigned) => {
        Some((gl::RED_INTEGER, gl::R16, gl::UNSIGNED_SHORT))
      }
      (Format::R(Size::Sixteen), Type::NormIntegral) => {
        Some((gl::RED_INTEGER, gl::R16_SNORM, gl::SHORT))
      }
      (Format::R(Size::Sixteen), Type::Integral) => Some((gl::RED_INTEGER, gl::R16I, gl::SHORT)),
      (Format::R(Size::Sixteen), Type::Unsigned) => {
        Some((gl::RED_INTEGER, gl::R16UI, gl::UNSIGNED_SHORT))
      }

      (Format::R(Size::ThirtyTwo), Type::NormUnsigned) => {
        Some((gl::RED_INTEGER, gl::RED, gl::UNSIGNED_INT))
      }
      (Format::R(Size::ThirtyTwo), Type::NormIntegral) => Some((gl::RED_INTEGER, gl::RED, gl::INT)),
      (Format::R(Size::ThirtyTwo), Type::Integral) => Some((gl::RED_INTEGER, gl::R32I, gl::INT)),
      (Format::R(Size::ThirtyTwo), Type::Unsigned) => {
        Some((gl::RED_INTEGER, gl::R32UI, gl::UNSIGNED_INT))
      }
      (Format::R(Size::ThirtyTwo), Type::Floating) => Some((gl::RED, gl::R32F, gl::FLOAT)),

      // red, blue channels
      (Format::RG(Size::Eight, Size::Eight), Type::NormUnsigned) => {
        Some((gl::RG, gl::RG8, gl::UNSIGNED_BYTE))
      }
      (Format::RG(Size::Eight, Size::Eight), Type::NormIntegral) => {
        Some((gl::RG, gl::RG8_SNORM, gl::BYTE))
      }
      (Format::RG(Size::Eight, Size::Eight), Type::Integral) => {
        Some((gl::RG_INTEGER, gl::RG8I, gl::BYTE))
      }
      (Format::RG(Size::Eight, Size::Eight), Type::Unsigned) => {
        Some((gl::RG_INTEGER, gl::RG8UI, gl::UNSIGNED_BYTE))
      }

      (Format::RG(Size::Sixteen, Size::Sixteen), Type::NormUnsigned) => {
        Some((gl::RG, gl::RG16, gl::UNSIGNED_SHORT))
      }
      (Format::RG(Size::Sixteen, Size::Sixteen), Type::NormIntegral) => {
        Some((gl::RG, gl::RG16_SNORM, gl::SHORT))
      }
      (Format::RG(Size::Sixteen, Size::Sixteen), Type::Integral) => {
        Some((gl::RG_INTEGER, gl::RG16I, gl::SHORT))
      }
      (Format::RG(Size::Sixteen, Size::Sixteen), Type::Unsigned) => {
        Some((gl::RG_INTEGER, gl::RG16UI, gl::UNSIGNED_SHORT))
      }

      (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::NormUnsigned) => {
        Some((gl::RG, gl::RG, gl::UNSIGNED_INT))
      }
      (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::NormIntegral) => {
        Some((gl::RG, gl::RG, gl::INT))
      }
      (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => {
        Some((gl::RG_INTEGER, gl::RG32I, gl::INT))
      }
      (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => {
        Some((gl::RG_INTEGER, gl::RG32UI, gl::UNSIGNED_INT))
      }
      (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => {
        Some((gl::RG, gl::RG32F, gl::FLOAT))
      }

      // red, blue, green channels
      (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
        Some((gl::RGB, gl::RGB8, gl::UNSIGNED_BYTE))
      }
      (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => {
        Some((gl::RGB, gl::RGB8_SNORM, gl::BYTE))
      }
      (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Integral) => {
        Some((gl::RGB_INTEGER, gl::RGB8I, gl::BYTE))
      }
      (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => {
        Some((gl::RGB_INTEGER, gl::RGB8UI, gl::UNSIGNED_BYTE))
      }

      (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormUnsigned) => {
        Some((gl::RGB, gl::RGB16, gl::UNSIGNED_SHORT))
      }
      (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormIntegral) => {
        Some((gl::RGB, gl::RGB16_SNORM, gl::SHORT))
      }
      (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Integral) => {
        Some((gl::RGB_INTEGER, gl::RGB16I, gl::SHORT))
      }
      (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Unsigned) => {
        Some((gl::RGB_INTEGER, gl::RGB16UI, gl::UNSIGNED_SHORT))
      }

      (Format::RGB(Size::Eleven, Size::Eleven, Size::Ten), Type::Floating) => {
        Some((gl::RGB, gl::R11F_G11F_B10F, gl::FLOAT))
      }

      (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormUnsigned) => {
        Some((gl::RGB, gl::RGB, gl::UNSIGNED_INT))
      }
      (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormIntegral) => {
        Some((gl::RGB, gl::RGB, gl::INT))
      }
      (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => {
        Some((gl::RGB_INTEGER, gl::RGB32I, gl::INT))
      }
      (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => {
        Some((gl::RGB_INTEGER, gl::RGB32UI, gl::UNSIGNED_INT))
      }
      (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => {
        Some((gl::RGB, gl::RGB32F, gl::FLOAT))
      }

      // red, blue, green, alpha channels
      (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
        Some((gl::RGBA, gl::RGBA8, gl::UNSIGNED_BYTE))
      }
      (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => {
        Some((gl::RGBA, gl::RGBA8_SNORM, gl::BYTE))
      }
      (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Integral) => {
        Some((gl::RGBA_INTEGER, gl::RGBA8I, gl::BYTE))
      }
      (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => {
        Some((gl::RGBA_INTEGER, gl::RGBA8UI, gl::UNSIGNED_BYTE))
      }

      (
        Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
        Type::NormUnsigned,
      ) => Some((gl::RGBA, gl::RGBA16, gl::UNSIGNED_SHORT)),
      (
        Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
        Type::NormIntegral,
      ) => Some((gl::RGBA, gl::RGBA16_SNORM, gl::SHORT)),
      (
        Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
        Type::Integral,
      ) => Some((gl::RGBA_INTEGER, gl::RGBA16I, gl::SHORT)),
      (
        Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
        Type::Unsigned,
      ) => Some((gl::RGBA_INTEGER, gl::RGBA16UI, gl::UNSIGNED_SHORT)),

      (
        Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
        Type::NormUnsigned,
      ) => Some((gl::RGBA, gl::RGBA, gl::UNSIGNED_INT)),
      (
        Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
        Type::NormIntegral,
      ) => Some((gl::RGBA, gl::RGBA, gl::INT)),
      (
        Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
        Type::Integral,
      ) => Some((gl::RGBA_INTEGER, gl::RGBA32I, gl::INT)),
      (
        Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
        Type::Unsigned,
      ) => Some((gl::RGBA_INTEGER, gl::RGBA32UI, gl::UNSIGNED_INT)),
      (
        Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
        Type::Floating,
      ) => Some((gl::RGBA, gl::RGBA32F, gl::FLOAT)),

      // sRGB
      (Format::SRGB(Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
        Some((gl::RGB, gl::SRGB8, gl::UNSIGNED_BYTE))
      }
      (Format::SRGB(Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => {
        Some((gl::RGB, gl::SRGB8, gl::BYTE))
      }
      (Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
        Some((gl::RGBA, gl::SRGB8_ALPHA8, gl::UNSIGNED_BYTE))
      }
      (Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => {
        Some((gl::RGBA, gl::SRGB8_ALPHA8, gl::BYTE))
      }

      (Format::Depth(Size::ThirtyTwo), Type::Floating) => {
        Some((gl::DEPTH_COMPONENT, gl::DEPTH_COMPONENT32F, gl::FLOAT))
      }

      (Format::DepthStencil(Size::ThirtyTwo, Size::Eight), Type::Floating) => Some((
        gl::DEPTH_STENCIL,
        gl::DEPTH32F_STENCIL8,
        gl::FLOAT_32_UNSIGNED_INT_24_8_REV,
      )),

      _ => None,
    }
  }

  fn opengl_blending_equation(equation: &Equation) -> GLenum {
    match equation {
      Equation::Additive => gl::FUNC_ADD,
      Equation::Subtract => gl::FUNC_SUBTRACT,
      Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
      Equation::Min => gl::MIN,
      Equation::Max => gl::MAX,
    }
  }

  fn opengl_blending_factor(factor: &Factor) -> GLenum {
    match factor {
      Factor::One => gl::ONE,
      Factor::Zero => gl::ZERO,
      Factor::SrcColor => gl::SRC_COLOR,
      Factor::SrcColorComplement => gl::ONE_MINUS_SRC_COLOR,
      Factor::DestColor => gl::DST_COLOR,
      Factor::DestColorComplement => gl::ONE_MINUS_DST_COLOR,
      Factor::SrcAlpha => gl::SRC_ALPHA,
      Factor::SrcAlphaComplement => gl::ONE_MINUS_SRC_ALPHA,
      Factor::DstAlpha => gl::DST_ALPHA,
      Factor::DstAlphaComplement => gl::ONE_MINUS_DST_ALPHA,
      Factor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE,
    }
  }
}

unsafe impl VertexEntityBackend for GL33 {
  unsafe fn new_vertex_entity<V, P, S, I>(
    &mut self,
    mut storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, P, S>, VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
    I: Into<Vec<u32>>,
  {
    let indices = indices.into();

    let mut vao: GLuint = 0;

    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    self.state.borrow_mut().bound_vertex_array.set(vao);

    let built_vertex_buffers = self.build_vertex_buffers(&mut storage)?;
    let vertex_buffers = built_vertex_buffers.buffers;
    let vertex_count = built_vertex_buffers.len;
    let primitive_restart = built_vertex_buffers.primitive_restart;
    let index_buffer = self.build_index_buffer(&indices);

    let data = VertexEntityData {
      vao,
      vertex_buffers,
      index_buffer,
    };

    let vao = vao as usize;
    let mut st = self.state.borrow_mut();
    st.vertex_entities.insert(vao, data);

    let state = self.state.clone();
    let dropper = Box::new(move |vao| {
      state.borrow_mut().drop_vertex_entity(vao);
    });

    Ok(VertexEntity::new(
      vao,
      storage,
      indices,
      vertex_count,
      primitive_restart,
      dropper,
    ))
  }

  unsafe fn vertex_entity_render<V, P>(
    &self,
    handle: usize,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
    primitive_restart: bool,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
  {
    // early return if we want zero instance
    if inst_count == 0 {
      return Ok(());
    }

    let vao = handle as GLuint;

    self
      .state
      .borrow_mut()
      .bound_vertex_array
      .set_if_invalid(vao, || {
        gl::BindVertexArray(vao);
      });

    let st = self.state.borrow();
    let data = st
      .vertex_entities
      .get(&handle)
      .ok_or_else(|| VertexEntityError::Render { cause: None })?;

    if data.index_buffer.is_some() {
      // indexed render
      let first = (mem::size_of::<u32>() * start_index) as *const c_void;

      self
        .state
        .borrow_mut()
        .primitive_restart
        .set_if_invalid(primitive_restart, || {
          if primitive_restart {
            gl::Enable(gl::PRIMITIVE_RESTART);
          } else {
            gl::Disable(gl::PRIMITIVE_RESTART);
          }
        });

      if inst_count == 1 {
        gl::DrawElements(
          GL33::opengl_connector(P::CONNECTOR),
          vert_count as _,
          gl::UNSIGNED_INT,
          first,
        );
      } else {
        gl::DrawElementsInstanced(
          GL33::opengl_connector(P::CONNECTOR),
          vert_count as _,
          gl::UNSIGNED_INT,
          first,
          inst_count as _,
        );
      }
    } else {
      // direct render

      if inst_count == 1 {
        gl::DrawArrays(
          GL33::opengl_connector(P::CONNECTOR),
          start_index as _,
          vert_count as _,
        );
      } else {
        gl::DrawArraysInstanced(
          GL33::opengl_connector(P::CONNECTOR),
          start_index as _,
          vert_count as _,
          inst_count as _,
        );
      }
    }

    Ok(())
  }

  unsafe fn vertex_entity_update_vertices<V, S>(
    &mut self,
    handle: usize,
    storage: &mut S,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: AsVertexStorage<V>,
  {
    // get the associated data with the handle first
    let st = self.state.borrow();
    let data = st
      .vertex_entities
      .get(&handle)
      .ok_or_else(|| VertexEntityError::UpdateVertexStorage { cause: None })?;

    // allow updating only if the vertex storage has the same shape as the one used to create the vertex entity
    // (interleaved/interleaved or deinterleaved/deinterleaved)
    match (storage.as_vertex_storage(), &data.vertex_buffers) {
      (VertexStorage::Interleaved(storage), Some(VertexEntityBuffers::Interleaved(ref buffer))) => {
        let vertices = storage.vertices();
        buffer.update(&vertices, 0, vertices.len()).map_err(|e| {
          VertexEntityError::UpdateVertexStorage {
            cause: Some(Box::new(e)),
          }
        })
      }

      (
        VertexStorage::Deinterleaved(storage),
        Some(VertexEntityBuffers::Deinterleaved(buffers)),
      ) => {
        for (comp, buffer) in storage.components_list().iter().zip(buffers) {
          buffer.update(&comp, 0, comp.len()).map_err(|e| {
            VertexEntityError::UpdateVertexStorage {
              cause: Some(Box::new(e)),
            }
          })?;
        }

        Ok(())
      }

      _ => Err(VertexEntityError::UpdateVertexStorage { cause: None }),
    }
  }

  unsafe fn vertex_entity_update_indices(
    &mut self,
    handle: usize,
    indices: &mut Vec<u32>,
  ) -> Result<(), VertexEntityError> {
    // get the associated data with the handle first
    let st = self.state.borrow();
    let data = st
      .vertex_entities
      .get(&handle)
      .ok_or_else(|| VertexEntityError::UpdateIndices { cause: None })?;

    // update the index buffer if it exists
    match &data.index_buffer {
      Some(buffer) => {
        buffer
          .update(&indices, 0, indices.len())
          .map_err(|e| VertexEntityError::UpdateIndices {
            cause: Some(Box::new(e)),
          })
      }
      None => Err(VertexEntityError::UpdateIndices { cause: None }),
    }
  }
}

unsafe impl FramebufferBackend for GL33 {
  unsafe fn new_render_layer<D, RC>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: usize,
    index: usize,
  ) -> Result<RenderLayer<RC>, FramebufferError>
  where
    D: Dimensionable,
    RC: RenderChannel,
  {
    // a render layer is a texture, so there is no need to wrap it
    let pixel_format = RC::CHANNEL_TY.to_pixel_format();
    let tex = TextureData::new::<D>(
      &self.state,
      GL33::opengl_target(D::dim()),
      size,
      mipmaps,
      pixel_format,
      Sampler::default(),
    )
    .map_err(|e| FramebufferError::RenderLayerCreation {
      cause: Some(Box::new(e)),
    })?;

    // attach the texture to the framebuffer
    gl::FramebufferTexture(
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0 + index as GLenum,
      tex as GLuint,
      0,
    );

    Ok(RenderLayer::new(tex))
  }

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: usize,
  ) -> Result<RenderLayer<DC>, FramebufferError>
  where
    D: Dimensionable,
    DC: DepthChannel,
  {
    // a depth render layer is also a texture, attached to the depth attachment of the framebuffer
    let pixel_format = DC::CHANNEL_TY.to_pixel_format();
    let tex = TextureData::new::<D>(
      &self.state,
      GL33::opengl_target(D::dim()),
      size,
      mipmaps,
      pixel_format,
      Sampler::default(),
    )
    .map_err(|e| FramebufferError::RenderLayerCreation {
      cause: Some(Box::new(e)),
    })?;

    // attach the texture to the framebuffer
    gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, tex as GLuint, 0);

    Ok(RenderLayer::new(tex))
  }

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: usize,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    let mut st = self.state.borrow_mut();

    // generate the framebuffer
    let mut handle: GLuint = 0;
    gl::GenFramebuffers(1, &mut handle);
    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, handle);
    st.bound_draw_framebuffer.set(handle);

    // set the color channels
    let color_channels = RS::color_channel_descs();
    if color_channels.is_empty() {
      gl::DrawBuffer(gl::NONE);
    } else {
      let color_buf_nb = color_channels.len() as GLsizei;
      let color_buffers: Vec<_> =
        (gl::COLOR_ATTACHMENT0..gl::COLOR_ATTACHMENT0 + color_buf_nb as GLenum).collect();

      gl::DrawBuffers(color_buf_nb, color_buffers.as_ptr());
    }

    // depth channel
    let depth_channel = DS::DEPTH_CHANNEL_TY;
    let renderbuffer = if depth_channel.is_none() {
      let mut renderbuffer: GLuint = 0;

      gl::GenRenderbuffers(1, &mut renderbuffer);
      gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
      gl::RenderbufferStorage(
        gl::RENDERBUFFER,
        gl::DEPTH_COMPONENT32F,
        D::width(size) as GLsizei,
        D::height(size) as GLsizei,
      );

      gl::FramebufferRenderbuffer(
        gl::FRAMEBUFFER,
        gl::DEPTH_ATTACHMENT,
        gl::RENDERBUFFER,
        renderbuffer,
      );

      Some(renderbuffer)
    } else {
      None
    };

    let data = FramebufferData {
      handle,
      renderbuffer,
    };

    drop(st);
    let handle = handle as usize;

    // create render and depth render layers
    let layers = RS::new_render_layers::<_, D>(self, handle, size, mipmaps)?;
    let depth_layer = DS::new_depth_render_layer::<_, D>(self, handle, size, mipmaps)?;

    // validate the state of the framebuffer (the framebuffer is already bound)
    FramebufferData::validate()?;

    // dropper
    let state = self.state.clone();
    let dropper = Box::new(move |handle| {
      state.borrow_mut().drop_framebuffer(handle);
    });

    self.state.borrow_mut().framebuffers.insert(handle, data);

    Ok(Framebuffer::new(handle, size, layers, depth_layer, dropper))
  }

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, Back<RS>, Back<DS>>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    Ok(Framebuffer::new(0, size, (), (), Box::new(|_| {})))
  }
}

// a cache for implementors needing to switch from [bool; N] to [u32; N]
static mut BOOL_CACHE: Vec<u32> = Vec::new();

unsafe impl ShaderBackend for GL33 {
  unsafe fn new_program<V, P, S, E>(
    &mut self,
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
  ) -> Result<Program<V, P, S, E>, ShaderError>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: Uniforms,
  {
    // create the shader stages first
    let vertex_stage = StageHandle::new_stage(gl::VERTEX_SHADER, &vertex_code)?;

    let primitive_stage = if primitive_code.is_empty() {
      None
    } else {
      Some(StageHandle::new_stage(
        gl::GEOMETRY_SHADER,
        &primitive_code,
      )?)
    };

    let fragment_stage = StageHandle::new_stage(gl::FRAGMENT_SHADER, &shading_code)?;

    // then attach and link them all
    let handle = gl::CreateProgram();

    gl::AttachShader(handle, vertex_stage.handle);

    if let Some(primitive_stage) = primitive_stage {
      gl::AttachShader(handle, primitive_stage.handle);
    }

    gl::AttachShader(handle, fragment_stage.handle);

    let data = ProgramData { handle };
    data.link()?;
    data.bind_vertex_attribs(V::vertex_desc())?;

    // everything went okay, just track the program and let’s gooooooo
    let handle = handle as usize;
    self.state.borrow_mut().programs.insert(handle, data);

    let state = self.state.clone();
    let dropper = Box::new(move |handle| {
      state.borrow_mut().drop_program(handle);
    });

    // build the uniforms
    let uniforms = E::build_uniforms(self, handle)?;

    Ok(Program::new(handle, uniforms, dropper))
  }

  unsafe fn new_shader_uni<T>(&mut self, handle: usize, name: &str) -> Result<Uni<T>, ShaderError>
  where
    T: Uniform,
  {
    // TODO: pattern match for buffer bindings / that kind of stuff
    ProgramData::ask_uniform(handle as GLuint, name).ok_or_else(|| ShaderError::UniCreation {
      name: name.to_owned(),
      cause: None,
    })
  }

  unsafe fn new_shader_uni_unbound<T>(&mut self, _: usize) -> Result<Uni<T>, ShaderError>
  where
    T: Uniform,
  {
    Ok(Uni::new(0))
  }

  unsafe fn set_shader_uni<T>(
    &mut self,
    handle: usize,
    uni: &Uni<T>,
    value: T,
  ) -> Result<(), ShaderError>
  where
    T: Uniform,
  {
    todo!()
  }

  fn visit_i32(&mut self, uni: &Uni<i32>, value: &i32) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1i(uni.handle() as GLint, *value);
    }

    Ok(())
  }

  fn visit_u32(&mut self, uni: &Uni<u32>, value: &u32) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1ui(uni.handle() as GLint, *value);
    }

    Ok(())
  }

  fn visit_f32(&mut self, uni: &Uni<f32>, value: &f32) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1f(uni.handle() as GLint, *value);
    }

    Ok(())
  }

  fn visit_bool(&mut self, uni: &Uni<bool>, value: &bool) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1ui(uni.handle() as GLint, *value as u32);
    }

    Ok(())
  }

  fn visit_i32_array<const N: usize>(
    &mut self,
    uni: &Uni<[i32; N]>,
    value: &[i32; N],
  ) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1iv(uni.handle() as GLint, N as GLsizei, value.as_ptr());
    }

    Ok(())
  }

  fn visit_u32_array<const N: usize>(
    &mut self,
    uni: &Uni<[u32; N]>,
    value: &[u32; N],
  ) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform1uiv(uni.handle() as GLint, N as GLsizei, value.as_ptr());
    }

    Ok(())
  }

  fn visit_f32_array<const N: usize>(
    &mut self,
    uni: &Uni<[f32; N]>,
    value: &[f32; N],
  ) -> Result<(), ShaderError> {
    todo!()
  }

  fn visit_bool_array<const N: usize>(
    &mut self,
    uni: &Uni<[bool; N]>,
    value: &[bool; N],
  ) -> Result<(), ShaderError> {
    unsafe {
      BOOL_CACHE.clear();
      BOOL_CACHE.extend(value.iter().map(|x| *x as u32));

      gl::Uniform1uiv(uni.handle() as GLint, N as GLsizei, BOOL_CACHE.as_ptr());
    }

    Ok(())
  }

  fn visit_ivec2(&mut self, uni: &Uni<[i32; 2]>, value: &[i32; 2]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform2i(uni.handle() as GLint, value[0], value[1]);
    }

    Ok(())
  }

  fn visit_uvec2(&mut self, uni: &Uni<[u32; 2]>, value: &[u32; 2]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform2ui(uni.handle() as GLint, value[0], value[1]);
    }

    Ok(())
  }

  fn visit_vec2(&mut self, uni: &Uni<[f32; 2]>, value: &[f32; 2]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform2f(uni.handle() as GLint, value[0], value[1]);
    }

    Ok(())
  }

  fn visit_bvec2(&mut self, uni: &Uni<[bool; 2]>, value: &[bool; 2]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform2ui(uni.handle() as GLint, value[0] as u32, value[1] as u32);
    }

    Ok(())
  }

  fn visit_ivec3(&mut self, uni: &Uni<[i32; 3]>, value: &[i32; 3]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform3i(uni.handle() as GLint, value[0], value[1], value[2]);
    }

    Ok(())
  }

  fn visit_uvec3(&mut self, uni: &Uni<[u32; 3]>, value: &[u32; 3]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform3ui(uni.handle() as GLint, value[0], value[1], value[2]);
    }

    Ok(())
  }

  fn visit_vec3(&mut self, uni: &Uni<[f32; 3]>, value: &[f32; 3]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform3f(uni.handle() as GLint, value[0], value[1], value[2]);
    }

    Ok(())
  }

  fn visit_bvec3(&mut self, uni: &Uni<[bool; 3]>, value: &[bool; 3]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform3ui(
        uni.handle() as GLint,
        value[0] as u32,
        value[1] as u32,
        value[2] as u32,
      );
    }

    Ok(())
  }

  fn visit_ivec4(&mut self, uni: &Uni<[i32; 4]>, value: &[i32; 4]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform4i(
        uni.handle() as GLint,
        value[0],
        value[1],
        value[2],
        value[3],
      );
    }

    Ok(())
  }

  fn visit_uvec4(&mut self, uni: &Uni<[u32; 4]>, value: &[u32; 4]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform4ui(
        uni.handle() as GLint,
        value[0],
        value[1],
        value[2],
        value[3],
      );
    }

    Ok(())
  }

  fn visit_vec4(&mut self, uni: &Uni<[f32; 4]>, value: &[f32; 4]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform4f(
        uni.handle() as GLint,
        value[0],
        value[1],
        value[2],
        value[3],
      );
    }

    Ok(())
  }

  fn visit_bvec4(&mut self, uni: &Uni<[bool; 4]>, value: &[bool; 4]) -> Result<(), ShaderError> {
    unsafe {
      gl::Uniform4ui(
        uni.handle() as GLint,
        value[0] as u32,
        value[1] as u32,
        value[2] as u32,
        value[3] as u32,
      );
    }

    Ok(())
  }

  fn visit_mat22(
    &mut self,
    uni: &Uni<[[f32; 2]; 2]>,
    value: &[[f32; 2]; 2],
  ) -> Result<(), ShaderError> {
    unsafe {
      gl::UniformMatrix2fv(uni.handle() as GLint, 1, gl::FALSE, value.as_ptr() as _);
    }

    Ok(())
  }

  fn visit_mat33(
    &mut self,
    uni: &Uni<[[f32; 3]; 3]>,
    value: &[[f32; 3]; 3],
  ) -> Result<(), ShaderError> {
    unsafe {
      gl::UniformMatrix3fv(uni.handle() as GLint, 1, gl::FALSE, value.as_ptr() as _);
    }

    Ok(())
  }

  fn visit_mat44(
    &mut self,
    uni: &Uni<[[f32; 4]; 4]>,
    value: &[[f32; 4]; 4],
  ) -> Result<(), ShaderError> {
    unsafe {
      gl::UniformMatrix4fv(uni.handle() as GLint, 1, gl::FALSE, value.as_ptr() as _);
    }

    Ok(())
  }
}

unsafe impl PipelineBackend for GL33 {
  unsafe fn with_framebuffer<'a, D, CS, DS, Err>(
    &'a mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    pipeline_state: &PipelineState,
    f: impl FnOnce(WithFramebuffer<'a, Self, CS>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<PipelineError>,
  {
    let mut st = self.state.borrow_mut();

    let framebuffer_handle = framebuffer.handle() as GLuint;
    st.bound_draw_framebuffer
      .set_if_invalid(framebuffer_handle, || {
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer_handle);
      });

    match pipeline_state.viewport {
      Viewport::Whole => {
        let size = framebuffer.size();
        let viewport @ [x, y, width, height] =
          [0, 0, D::width(size) as GLint, D::height(size) as GLint];
        st.viewport.set_if_invalid(viewport, || {
          gl::Viewport(x, y, width, height);
        });
      }
      Viewport::Specific {
        x,
        y,
        width,
        height,
      } => {
        st.viewport.set_if_invalid(
          [x as GLint, y as GLint, width as GLint, height as GLint],
          || {
            gl::Viewport(x as GLint, y as GLint, width as GLint, height as GLint);
          },
        );
      }
    }

    let mut clear_buffer_bits = 0;
    if let Some(clear_color) = pipeline_state.clear_color() {
      st.clear_color.set_if_invalid(*clear_color, || {
        gl::ClearColor(
          clear_color[0],
          clear_color[1],
          clear_color[2],
          clear_color[3],
        );
      });

      clear_buffer_bits |= gl::COLOR_BUFFER_BIT;
    }

    if let Some(clear_depth) = pipeline_state.clear_depth {
      st.clear_depth.set_if_invalid(clear_depth, || {
        gl::ClearDepth(clear_depth as _);
      });

      st.depth_write.set_if_invalid(DepthWrite::On, || {
        gl::DepthMask(gl::TRUE);
      });

      clear_buffer_bits |= gl::DEPTH_BUFFER_BIT;
    }

    if let Some(clear_stencil) = pipeline_state.clear_stencil {
      st.clear_stencil.set_if_invalid(clear_stencil, || {
        gl::ClearStencil(clear_stencil);
      });

      clear_buffer_bits |= gl::STENCIL_BUFFER_BIT;
    }

    match pipeline_state.scissor() {
      Scissor::Off => {
        st.scissor.set_if_invalid(false, || {
          gl::Disable(gl::SCISSOR_TEST);
        });
      }

      Scissor::On {
        x,
        y,
        width,
        height,
      } => {
        st.scissor.set_if_invalid(true, || {
          gl::Enable(gl::SCISSOR_TEST);
        });

        let region = [*x as GLint, *y as GLint, *width as GLint, *height as GLint];
        st.scissor_region.set_if_invalid(region, || {
          gl::Scissor(region[0], region[1], region[2], region[3]);
        });
      }
    }

    if clear_buffer_bits != 0 {
      gl::Clear(clear_buffer_bits);
    }

    let srgb_enabled = pipeline_state.srgb_enabled;
    st.srgb_framebuffer_enabled
      .set_if_invalid(srgb_enabled, || {
        if srgb_enabled {
          gl::Enable(gl::FRAMEBUFFER_SRGB);
        } else {
          gl::Disable(gl::FRAMEBUFFER_SRGB);
        }
      });

    drop(st);

    f(WithFramebuffer::new(self))
  }

  unsafe fn with_program<'a, V, P, S, E, Err>(
    &'a mut self,
    program: &Program<V, P, S, E>,
    f: impl FnOnce(luminance::pipeline::WithProgram<'a, Self, V, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: Uniforms,
    Err: From<PipelineError>,
  {
    let program_handle = program.handle() as GLuint;
    self
      .state
      .borrow_mut()
      .current_program
      .set_if_invalid(program_handle, || {
        gl::UseProgram(program_handle);
      });

    f(WithProgram::new(self))
  }

  unsafe fn with_render_state<'a, V, P, Err>(
    &'a mut self,
    render_state: &luminance::render_state::RenderState,
    f: impl FnOnce(luminance::pipeline::WithRenderState<'a, Self, V, P>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    P: Primitive,
    Err: From<PipelineError>,
  {
    let mut st = self.state.borrow_mut();

    st.blending_state.set_if_invalid(true, || {
      gl::Enable(gl::BLEND);
    });

    match render_state.blending {
      BlendingMode::Combined(blending) => {
        st.blending_equations
          .set_if_invalid([blending.equation, blending.equation], || {
            gl::BlendEquation(Self::opengl_blending_equation(&blending.equation));
          });

        st.blending_factors.set_if_invalid(
          [blending.src, blending.dst, blending.src, blending.dst],
          || {
            gl::BlendFunc(
              Self::opengl_blending_factor(&blending.src),
              Self::opengl_blending_factor(&blending.dst),
            );
          },
        );
      }

      BlendingMode::Separate { rgb, alpha } => {
        st.blending_equations
          .set_if_invalid([rgb.equation, alpha.equation], || {
            gl::BlendEquationSeparate(
              Self::opengl_blending_equation(&rgb.equation),
              Self::opengl_blending_equation(&alpha.equation),
            );
          });

        st.blending_factors
          .set_if_invalid([rgb.src, rgb.dst, alpha.src, alpha.dst], || {
            gl::BlendFuncSeparate(
              Self::opengl_blending_factor(&rgb.src),
              Self::opengl_blending_factor(&rgb.dst),
              Self::opengl_blending_factor(&alpha.src),
              Self::opengl_blending_factor(&alpha.dst),
            );
          });
      }
    }

    match render_state.depth_test {
      DepthTest::Off => {
        st.depth_test.set_if_invalid(false, || {
          gl::Disable(gl::DEPTH_TEST);
        });
      }
      DepthTest::On(comparison) => {
        st.depth_test.set_if_invalid(true, || {
          gl::Enable(gl::DEPTH_TEST);
        });

        st.depth_test_comparison.set_if_invalid(comparison, || {
          gl::DepthFunc(Self::opengl_comparison(comparison));
        });
      }
    }

    st.depth_write.set_if_invalid(render_state.depth_write, || {
      gl::DepthMask(Self::opengl_depth_write(render_state.depth_write));
    });

    match render_state.stencil_test {
      StencilTest::Off => {
        st.stencil_test.set_if_invalid(false, || {
          gl::Disable(gl::STENCIL_TEST);
        });
      }

      StencilTest::On {
        comparison,
        reference,
        mask,
        depth_passes_stencil_fails,
        depth_fails_stencil_passes,
        depth_stencil_pass,
      } => {
        st.stencil_test.set_if_invalid(true, || {
          gl::Enable(gl::STENCIL_TEST);
        });

        let func = (comparison, reference, mask);
        st.stencil_func.set_if_invalid(func, || {
          gl::StencilFunc(
            Self::opengl_comparison(comparison),
            reference as GLint,
            mask as GLuint,
          );
        });

        let ops = [
          depth_passes_stencil_fails,
          depth_fails_stencil_passes,
          depth_stencil_pass,
        ];
        st.stencil_ops.set_if_invalid(ops, || {
          gl::StencilOp(
            Self::opengl_stencil_op(depth_passes_stencil_fails),
            Self::opengl_stencil_op(depth_fails_stencil_passes),
            Self::opengl_stencil_op(depth_stencil_pass),
          );
        });
      }
    }

    match render_state.face_culling {
      FaceCulling::Off => {
        st.face_culling.set_if_invalid(false, || {
          gl::Disable(gl::CULL_FACE);
        });
      }

      FaceCulling::On { order, face } => {
        st.face_culling.set_if_invalid(true, || {
          gl::Enable(gl::CULL_FACE);
        });

        st.face_culling_order.set_if_invalid(order, || {
          gl::FrontFace(Self::opengl_face_culling_order(order));
        });

        st.face_culling_face.set_if_invalid(face, || {
          gl::CullFace(Self::opengl_face_culling_face(face));
        });
      }
    }

    match render_state.scissor {
      Scissor::Off => {
        st.scissor.set_if_invalid(false, || {
          gl::Disable(gl::SCISSOR_TEST);
        });
      }
      Scissor::On {
        x,
        y,
        width,
        height,
      } => {
        st.scissor.set_if_invalid(true, || {
          gl::Enable(gl::SCISSOR_TEST);
        });

        let region = [x as GLint, y as GLint, width as GLint, height as GLint];
        st.scissor_region.set_if_invalid(region, || {
          gl::Scissor(region[0], region[1], region[2], region[3]);
        });
      }
    }

    drop(st);

    f(WithRenderState::new(self))
  }

  unsafe fn render_vertex_entity<V, P>(
    &mut self,
    view: VertexEntityView<V, P>,
  ) -> Result<(), PipelineError>
  where
    V: Vertex,
    P: Primitive,
  {
    self
      .vertex_entity_render::<V, P>(
        view.handle(),
        view.start_vertex(),
        view.vertex_count(),
        view.instance_count(),
        view.primitive_restart(),
      )
      .map_err(|e| PipelineError::RenderVertexEntity {
        start_vertex: view.start_vertex(),
        vertex_count: view.vertex_count(),
        instance_count: view.instance_count(),
        cause: Some(Box::new(e)),
      })
  }
}
