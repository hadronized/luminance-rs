//! Shader stages, programs and uniforms.
//!
//! This module contains everything related to _shaders_. Shader programs — shaders, for short —
//! are GPU binaries that run to perform a series of transformation. Typically run when a draw
//! command is issued, they are responsible for:
//!
//! - Transforming vertex data. This is done in a _vertex shader_. Vertex data, such as the
//!   positions, colors, UV coordinates, bi-tangents, etc. of each vertices will go through the
//!   vertex shader and get transformed based on the code provided inside the stage. For example,
//!   vertices can be projected on the screen with a perspective and view matrices.
//! - Tessellating primitive patches. This is done in _tessellation shaders_.
//! - Filtering, transforming again or even generating new vertices and primitives. This is done
//!   by the _geometry shader_.
//! - And finally, outputting a color for each _fragment_ covered by the objects you render. This
//!   is done by the _fragment shader_.
//!
//! # Shader stages
//!
//! Right now, five shader stages  — [`Stage`] — are supported, ordered by usage in the graphics
//! pipeline:
//!
//! 1. [`StageType::VertexShader`].
//! 2. [`StageType::TessellationControlShader`].
//! 3. [`StageType::TessellationEvaluationShader`].
//! 4. [`StageType::GeometryShader`].
//! 5. [`StageType::FragmentShader`].
//!
//! Those are not all mandatory: only the _vertex_ stage and _fragment_ stages are mandatory. If
//! you want tessellation shaders, you have to provide both of them.
//!
//! Shader stages — [`Stage`] — are compiled independently at runtime by your GPU driver, and then
//! _linked_ into a shader program. The creation of a [`Stage`] implies using an input string,
//! representing the _source code_ of the stage. This is an opaque [`String`] that must represent
//! a GLSL stage. The backend will transform the string into its own representation if needed.
//!
//! > For this version of the crate, the GLSL string must be at least 330-compliant. It is possible
//! > that this changes in the future to be more flexible, but right now GLSL 150, for instance, is
//! > not allowed.
//!
//! # Shader program
//!
//! A shader program — [`Program`] is akin to a binary program, but runs on GPU. It is invoked when
//! you issue draw commands. It will run each stages you’ve put in it to transform vertices and
//! rasterize fragments inside a framebuffer. Once this is done, the framebuffer will contain
//! altered fragments by the final stage (fragment shader). If the shader program outputs several
//! properties, we call that situation _MRT_ (Multiple Render Target) and the framebuffer must be
//! configured to be able to receive those outputs — basically, it means that its _color slots_
//! and/or _depth slots_ must adapt to the output of the shader program.
//!
//! Creating shader programs is done by gathering the [`Stage`] you want and _linking_ them. Some
//! helper methods allow to create a shader [`Program`] directly from the string source for each
//! stage, removing the need to build each stage individually.
//!
//! Shader programs are typed with three important piece of information:
//!
//! - The vertex [`Semantics`].
//! - The render target outputs.
//! - The [`UniformInterface`].
//!
//!
//! # Vertex semantics
//!
//! When a shader program runs, it first executes the mandatory _vertex stage on a set of
//! vertices. Those vertices have a given format — that is described by the [`Vertex`] trait.
//! Because running a shader on an incompatible vertex would yield wrong results, both the
//! vertices and the shader program must be tagged with a type which must implement the
//! [`Semantics`] trait. More on that on the documentation of [`Semantics`].
//!
//! # Render target outputs
//!
//! A shader program, in its final mandatory _fragment stage_, will write values into the currently
//! in-use framebuffer. The number of “channels” to write to represents the render targets.
//! Typically, simple renders will simply write the color of a pixel — so only one render target.
//! In that case, the type of the output of the shader program must match the color slot of the
//! framebuffer it is used with.
//!
//! However, it is possible to write more data. For instance,
//! [deferred shading](https://en.wikipedia.org/wiki/Deferred_shading) is a technique that requires
//! to write several data to a framebuffer, called G-buffer (for geometry buffer): space
//! coordinates, normals, tangents, bi-tangents, etc. In that case, your framebuffer must have
//! a type matching the outputs of the fragment shader, too.
//!
//! # Shader customization
//!
//! A shader [`Program`] represents some code, in a binary form, that transform data. If you
//! consider such code, it can adapt to the kind of data it receives, but the behavior is static.
//! That means that it shouldn’t be possible to ask the program to do something else — shader
//! programs don’t have a state as they must be spawned in parallel for your vertices, pixels, etc.
//! However, there is a way to dynamically change what happens inside a shader program. That way
//!
//! The concept is similar to environment variables: you can declare, in your shader stages,
//! _environment variables_ that will receive values from the host (i.e. on the Rust side). It is
//! not possible to change those values while a draw command is issued: you have to change them
//! in between draw commands. For this reason, those environment variables are often called
//! _constant buffers_, _uniform_, _uniform buffers_, etc. by several graphics libraries. In our
//! case, right now, we call them [`Uniform`].
//!
//! ## Uniforms
//!
//! A [`Uniform`] is parametric type that accepts the type of the value it will be able to change.
//! For instance, `Uniform<f32>` represents a `f32` that can be changed in a shader program. That
//! value can be set by the Rust program when the shader program is not currently in use — no
//! draw commands.
//!
//! A [`Uniform`] is a _single_ variable that allows the most basic form of customization. It’s
//! very similar to environment variables. You can declare several ones as you would declare
//! several environment variables. More on that on the documentation of [`Uniform`].
//!
//! ## Shader data
//!
//! Another way to pass data to shader is to use a [`ShaderData`]. This kind of object allows to
//! shared data between shaders: set the data once, it will be available to any shader you pass the
//! [`ShaderData`] to.
//!
//! Most implementation also allows much more data via this mechanism, allowing to pass huge amount
//! of data to implement various techniques, such as _geometry instancing_ for instance.
//!
//! ## Uniform interfaces
//!
//! As with vertex semantics and render targets, the uniforms that can be used with a shader program
//! are part of its type, too, and represented by a single type that must implement
//! [`UniformInterface`]. That type can contain anything, but it is advised to just put [`Uniform`]
//! fields in it. More on the [`UniformInterface`] documentation.
//!
//! [`Vertex`]: crate::vertex::Vertex
//! [`Pipeline`]: crate::pipeline::Pipeline
//! [`ShaderData`]: crate::shader::ShaderData

pub mod types;

use crate::{
  backend::shader::{Shader, ShaderData as ShaderDataBackend, Uniformable},
  context::GraphicsContext,
  vertex::Vertex,
};
use std::{error, fmt, marker::PhantomData};

/// A shader stage type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageType {
  /// Vertex shader.
  VertexShader,
  /// Tessellation control shader.
  TessellationControlShader,
  /// Tessellation evaluation shader.
  TessellationEvaluationShader,
  /// Geometry shader.
  GeometryShader,
  /// Fragment shader.
  FragmentShader,
}

impl fmt::Display for StageType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StageType::VertexShader => f.write_str("vertex shader"),
      StageType::TessellationControlShader => f.write_str("tessellation control shader"),
      StageType::TessellationEvaluationShader => f.write_str("tessellation evaluation shader"),
      StageType::GeometryShader => f.write_str("geometry shader"),
      StageType::FragmentShader => f.write_str("fragment shader"),
    }
  }
}

/// Errors that shader stages can emit.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StageError {
  /// Occurs when a shader fails to compile.
  CompilationFailed(StageType, String),
  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(StageType),
}

impl StageError {
  /// Occurs when a shader fails to compile.
  pub fn compilation_failed(ty: StageType, reason: impl Into<String>) -> Self {
    StageError::CompilationFailed(ty, reason.into())
  }

  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  pub fn unsupported_type(ty: StageType) -> Self {
    StageError::UnsupportedType(ty)
  }
}

impl fmt::Display for StageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StageError::CompilationFailed(ref ty, ref r) => write!(f, "{} compilation error: {}", ty, r),

      StageError::UnsupportedType(ty) => write!(f, "unsupported {}", ty),
    }
  }
}

impl error::Error for StageError {}

impl From<StageError> for ProgramError {
  fn from(e: StageError) -> Self {
    ProgramError::StageError(e)
  }
}

/// Tessellation stages.
///
/// - The `control` stage represents the _tessellation control stage_, which is invoked first.
/// - The `evaluation` stage represents the _tessellation evaluation stage_, which is invoked after
///   the control stage has finished.
///
/// # Parametricity
///
/// - `S` is the representation of the stage. Depending on the interface you choose to create a
///   [`Program`], it might be a [`Stage`] or something akin to [`&str`] / [`String`].
///
/// [`&str`]: str
pub struct TessellationStages<'a, S>
where
  S: ?Sized,
{
  /// Tessellation control representation.
  pub control: &'a S,
  /// Tessellation evaluation representation.
  pub evaluation: &'a S,
}

/// Errors that a [`Program`] can generate.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum ProgramError {
  /// Creating the program failed.
  CreationFailed(String),
  /// A shader stage failed to compile or validate its state.
  StageError(StageError),
  /// Program link failed. You can inspect the reason by looking at the contained [`String`].
  LinkFailed(String),
  /// A program warning.
  Warning(ProgramWarning),
}

impl ProgramError {
  /// Creating the program failed.
  pub fn creation_failed(reason: impl Into<String>) -> Self {
    ProgramError::CreationFailed(reason.into())
  }

  /// A shader stage failed to compile or validate its state.
  pub fn stage_error(e: StageError) -> Self {
    ProgramError::StageError(e)
  }

  /// Program link failed. You can inspect the reason by looking at the contained [`String`].
  pub fn link_failed(reason: impl Into<String>) -> Self {
    ProgramError::LinkFailed(reason.into())
  }

  /// A program warning.
  pub fn warning(w: ProgramWarning) -> Self {
    ProgramError::Warning(w)
  }
}

impl fmt::Display for ProgramError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ProgramError::CreationFailed(ref e) => write!(f, "cannot create shader program: {}", e),

      ProgramError::StageError(ref e) => write!(f, "shader program has stage error: {}", e),

      ProgramError::LinkFailed(ref s) => write!(f, "shader program failed to link: {}", s),

      ProgramError::Warning(ref e) => write!(f, "shader program warning: {}", e),
    }
  }
}

impl error::Error for ProgramError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      ProgramError::StageError(e) => Some(e),
      _ => None,
    }
  }
}

/// Program warnings, not necessarily considered blocking errors.
#[derive(Debug, Eq, PartialEq)]
pub enum ProgramWarning {
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the [`UniformWarning`] type for more information.
  Uniform(UniformWarning),
  /// Some vertex attribute is ill-formed.
  VertexAttrib(VertexAttribWarning),
}

impl fmt::Display for ProgramWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ProgramWarning::Uniform(ref e) => write!(f, "uniform warning: {}", e),
      ProgramWarning::VertexAttrib(ref e) => write!(f, "vertex attribute warning: {}", e),
    }
  }
}

impl error::Error for ProgramWarning {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      ProgramWarning::Uniform(e) => Some(e),
      ProgramWarning::VertexAttrib(e) => Some(e),
    }
  }
}

impl From<ProgramWarning> for ProgramError {
  fn from(e: ProgramWarning) -> Self {
    ProgramError::Warning(e)
  }
}

/// Warnings related to uniform issues.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),

  /// Type mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The [`String`] is the name of the uniform; the second one gives the type mismatch.
  ///
  /// [`Uniform<T>`]: crate::shader::Uniform
  TypeMismatch(String, UniformType),

  /// The requested type is unsupported by the backend.
  ///
  /// The [`String`] is the name of the uniform. The [`UniformType`] is the type that is not
  /// supported by the backend.
  UnsupportedType(String, UniformType),

  /// Size mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
  /// and the size that got reflected from the backend in the shaders.
  ///
  /// [`Uniform<T>`]: crate::shader::Uniform
  SizeMismatch {
    /// Name of the uniform.
    name: String,

    /// Size of the uniform (static).
    size: usize,

    /// Found size of the uniform (in the shader).
    found_size: usize,
  },
}

impl UniformWarning {
  /// Create an inactive uniform warning.
  pub fn inactive<N>(name: N) -> Self
  where
    N: Into<String>,
  {
    UniformWarning::Inactive(name.into())
  }

  /// Create a type mismatch.
  pub fn type_mismatch<N>(name: N, ty: UniformType) -> Self
  where
    N: Into<String>,
  {
    UniformWarning::TypeMismatch(name.into(), ty)
  }

  /// Create an unsupported type error.
  pub fn unsupported_type<N>(name: N, ty: UniformType) -> Self
  where
    N: Into<String>,
  {
    UniformWarning::UnsupportedType(name.into(), ty)
  }

  /// Create a size mismatch error.
  pub fn size_mismatch(name: impl Into<String>, size: usize, found_size: usize) -> Self {
    UniformWarning::SizeMismatch {
      name: name.into(),
      size,
      found_size,
    }
  }
}

impl fmt::Display for UniformWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      UniformWarning::Inactive(ref s) => write!(f, "inactive {} uniform", s),

      UniformWarning::TypeMismatch(ref n, ref t) => {
        write!(f, "type mismatch for uniform {}: {}", n, t)
      }

      UniformWarning::UnsupportedType(ref name, ref ty) => {
        write!(f, "unsupported type {} for uniform {}", ty, name)
      }

      UniformWarning::SizeMismatch {
        ref name,
        size,
        found_size,
      } => {
        write!(
          f,
          "size mismatch for uniform {}: {} (tdetected size={}",
          name, size, found_size
        )
      }
    }
  }
}

impl From<UniformWarning> for ProgramWarning {
  fn from(e: UniformWarning) -> Self {
    ProgramWarning::Uniform(e)
  }
}

impl error::Error for UniformWarning {}

/// Warnings related to vertex attributes issues.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum VertexAttribWarning {
  /// Inactive vertex attribute (not read).
  Inactive(String),
}

impl VertexAttribWarning {
  /// Inactive vertex attribute (not read).
  pub fn inactive(attrib: impl Into<String>) -> Self {
    VertexAttribWarning::Inactive(attrib.into())
  }
}

impl fmt::Display for VertexAttribWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      VertexAttribWarning::Inactive(ref s) => write!(f, "inactive {} vertex attribute", s),
    }
  }
}

impl From<VertexAttribWarning> for ProgramWarning {
  fn from(e: VertexAttribWarning) -> Self {
    ProgramWarning::VertexAttrib(e)
  }
}

impl error::Error for VertexAttribWarning {}

/// A GPU shader program environment variable.
///
/// A uniform is a special variable that can be used to send data to a GPU. Several
/// forms exist, but the idea is that `T` represents the data you want to send. Some exceptions
/// exist that allow to pass shared data — such as [`ShaderDataBinding`] to pass a
/// [`ShaderData`], or [`TextureBinding`] to pass a [`Texture`] in order to fetch from it in a
/// shader stage.
///
/// You will never be able to store them by your own. Instead, you must use a [`UniformInterface`],
/// which provides a _contravariant_ interface for you. Creation is `unsafe` and should be
/// avoided. The [`UniformInterface`] is the only safe way to create those.
///
/// # Parametricity
///
/// - `T` is the type of data you want to be able to set in a shader program.
///
/// [`Texture`]: crate::texture::Texture
/// [`TextureBinding`]: crate::pipeline::TextureBinding
/// [`ShaderDataBinding`]: crate::pipeline::ShaderDataBinding
#[derive(Debug)]
pub struct Uniform<T>
where
  T: ?Sized,
{
  index: i32,
  _t: PhantomData<*const T>,
}

impl<T> Uniform<T>
where
  T: ?Sized,
{
  /// Create a new [`Uniform`].
  ///
  /// # Safety
  ///
  /// This method must be used **only** by backends. If you end up using it,
  /// then you’re doing something wrong. Read on [`UniformInterface`] for further
  /// information.
  pub unsafe fn new(index: i32) -> Self {
    Uniform {
      index,
      _t: PhantomData,
    }
  }

  /// Retrieve the internal index.
  ///
  /// Even though that function is safe, you have no reason to use it. Read on
  /// [`UniformInterface`] for further details.
  pub fn index(&self) -> i32 {
    self.index
  }
}

/// Type of a uniform.
///
/// This is an exhaustive list of possible types of value you can send to a shader program.
/// A [`UniformType`] is associated to any type that can be considered sent via the
/// [`Uniformable`] trait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniformType {
  // scalars
  /// 32-bit signed integer.
  Int,
  /// 32-bit unsigned integer.
  UInt,
  /// 32-bit floating-point number.
  Float,
  /// 64-bit floating-point number.
  Double,
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
  /// 2D floating-point (double) vector.
  DVec2,
  /// 3D floating-point (double) vector.
  DVec3,
  /// 4D floating-point (double) vector.
  DVec4,
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
  /// 2×2 floating-point (double) matrix.
  DM22,
  /// 3×3 floating-point (double) matrix.
  DM33,
  /// 4×4 floating-point (double) matrix.
  DM44,

  // textures
  /// Signed integral 1D texture sampler.
  ISampler1D,
  /// Signed integral 2D texture sampler.
  ISampler2D,
  /// Signed integral 3D texture sampler.
  ISampler3D,
  /// Signed integral 1D array texture sampler.
  ISampler1DArray,
  /// Signed integral 2D array texture sampler.
  ISampler2DArray,
  /// Unsigned integral 1D texture sampler.
  UISampler1D,
  /// Unsigned integral 2D texture sampler.
  UISampler2D,
  /// Unsigned integral 3D texture sampler.
  UISampler3D,
  /// Unsigned integral 1D array texture sampler.
  UISampler1DArray,
  /// Unsigned integral 2D array texture sampler.
  UISampler2DArray,
  /// Floating-point 1D texture sampler.
  Sampler1D,
  /// Floating-point 2D texture sampler.
  Sampler2D,
  /// Floating-point 3D texture sampler.
  Sampler3D,
  /// Floating-point 1D array texture sampler.
  Sampler1DArray,
  /// Floating-point 2D array texture sampler.
  Sampler2DArray,
  /// Signed cubemap sampler.
  ICubemap,
  /// Unsigned cubemap sampler.
  UICubemap,
  /// Floating-point cubemap sampler.
  Cubemap,

  /// Shader data binding.
  ShaderDataBinding,
}

impl fmt::Display for UniformType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      UniformType::Int => f.write_str("int"),
      UniformType::UInt => f.write_str("uint"),
      UniformType::Float => f.write_str("float"),
      UniformType::Double => f.write_str("double"),
      UniformType::Bool => f.write_str("bool"),
      UniformType::IVec2 => f.write_str("ivec2"),
      UniformType::IVec3 => f.write_str("ivec3"),
      UniformType::IVec4 => f.write_str("ivec4"),
      UniformType::UIVec2 => f.write_str("uvec2"),
      UniformType::UIVec3 => f.write_str("uvec3"),
      UniformType::UIVec4 => f.write_str("uvec4"),
      UniformType::Vec2 => f.write_str("vec2"),
      UniformType::Vec3 => f.write_str("vec3"),
      UniformType::Vec4 => f.write_str("vec4"),
      UniformType::DVec2 => f.write_str("dvec2"),
      UniformType::DVec3 => f.write_str("dvec3"),
      UniformType::DVec4 => f.write_str("dvec4"),
      UniformType::BVec2 => f.write_str("bvec2"),
      UniformType::BVec3 => f.write_str("bvec3"),
      UniformType::BVec4 => f.write_str("bvec4"),
      UniformType::M22 => f.write_str("mat2"),
      UniformType::M33 => f.write_str("mat3"),
      UniformType::M44 => f.write_str("mat4"),
      UniformType::DM22 => f.write_str("dmat2"),
      UniformType::DM33 => f.write_str("dmat3"),
      UniformType::DM44 => f.write_str("dmat4"),
      UniformType::ISampler1D => f.write_str("isampler1D"),
      UniformType::ISampler2D => f.write_str("isampler2D"),
      UniformType::ISampler3D => f.write_str("isampler3D"),
      UniformType::ISampler1DArray => f.write_str("isampler1DArray"),
      UniformType::ISampler2DArray => f.write_str("isampler2DArray"),
      UniformType::UISampler1D => f.write_str("usampler1D"),
      UniformType::UISampler2D => f.write_str("usampler2D"),
      UniformType::UISampler3D => f.write_str("usampler3D"),
      UniformType::UISampler1DArray => f.write_str("usampler1DArray"),
      UniformType::UISampler2DArray => f.write_str("usampler2DArray"),
      UniformType::Sampler1D => f.write_str("sampler1D"),
      UniformType::Sampler2D => f.write_str("sampler2D"),
      UniformType::Sampler3D => f.write_str("sampler3D"),
      UniformType::Sampler1DArray => f.write_str("sampler1DArray"),
      UniformType::Sampler2DArray => f.write_str("sampler2DArray"),
      UniformType::ICubemap => f.write_str("isamplerCube"),
      UniformType::UICubemap => f.write_str("usamplerCube"),
      UniformType::Cubemap => f.write_str("samplerCube"),
      UniformType::ShaderDataBinding => f.write_str("shader data binding"),
    }
  }
}

/// A shader stage.
///
/// # Parametricity
///
/// - `B` is the backend type.
///
/// [`&str`]: str
pub struct Stage<B>
where
  B: ?Sized + Shader,
{
  repr: B::StageRepr,
}

impl<B> Stage<B>
where
  B: ?Sized + Shader,
{
  /// Create a new stage of type `ty` by compiling `src`.
  ///
  /// # Parametricity
  ///
  /// - `C` is the graphics context. `C::Backend` must implement the [`Shader`] trait.
  /// - `R` is the source code to use in the stage. It must implement [`AsRef<str>`].
  ///
  /// # Notes
  ///
  /// Feel free to consider using [`GraphicsContext::new_shader_stage`] for a simpler form of
  /// this method.
  ///
  /// [`AsRef<str>`]: AsRef
  pub fn new<C, R>(ctx: &mut C, ty: StageType, src: R) -> Result<Self, StageError>
  where
    C: GraphicsContext<Backend = B>,
    R: AsRef<str>,
  {
    unsafe {
      ctx
        .backend()
        .new_stage(ty, src.as_ref())
        .map(|repr| Stage { repr })
    }
  }
}

/// A builder of [`Uniform`].
///
/// A [`UniformBuilder`] is an important type as it’s the only one that allows to safely create
/// [`Uniform`] values.
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement the [`Shader`] trait.
pub struct UniformBuilder<'a, B>
where
  B: ?Sized + Shader,
{
  repr: B::UniformBuilderRepr,
  warnings: Vec<UniformWarning>,
  _a: PhantomData<&'a mut ()>,
}

impl<'a, B> UniformBuilder<'a, B>
where
  B: ?Sized + Shader,
{
  /// Ask the creation of a [`Uniform`], identified by its `name`.
  pub fn ask<T>(&mut self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    B: for<'u> Uniformable<'u, T>,
  {
    unsafe { B::ask_uniform(&mut self.repr, name) }
  }

  /// Ask the creation of a [`Uniform`], identified by its `name`.
  ///
  /// If the name is not found, an _unbound_ [`Uniform`] is returned (i.e. a [`Uniform`]) that does
  /// nothing.
  pub fn ask_or_unbound<T>(&mut self, name: &str) -> Uniform<T>
  where
    B: for<'u> Uniformable<'u, T>,
  {
    match self.ask(name) {
      Ok(uniform) => uniform,
      Err(err) => {
        self.warnings.push(err);
        unsafe { B::unbound(&mut self.repr) }
      }
    }
  }
}

/// [`Uniform`] interface.
///
/// When a type implements [`UniformInterface`], it means that it can be used as part of a shader
/// [`Program`] type. When a [`Program`] is in use in a graphics pipeline, its [`UniformInterface`]
/// is automatically provided to the user, giving them access to all the fields declared in. Then,
/// they can pass data to shaders before issuing draw commands.
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement [`Shader`].
/// - `E` is the environment type. Set by default to `()`, it allows to pass a mutable
///   object at construction-site of the [`UniformInterface`]. It can be useful to generate
///   events or customize the way the [`Uniform`] are built by doing some lookups in hashmaps, etc.
///
/// # Notes
///
/// Implementing this trait — especially [`UniformInterface::uniform_interface`] can be a bit
/// overwhelming. It is highly recommended to use [luminance-derive]’s `UniformInterface`
/// proc-macro, which will do that for you by scanning your type declaration.
///
/// [luminance-derive]: https://crates.io/crates/luminance-derive
pub trait UniformInterface<B, E = ()>: Sized
where
  B: Shader,
{
  /// Create a [`UniformInterface`] by constructing [`Uniform`]s with a [`UniformBuilder`] and an
  /// optional environment object.
  ///
  /// This method is the only place where `Self` should be created. In theory, you could create it
  /// the way you want (since the type is provided by you) but not all types make sense. You will
  /// likely want to have some [`Uniform`] objects in your type, and the [`UniformBuilder`] that is
  /// provided as argument is the only way to create them.
  fn uniform_interface<'a>(
    builder: &mut UniformBuilder<'a, B>,
    env: &mut E,
  ) -> Result<Self, UniformWarning>;
}

impl<B, E> UniformInterface<B, E> for ()
where
  B: Shader,
{
  fn uniform_interface<'a>(
    _: &mut UniformBuilder<'a, B>,
    _: &mut E,
  ) -> Result<Self, UniformWarning> {
    Ok(())
  }
}

/// A built program with potential warnings.
///
/// The sole purpose of this type is to be destructured when a program is built.
pub struct BuiltProgram<B, V, Out, Uni>
where
  B: Shader,
{
  /// Built program.
  pub program: Program<B, V, Out, Uni>,
  /// Potential warnings.
  pub warnings: Vec<ProgramError>,
}

impl<B, Sem, Out, Uni> BuiltProgram<B, Sem, Out, Uni>
where
  B: Shader,
{
  /// Get the program and ignore the warnings.
  pub fn ignore_warnings(self) -> Program<B, Sem, Out, Uni> {
    self.program
  }
}

/// A [`Program`] uniform adaptation that has failed.
pub struct AdaptationFailure<B, V, Out, Uni>
where
  B: Shader,
{
  /// Program used before trying to adapt.
  pub program: Program<B, V, Out, Uni>,
  /// Program error that prevented to adapt.
  pub error: ProgramError,
}

impl<B, V, Out, Uni> AdaptationFailure<B, V, Out, Uni>
where
  B: Shader,
{
  pub(crate) fn new(program: Program<B, V, Out, Uni>, error: ProgramError) -> Self {
    AdaptationFailure { program, error }
  }

  /// Get the program and ignore the error.
  pub fn ignore_error(self) -> Program<B, V, Out, Uni> {
    self.program
  }
}

/// Interact with the [`UniformInterface`] carried by a [`Program`] and/or perform dynamic
/// uniform lookup.
///
/// This type allows to set — [`ProgramInterface::set`] – uniforms for a [`Program`].
///
/// In the case where you don’t have a uniform interface or need to dynamically lookup uniforms,
/// you can use the [`ProgramInterface::query`] method.
///
/// # Parametricity
///
/// `B` is the backend type.
pub struct ProgramInterface<'a, B>
where
  B: Shader,
{
  pub(crate) program: &'a mut B::ProgramRepr,
}

impl<'a, B> ProgramInterface<'a, B>
where
  B: Shader,
{
  /// Set a value on a [`Uniform`].
  ///
  /// The value that is passed depends on the associated [`Uniformable::Target`] type. Most of the time, it will be the
  /// same as `T`, but it might sometimes be something different if you are using existential types, such as with types
  /// with lifetimes.
  pub fn set<'u, T>(&'u mut self, uniform: &'u Uniform<T>, value: B::Target)
  where
    B: Uniformable<'u, T>,
  {
    unsafe { B::update(self.program, uniform, value) };
  }

  /// Get back a [`UniformBuilder`] to dynamically access [`Uniform`] objects.
  pub fn query(&mut self) -> Result<UniformBuilder<'a, B>, ProgramError> {
    unsafe {
      B::new_uniform_builder(&mut self.program).map(|repr| UniformBuilder {
        repr,
        warnings: Vec::new(),
        _a: PhantomData,
      })
    }
  }
}

/// A [`Program`] builder.
///
/// This type allows to create shader programs without having to worry too much about the highly
/// generic API.
pub struct ProgramBuilder<'a, C, V, Out, Uni> {
  ctx: &'a mut C,
  _phantom: PhantomData<(V, Out, Uni)>,
}

impl<'a, C, V, Out, Uni> ProgramBuilder<'a, C, V, Out, Uni>
where
  C: GraphicsContext,
  C::Backend: Shader,
  V: Vertex,
{
  /// Create a new [`ProgramBuilder`] from a [`GraphicsContext`].
  pub fn new(ctx: &'a mut C) -> Self {
    ProgramBuilder {
      ctx,
      _phantom: PhantomData,
    }
  }

  /// Create a [`Program`] by linking [`Stage`]s and accessing a mutable environment variable.
  ///
  /// # Parametricity
  ///
  /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`Stage`] inside.
  /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
  /// - `E` is the mutable environment variable.
  ///
  /// # Notes
  ///
  /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
  /// a simpler interface.
  pub fn from_stages_env<'b, E>(
    &mut self,
    vertex: &'b Stage<C::Backend>,
    tess: impl Into<Option<TessellationStages<'b, Stage<C::Backend>>>>,
    geometry: impl Into<Option<&'b Stage<C::Backend>>>,
    fragment: &'b Stage<C::Backend>,
    env: &mut E,
  ) -> Result<BuiltProgram<C::Backend, V, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<C::Backend, E>,
  {
    let tess = tess.into();
    let geometry = geometry.into();

    unsafe {
      let mut repr = self.ctx.backend().new_program(
        &vertex.repr,
        tess.map(|stages| TessellationStages {
          control: &stages.control.repr,
          evaluation: &stages.evaluation.repr,
        }),
        geometry.map(|stage| &stage.repr),
        &fragment.repr,
      )?;

      let warnings = C::Backend::apply_semantics::<V>(&mut repr)?
        .into_iter()
        .map(|w| ProgramError::Warning(w.into()))
        .collect();

      let mut uniform_builder =
        C::Backend::new_uniform_builder(&mut repr).map(|repr| UniformBuilder {
          repr,
          warnings: Vec::new(),
          _a: PhantomData,
        })?;

      let uni =
        Uni::uniform_interface(&mut uniform_builder, env).map_err(ProgramWarning::Uniform)?;

      let program = Program {
        repr,
        uni,
        _phantom: PhantomData,
      };

      Ok(BuiltProgram { program, warnings })
    }
  }

  /// Create a [`Program`] by linking [`Stage`]s.
  ///
  /// # Parametricity
  ///
  /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`Stage`] inside.
  /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
  ///
  /// # Notes
  ///
  /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
  /// a simpler interface.
  pub fn from_stages<'b>(
    &mut self,
    vertex: &'b Stage<C::Backend>,
    tess: impl Into<Option<TessellationStages<'b, Stage<C::Backend>>>>,
    geometry: impl Into<Option<&'b Stage<C::Backend>>>,
    fragment: &'b Stage<C::Backend>,
  ) -> Result<BuiltProgram<C::Backend, V, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<C::Backend>,
  {
    Self::from_stages_env(self, vertex, tess, geometry, fragment, &mut ())
  }

  /// Create a [`Program`] by linking [`&str`]s and accessing a mutable environment variable.
  ///
  /// # Parametricity
  ///
  /// - `C` is the graphics context.
  /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`&str`] inside.
  /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
  /// - `E` is the mutable environment variable.
  ///
  /// # Notes
  ///
  /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
  /// a simpler interface.
  ///
  /// [`&str`]: str
  pub fn from_strings_env<'b, T, G, E>(
    &mut self,
    vertex: &'b str,
    tess: T,
    geometry: G,
    fragment: &'b str,
    env: &mut E,
  ) -> Result<BuiltProgram<C::Backend, V, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<C::Backend, E>,
    T: Into<Option<TessellationStages<'b, str>>>,
    G: Into<Option<&'b str>>,
  {
    let vs_stage = Stage::new(self.ctx, StageType::VertexShader, vertex)?;

    let tess_stages = match tess.into() {
      Some(TessellationStages {
        control,
        evaluation,
      }) => {
        let control_stage = Stage::new(self.ctx, StageType::TessellationControlShader, control)?;
        let evaluation_stage = Stage::new(
          self.ctx,
          StageType::TessellationEvaluationShader,
          evaluation,
        )?;
        Some((control_stage, evaluation_stage))
      }
      None => None,
    };
    let tess_stages =
      tess_stages
        .as_ref()
        .map(|(ref control, ref evaluation)| TessellationStages {
          control,
          evaluation,
        });

    let gs_stage = match geometry.into() {
      Some(geometry) => Some(Stage::new(self.ctx, StageType::GeometryShader, geometry)?),
      None => None,
    };

    let fs_stage = Stage::new(self.ctx, StageType::FragmentShader, fragment)?;

    Self::from_stages_env(
      self,
      &vs_stage,
      tess_stages,
      gs_stage.as_ref(),
      &fs_stage,
      env,
    )
  }

  /// Create a [`Program`] by linking [`&str`]s.
  ///
  /// # Parametricity
  ///
  /// - `C` is the graphics context.
  /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`&str`] inside.
  /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
  ///
  /// # Notes
  ///
  /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
  /// a simpler interface.
  ///
  /// [`&str`]: str
  pub fn from_strings<'b, T, G>(
    &mut self,
    vertex: &'b str,
    tess: T,
    geometry: G,
    fragment: &'b str,
  ) -> Result<BuiltProgram<C::Backend, V, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<C::Backend>,
    T: Into<Option<TessellationStages<'b, str>>>,
    G: Into<Option<&'b str>>,
  {
    Self::from_strings_env(self, vertex, tess, geometry, fragment, &mut ())
  }
}

/// A shader program.
///
/// Shader programs are GPU binaries that execute when a draw command is issued.
///
/// # Parametricity
///
/// - `B` is the backend type.
/// - `Sem` is the [`Semantics`] type.
/// - `Out` is the render target type.
/// - `Uni` is the [`UniformInterface`] type.
pub struct Program<B, V, Out, Uni>
where
  B: Shader,
{
  pub(crate) repr: B::ProgramRepr,
  pub(crate) uni: Uni,
  _phantom: PhantomData<*const (V, Out)>,
}

impl<B, V, Out, Uni> Program<B, V, Out, Uni>
where
  B: Shader,
  V: Vertex,
{
  /// Create a new [`UniformInterface`] but keep the [`Program`] around without rebuilding it.
  ///
  /// # Parametricity
  ///
  /// - `Q` is the new [`UniformInterface`].
  pub fn adapt<Q>(self) -> Result<BuiltProgram<B, V, Out, Q>, AdaptationFailure<B, V, Out, Uni>>
  where
    Q: UniformInterface<B>,
  {
    self.adapt_env(&mut ())
  }

  /// Create a new [`UniformInterface`] but keep the [`Program`] around without rebuilding it, by
  /// using a mutable environment variable.
  ///
  /// # Parametricity
  ///
  /// - `Q` is the new [`UniformInterface`].
  /// - `E` is the mutable environment variable.
  pub fn adapt_env<Q, E>(
    mut self,
    env: &mut E,
  ) -> Result<BuiltProgram<B, V, Out, Q>, AdaptationFailure<B, V, Out, Uni>>
  where
    Q: UniformInterface<B, E>,
  {
    // first, try to create the new uniform interface
    let mut uniform_builder: UniformBuilder<B> =
      match unsafe { B::new_uniform_builder(&mut self.repr) } {
        Ok(repr) => UniformBuilder {
          repr,
          warnings: Vec::new(),
          _a: PhantomData,
        },

        Err(e) => return Err(AdaptationFailure::new(self, e)),
      };

    let uni = match Q::uniform_interface(&mut uniform_builder, env) {
      Ok(uni) => uni,
      Err(e) => {
        return Err(AdaptationFailure::new(
          self,
          ProgramWarning::Uniform(e).into(),
        ))
      }
    };

    let warnings = uniform_builder
      .warnings
      .into_iter()
      .map(|w| ProgramError::Warning(w.into()))
      .collect();

    let program = Program {
      repr: self.repr,
      uni,
      _phantom: PhantomData,
    };

    Ok(BuiltProgram { program, warnings })
  }

  /// Re-create the [`UniformInterface`] but keep the [`Program`] around without rebuilding it.
  ///
  /// # Parametricity
  ///
  /// - `E` is the mutable environment variable.
  pub fn readapt_env<E>(
    self,
    env: &mut E,
  ) -> Result<BuiltProgram<B, V, Out, Uni>, AdaptationFailure<B, V, Out, Uni>>
  where
    Uni: UniformInterface<B, E>,
  {
    self.adapt_env(env)
  }
}

/// Shader data.
///
/// # Parametricity
///
/// - `B` is the backend type.
/// - `T` is the type of the carried items.
pub struct ShaderData<B, T>
where
  B: ?Sized + ShaderDataBackend<T>,
{
  pub(crate) repr: B::ShaderDataRepr,
}

impl<B, T> ShaderData<B, T>
where
  B: ?Sized + ShaderDataBackend<T>,
{
  /// Create a [`ShaderData`] via an iterator of values.
  pub fn new(
    ctx: &mut impl GraphicsContext<Backend = B>,
    values: impl IntoIterator<Item = T>,
  ) -> Result<Self, ShaderDataError> {
    let repr = unsafe { ctx.backend().new_shader_data(values.into_iter())? };
    Ok(Self { repr })
  }

  /// Get the value at index `i`.
  pub fn at(&self, i: usize) -> Result<T, ShaderDataError> {
    unsafe { B::get_shader_data_at(&self.repr, i) }
  }

  /// Set the item at index `i` with the value `x`.
  ///
  /// Return the previous value.
  pub fn set(&mut self, i: usize, x: T) -> Result<T, ShaderDataError> {
    unsafe { B::set_shader_data_at(&mut self.repr, i, x) }
  }

  /// Replace all the values with the one provided by the iterator.
  pub fn replace(&mut self, values: impl IntoIterator<Item = T>) -> Result<(), ShaderDataError> {
    unsafe { B::set_shader_data_values(&mut self.repr, values.into_iter()) }
  }
}

/// Possible errors that can occur with shader data.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShaderDataError {
  /// Cannot create the shader data on the backend side.
  CannotCreate,

  /// Index out of bounds.,
  OutOfBounds {
    /// Tried (incorrect) index.
    index: usize,
  },

  /// Cannot set data.
  CannotSetData {
    /// Tried (incorrect) index.
    index: usize,
  },

  /// Cannot replace data.
  CannotReplaceData,
}

impl fmt::Display for ShaderDataError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      ShaderDataError::CannotCreate => f.write_str("cannot create shader data"),

      ShaderDataError::OutOfBounds { index } => {
        write!(f, "cannot get shader data item; out of bounds {}", index)
      }

      ShaderDataError::CannotSetData { index } => {
        write!(f, "cannot get shader data item; out of bounds {}", index)
      }

      ShaderDataError::CannotReplaceData => f.write_str("cannot replace shader data"),
    }
  }
}

impl std::error::Error for ShaderDataError {}
