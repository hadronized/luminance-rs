//! Shader backend interface.
//!
//! This interface defines the low-level API shaders must implement to be usable.
//!
//! Shader support is quite complex and requires several concepts to be implemented by the backend. The first one is the
//! concept of « shader stage ». A shader stage represents a single logic of shader processing. Oftentimes, backends
//! support at least two of them:
//!
//! - Vertex shader, which is run for all vertices.
//! - Fragment shader, which is run for all fragments rasterized by the backend.
//!
//! Other backends support optional backend stages, such as:
//!
//! - Geometry shader, which is run for every primitive (point, lines, triangles, etc.).
//! - Tessellation shaders, run to tessellate the vertex stream.
//! - Compute shaders, special kind of shaders used to compute non-image related data on the backend using the shader
//!   pipeline.
//!
//! Then, the concept of a « shader program », which agregates shader stages into a single entity after a process of «
//! linking » the various shader stages. A shader program is a pipeline resource, so it will be used inside a graphics
//! pipeline to shade a scene. At the higher level, shader programs are typed with different type variables that don’t
//! leak in the backend, but some have meaning, which is a good transition to the next concept: uniforms. In this
//! backend, uniforms are user-defined structures that can only be built by backend-specific ways. This is why another
//! trait must be implement do perform all the lookups and uniforms construction.
//!
//! Finally, some traits exist to provide more features, such as [`ShaderData`] to support shader data operations.

use crate::{
  shader::{
    ProgramError, ShaderDataError, StageError, StageType, TessellationStages, Uniform, UniformType,
    UniformWarning, VertexAttribWarning,
  },
  vertex::Vertex,
};

/// Backend support for uniforms.
///
/// When a backend implements [`Uniformable`], it adds support for the type parameter as being a recognized _uniform
/// type_ and then can be mapped in a uniform interface via [`Uniform`].
///
/// Implementing such a trait is relatively trivial:
///
/// - You must implement [`Uniformable::ty`], which reifies the type of the uniform using [`UniformType`]. If your
///   uniform type is not supported in [`UniformType`], it means the API doesn’t know about it and then that type cannot
///   be supported.
/// - You must implement [`Uniformable::update`], which updates the value of the [`Uniform`] in a given shader program.
///   For indirect values such as bound resources (textures, shader data, etc.), uploading will most of the time be a
///   binding update on the backend side.
/// - You must provide an associated type that will be the actual type users will pass. This is needed because some
///   uniform types cannot be expressed directly, such as existentials (think of types with lifetimes, for instance).
pub unsafe trait Uniformable<'a, T>: Shader {
  type Target: 'a;

  /// Return the size of the uniform.
  ///
  /// For regular uniform variables, this should be `1`. For arrays, it should be the length of the array.
  /// For anything that is not sized, such as texture bindings, shader data, etc., set it to `0`.
  const SIZE: usize;

  /// Reify the type of the uniform as a [`UniformType`].
  unsafe fn ty() -> UniformType;

  /// Update the associated value of the [`Uniform`] in the given shader program.
  unsafe fn update(program: &mut Self::ProgramRepr, uniform: &'a Uniform<T>, value: Self::Target);
}

/// Shader support.
///
/// This trait provides several concepts as once, as they all depend on each other:
///
/// - Shader stages.
/// - Shader programs.
/// - Uniform builders.
///
/// The associated type [`Shader::StageRepr`] is the backend representation of a shader stage. They are created with
/// [`Shader::new_stage`] with a [`StageType`] representing the shader stage type that must be created. Because the
/// backend might not support this type of shader stage, it might fail with a [`StageError`].
pub unsafe trait Shader {
  /// Backend representation of a shader stage.
  type StageRepr;

  /// Backend representation of a shader program.
  type ProgramRepr;

  /// Backend representation of a uniform builder.
  type UniformBuilderRepr;

  /// Create a new shader stage of type [`StageType`].
  unsafe fn new_stage(&mut self, ty: StageType, src: &str) -> Result<Self::StageRepr, StageError>;

  /// Create a new shader program by combining several shader stages.
  ///
  /// The vertex and fragment stages are mandatory. The other ones are optional and then must be inspected to check
  /// whether they were provided by the user.
  unsafe fn new_program(
    &mut self,
    vertex: &Self::StageRepr,
    tess: Option<TessellationStages<Self::StageRepr>>,
    geometry: Option<&Self::StageRepr>,
    fragment: &Self::StageRepr,
  ) -> Result<Self::ProgramRepr, ProgramError>;

  /// Apply semantics.
  ///
  /// This is a very specific operations that happen right after the shader program got successfully created by the
  /// backend. This function is responsible in setting whatever might be needed by the backend to allocate, prepare or
  /// validate the semantics — i.e. `Sem` which implements [`Semantics`].
  unsafe fn apply_semantics<V>(
    program: &mut Self::ProgramRepr,
  ) -> Result<Vec<VertexAttribWarning>, ProgramError>
  where
    V: Vertex;

  /// Construct a new uniform builder.
  ///
  /// This method must create a uniform builder, which will be used when passed to the user.
  unsafe fn new_uniform_builder(
    program: &mut Self::ProgramRepr,
  ) -> Result<Self::UniformBuilderRepr, ProgramError>;

  /// Lookup a [`Uniform`].
  ///
  /// This method must lookup a [`Uniform`] and map it, or return the appropriate error.
  unsafe fn ask_uniform<T>(
    uniform_builder: &mut Self::UniformBuilderRepr,
    name: &str,
  ) -> Result<Uniform<T>, UniformWarning>
  where
    Self: for<'u> Uniformable<'u, T>;

  /// Backend representation of an _unbound_ [`Uniform`] (i.e. that is inactive in the shader program).
  ///
  /// This is a method taking a uniform builder so that the builder can accumulate a state.
  unsafe fn unbound<T>(uniform_builder: &mut Self::UniformBuilderRepr) -> Uniform<T>
  where
    Self: for<'u> Uniformable<'u, T>;
}

/// Shader data backend.
pub unsafe trait ShaderData<T> {
  /// Representation of the data by the backend.
  type ShaderDataRepr;

  /// Build a new shader data from some values represented via an iterator.
  unsafe fn new_shader_data(
    &mut self,
    values: impl Iterator<Item = T>,
  ) -> Result<Self::ShaderDataRepr, ShaderDataError>;

  /// Access an item at index `i`.
  unsafe fn get_shader_data_at(
    shader_data: &Self::ShaderDataRepr,
    i: usize,
  ) -> Result<T, ShaderDataError>;

  /// Set an item at index `i`.
  ///
  /// Return the previous value.
  unsafe fn set_shader_data_at(
    shader_data: &mut Self::ShaderDataRepr,
    i: usize,
    x: T,
  ) -> Result<T, ShaderDataError>;

  /// Set values by providing an iterator.
  unsafe fn set_shader_data_values(
    shader_data: &mut Self::ShaderDataRepr,
    values: impl Iterator<Item = T>,
  ) -> Result<(), ShaderDataError>;
}
