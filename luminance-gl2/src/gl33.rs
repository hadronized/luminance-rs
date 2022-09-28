use core::fmt;
use gl::types::{GLboolean, GLenum, GLfloat, GLint, GLsizei, GLubyte, GLuint};
use luminance::{
  backend::{
    FramebufferBackend, FramebufferError, TextureError, VertexEntityBackend, VertexEntityError,
  },
  blending::{Equation, Factor},
  context::ContextActive,
  depth_stencil::{Comparison, StencilOp},
  dim::{Dim, Dimensionable},
  face_culling::{FaceCullingFace, FaceCullingOrder},
  framebuffer::Framebuffer,
  pixel::{Format, Pixel, PixelFormat, Size, Type},
  primitive::{Connector, Primitive},
  render_channel::{IsDepthChannelType, IsRenderChannelType},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  texture::{MagFilter, MinFilter, Sampler, TexelUpload, Wrap},
  vertex::{
    Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType, VertexBufferDesc,
    VertexInstancing,
  },
  vertex_entity::VertexEntity,
  vertex_storage::{AsVertexStorage, Deinterleaved, Interleaved, VertexStorage},
};
use std::{
  cell::RefCell,
  collections::HashMap,
  ffi::c_void,
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
  ptr,
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

  // viewport
  viewport: Cached<[GLint; 4]>,

  // clear buffers
  clear_color: Cached<[GLfloat; 4]>,
  clear_depth: Cached<GLfloat>,
  clear_stencil: Cached<GLint>,

  // blending
  blending_state: Cached<GLboolean>,
  blending_rgb_equation: Cached<Equation>,
  blending_alpha_equation: Cached<Equation>,
  blending_rgb_src: Cached<Factor>,
  blending_rgb_dst: Cached<Factor>,
  blending_alpha_src: Cached<Factor>,
  blending_alpha_dst: Cached<Factor>,

  // depth test
  depth_test: Cached<GLboolean>,
  depth_test_comparison: Cached<Comparison>,
  depth_write: Cached<GLboolean>,

  // stencil test
  stencil_test: Cached<GLboolean>,
  stencil_test_comparison: Cached<Comparison>,
  stencil_test_reference: Cached<GLubyte>,
  stencil_test_mask: Cached<GLubyte>,
  stencil_test_depth_passes_stencil_fails: Cached<StencilOp>,
  stencil_test_depth_fails_stencil_passes: Cached<StencilOp>,
  stencil_test_depth_pass: Cached<StencilOp>,

  // face culling
  face_culling: Cached<bool>,
  face_culling_order: Cached<FaceCullingOrder>,
  face_culling_face: Cached<FaceCullingFace>,

  // scissor
  scissor: Cached<bool>,
  scissor_x: Cached<u32>,
  scissor_y: Cached<u32>,
  scissor_width: Cached<u32>,
  scissor_height: Cached<u32>,

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
      context_active,
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
      primitive_restart: vertex_restart,
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
  ) -> Result<TextureData, TextureError>
  where
    D: Dimensionable,
  {
    let mut texture: GLuint = 0;

    unsafe {
      gl::GenTextures(1, &mut texture);
    }

    todo!("bind");

    Self::set_texture_levels(target, mipmaps);
    Self::apply_sampler_to_texture(target, sampler);
    Self::create_texture_storage::<D>(size, mipmaps + 1, pf);

    todo!()
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

#[derive(Debug)]
pub struct GL33 {
  state: StateRef,
}

impl GL33 {
  pub fn new(state: StateRef) -> Self {
    Self { state }
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

  unsafe fn vertex_entity_render<V, P, S>(
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
    S: AsVertexStorage<V>,
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
    size: D::Size,
  ) -> Result<RenderLayer<RC>, FramebufferError>
  where
    D: Dimensionable,
    RC: IsRenderChannelType,
  {
    todo!()
  }

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<DC>, FramebufferError>
  where
    D: Dimensionable,
    DC: IsDepthChannelType,
  {
    todo!()
  }

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    todo!()
  }

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    todo!()
  }
}
