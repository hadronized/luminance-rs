pub mod types;

use crate::{
  backend::{Backend, ShaderError},
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
  pub fn add_primitive_code<P, Q>(self, stage: Stage<P, Q, E>) -> ProgramBuilder<V, W, P, Q, S, E>
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
  pub fn add_shading_code<S>(
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

#[derive(Debug)]
pub struct Program<V, P, S, E> {
  handle: usize,
  pub(crate) environment: E,
  _phantom: PhantomData<*const (V, P, S, E)>,
}

impl<V, P, S, E> Program<V, P, S, E> {
  pub unsafe fn new(handle: usize, environment: E) -> Self {
    Self {
      handle,
      environment,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

#[derive(Debug)]
pub struct Env<T> {
  handle: usize,
  _phantom: PhantomData<T>,
}

impl<T> Env<T> {
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
pub enum EnvType {
  Integral(EnvDim),

  Unsigned(EnvDim),

  Floating(EnvDim),

  Boolean(EnvDim),

  Matrix(EnvMatDim),

  IntegralSampler(EnvSamplerDim),

  UnsignedSampler(EnvSamplerDim),

  FloatingSampler(EnvSamplerDim),

  IntegralCubemapSampler,

  UnsignedCubemapSampler,

  FloatingCubemapSampler,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvDim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvMatDim {
  Mat22,
  Mat33,
  Mat44,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvSamplerDim {
  Dim1,
  Dim2,
  Dim3,
  Dim1Array,
  Dim2Array,
}

pub trait IsEnv {
  const ENV_TY: EnvType;
}

macro_rules! impl_IsEnv {
  // scalar / vectors / matrices
  ($t:ty, $v:ident $(, $dim:path)?) => {
    impl IsEnv for $t {
      const ENV_TY: EnvType = EnvType::$v $(($dim))?;
    }
  }
}

impl_IsEnv!(i32, Integral, EnvDim::Dim1);
impl_IsEnv!(u32, Unsigned, EnvDim::Dim1);
impl_IsEnv!(f32, Floating, EnvDim::Dim1);
impl_IsEnv!(bool, Boolean, EnvDim::Dim1);

#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector2<i32>, Integral, EnvDim::Dim2);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector2<u32>, Unsigned, EnvDim::Dim2);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector2<f32>, Floating, EnvDim::Dim2);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector2<bool>, Boolean, EnvDim::Dim2);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector3<i32>, Integral, EnvDim::Dim3);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector3<u32>, Unsigned, EnvDim::Dim3);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector3<f32>, Floating, EnvDim::Dim3);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector3<bool>, Boolean, EnvDim::Dim3);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector4<i32>, Integral, EnvDim::Dim4);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector4<u32>, Unsigned, EnvDim::Dim4);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector4<f32>, Floating, EnvDim::Dim4);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::Vector4<bool>, Boolean, EnvDim::Dim4);

#[cfg(feature = "mint")]
impl_IsEnv!(mint::ColumnMatrix2<f32>, Matrix, EnvMatDim::Mat22);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::ColumnMatrix3<f32>, Matrix, EnvMatDim::Mat33);
#[cfg(feature = "mint")]
impl_IsEnv!(mint::ColumnMatrix4<f32>, Matrix, EnvMatDim::Mat44);

// TODO: samplers

pub trait FromEnv: Sized {
  fn from_env<B>(backend: &mut B, program_handle: usize) -> Result<Self, ShaderError>
  where
    B: Backend;
}

impl FromEnv for () {
  fn from_env<B>(_: &mut B, _: usize) -> Result<Self, ShaderError>
  where
    B: Backend,
  {
    Ok(())
  }
}

pub struct SharedEnv<T> {
  handle: usize,
  _phantom: PhantomData<*const T>,
}

impl<T> SharedEnv<T> {
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

pub trait IsSharedEnv {}

#[derive(Debug)]
pub struct ProgramUpdate<'a, B> {
  pub(crate) backend: &'a mut B,
  pub(crate) program_handle: usize,
}

impl<'a, B> ProgramUpdate<'a, B>
where
  B: Backend,
{
  pub fn set<T>(&mut self, env: &Env<T>, value: T) -> Result<(), ShaderError>
  where
    T: IsEnv,
  {
    unsafe {
      self
        .backend
        .set_program_env(self.program_handle, env.handle(), value)
    }
  }
}
