use gl::types::{GLboolean, GLenum, GLfloat, GLint, GLsizei, GLubyte, GLuint};
use luminance::{
  backend::{VertexEntityBackend, VertexEntityError},
  blending::{Equation, Factor},
  context::ContextActive,
  depth_stencil::{Comparison, StencilOp},
  face_culling::{FaceCullingFace, FaceCullingOrder},
  primitive::{Connector, Primitive},
  vertex::{
    Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType, VertexBufferDesc,
    VertexInstancing,
  },
  vertex_entity::{Indices, VertexEntity, Vertices},
  vertex_storage::{self, Deinterleaved, Interleaved, VertexStorage, VertexStorageVisitor},
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
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

  // Set the value if invalid or never set (and call the function).
  //
  // If the value was still valid, returns `true`.
  pub fn set_if_invalid(&mut self, value: &T, f: impl FnOnce()) -> bool {
    match self.0 {
      Some(ref x) if x == value => false,

      _ => {
        self.0 = Some(*value);
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
  buffers: HashSet<GLuint>,
  vertex_storage_data: HashMap<usize, VertexStorageData>,
  vertex_entities: HashMap<usize, VertexEntityData>,

  // binding points
  next_texture_unit: GLuint,
  free_texture_units: Vec<GLuint>,
  next_uni_buffer: GLuint,
  free_uni_buffers: Vec<GLuint>,

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

  // texture
  current_texture_unit: Cached<GLenum>,
  bound_textures: Vec<GLuint>,
  // texture pool used to optimize texture creation; regular textures typically will never ask
  // for fetching from this set but framebuffers, who often generate several textures, might use
  // this opportunity to get N textures (color, depth and stencil) at once, in a single CPU / GPU
  // roundtrip
  //
  // fishy fishy
  texture_swimming_pool: Vec<GLuint>,

  // uniform buffer
  bound_uniform_buffers: Vec<GLuint>,

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
    let buffers = HashSet::new();
    let vertex_storage_data = HashMap::new();
    let vertex_entities = HashMap::new();
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

      buffers,
      vertex_storage_data,
      vertex_entities,
      context_active,
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
      primitive_restart: vertex_restart,
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
    if self.state.borrow().context_active.is_active() {
      unsafe { gl::DeleteBuffers(1, &self.handle) };
      self.state.borrow_mut().buffers.remove(&self.handle);
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

    st.buffers.insert(handle);

    Buffer {
      handle,
      state: state.clone(),
    }
  }
}

#[derive(Debug)]
pub struct GL33 {
  state: StateRef,
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

#[derive(Debug)]
struct VertexStorageData {
  ptr: *mut u8,
}

impl VertexStorageData {
  fn new<S>(storage: S) -> Self {
    let boxed = Box::new(storage);
    let ptr = Box::leak(boxed) as *mut S as *mut u8;

    VertexStorageData { ptr }
  }

  unsafe fn into<S>(self) -> S {
    let boxed = Box::from_raw(self.ptr as *mut S);
    *boxed
  }
}

struct BuiltVertexBuffers<'a> {
  state: &'a StateRef,
  buffers: Option<VertexEntityBuffers>,
  len: usize,
  primitive_restart: bool,
}

impl<'a, 'b, V> VertexStorageVisitor<'a, V> for BuiltVertexBuffers<'b>
where
  V: Vertex,
{
  fn visit_interleaved(&mut self, storage: &'a Interleaved<V>) -> Result<(), VertexEntityError> {
    let vertices = storage.vertices();
    let len = vertices.len();

    if vertices.is_empty() {
      // no need do create a vertex buffer
      return Ok(());
    }

    let buffer = Buffer::from_vec(self.state, storage.vertices());

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
    self.buffers = Some(VertexEntityBuffers::Interleaved(buffer));
    self.len = len;
    self.primitive_restart = storage.primitive_restart();

    Ok(())
  }

  fn visit_deinterleaved(
    &mut self,
    storage: &'a Deinterleaved<V>,
  ) -> Result<(), VertexEntityError> {
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

    self.buffers = Some(VertexEntityBuffers::Deinterleaved(buffers));
    self.len = len;
    self.primitive_restart = storage.primitive_restart();

    Ok(())
  }
}

impl GL33 {
  pub fn new(state: StateRef) -> Self {
    Self { state }
  }

  fn build_vertex_buffers<V>(
    &self,
    storage: &impl VertexStorage<V>,
  ) -> Result<BuiltVertexBuffers, VertexEntityError>
  where
    V: Vertex,
  {
    let mut built = BuiltVertexBuffers {
      state: &self.state,
      buffers: None,
      len: 0,
      primitive_restart: false,
    };

    storage.visit(&mut built)?;
    Ok(built)
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
}

unsafe impl VertexEntityBackend for GL33 {
  unsafe fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, S>, VertexEntityError>
  where
    V: Vertex,
    S: Into<VertexStorage<V>>,
    I: Into<Vec<u32>>,
  {
    let indices = indices.into();

    let mut vao: GLuint = 0;

    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    self.state.borrow_mut().bound_vertex_array.set(vao);

    let built_vertex_buffers = self.build_vertex_buffers(&storage)?;
    let vertex_buffers = built_vertex_buffers.buffers;
    let vertex_count = built_vertex_buffers.len;
    let primite_restart = built_vertex_buffers.primitive_restart;

    let index_buffer = self.build_index_buffer(&indices);

    let data = VertexEntityData {
      vao,
      vertex_buffers,
      index_buffer,
    };

    let vao = vao as usize;

    let mut st = self.state.borrow_mut();
    st.vertex_entities.insert(vao, data);

    // store the storage so that we can hand it back if the user wants to map vertices again
    st.vertex_storage_data
      .insert(vao, VertexStorageData::new(storage));

    Ok(VertexEntity::new(
      vao,
      vertex_count,
      indices.len(),
      primite_restart,
    ))
  }

  unsafe fn vertex_entity_render<V, P, S>(
    &self,
    entity: &VertexEntity<V, S>,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: Into<VertexStorage<V>>,
  {
    // early return if we want zero instance
    if inst_count == 0 {
      return Ok(());
    }

    let st = self.state.borrow();
    let vao = entity.handle();
    let data = st
      .vertex_entities
      .get(&vao)
      .ok_or_else(|| VertexEntityError::Render { cause: None })?;

    let vao = entity.handle() as GLuint;
    st.bound_vertex_array.set_if_invalid(&vao, || {
      gl::BindVertexArray(vao);
    });

    if let Some(index_buffer) = data.index_buffer {
      // indexed render
      let first = (mem::size_of::<u32>() * start_index) as *const c_void;

      let primitive_restart = entity.primitive_restart();
      st.primitive_restart.set_if_invalid(&primitive_restart, || {
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

  unsafe fn vertex_entity_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Vertices<'a, V, S>, VertexEntityError>
  where
    V: Vertex,
    S: Into<VertexStorage<V>>,
  {
    todo!()
  }

  unsafe fn vertex_entity_update_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    vertices: Vertices<'a, V, S>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: Into<VertexStorage<V>>,
  {
    todo!()
  }

  unsafe fn vertex_entity_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Indices<'a>, VertexEntityError>
  where
    V: Vertex,
    S: Into<VertexStorage<V>>,
  {
    todo!()
  }

  unsafe fn vertex_entity_update_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    indices: Indices<'a>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: Into<VertexStorage<V>>,
  {
    todo!()
  }
}
