pub mod types;

use crate::{
  backend::{ShaderBackend, ShaderError},
  dim::{Dim, Dimensionable},
  pixel::{self, PixelType},
  primitive::Primitive,
  render_slots::RenderSlots,
  texture::InUseTexture,
  vertex::Vertex,
};
use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

pub struct ProgramBuilder<V, W, P, S, E> {
  pub(crate) vertex_code: String,
  pub(crate) primitive_code: String,
  pub(crate) shading_code: String,
  _phantom: PhantomData<*const (V, W, P, S, E)>,
}

impl<E> ProgramBuilder<(), (), (), (), E> {
  pub fn new() -> Self {
    Self {
      vertex_code: String::new(),
      primitive_code: String::new(),
      shading_code: String::new(),
      _phantom: PhantomData,
    }
  }
}

impl<P, S, E> ProgramBuilder<(), (), P, S, E> {
  pub fn add_vertex_stage<V, W>(self, code: impl Into<String>) -> ProgramBuilder<V, W, P, S, E>
  where
    V: Vertex,
    W: Vertex,
  {
    ProgramBuilder {
      vertex_code: code.into(),
      primitive_code: self.primitive_code,
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }
}

impl<V, W, S, E> ProgramBuilder<V, W, (), S, E> {
  pub fn add_primitive_stage<P>(self, code: impl Into<String>) -> ProgramBuilder<V, W, P, S, E>
  where
    P: Primitive,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: code.into(),
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }

  pub fn no_primitive_stage<P>(self) -> ProgramBuilder<V, W, P, S, E>
  where
    P: Primitive,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: String::new(),
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }
}

impl<V, W, P, E> ProgramBuilder<V, W, P, (), E> {
  pub fn add_shading_stage<S>(self, code: impl Into<String>) -> ProgramBuilder<V, W, P, S, E>
  where
    S: RenderSlots,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: self.primitive_code,
      shading_code: code.into(),
      _phantom: PhantomData,
    }
  }
}

pub struct Program<V, W, P, S, E> {
  handle: usize,
  pub(crate) uniforms: E,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (V, W, P, S, E)>,
}

impl<V, W, P, S, E> Program<V, W, P, S, E>
where
  V: Vertex,
  W: Vertex,
  P: Primitive,
  S: RenderSlots,
{
  pub unsafe fn new(handle: usize, uniforms: E, dropper: Box<dyn FnMut(usize)>) -> Self {
    Self {
      handle,
      uniforms,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

impl<V, W, P, S, E> Drop for Program<V, W, P, S, E> {
  fn drop(&mut self) {
    (self.dropper)(self.handle);
  }
}

#[derive(Debug)]
pub struct Uni<T>
where
  T: ?Sized,
{
  handle: usize,
  _phantom: PhantomData<T>,
}

impl<T> Uni<T> {
  pub unsafe fn new(handle: usize) -> Self {
    Self {
      handle,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UniType {
  Integral(UniDim),

  Unsigned(UniDim),

  Floating(UniDim),

  #[cfg(feature = "shader-f64")]
  Floating64(UniDim),

  Boolean(UniDim),

  Matrix(UniMatDim),

  #[cfg(feature = "shader-f64")]
  Matrix64(UniMatDim),

  Sampler(pixel::Type, Dim),

  Buffer,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UniDim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UniMatDim {
  Mat22,
  Mat33,
  Mat44,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UniSamplerDim {
  Dim1,
  Dim2,
  Dim3,
  Dim1Array,
  Dim2Array,
}

pub trait Uniform {
  type Value;

  const LEN: usize;

  fn uni_type() -> UniType;

  fn set(
    backend: &mut impl ShaderBackend,
    uni: &Uni<Self>,
    value: &Self::Value,
  ) -> Result<(), ShaderError>;
}

macro_rules! impl_Uniform {
  // scalar
  ($t:ty, $v:ident, $visit_fn:ident $(, $dim:path)?) => {
    impl Uniform for $t {
      type Value = Self;

      const LEN: usize = 1;

      fn uni_type() -> UniType {
        UniType::$v $(($dim))?
      }

      fn set(backend: &mut impl ShaderBackend, uni: &Uni<Self>, value: &Self::Value) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, value)
      }
    }
  };

  // via as_ref (vec, matrix)
  (as_ref $t:ty, $q:ty, $v:ident, $visit_fn:ident $(, $dim:path)?) => {
    impl Uniform for $t {
      type Value = $t;

      const LEN: usize = 1;

      fn uni_type() -> UniType {
        UniType::$v $(($dim))?
      }

      fn set(backend: &mut impl ShaderBackend, uni: &Uni<Self>, value: &Self::Value) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, value.as_ref())
      }
    }
  };

  // array version
  (array $t:ty, $visit_fn:ident) => {
    impl<const N: usize> Uniform for [$t; N] {
      type Value = Self;

      const LEN: usize = N;

      fn uni_type() -> UniType {
        <$t>::uni_type()
      }

      fn set(backend: &mut impl ShaderBackend, uni: &Uni<Self>, value: &Self::Value) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, value.into())
      }
    }
  };
}

impl_Uniform!(i32, Integral, visit_i32, UniDim::Dim1);
impl_Uniform!(u32, Unsigned, visit_u32, UniDim::Dim1);
impl_Uniform!(f32, Floating, visit_f32, UniDim::Dim1);
impl_Uniform!(bool, Boolean, visit_bool, UniDim::Dim1);
impl_Uniform!(array i32, visit_i32_array);
impl_Uniform!(array u32, visit_u32_array);
impl_Uniform!(array f32, visit_f32_array);
impl_Uniform!(array bool, visit_bool_array);

#[cfg(feature = "shader-f64")]
impl_Uniform!(f64, Floating64, visit_f64, UniDim::Dim1);

#[cfg(feature = "shader-f64")]
impl_Uniform!(array f64, visit_f64_array);

#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector2<i32>,
  [i32; 2],
  Integral,
  visit_ivec2,
  UniDim::Dim2
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector2<u32>,
  [u32; 2],
  Unsigned,
  visit_uvec2,
  UniDim::Dim2
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector2<f32>,
  [f32; 2],
  Floating,
  visit_vec2,
  UniDim::Dim2
);
#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::Vector2<f64>,
  [f64; 2],
  Floating64,
  visit_dvec2,
  UniDim::Dim2
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector2<bool>,
  [bool; 2],
  Boolean,
  visit_bvec2,
  UniDim::Dim2
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector3<i32>,
  [i32; 3],
  Integral,
  visit_ivec3,
  UniDim::Dim3
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector3<u32>,
  [u32; 3],
  Unsigned,
  visit_uvec3,
  UniDim::Dim3
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector3<f32>,
  [f32; 3],
  Floating,
  visit_vec3,
  UniDim::Dim3
);
#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::Vector3<f64>,
  [f64; 3],
  Floating64,
  visit_dvec3,
  UniDim::Dim3
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector3<bool>,
  [bool; 3],
  Boolean,
  visit_bvec3,
  UniDim::Dim3
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector4<i32>,
  [i32; 4],
  Integral,
  visit_ivec4,
  UniDim::Dim4
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector4<u32>,
  [u32; 4],
  Unsigned,
  visit_uvec4,
  UniDim::Dim4
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector4<f32>,
  [f32; 4],
  Floating,
  visit_vec4,
  UniDim::Dim4
);
#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::Vector4<f64>,
  [f64; 4],
  Floating64,
  visit_dvec4,
  UniDim::Dim4
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::Vector4<bool>,
  [bool; 4],
  Boolean,
  visit_bvec4,
  UniDim::Dim4
);

#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix2<f32>,
  [[f32; 2]; 2],
  Matrix,
  visit_mat22,
  UniMatDim::Mat22
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix3<f32>,
  [[f32; 3]; 3],
  Matrix,
  visit_mat33,
  UniMatDim::Mat33
);
#[cfg(feature = "mint")]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix4<f32>,
  [[f32; 4]; 4],
  Matrix,
  visit_mat44,
  UniMatDim::Mat44
);

#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix2<f64>,
  [[f64; 2]; 2],
  Matrix64,
  visit_dmat22,
  UniMatDim::Mat22
);
#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix3<f64>,
  [[f64; 3]; 3],
  Matrix64,
  visit_dmat33,
  UniMatDim::Mat33
);
#[cfg(all(feature = "mint", feature = "shader-f64"))]
impl_Uniform!(
  as_ref
  mint::ColumnMatrix4<f64>,
  [[f64; 4]; 4],
  Matrix64,
  visit_dmat44,
  UniMatDim::Mat44
);

// FIXME: I think we should be using Texture<D, P> and use InUseTexture<D, P::Type> as Value?!
impl<D, P> Uniform for InUseTexture<D, P>
where
  D: Dimensionable,
  P: PixelType,
{
  type Value = InUseTexture<D, P>;

  const LEN: usize = 1;

  fn uni_type() -> UniType {
    UniType::Sampler(P::pixel_type(), D::dim())
  }

  fn set(
    backend: &mut impl ShaderBackend,
    uni: &Uni<Self>,
    value: &Self::Value,
  ) -> Result<(), ShaderError> {
    backend.visit_texture(uni, value)
  }
}

impl<T, Scheme> Uniform for UniBuffer<T, Scheme>
where
  T: MemoryLayout<Scheme>,
{
  type Value = InUseUniBuffer<T, Scheme>;

  const LEN: usize = 1;

  fn uni_type() -> UniType {
    UniType::Buffer
  }

  fn set(
    backend: &mut impl ShaderBackend,
    uni: &Uni<Self>,
    value: &Self::Value,
  ) -> Result<(), ShaderError> {
    backend.visit_uni_buffer(uni, value)
  }
}

pub trait Uniforms: Sized {
  fn build_uniforms<B>(backend: &mut B, program_handle: usize) -> Result<Self, ShaderError>
  where
    B: ShaderBackend;
}

impl Uniforms for () {
  fn build_uniforms<B>(_: &mut B, _: usize) -> Result<Self, ShaderError>
  where
    B: ShaderBackend,
  {
    Ok(())
  }
}

#[derive(Debug)]
pub enum Std140 {}

#[derive(Debug)]
pub enum Std430 {}

pub unsafe trait MemoryLayout<Scheme>: Sized {
  /// The aligned associated type.
  type Aligned: From<Self>;
}

/// Aligment rules.
pub unsafe trait MemoryAlign<Scheme> {
  const ALIGNMENT: usize;
}

macro_rules! impl_MemoryAlign {
  ($t:ty, $scheme:ty, $alignment:expr) => {
    unsafe impl MemoryAlign<$scheme> for $t {
      const ALIGNMENT: usize = $alignment;
    }
  };
}

impl_MemoryAlign!(f32, Std140, 4);
impl_MemoryAlign!(u32, Std140, 4);
impl_MemoryAlign!(i32, Std140, 4);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<f32>, Std140, 8);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<u32>, Std140, 8);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<i32>, Std140, 8);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<f32>, Std140, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<u32>, Std140, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<i32>, Std140, 16);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<f32>, Std140, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<u32>, Std140, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<i32>, Std140, 16);

unsafe impl<T, const N: usize> MemoryAlign<Std140> for [T; N]
where
  T: MemoryAlign<Std140>,
{
  // T::ALIGNMENT rounded-up to the next 16 multiple (alignment of vec4)
  const ALIGNMENT: usize = (T::ALIGNMENT + 15) & !15;
}

impl_MemoryAlign!(f32, Std430, 4);
impl_MemoryAlign!(u32, Std430, 4);
impl_MemoryAlign!(i32, Std430, 4);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<f32>, Std430, 8);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<u32>, Std430, 8);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector2<i32>, Std430, 8);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<f32>, Std430, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<u32>, Std430, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector3<i32>, Std430, 16);

#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<f32>, Std430, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<u32>, Std430, 16);
#[cfg(feature = "mint")]
impl_MemoryAlign!(mint::Vector4<i32>, Std430, 16);

unsafe impl<T, const N: usize> MemoryAlign<Std430> for [T; N]
where
  T: MemoryAlign<Std430>,
{
  const ALIGNMENT: usize = T::ALIGNMENT;
}

pub struct UniBuffer<T, Scheme> {
  handle: usize,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (T, Scheme)>,
}

impl<T, Scheme> UniBuffer<T, Scheme> {
  pub unsafe fn new(handle: usize, dropper: Box<dyn FnMut(usize)>) -> Self {
    Self {
      handle,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

impl<T, Scheme> Drop for UniBuffer<T, Scheme> {
  fn drop(&mut self) {
    (self.dropper)(self.handle)
  }
}

pub struct InUseUniBuffer<T, Scheme> {
  handle: usize,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (T, Scheme)>,
}

impl<T, Scheme> InUseUniBuffer<T, Scheme> {
  pub unsafe fn new(handle: usize, dropper: Box<dyn FnMut(usize)>) -> Self {
    Self {
      handle,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

impl<T, Scheme> Drop for InUseUniBuffer<T, Scheme> {
  fn drop(&mut self) {
    (self.dropper)(self.handle)
  }
}

#[derive(Debug)]
pub struct UniBufferRef<'a, B, T, Scheme>
where
  B: ?Sized + ShaderBackend,
  T: MemoryLayout<Scheme>,
{
  backend: &'a mut B,
  uni_buffer_handle: usize,
  ptr: *mut T,
  _phantom: PhantomData<*const Scheme>,
}

impl<'a, B, T, Scheme> UniBufferRef<'a, B, T, Scheme>
where
  B: ?Sized + ShaderBackend,
  T: MemoryLayout<Scheme>,
{
  pub unsafe fn new(backend: &'a mut B, uni_buffer_handle: usize, ptr: *mut T) -> Self {
    Self {
      backend,
      uni_buffer_handle,
      ptr,
      _phantom: PhantomData,
    }
  }

  pub fn sync(self) {
    // will just drop and automatically sync; magic!
  }
}

impl<'a, B, T, Scheme> Drop for UniBufferRef<'a, B, T, Scheme>
where
  B: ?Sized + ShaderBackend,
  T: MemoryLayout<Scheme>,
{
  fn drop(&mut self) {
    unsafe {
      // NOTE: maybe we want an optional dependency on log here? not sure
      let _ = self
        .backend
        .unsync_uni_buffer::<T, Scheme>(self.uni_buffer_handle);
    }
  }
}

impl<'a, B, T, Scheme> Deref for UniBufferRef<'a, B, T, Scheme>
where
  B: ?Sized + ShaderBackend,
  T: MemoryLayout<Scheme>,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl<'a, B, T, Scheme> DerefMut for UniBufferRef<'a, B, T, Scheme>
where
  B: ?Sized + ShaderBackend,
  T: MemoryLayout<Scheme>,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

#[derive(Debug)]
pub struct ProgramUpdate<'a, B> {
  pub(crate) backend: &'a mut B,
  pub(crate) program_handle: usize,
}

impl<'a, B> ProgramUpdate<'a, B>
where
  B: ShaderBackend,
{
  pub fn set<T>(&mut self, uni: &Uni<T>, value: &T::Value) -> Result<(), ShaderError>
  where
    T: Uniform,
  {
    unsafe { self.backend.set_shader_uni(self.program_handle, uni, value) }
  }

  pub fn query_set<T>(&mut self, name: impl AsRef<str>, value: &T::Value) -> Result<(), ShaderError>
  where
    T: Uniform,
  {
    let uni = unsafe {
      self
        .backend
        .new_shader_uni::<T>(self.program_handle, name.as_ref())?
    };

    self.set(&uni, value)
  }
}
