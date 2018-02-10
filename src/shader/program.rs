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

use gl;
use gl::types::*;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::null_mut;

use linear::{M22, M33, M44};
use shader::stage::{self, Stage, StageError};
use vertex::Vertex;

pub type Result<A> = ::std::result::Result<A, ProgramError>;

/// A raw shader program.
///
/// This is a type-erased version of a `Program`.
#[derive(Debug)]
pub struct RawProgram {
  handle: GLuint
}

impl RawProgram {
  /// Create a new program by linking shader stages.
  fn new<'a, T, G>(tess: T,
                       vertex: &Stage,
                       geometry: G,
                       fragment: &Stage)
                       -> Result<Self>
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

      gl::LinkProgram(handle);

      let mut linked: GLint = gl::FALSE as GLint;
      gl::GetProgramiv(handle, gl::LINK_STATUS, &mut linked);

      if linked == (gl::TRUE as GLint) {
        Ok(RawProgram { handle: handle })
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
pub struct Program<In, Out, Uni> {
  raw: RawProgram,
  uni_iface: Uni,
  _in: PhantomData<*const In>,
  _out: PhantomData<*const Out>
}

impl<In, Out, Uni> Program<In, Out, Uni> where In: Vertex, Uni: UniformInterface {
  /// Create a new program by consuming `Stage`s.
  pub fn from_stages<'a, T, G>(tess: T,
                               vertex: &Stage,
                               geometry: G,
                               fragment: &Stage)
                               -> Result<(Self, Vec<UniformWarning>)>
                               where T: Into<Option<(&'a Stage, &'a Stage)>>,
                                     G: Into<Option<&'a Stage>> {
    let raw = RawProgram::new(tess, vertex, geometry, fragment)?;
    let (iface, warnings) = Uni::uniform_interface(UniformBuilder::new(&raw))?;

    let program = Program {
      raw: raw,
      uni_iface: iface,
      _in: PhantomData,
      _out: PhantomData
    };

    Ok((program, warnings))
  }

  /// Create a new program by consuming strings.
  pub fn from_strings<'a, T, G>(tess: T,
                                vertex: &str,
                                geometry: G,
                                fragment: &str)
                                -> Result<(Self, Vec<UniformWarning>)>
                                where T: Into<Option<(&'a str, &'a str)>>,
                                      G: Into<Option<&'a str>> {
    let tess = match tess.into() {
      Some((tcs_str, tes_str)) => {
        let tcs = Stage::new(stage::Type::TessellationControlShader, tcs_str).map_err(ProgramError::StageError)?;
        let tes = Stage::new(stage::Type::TessellationControlShader, tes_str).map_err(ProgramError::StageError)?;
        Some((tcs, tes))
      },
      None => None
    };

    let gs = match geometry.into() {
      Some(gs_str) => Some(Stage::new(stage::Type::GeometryShader, gs_str).map_err(ProgramError::StageError)?),
      None => None
    };

    let vs = Stage::new(stage::Type::VertexShader, vertex).map_err(ProgramError::StageError)?;
    let fs = Stage::new(stage::Type::FragmentShader, fragment).map_err(ProgramError::StageError)?;

    Self::from_stages(tess.as_ref().map(|&(ref tcs, ref tes)| (tcs, tes)), &vs, gs.as_ref(), &fs)
  }

  /// Get the uniform interface associated with this program.
  pub(crate) fn uniform_interface(&self) -> &Uni {
    &self.uni_iface
  }
}

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = RawProgram;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

/// Class of types that can act as uniform interfaces in typed programs.
///
/// A uniform interface is a value that contains uniforms. The purpose of a uniform interface is to
/// be stored in a typed program and handed back to the programmer when the program is available in
/// a pipeline.
pub trait UniformInterface: Sized {
  /// Build the uniform interface.
  ///
  /// When mapping a uniform, if you want to accept failures, you can discard the error and use
  /// `UniformBuilder::unbound` to let the uniform pass through, and collect the uniform warning.
  fn uniform_interface<'a>(builder: UniformBuilder<'a>) -> Result<(Self, Vec<UniformWarning>)>;
}

impl UniformInterface for () {
  fn uniform_interface<'a>(_: UniformBuilder<'a>) -> Result<(Self, Vec<UniformWarning>)> {
    Ok(((), Vec::new()))
  }
}

/// Build uniforms to fold them to a uniform interface.
pub struct UniformBuilder<'a> {
  raw: &'a RawProgram
}

impl<'a> UniformBuilder<'a> {
  fn new(raw: &'a RawProgram) -> Self {
    UniformBuilder {
      raw: raw
    }
  }

  /// Have the builder hand you a `Uniform` of the type of your choice. Keep in mind that it’s
  /// possible that this function fails if you ask for a type that is not the one defined in the
  /// shader.
  pub fn ask<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let uniform = match T::ty() {
      Type::BufferBinding => self.ask_uniform_block(name)?,
      _ => self.ask_uniform(name)?
    };

    uniform_type_match(self.raw.handle, name, T::ty(), T::dim()).map_err(|err| UniformWarning::TypeMismatch(name.to_owned(), err))?;

    Ok(uniform)
  }

  fn ask_uniform<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let c_name = CString::new(name.as_bytes()).unwrap();
    let location = unsafe { gl::GetUniformLocation(self.raw.handle, c_name.as_ptr() as *const GLchar) };

    if location < 0 {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location))
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let c_name = CString::new(name.as_bytes()).unwrap();
    let location = unsafe { gl::GetUniformBlockIndex(self.raw.handle, c_name.as_ptr() as *const GLchar) };

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

/// Errors that a `Program` can generate.
#[derive(Clone, Debug)]
pub enum ProgramError {
  StageError(StageError),
  /// Program link failed. You can inspect the reason by looking at the contained `String`.
  LinkFailed(String),
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
  UniformWarning(UniformWarning)
}

impl fmt::Display for ProgramError {
  fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for ProgramError {
  fn description(&self) -> &str {
    match *self {
      ProgramError::StageError(_) => "stage error",
      ProgramError::LinkFailed(ref s) => &s,
      ProgramError::UniformWarning(_) => "uniform warning"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      ProgramError::StageError(ref e) => Some(e),
      ProgramError::UniformWarning(ref e) => Some(e),
      _ => None
    }
  }
}

/// Warnings related to uniform issues.
#[derive(Clone, Debug)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),
  /// Type mismatch between the static requested type (i.e. the `T` in `Uniform<T>` for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, String)
}

impl fmt::Display for UniformWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for UniformWarning {
  fn description(&self) -> &str {
    match *self {
      UniformWarning::Inactive(ref s) => &s,
      UniformWarning::TypeMismatch(..) => "type mismatch"
    }
  }
}

/// A shader uniform. `Uniform<T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<T> {
  program: GLuint,
  index: GLint,
  _t: PhantomData<*const T>
}

impl<T> Uniform<T> where T: Uniformable {
  fn new(program: GLuint, index: GLint) -> Self {
    Uniform {
      program: program,
      index: index,
      _t: PhantomData
    }
  }

  fn unbound(program: GLuint) -> Self {
    Uniform {
      program: program,
      index: -1,
      _t: PhantomData
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
  Integral,
  Unsigned,
  Floating,
  Boolean,
  TextureUnit,
  BufferBinding
}

/// Dimension of the uniform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
  Dim22,
  Dim33,
  Dim44,
  Cubemap
}

/// Types that can behave as `Uniform`.
pub trait Uniformable: Sized {
  /// Update the uniform with a new value.
  fn update(self, u: &Uniform<Self>);
  /// Retrieve the `Type` of the uniform.
  fn ty() -> Type;
  /// Retrieve the `Dim` of the uniform.
  fn dim() -> Dim;
}

impl Uniformable for i32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1i(u.index, self) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [i32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [i32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [i32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [i32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1iv(u.index, self.len() as GLsizei, self.as_ptr()) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[i32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[i32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[i32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for u32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [u32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [u32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [u32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [u32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[u32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[u32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[u32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for f32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1f(u.index, self) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [f32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [f32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [f32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [f32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[f32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[f32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[f32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for M22 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix2fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl Uniformable for M33 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix3fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl Uniformable for M44 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix4fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<'a> Uniformable for &'a [M22] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix2fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<'a> Uniformable for &'a [M33] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix3fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<'a> Uniformable for &'a [M44] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix4fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl Uniformable for bool {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self as GLuint) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [bool; 2] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32];
    unsafe { gl::Uniform2uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [bool; 3] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32];
    unsafe { gl::Uniform3uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [bool; 4] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32, self[3] as u32];
    unsafe { gl::Uniform4uiv(u.index, 1,  &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [bool] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(u.index, v.len() as GLsizei, v.as_ptr()) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[bool; 2]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[bool; 3]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32]).collect();
    unsafe { gl::Uniform3uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[bool; 4]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32]).collect();
    unsafe { gl::Uniform4uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

fn uniform_type_match(program: GLuint, name: &str, ty: Type, dim: Dim) -> ::std::result::Result<(), String> {
  let mut size: GLint = 0;
  let mut typ: GLuint = 0;
  let c_name = CString::new(name.as_bytes()).unwrap();

  unsafe {
    // get the max length of the returned names
    let mut max_len = 0;
    gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_len);

    // get the index of the uniform
    let mut index = 0;
    let mut name_ = Vec::<i8>::with_capacity(max_len as usize);
    gl::GetUniformIndices(program, 1, [c_name.as_ptr() as *const i8].as_ptr(), &mut index);

    // get its size and type
    gl::GetActiveUniform(program, index, max_len, null_mut(), &mut size, &mut typ, name_.as_mut_ptr());
  }

  // FIXME
  // early-return if array – we don’t support them yet
  if size != 1 {
    return Ok(());
  }

  match (ty, dim) {
    (Type::Integral, Dim::Dim1) if typ != gl::INT => Err("requested int doesn't match".to_owned()),
    (Type::Integral, Dim::Dim2) if typ != gl::INT_VEC2 => Err("requested ivec2 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim3) if typ != gl::INT_VEC3 => Err("requested ivec3 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim4) if typ != gl::INT_VEC4 => Err("requested ivec4 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim1) if typ != gl::UNSIGNED_INT => Err("requested uint doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim2) if typ != gl::UNSIGNED_INT_VEC2 => Err("requested uvec2 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim3) if typ != gl::UNSIGNED_INT_VEC3 => Err("requested uvec3 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim4) if typ != gl::UNSIGNED_INT_VEC4 => Err("requested uvec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim1) if typ != gl::FLOAT => Err("requested float doesn't match".to_owned()),
    (Type::Floating, Dim::Dim2) if typ != gl::FLOAT_VEC2 => Err("requested vec2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim3) if typ != gl::FLOAT_VEC3 => Err("requested vec3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim4) if typ != gl::FLOAT_VEC4 => Err("requested vec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim22) if typ != gl::FLOAT_MAT2 => Err("requested mat2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim33) if typ != gl::FLOAT_MAT3 => Err("requested mat3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim44) if typ != gl::FLOAT_MAT4 => Err("requested mat4 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim1) if typ != gl::BOOL => Err("requested bool doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim2) if typ != gl::BOOL_VEC2 => Err("requested bvec2 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim3) if typ != gl::BOOL_VEC3 => Err("requested bvec3 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim4) if typ != gl::BOOL_VEC4 => Err("requested bvec4 doesn't match".to_owned()),
    _ => Ok(())
  }
}

#[macro_export]
macro_rules! uniform_interface {
  (struct $struct_name:ident { $($fields:tt)* }) => {
    uniform_interface_build_struct!($struct_name, $($fields)*);
    uniform_interface_impl_trait!($struct_name, $($fields)*);
  }
}

#[macro_export]
macro_rules! uniform_interface_build_struct {
  ($struct_name:ident, $($(#[$($field_attrs:tt)*])* $field_name:ident : $field_ty:ty),+) => {
    struct $struct_name {
      $(
        $field_name: $crate::shader::program::Uniform<$field_ty>
      ),+
    }
  }
}

#[macro_export]
macro_rules! uniform_interface_impl_trait {
  ($struct_name:ident, $($(#[$($field_attrs:tt)*])* $field_name:ident : $field_ty:ty),+) => {
    impl $crate::shader::program::UniformInterface for $struct_name {
      fn uniform_interface(
        builder: $crate::shader::program::UniformBuilder
      ) -> ::std::result::Result<(Self, Vec<$crate::shader::program::UniformWarning>), $crate::shader::program::ProgramError> {
        #[allow(unused_mut)]
        let mut warnings = Vec::new();

        $(
          uniform_interface_impl_trait_map!(builder, warnings, $field_name $(#[$($field_attrs)*])*);
        )+

        let iface = $struct_name { $($field_name),+ };
        Ok((iface, warnings))
      }
    }
  }
}

#[macro_export]
macro_rules! uniform_interface_impl_trait_map {
  // this form authorizes to specify the mapping
  // this form authorizes unmapped uniforms by overriding them with an unbound uniform
  ($builder:ident, $warnings:ident, $field_name:ident #[as($field_mapping:expr), unbound]) => {
    let $field_name = $builder.ask($field_mapping).unwrap_or_else(|warning| {
      $warnings.push(warning);
      $builder.unbound()
    });
  };

  // same form as above but with flipped annotations
  ($builder:ident, $warnings:ident, $field_name:ident #[unbound, as($field_mapping:expr)]) => {
    let $field_name = $builder.ask($field_mapping).unwrap_or_else(|warning| {
      $warnings.push(warning);
      $builder.unbound()
    });
  };

  // this form authorizes to specify the mapping
  // this form will make the whole uniform interface not to build on any error
  ($builder:ident, $warnings:ident, $field_name:ident #[as($field_mapping:expr)]) => {
    let $field_name = $builder.ask($field_mapping).map_err($crate::shader::program::ProgramError::UniformWarning)?;
  };

  // this form authorizes unmapped uniforms by overriding them with an unbound uniform
  ($builder:ident, $warnings:ident, $field_name:ident #[unbound]) => {
    let $field_name = $builder.ask(stringify!($field_name)).unwrap_or_else(|warning| {
      $warnings.push(warning);
      $builder.unbound()
    });
  };

  // this form will make the whole uniform interface not to build on any error
  ($builder:ident, $warnings:ident, $field_name:ident) => {
    let $field_name = $builder.ask(stringify!($field_name)).map_err($crate::shader::program::ProgramError::UniformWarning)?;
  }
}
