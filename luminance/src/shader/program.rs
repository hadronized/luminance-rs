//! Shader programs related types and functions.
//!
//! A shader `Program` is an object representing several operations. It’s a streaming program that
//! will operate on vertices, vertex patches, primitives and/or fragments.
//!
//! > *Note: shader programs don’t have to run on all those objects; they can be ran only on
//! vertices and fragments, for instance*.
//!
//! Creating a shader program is very simple. You need shader `Stage`s representing each step of the
//! processing.
//!
//! You *have* to provide at least a vertex and a fragment stages. If you want tessellation
//! processing, you need to provide a tessellation control and tessellation evaluation stages. If
//! you want primitives processing, you need to add a geometry stage.
//!
//! In order to customize the behavior of your shader programs, you have access to *uniforms*. For
//! more details about them, see the documentation for the type `Uniform` and trait `Uniformable`.
//! When creating a new shader program, you have to provide code to declare its *uniform semantics*.
//!
//! The *uniform semantics* represent a mapping between the variables declared in your shader
//! sources and variables you have access in your host code in Rust. Typically, you declare your
//! variable – `Uniform` – in Rust as `const` and use the function `Uniform::sem` to get the
//! semantic associated with the string you pass in.
//!
//! > **Becareful: currently, uniforms are a bit messy as you have to provide a per-program unique
//! number when you use the `Uniform::new` method. Efforts will be done in that direction in later
//! releases.
//!
//! You can create a `Program` with its `new` associated function.

#[cfg(feature = "std")]
use std::ffi::CString;
#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::marker::PhantomData;
#[cfg(feature = "std")]
use std::ops::Deref;
#[cfg(feature = "std")]
use std::ptr::null_mut;

#[cfg(not(feature = "std"))]
use alloc::prelude::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::fmt::{self, Write};
#[cfg(not(feature = "std"))]
use core::marker::PhantomData;
#[cfg(not(feature = "std"))]
use core::ops::Deref;
#[cfg(not(feature = "std"))]
use core::ptr::null_mut;

use crate::linear::{M22, M33, M44};
use crate::metagl::*;
use crate::shader::stage::{self, Stage, StageError};
use crate::vertex::Semantics;

/// A raw shader program.
///
/// This is a type-erased version of a `Program`.
#[derive(Debug)]
pub struct RawProgram {
  handle: GLuint,
}

impl RawProgram {
  /// Create a new program by attaching shader stages.
  fn new<'a, T, G>(tess: T, vertex: &Stage, geometry: G, fragment: &Stage) -> Result<Self, ProgramError>
  where T: Into<Option<(&'a Stage, &'a Stage)>>,
        G: Into<Option<&'a Stage>> {
    unsafe {
      let handle = gl::CreateProgram();

      if let Some((tcs, tes)) = tess.into() {
        gl::AttachShader(handle, tcs.handle());
        gl::AttachShader(handle, tes.handle());
      }

      gl::AttachShader(handle, vertex.handle());

      if let Some(geometry) = geometry.into() {
        gl::AttachShader(handle, geometry.handle());
      }

      gl::AttachShader(handle, fragment.handle());

      let program = RawProgram { handle };
      program.link().map(move |_| program)
    }
  }

  /// Link a program.
  fn link(&self) -> Result<(), ProgramError> {
    let handle = self.handle;

    unsafe {
      gl::LinkProgram(handle);

      let mut linked: GLint = gl::FALSE as GLint;
      gl::GetProgramiv(handle, gl::LINK_STATUS, &mut linked);

      if linked == (gl::TRUE as GLint) {
        Ok(())
      } else {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetProgramInfoLog(handle, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

        gl::DeleteProgram(handle);

        log.set_len(log_len as usize);

        Err(ProgramError::LinkFailed(String::from_utf8(log).unwrap()))
      }
    }
  }


  #[inline]
  pub(crate) fn handle(&self) -> GLuint {
    self.handle
  }
}

impl Drop for RawProgram {
  fn drop(&mut self) {
    unsafe { gl::DeleteProgram(self.handle) }
  }
}

/// A typed shader program.
///
/// Typed shader programs represent their inputs, outputs and environment (uniforms) directly in
/// their types. This is very interesting as it adds more static safety and enables such programs
/// to *“store”* information like the uniform interface and such.
pub struct Program<S, Out, Uni> {
  raw: RawProgram,
  uni_iface: Uni,
  _in: PhantomData<*const S>,
  _out: PhantomData<*const Out>,
}

impl<S, Out, Uni> Program<S, Out, Uni> where S: Semantics {
  /// Create a new program by consuming `Stage`s.
  pub fn from_stages<'a, T, G>(
    tess: T,
    vertex: &Stage,
    geometry: G,
    fragment: &Stage,
  ) -> Result<(Self, Vec<ProgramWarning>), ProgramError>
  where Uni: UniformInterface,
        T: Into<Option<(&'a Stage, &'a Stage)>>,
        G: Into<Option<&'a Stage>> {
    Self::from_stages_env(tess, vertex, geometry, fragment, ())
  }

  /// Create a new program by consuming strings.
  pub fn from_strings<'a, T, G>(
    tess: T,
    vertex: &str,
    geometry: G,
    fragment: &str,
  ) -> Result<(Self, Vec<ProgramWarning>), ProgramError>
  where Uni: UniformInterface,
        T: Into<Option<(&'a str, &'a str)>>,
        G: Into<Option<&'a str>> {
    Self::from_strings_env(tess, vertex, geometry, fragment, ())
  }

  /// Create a new program by consuming `Stage`s and by looking up an environment.
  pub fn from_stages_env<'a, E, T, G>(
    tess: T,
    vertex: &Stage,
    geometry: G,
    fragment: &Stage,
    env: E,
  ) -> Result<(Self, Vec<ProgramWarning>), ProgramError>
  where Uni: UniformInterface<E>,
        T: Into<Option<(&'a Stage, &'a Stage)>>,
        G: Into<Option<&'a Stage>> {
    let raw = RawProgram::new(tess, vertex, geometry, fragment)?;

    let mut warnings = bind_vertex_attribs_locations::<S>(&raw);

    raw.link()?;

    let (uni_iface, uniform_warnings) = create_uniform_interface(&raw, env)?;
    warnings.extend(uniform_warnings.into_iter().map(ProgramWarning::Uniform));

    let program = Program {
      raw,
      uni_iface,
      _in: PhantomData,
      _out: PhantomData,
    };

    Ok((program, warnings))
  }

  /// Create a new program by consuming strings.
  pub fn from_strings_env<'a, E, T, G>(
    tess: T,
    vertex: &str,
    geometry: G,
    fragment: &str,
    env: E,
  ) -> Result<(Self, Vec<ProgramWarning>), ProgramError>
  where Uni: UniformInterface<E>,
        T: Into<Option<(&'a str, &'a str)>>,
        G: Into<Option<&'a str>> {
    let tess = match tess.into() {
      Some((tcs_str, tes_str)) => {
        let tcs =
          Stage::new(stage::Type::TessellationControlShader, tcs_str).map_err(ProgramError::StageError)?;
        let tes =
          Stage::new(stage::Type::TessellationControlShader, tes_str).map_err(ProgramError::StageError)?;
        Some((tcs, tes))
      }
      None => None,
    };

    let gs = match geometry.into() {
      Some(gs_str) => {
        Some(Stage::new(stage::Type::GeometryShader, gs_str).map_err(ProgramError::StageError)?)
      }
      None => None,
    };

    let vs = Stage::new(stage::Type::VertexShader, vertex).map_err(ProgramError::StageError)?;
    let fs = Stage::new(stage::Type::FragmentShader, fragment).map_err(ProgramError::StageError)?;

    Self::from_stages_env(
      tess.as_ref().map(|&(ref tcs, ref tes)| (tcs, tes)),
      &vs,
      gs.as_ref(),
      &fs,
      env,
    )
  }

  /// Get the program interface associated with this program.
  pub(crate) fn interface<'a>(&'a self) -> ProgramInterface<'a, Uni> {
    let raw_program = &self.raw;
    let uniform_interface = &self.uni_iface;

    ProgramInterface {
      raw_program,
      uniform_interface,
    }
  }

  /// Transform the program to adapt the uniform interface.
  ///
  /// This function will not re-allocate nor recreate the GPU data. It will try to change the
  /// uniform interface and if the new uniform interface is correctly generated, return the same
  /// shader program updated with the new uniform interface. If the generation of the new uniform
  /// interface fails, this function will return the program with the former uniform interface.
  pub fn adapt<Q>(self) -> Result<(Program<S, Out, Q>, Vec<UniformWarning>), (ProgramError, Self)>
  where Q: UniformInterface {
    self.adapt_env(())
  }

  /// Transform the program to adapt the uniform interface by looking up an environment.
  ///
  /// This function will not re-allocate nor recreate the GPU data. It will try to change the
  /// uniform interface and if the new uniform interface is correctly generated, return the same
  /// shader program updated with the new uniform interface. If the generation of the new uniform
  /// interface fails, this function will return the program with the former uniform interface.
  pub fn adapt_env<Q, E>(
    self,
    env: E,
  ) -> Result<(Program<S, Out, Q>, Vec<UniformWarning>), (ProgramError, Self)>
  where Q: UniformInterface<E> {
    // first, try to create the new uniform interface
    let new_uni_iface = create_uniform_interface(&self.raw, env);

    match new_uni_iface {
      Ok((uni_iface, warnings)) => {
        // if we have succeeded, return self with the new uniform interface
        let program = Program {
          raw: self.raw,
          uni_iface,
          _in: PhantomData,
          _out: PhantomData,
        };

        Ok((program, warnings))
      }

      Err(iface_err) => {
        // we couldn’t generate the new uniform interface; return the error(s) that occurred and the
        // the untouched former program
        Err((iface_err, self))
      }
    }
  }

  /// A version of [`Program::adapt_env`] that doesn’t change the uniform interface type.
  ///
  /// This function might be needed for when you want to update the uniform interface but still
  /// enforce that the type must remain the same.
  pub fn readapt_env<E>(self, env: E) -> Result<(Self, Vec<UniformWarning>), (ProgramError, Self)>
  where Uni: UniformInterface<E> {
    self.adapt_env(env)
  }
}

impl<S, Out, Uni> Deref for Program<S, Out, Uni> {
  type Target = RawProgram;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

/// Class of types that can act as uniform interfaces in typed programs.
///
/// A uniform interface is a value that contains uniforms. The purpose of a uniform interface is to
/// be stored in a typed program and handed back when the program is made available in a pipeline.
///
/// The `E` type variable represents the environment and might be used to drive the implementation
/// from a value. It’s defaulted to `()` so that if you don’t use the environment, you don’t have to
/// worry about that value when creating the shader program.
pub trait UniformInterface<E = ()>: Sized {
  /// Build the uniform interface.
  ///
  /// When mapping a uniform, if you want to accept failures, you can discard the error and use
  /// `UniformBuilder::unbound` to let the uniform pass through, and collect the uniform warning.
  fn uniform_interface<'a>(builder: &mut UniformBuilder<'a>, env: E) -> Result<Self, ProgramError>;
}

impl UniformInterface for () {
  fn uniform_interface<'a>(_: &mut UniformBuilder<'a>, _: ()) -> Result<Self, ProgramError> {
    Ok(())
  }
}

/// Build uniforms to fold them to a uniform interface.
pub struct UniformBuilder<'a> {
  raw: &'a RawProgram,
  warnings: Vec<UniformWarning>,
}

impl<'a> UniformBuilder<'a> {
  fn new(raw: &'a RawProgram) -> Self {
    UniformBuilder {
      raw,
      warnings: Vec::new(),
    }
  }

  /// Have the builder hand you a `Uniform` of the type of your choice.
  ///
  /// Keep in mind that it’s possible that this function fails if you ask for a type for which the
  /// one defined in the shader doesn’t type match. If you don’t want a failure but an *unbound*
  /// uniform, head over to the `ask_unbound` function.
  pub fn ask<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where T: Uniformable {
    let uniform = match T::ty() {
      Type::BufferBinding => self.ask_uniform_block(name)?,
      _ => self.ask_uniform(name)?,
    };

    uniform_type_match(self.raw.handle, name, T::ty())
      .map_err(|err| UniformWarning::TypeMismatch(name.to_owned(), err))?;

    Ok(uniform)
  }

  pub fn ask_unbound<T>(&mut self, name: &str) -> Uniform<T>
  where T: Uniformable {
    match self.ask(name) {
      Ok(uniform) => uniform,
      Err(warning) => {
        self.warnings.push(warning);
        self.unbound()
      }
    }
  }

  fn ask_uniform<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where T: Uniformable {
    let location = {
      #[cfg(feature = "std")]
      {
        let c_name = CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetUniformLocation(self.raw.handle, c_name.as_ptr() as *const GLchar) }
      }

      #[cfg(not(feature = "std"))]
      {
        unsafe { with_cstring(name, |c_name| gl::GetUniformLocation(self.raw.handle, c_name)).unwrap_or(-1) }
      }
    };

    if location < 0 {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location))
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where T: Uniformable {
    let location = {
      #[cfg(feature = "std")]
      {
        let c_name = CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetUniformBlockIndex(self.raw.handle, c_name.as_ptr() as *const GLchar) }
      }

      #[cfg(not(feature = "std"))]
      {
        unsafe {
          with_cstring(name, |c_name| gl::GetUniformBlockIndex(self.raw.handle, c_name))
            .unwrap_or(gl::INVALID_INDEX)
        }
      }
    };

    if location == gl::INVALID_INDEX {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location as GLint))
    }
  }

  /// Special uniform that won’t do anything.
  ///
  /// Use that function when you need a uniform to complete a uniform interface but you’re sure you
  /// won’t use it.
  pub fn unbound<T>(&self) -> Uniform<T> where T: Uniformable {
    Uniform::unbound(self.raw.handle)
  }
}

/// The shader program interface.
///
/// This struct gives you access to several capabilities, among them:
///
///   - The typed *uniform interface* you would have acquired earlier.
///   - Some functions to query more data dynamically.
pub struct ProgramInterface<'a, Uni> {
  raw_program: &'a RawProgram,
  uniform_interface: &'a Uni,
}

impl<'a, Uni> Deref for ProgramInterface<'a, Uni> {
  type Target = Uni;

  fn deref(&self) -> &Self::Target {
    self.uniform_interface
  }
}

impl<'a, Uni> ProgramInterface<'a, Uni> {
  pub fn query(&'a self) -> UniformBuilder<'a> {
    UniformBuilder::new(self.raw_program)
  }
}

/// Errors that a `Program` can generate.
#[derive(Debug)]
pub enum ProgramError {
  StageError(StageError),
  /// Program link failed. You can inspect the reason by looking at the contained `String`.
  LinkFailed(String),
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
  UniformWarning(UniformWarning),
  /// Some vertex attribute is ill-formed.
  VertexAttribWarning(VertexAttribWarning)
}

impl fmt::Display for ProgramError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ProgramError::StageError(ref e) => write!(f, "shader program has stage error: {}", e),

      ProgramError::LinkFailed(ref s) => write!(f, "shader program failed to link: {}", s),

      ProgramError::UniformWarning(ref e) => write!(f, "shader program contains uniform warning(s): {}", e),
      ProgramError::VertexAttribWarning(ref e) => write!(f, "shader program contains vertex attribute warning(s): {}", e),
    }
  }
}

/// Program warnings, not necessarily considered blocking errors.
#[derive(Debug)]
pub enum ProgramWarning {
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
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

/// Warnings related to uniform issues.
#[derive(Debug)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),
  /// Type mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, String),
}

impl fmt::Display for UniformWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      UniformWarning::Inactive(ref s) => write!(f, "inactive {} uniform", s),

      UniformWarning::TypeMismatch(ref n, ref t) => write!(f, "type mismatch for uniform {}: {}", n, t),
    }
  }
}

/// Warnings related to vertex attributes issues.
#[derive(Debug)]
pub enum VertexAttribWarning {
  /// Inactive vertex attribute (not read).
  Inactive(String)
}

impl fmt::Display for VertexAttribWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      VertexAttribWarning::Inactive(ref s) => write!(f, "inactive {} vertex attribute", s)
    }
  }
}

/// A contravariant shader uniform. `Uniform<T>` doesn’t hold any value. It’s more like a mapping
/// between the host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<T> {
  program: GLuint,
  index: GLint,
  _t: PhantomData<*const T>,
}

impl<T> Uniform<T> where T: Uniformable {
  fn new(program: GLuint, index: GLint) -> Self {
    Uniform {
      program,
      index,
      _t: PhantomData,
    }
  }

  fn unbound(program: GLuint) -> Self {
    Uniform {
      program,
      index: -1,
      _t: PhantomData,
    }
  }

  pub(crate) fn program(&self) -> GLuint {
    self.program
  }

  pub(crate) fn index(&self) -> GLint {
    self.index
  }

  /// Update the value pointed by this uniform.
  pub fn update(&self, x: T) {
    x.update(self);
  }
}

/// Type of a uniform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  // scalars
  Int,
  UInt,
  Float,
  Bool,
  // vectors
  IVec2,
  IVec3,
  IVec4,
  UIVec2,
  UIVec3,
  UIVec4,
  Vec2,
  Vec3,
  Vec4,
  BVec2,
  BVec3,
  BVec4,
  // matrices
  M22,
  M33,
  M44,
  // textures
  ISampler1D,
  ISampler2D,
  ISampler3D,
  UISampler1D,
  UISampler2D,
  UISampler3D,
  Sampler1D,
  Sampler2D,
  Sampler3D,
  ICubemap,
  UICubemap,
  Cubemap,
  // buffer
  BufferBinding,
}

/// Types that can behave as `Uniform`.
pub unsafe trait Uniformable: Sized {
  /// Update the uniform with a new value.
  fn update(self, u: &Uniform<Self>);
  /// Retrieve the `Type` of the uniform.
  fn ty() -> Type;
}

unsafe impl Uniformable for i32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1i(u.index, self) }
  }

  fn ty() -> Type {
    Type::Int
  }
}

unsafe impl Uniformable for [i32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec2
  }
}

unsafe impl Uniformable for [i32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec3
  }
}

unsafe impl Uniformable for [i32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec4
  }
}

unsafe impl<'a> Uniformable for &'a [i32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1iv(u.index, self.len() as GLsizei, self.as_ptr()) }
  }

  fn ty() -> Type {
    Type::Int
  }
}

unsafe impl<'a> Uniformable for &'a [[i32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec2
  }
}

unsafe impl<'a> Uniformable for &'a [[i32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec3
  }
}

unsafe impl<'a> Uniformable for &'a [[i32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type {
    Type::IVec4
  }
}

unsafe impl Uniformable for u32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self) }
  }

  fn ty() -> Type {
    Type::UInt
  }
}

unsafe impl Uniformable for [u32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec2
  }
}

unsafe impl Uniformable for [u32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec3
  }
}

unsafe impl Uniformable for [u32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec4
  }
}

unsafe impl<'a> Uniformable for &'a [u32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::UInt
  }
}

unsafe impl<'a> Uniformable for &'a [[u32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec2
  }
}

unsafe impl<'a> Uniformable for &'a [[u32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec3
  }
}

unsafe impl<'a> Uniformable for &'a [[u32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::UIVec4
  }
}

unsafe impl Uniformable for f32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1f(u.index, self) }
  }

  fn ty() -> Type {
    Type::Float
  }
}

unsafe impl Uniformable for [f32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec2
  }
}

unsafe impl Uniformable for [f32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec3
  }
}

unsafe impl Uniformable for [f32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec4
  }
}

unsafe impl<'a> Uniformable for &'a [f32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::Float
  }
}

unsafe impl<'a> Uniformable for &'a [[f32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec2
  }
}

unsafe impl<'a> Uniformable for &'a [[f32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec3
  }
}

unsafe impl<'a> Uniformable for &'a [[f32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::Vec4
  }
}

unsafe impl Uniformable for M22 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix2fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::M22
  }
}

unsafe impl Uniformable for M33 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix3fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::M33
  }
}

unsafe impl Uniformable for M44 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix4fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type {
    Type::M44
  }
}

unsafe impl<'a> Uniformable for &'a [M22] {
  fn update(self, u: &Uniform<Self>) {
    unsafe {
      gl::UniformMatrix2fv(
        u.index,
        self.len() as GLsizei,
        gl::FALSE,
        self.as_ptr() as *const f32,
      )
    }
  }

  fn ty() -> Type {
    Type::M22
  }
}

unsafe impl<'a> Uniformable for &'a [M33] {
  fn update(self, u: &Uniform<Self>) {
    unsafe {
      gl::UniformMatrix3fv(
        u.index,
        self.len() as GLsizei,
        gl::FALSE,
        self.as_ptr() as *const f32,
      )
    }
  }

  fn ty() -> Type {
    Type::M33
  }
}

unsafe impl<'a> Uniformable for &'a [M44] {
  fn update(self, u: &Uniform<Self>) {
    unsafe {
      gl::UniformMatrix4fv(
        u.index,
        self.len() as GLsizei,
        gl::FALSE,
        self.as_ptr() as *const f32,
      )
    }
  }

  fn ty() -> Type {
    Type::M44
  }
}

unsafe impl Uniformable for bool {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self as GLuint) }
  }

  fn ty() -> Type {
    Type::Bool
  }
}

unsafe impl Uniformable for [bool; 2] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32];
    unsafe { gl::Uniform2uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec2
  }
}

unsafe impl Uniformable for [bool; 3] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32];
    unsafe { gl::Uniform3uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec3
  }
}

unsafe impl Uniformable for [bool; 4] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32, self[3] as u32];
    unsafe { gl::Uniform4uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec4
  }
}

unsafe impl<'a> Uniformable for &'a [bool] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(u.index, v.len() as GLsizei, v.as_ptr()) }
  }

  fn ty() -> Type {
    Type::Bool
  }
}

unsafe impl<'a> Uniformable for &'a [[bool; 2]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec2
  }
}

unsafe impl<'a> Uniformable for &'a [[bool; 3]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32])
      .collect();
    unsafe { gl::Uniform3uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec3
  }
}

unsafe impl<'a> Uniformable for &'a [[bool; 4]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32])
      .collect();
    unsafe { gl::Uniform4uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type {
    Type::BVec4
  }
}

// Check whether a shader program’s uniform type matches the type we have chosen.
fn uniform_type_match(program: GLuint, name: &str, ty: Type) -> Result<(), String> {
  let mut size: GLint = 0;
  let mut typ: GLuint = 0;

  unsafe {
    // get the max length of the returned names
    let mut max_len = 0;
    gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_len);

    // get the index of the uniform
    let mut index = 0;

    #[cfg(feature = "std")]
    {
      let c_name = CString::new(name.as_bytes()).unwrap();
      gl::GetUniformIndices(program, 1, [c_name.as_ptr() as *const GLchar].as_ptr(), &mut index);
    }

    #[cfg(not(feature = "std"))]
    {
      let r = with_cstring(name, |c_name| {
        gl::GetUniformIndices(program, 1, [c_name].as_ptr(), &mut index);
      });

      if let Err(_) = r {
        #[cfg(feature = "std")]
        {
          return Err(format!("unable to find the index of {}", name));
        }

        #[cfg(not(feature = "std"))]
        {
          let mut reason = String::new();
          let _ = write!(&mut reason, "unable to find the index of {}", name);
          return Err(reason);
        }
      }
    }

    // get its size and type
    let mut name_ = Vec::<GLchar>::with_capacity(max_len as usize);
    gl::GetActiveUniform(
      program,
      index,
      max_len,
      null_mut(),
      &mut size,
      &mut typ,
      name_.as_mut_ptr(),
    );
  }

  // early-return if array – we don’t support them yet
  if size != 1 {
    return Ok(());
  }

  // helper function for error reporting
  let type_mismatch = |t| {
    #[cfg(feature = "std")]
    {
      Err(format!("requested {} doesn’t match", t))
    }

    #[cfg(not(feature = "std"))]
    {
      let mut reason = String::new();
      let _ = write!(&mut reason, "requested {} doesn’t match", t);
      Err(reason)
    }
  };

  match ty {
    // scalars
    Type::Int if typ != gl::INT => type_mismatch("int"),
    Type::UInt if typ != gl::UNSIGNED_INT => type_mismatch("uint"),
    Type::Float if typ != gl::FLOAT => type_mismatch("float"),
    Type::Bool if typ != gl::BOOL => type_mismatch("bool"),
    // vectors
    Type::IVec2 if typ != gl::INT_VEC2 => type_mismatch("ivec2"),
    Type::IVec3 if typ != gl::INT_VEC3 => type_mismatch("ivec3"),
    Type::IVec4 if typ != gl::INT_VEC4 => type_mismatch("ivec4"),
    Type::UIVec2 if typ != gl::UNSIGNED_INT_VEC2 => type_mismatch("uvec2"),
    Type::UIVec3 if typ != gl::UNSIGNED_INT_VEC3 => type_mismatch("uvec3"),
    Type::UIVec4 if typ != gl::UNSIGNED_INT_VEC4 => type_mismatch("uvec4"),
    Type::Vec2 if typ != gl::FLOAT_VEC2 => type_mismatch("vec2"),
    Type::Vec3 if typ != gl::FLOAT_VEC3 => type_mismatch("vec3"),
    Type::Vec4 if typ != gl::FLOAT_VEC4 => type_mismatch("vec4"),
    Type::BVec2 if typ != gl::BOOL_VEC2 => type_mismatch("bvec2"),
    Type::BVec3 if typ != gl::BOOL_VEC3 => type_mismatch("bvec3"),
    Type::BVec4 if typ != gl::BOOL_VEC4 => type_mismatch("bvec4"),
    // matrices
    Type::M22 if typ != gl::FLOAT_MAT2 => type_mismatch("mat2"),
    Type::M33 if typ != gl::FLOAT_MAT3 => type_mismatch("mat3"),
    Type::M44 if typ != gl::FLOAT_MAT4 => type_mismatch("mat4"),
    // textures
    Type::ISampler1D if typ != gl::INT_SAMPLER_1D => type_mismatch("isampler1D"),
    Type::ISampler2D if typ != gl::INT_SAMPLER_2D => type_mismatch("isampler2D"),
    Type::ISampler3D if typ != gl::INT_SAMPLER_3D => type_mismatch("isampler3D"),
    Type::UISampler1D if typ != gl::UNSIGNED_INT_SAMPLER_1D => type_mismatch("usampler1D"),
    Type::UISampler2D if typ != gl::UNSIGNED_INT_SAMPLER_2D => type_mismatch("usampler2D"),
    Type::UISampler3D if typ != gl::UNSIGNED_INT_SAMPLER_3D => type_mismatch("usampler3D"),
    Type::Sampler1D if typ != gl::SAMPLER_1D => type_mismatch("sampler1D"),
    Type::Sampler2D if typ != gl::SAMPLER_2D => type_mismatch("sampler2D"),
    Type::Sampler3D if typ != gl::SAMPLER_3D => type_mismatch("sampler3D"),
    Type::ICubemap if typ != gl::INT_SAMPLER_CUBE => type_mismatch("isamplerCube"),
    Type::UICubemap if typ != gl::UNSIGNED_INT_SAMPLER_CUBE => type_mismatch("usamplerCube"),
    Type::Cubemap if typ != gl::SAMPLER_CUBE => type_mismatch("samplerCube"),
    _ => Ok(()),
  }
}

// Generate a uniform interface and collect warnings.
fn create_uniform_interface<Uni, E>(
  raw: &RawProgram,
  env: E,
) -> Result<(Uni, Vec<UniformWarning>), ProgramError>
where Uni: UniformInterface<E> {
  let mut builder = UniformBuilder::new(raw);
  let iface = Uni::uniform_interface(&mut builder, env)?;
  Ok((iface, builder.warnings))
}

fn bind_vertex_attribs_locations<S>(
  raw: &RawProgram
) -> Vec<ProgramWarning>
where S: Semantics {
  let mut warnings = Vec::new();

  for desc in S::semantics_set() {
    match get_vertex_attrib_location(raw, &desc.name) {
      Ok(_) => {
        let index = desc.index as GLuint;

        // we are not interested in the location as we’re about to change it to what we’ve
        // decided in the semantics
        #[cfg(feature = "std")]
        {
          let c_name = CString::new(desc.name.as_bytes()).unwrap();
          unsafe { gl::BindAttribLocation(raw.handle, index, c_name.as_ptr() as *const GLchar) };
        }

        #[cfg(not(feature = "std"))]
        {
          unsafe {
            with_cstring(fmt.name, |c_name| {
              gl::BindAttribLocation(raw.handle, index, c_name.as_ptr() as *const GLchar);
            });
          }
        }
      }

      Err(warning) => warnings.push(ProgramWarning::VertexAttrib(warning))
    }
  }

  warnings
}

fn get_vertex_attrib_location(
  raw: &RawProgram,
  name: &str
) -> Result<GLuint, VertexAttribWarning> {
  let location = {
    #[cfg(feature = "std")]
    {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetAttribLocation(raw.handle, c_name.as_ptr() as *const GLchar) }
    }

    #[cfg(not(feature = "std"))]
    {
      unsafe {
        with_cstring(name, |c_name| gl::GetAttribLocation(raw.handle, c_name)).unwrap_or(-1)
      }
    }
  };

  if location < 0 {
    Err(VertexAttribWarning::Inactive(name.to_owned()))
  } else {
    Ok(location as _)
  }
}
