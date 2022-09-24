use std::{collections::HashSet, marker::PhantomData, ptr};

use gl::types::GLuint;
use luminance::{
  backend::{state::StateRef, VertexEntityBackend},
  vertex::Vertex,
  vertex_storage::{Interleaved, VertexStorage, Visitor},
};

#[derive(Debug)]
struct Buffer<T> {
  handle: GLuint,
  state: StateRef<Resources>,
  _phantom: PhantomData<T>,
}

impl<T> Drop for Buffer<T> {
  fn drop(&mut self) {
    if self.state.borrow().context_active {
      unsafe { gl::DeleteBuffers(1, &self.handle) };
      self.state.borrow_mut().spec.buffers.remove(&self.handle);
    }
  }
}

impl<T> Buffer<T> {
  fn from_vec(state: &StateRef<Resources>, vec: Vec<T>) -> Self {
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

    st.spec.buffers.insert(handle);

    Buffer {
      handle,
      state: state.clone(),
      _phantom: PhantomData,
    }
  }
}

#[derive(Debug)]
pub struct Resources {
  buffers: HashSet<GLuint>,
}

#[derive(Debug)]
pub struct GL33 {
  state: StateRef<Resources>,
}

impl GL33 {
  pub fn new(state: StateRef<Resources>) -> Self {
    Self { state }
  }

  pub fn build_vertex_buffer<V>(&self, s: impl VertexStorage<V>) -> Option<Buffer<V>>
  where
    V: Vertex,
  {
    let mut vertex_buffer = None;
    let fmt = V::vertex_desc();

    let mut visitor = Visitor::new(
      |interleaved: &mut Interleaved<V>| {
        let vertices = interleaved.vertices();

        if vertices.is_empty() {
          // no need do create a vertex buffer
          return;
        }

        let buffer = Buffer::from_vec(&self.state, interleaved.vertices().clone());

        // force binding as it’s meaningful when a vao is bound
        unsafe {
          gl::BindBuffer(gl::ARRAY_BUFFER, buffer.handle);
        }

        self
          .state
          .borrow_mut()
          .bound_array_buffer
          .set(buffer.handle);

        vertex_buffer = Some(buffer);
      },
      |deinterleaved: &mut Deinterleaved<V>| {},
    );

    Self::set_vertex_pointers(&fmt);

    vertex_buffer
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

  fn dim_as_size(d: VertexAttribDim) -> GLint {
    match d {
      VertexAttribDim::Dim1 => 1,
      VertexAttribDim::Dim2 => 2,
      VertexAttribDim::Dim3 => 3,
      VertexAttribDim::Dim4 => 4,
    }
  }
}

unsafe impl VertexEntityBackend for GL33 {
  unsafe fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<luminance::vertex_entity::VertexEntity<V, S>, luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
    I: Into<Vec<u32>>,
  {
    let mut vao: GLuint = 0;

    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    self.state.borrow_mut().bound_vertex_array.set(vao);
  }

  unsafe fn vertex_entity_render<V, S>(
    &self,
    entity: &luminance::vertex_entity::VertexEntity<V, S>,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
  ) -> Result<(), luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
  {
    todo!()
  }

  unsafe fn vertex_entity_vertices<'a, V, S>(
    &'a mut self,
    entity: &luminance::vertex_entity::VertexEntity<V, S>,
  ) -> Result<luminance::vertex_entity::Vertices<'a, V, S>, luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
  {
    todo!()
  }

  unsafe fn vertex_entity_update_vertices<'a, V, S>(
    &'a mut self,
    entity: &luminance::vertex_entity::VertexEntity<V, S>,
    vertices: luminance::vertex_entity::Vertices<'a, V, S>,
  ) -> Result<(), luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
  {
    todo!()
  }

  unsafe fn vertex_entity_indices<'a, V, S>(
    &'a mut self,
    entity: &luminance::vertex_entity::VertexEntity<V, S>,
  ) -> Result<luminance::vertex_entity::Indices<'a>, luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
  {
    todo!()
  }

  unsafe fn vertex_entity_update_indices<'a, V, S>(
    &'a mut self,
    entity: &luminance::vertex_entity::VertexEntity<V, S>,
    indices: luminance::vertex_entity::Indices<'a>,
  ) -> Result<(), luminance::backend::VertexEntityError>
  where
    V: luminance::vertex::Vertex,
    S: luminance::vertex_storage::VertexStorage<V>,
  {
    todo!()
  }
}
