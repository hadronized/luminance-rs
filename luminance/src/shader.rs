pub mod types;

use crate::{
  backend::{ShaderBackend, ShaderError},
  primitive::Primitive,
  render_slots::RenderSlots,
  vertex::Vertex,
};
use std::marker::PhantomData;

pub struct ProgramBuilder<V, W, P, Q, S, E> {
  pub(crate) vertex_code: String,
  pub(crate) primitive_code: String,
  pub(crate) shading_code: String,
  _phantom: PhantomData<*const (V, W, P, Q, S, E)>,
}

impl<E> ProgramBuilder<(), (), (), (), (), E> {
  pub fn new() -> Self {
    Self {
      vertex_code: String::new(),
      primitive_code: String::new(),
      shading_code: String::new(),
      _phantom: PhantomData,
    }
  }
}

impl<P, Q, S, E> ProgramBuilder<(), (), P, Q, S, E> {
  pub fn add_vertex_stage<V, W>(self, stage: Stage<V, W, E>) -> ProgramBuilder<V, W, P, Q, S, E>
  where
    V: Vertex,
    W: Vertex,
  {
    ProgramBuilder {
      vertex_code: stage.code,
      primitive_code: self.primitive_code,
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }
}

impl<V, W, S, E> ProgramBuilder<V, W, (), (), S, E> {
  pub fn add_primitive_stage<P, Q>(self, stage: Stage<P, Q, E>) -> ProgramBuilder<V, W, P, Q, S, E>
  where
    P: Primitive<Vertex = W>,
    Q: Primitive,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: stage.code,
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }

  pub fn no_primitive_stage<P>(self) -> ProgramBuilder<V, W, P, P, S, E>
  where
    P: Primitive<Vertex = W>,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: String::new(),
      shading_code: self.shading_code,
      _phantom: PhantomData,
    }
  }
}

impl<V, W, P, Q, E> ProgramBuilder<V, W, P, Q, (), E> {
  pub fn add_shading_stage<S>(
    self,
    stage: Stage<Q::Vertex, S, E>,
  ) -> ProgramBuilder<V, W, P, Q, S, E>
  where
    Q: Primitive,
    S: RenderSlots,
  {
    ProgramBuilder {
      vertex_code: self.vertex_code,
      primitive_code: self.primitive_code,
      shading_code: stage.code,
      _phantom: PhantomData,
    }
  }
}

#[derive(Debug)]
pub struct Stage<I, O, E> {
  code: String,
  _phantom: PhantomData<*const (I, O, E)>,
}

impl<I, O, E> Stage<I, O, E> {
  pub fn new(code: impl Into<String>) -> Self {
    Self {
      code: code.into(),
      _phantom: PhantomData,
    }
  }

  pub fn code(&self) -> &str {
    &self.code
  }
}

pub struct Program<V, P, S, E> {
  handle: usize,
  pub(crate) uniforms: E,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (V, P, S, E)>,
}

impl<V, P, S, E> Program<V, P, S, E>
where
  V: Vertex,
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

impl<V, P, S, E> Drop for Program<V, P, S, E> {
  fn drop(&mut self) {
    (self.dropper)(self.handle);
  }
}

#[derive(Debug)]
pub struct Uni<T> {
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

  Boolean(UniDim),

  Matrix(UniMatDim),

  IntegralSampler(UniSamplerDim),

  UnsignedSampler(UniSamplerDim),

  FloatingSampler(UniSamplerDim),

  IntegralCubemapSampler,

  UnsignedCubemapSampler,

  FloatingCubemapSampler,
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
  type UniType;

  const UNI_TY: UniType;
  const LEN: usize;

  fn set(
    &self,
    backend: &mut impl ShaderBackend,
    uni: &Uni<Self::UniType>,
  ) -> Result<(), ShaderError>;
}

macro_rules! impl_Uniform {
  // scalar
  ($t:ty, $v:ident, $visit_fn:ident $(, $dim:path)?) => {
    impl Uniform for $t {
      type UniType = Self;

      const UNI_TY: UniType = UniType::$v $(($dim))?;
      const LEN: usize = 1;

      fn set(&self, backend: &mut impl ShaderBackend, uni: &Uni<Self::UniType>) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, self)
      }
    }
  };

  // via as_ref (vec, matrix)
  (as_ref $t:ty, $q:ty, $v:ident, $visit_fn:ident $(, $dim:path)?) => {
    impl Uniform for $t where $t: AsRef<$q> {
      type UniType = $q;

      const UNI_TY: UniType = UniType::$v $(($dim))?;
      const LEN: usize = 1;

      fn set(&self, backend: &mut impl ShaderBackend, uni: &Uni<Self::UniType>) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, self.as_ref())
      }
    }
  };

  // array version
  (array $t:ty, $visit_fn:ident) => {
    impl<const N: usize> Uniform for [$t; N] {
      type UniType = Self;

      const UNI_TY: UniType = <$t>::UNI_TY;
      const LEN: usize = N;

      fn set(&self, backend: &mut impl ShaderBackend, uni: &Uni<Self::UniType>) -> Result<(), ShaderError> {
        backend.$visit_fn(uni, self.into())
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

// TODO: samplers

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
pub struct ProgramUpdate<'a, B> {
  pub(crate) backend: &'a mut B,
  pub(crate) program_handle: usize,
}

impl<'a, B> ProgramUpdate<'a, B>
where
  B: ShaderBackend,
{
  pub fn set<T>(&mut self, uni: &Uni<T>, value: T) -> Result<(), ShaderError>
  where
    T: Uniform,
  {
    unsafe { self.backend.set_shader_uni(self.program_handle, uni, value) }
  }
}
