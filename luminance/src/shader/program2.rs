use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

/// Types that can behave as `Uniform`.
pub unsafe trait Uniformable<T>: Sized {
  ///`Type` of the uniform.
  const TY: Type;

  /// Update the uniform with a new value.
  fn update(self, value: T);
}

/// Type of a uniform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  // scalars
  /// 32-bit signed integer.
  Int,
  /// 32-bit unsigned integer.
  UInt,
  /// 32-bit floating-point number.
  Float,
  /// Boolean.
  Bool,

  // vectors
  /// 2D signed integral vector.
  IVec2,
  /// 3D signed integral vector.
  IVec3,
  /// 4D signed integral vector.
  IVec4,
  /// 2D unsigned integral vector.
  UIVec2,
  /// 3D unsigned integral vector.
  UIVec3,
  /// 4D unsigned integral vector.
  UIVec4,
  /// 2D floating-point vector.
  Vec2,
  /// 3D floating-point vector.
  Vec3,
  /// 4D floating-point vector.
  Vec4,
  /// 2D boolean vector.
  BVec2,
  /// 3D boolean vector.
  BVec3,
  /// 4D boolean vector.
  BVec4,

  // matrices
  /// 2×2 floating-point matrix.
  M22,
  /// 3×3 floating-point matrix.
  M33,
  /// 4×4 floating-point matrix.
  M44,

  // textures
  /// Signed integral 1D texture sampler.
  ISampler1D,
  /// Signed integral 2D texture sampler.
  ISampler2D,
  /// Signed integral 3D texture sampler.
  ISampler3D,
  /// Unsigned integral 1D texture sampler.
  UISampler1D,
  /// Unsigned integral 2D texture sampler.
  UISampler2D,
  /// Unsigned integral 3D texture sampler.
  UISampler3D,
  /// Floating-point 1D texture sampler.
  Sampler1D,
  /// Floating-point 2D texture sampler.
  Sampler2D,
  /// Floating-point 3D texture sampler.
  Sampler3D,
  /// Signed cubemap sampler.
  ICubemap,
  /// Unsigned cubemap sampler.
  UICubemap,
  /// Floating-point cubemap sampler.
  Cubemap,

  // buffer
  /// Buffer binding; used for UBOs.
  BufferBinding,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Type::Int => f.write_str("int"),
      Type::UInt => f.write_str("uint"),
      Type::Float => f.write_str("float"),
      Type::Bool => f.write_str("bool"),
      Type::IVec2 => f.write_str("ivec2"),
      Type::IVec3 => f.write_str("ivec3"),
      Type::IVec4 => f.write_str("ivec4"),
      Type::UIVec2 => f.write_str("uvec2"),
      Type::UIVec3 => f.write_str("uvec3"),
      Type::UIVec4 => f.write_str("uvec4"),
      Type::Vec2 => f.write_str("vec2"),
      Type::Vec3 => f.write_str("vec3"),
      Type::Vec4 => f.write_str("vec4"),
      Type::BVec2 => f.write_str("bvec2"),
      Type::BVec3 => f.write_str("bvec3"),
      Type::BVec4 => f.write_str("bvec4"),
      Type::M22 => f.write_str("mat2"),
      Type::M33 => f.write_str("mat3"),
      Type::M44 => f.write_str("mat4"),
      Type::ISampler1D => f.write_str("isampler1D"),
      Type::ISampler2D => f.write_str("isampler2D"),
      Type::ISampler3D => f.write_str("isampler3D"),
      Type::UISampler1D => f.write_str("uSampler1D"),
      Type::UISampler2D => f.write_str("uSampler2D"),
      Type::UISampler3D => f.write_str("uSampler3D"),
      Type::Sampler1D => f.write_str("sampler1D"),
      Type::Sampler2D => f.write_str("sampler2D"),
      Type::Sampler3D => f.write_str("sampler3D"),
      Type::ICubemap => f.write_str("isamplerCube"),
      Type::UICubemap => f.write_str("usamplerCube"),
      Type::Cubemap => f.write_str("samplerCube"),
      Type::BufferBinding => f.write_str("buffer binding"),
    }
  }
}

pub trait UniformBuild<T>: UniformBuilder {
  type Uniform: Uniformable<T>;

  fn ask_specific<S>(&mut self, name: S) -> Result<Self::Uniform, Self::Err>
  where
    S: AsRef<str>;

  fn ask_unbound_specific<S>(&mut self, name: S) -> Self::Uniform
  where
    S: AsRef<str>;

  fn unbound_specific(&mut self) -> Self::Uniform;
}

pub trait UniformBuilder {
  type Err;

  fn ask<T, S>(&mut self, name: S) -> Result<Self::Uniform, Self::Err>
  where
    Self: UniformBuild<T>,
    S: AsRef<str>,
  {
    self.ask_specific(name)
  }

  fn ask_unbound<T, S>(&mut self, name: S) -> Self::Uniform
  where
    Self: UniformBuild<T>,
    S: AsRef<str>,
  {
    self.ask_unbound_specific(name)
  }

  fn unbound<T>(&mut self) -> Self::Uniform
  where
    Self: UniformBuild<T>,
  {
    self.unbound_specific()
  }
}

pub trait UniformInterface<E = ()>: Sized {
  fn uniform_interface<'a, B>(builder: &mut B, env: E) -> Result<Self, B::Err>
  where
    B: UniformBuilder;
}

impl<E> UniformInterface<E> for () {
  fn uniform_interface<'a, B>(_: &mut B, _: E) -> Result<Self, B::Err>
  where
    B: UniformBuilder,
  {
    Ok(())
  }
}

pub struct TessellationStages<'a, T>
where
  T: ?Sized,
{
  pub control: &'a T,
  pub evaluation: &'a T,
}

pub struct BuiltProgram<P, W> {
  pub program: P,
  pub warnings: Vec<W>,
}

pub trait Program<'program, S, Out, Uni>: Sized {
  type Stage;

  type Err;

  type UniformBuilder: UniformBuilder;

  fn from_stages_env<T, G, E>(
    vertex: &Self::Stage,
    tess: T,
    geometry: G,
    fragment: &Self::Stage,
    env: E,
  ) -> Result<BuiltProgram<Self, Self::Err>, Self::Err>
  where
    T: for<'a> Into<Option<TessellationStages<'a, Self::Stage>>>,
    G: for<'a> Into<Option<&'a Self::Stage>>;

  fn from_stages<T, G>(
    vertex: &Self::Stage,
    tess: T,
    geometry: G,
    fragment: &Self::Stage,
  ) -> Result<BuiltProgram<Self, Self::Err>, Self::Err>
  where
    T: for<'a> Into<Option<TessellationStages<'a, Self::Stage>>>,
    G: for<'a> Into<Option<&'a Self::Stage>>,
  {
    Self::from_stages_env(vertex, tess, geometry, fragment, ())
  }

  fn from_strings_env<T, G, E>(
    vertex: &str,
    tess: T,
    geometry: G,
    fragment: &str,
    env: E,
  ) -> Result<BuiltProgram<Self, Self::Err>, Self::Err>;

  fn from_strings<T, G>(
    vertex: &str,
    tess: T,
    geometry: G,
    fragment: &str,
  ) -> Result<BuiltProgram<Self, Self::Err>, Self::Err>
  where
    T: for<'a> Into<Option<TessellationStages<'a, str>>>,
    G: for<'a> Into<Option<&'a str>>,
  {
    Self::from_strings_env(vertex, tess, geometry, fragment, ())
  }

  fn link(&'program self) -> Result<(), Self::Err>;

  fn uniform_builder(&'program self) -> Self::UniformBuilder;

  fn interface(&'program self) -> ProgramInterface<'program, Self, S, Out, Uni>;
}

pub struct ProgramInterface<'program, P, S, Out, Uni>
where
  P: Program<'program, S, Out, Uni>,
{
  program: &'program P,
  interface: &'program Uni,
  _s: PhantomData<&'program S>,
  _out: PhantomData<&'program Out>,
}

impl<'program, P, S, Out, Uni> ProgramInterface<'program, P, S, Out, Uni>
where
  P: Program<'program, S, Out, Uni>,
{
  pub fn query(&'program self) -> P::UniformBuilder {
    self.program.uniform_builder()
  }
}

impl<'program, P, S, Out, Uni> Deref for ProgramInterface<'program, P, S, Out, Uni>
where
  P: Program<'program, S, Out, Uni>,
{
  type Target = Uni;

  fn deref(&self) -> &Self::Target {
    &self.interface
  }
}
