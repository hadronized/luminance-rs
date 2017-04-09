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
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::null_mut;

use buffer::Binding;
use linear::{M22, M33, M44};
use shader::stage::Stage;
use texture::Unit;

type Result<A> = ::std::result::Result<A, ProgramError>;

/// A shader program.
#[derive(Debug)]
pub struct Program {
  handle: GLuint,
  uni_sem_map: HashMap<SemIndex, GLint>, // mapping between user semantic (indexes) and OpenGL uniform locations
  ubo_sem_map: HashMap<SemIndex, GLint>, // mapping between user semantic (indexes) and OpenGL uniform block indexes
}

impl Program {
  /// Create a new program by linking shader stages.
  pub fn new<'a, T, G>(tess: T, vertex: &Stage, geometry: G, fragment: &Stage, sem_map: &[Sem]) -> Result<(Self, Vec<UniformWarning>)> where T: Into<Option<(&'a Stage, &'a Stage)>>, G: Into<Option<&'a Stage>> {
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
        let mut uni_sem_map = HashMap::new();
        let mut ubo_sem_map = HashMap::new();
        let mut warnings = Vec::new();

        for sem in sem_map {
          let (loc, warning) = get_uniform_location(handle, sem.name(), sem.ty(), sem.dim());

          match loc {
            Location::Uniform(location) => uni_sem_map.insert(sem.index(), location),
            Location::UniformBlock(index) => ubo_sem_map.insert(sem.index(), index)
          };

          // if there’s a warning, add it to the list of warnings
          if let Err(warning) = warning {
            warnings.push(warning);
          }
        }

        let program = Program {
          handle: handle,
          uni_sem_map: uni_sem_map,
          ubo_sem_map: ubo_sem_map,
        };

        Ok((program, warnings))
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

  /// Update a uniform variable in the program.
  #[inline]
  fn update<T>(&self, u: &Uniform<T>, value: T) where T: Uniformable {
    value.update(&self, u);
  }

  #[inline]
  pub unsafe fn handle(&self) -> GLuint {
    self.handle
  }
}

impl Drop for Program {
  fn drop(&mut self) {
    unsafe { gl::DeleteProgram(self.handle) }
  }
}

/// A shader uniform semantic. It holds information on the variable it represents such as a name,
/// its type and its dimension.
///
/// You can create them from `Uniform` with the `Uniform::sem` method.
#[derive(Clone, Debug)]
pub struct Sem {
  name: String,
  index: SemIndex,
  ty: Type,
  dim: Dim
}

impl Sem {
  /// Create a new sem.
  fn new(name: &str, index: SemIndex, ty: Type, dim: Dim) -> Self {
    Sem {
      name: name.to_owned(),
      index: index,
      ty: ty,
      dim: dim
    }
  }

  /// Name of the uniform in the shader program.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Index of the uniform in the shader program.
  pub fn index(&self) -> SemIndex {
    self.index
  }

  /// Type of the uniform in the shader program.
  pub fn ty(&self) -> Type {
    self.ty
  }

  /// Dimension of the uniform in the shader program.
  pub fn dim(&self) -> Dim {
    self.dim
  }
}

/// Semantic index.
pub type SemIndex = u32;

/// Errors that a `Program` can generate.
#[derive(Clone, Debug)]
pub enum ProgramError {
  /// Program link failed. You can inspect the reason by looking at the contained `String`.
  LinkFailed(String),
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
  UniformWarning(UniformWarning)
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

/// A shader uniform. `Uniform<T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<T> {
  sem_index: SemIndex,
  _t: PhantomData<*const T>
}

impl<T> Uniform<T> where T: Uniformable {
  /// Create a new uniform from a semantic index.
  pub const fn new(sem_index: SemIndex) -> Uniform<T> {
    Uniform {
      sem_index: sem_index,
      _t: PhantomData
    }
  }

  /// Create a `Sem` by giving a mapping name. The `Type` and `Dim` are reified using the static
  /// type of the uniform (`T`).
  pub fn sem(&self, name: &str) -> Sem {
    Sem::new(name, self.sem_index, T::reify_type(), T::dim())
  }

  /// Lazily alter the content of a uniform.
  pub fn alter(&self, value: T) -> AlterUniform {
    AlterUniform::new(self, value)
  }
}

/// A uniform altered with a value. Type erasure is performed on the type of the uniform so that
/// this type can be collected and pass down to whatever function needs heterogenous collections of
/// altered uniforms.
pub struct AlterUniform<'a> {
  alter: Box<for<'b> Fn(&'b Program) + 'a>
}

impl<'a> AlterUniform<'a> {
  /// Create a new `AlterUniform`.
  fn new<T>(uniform: &'a Uniform<T>, value: T) -> Self where T: Uniformable {
    AlterUniform {
      alter: Box::new(move |program: &Program| {
        program.update(uniform, value)
      })
    }
  }

  /// Alter the uniform. You must not use that function directly.
  pub unsafe fn alter(&self, program: &Program) {
    (self.alter)(program);
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
pub trait Uniformable: Copy + Sized {
  /// Update the uniform with a new value.
  fn update(self, program: &Program, u: &Uniform<Self>);
  /// Retrieve the `Type` of the uniform.
  fn reify_type() -> Type; // FIXME: call that ty() instead
  /// Retrieve the `Dim` of the uniform.
  fn dim() -> Dim;
}

impl Uniformable for i32 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1i(program.uni_sem_map[&u.sem_index], self) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [i32; 2] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2iv(program.uni_sem_map[&u.sem_index], 1, &self as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [i32; 3] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3iv(program.uni_sem_map[&u.sem_index], 1, &self as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [i32; 4] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4iv(program.uni_sem_map[&u.sem_index], 1, &self as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [i32] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1iv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr()) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[i32; 2]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2iv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[i32; 3]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3iv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[i32; 4]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4iv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for u32 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1ui(program.uni_sem_map[&u.sem_index], self) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [u32; 2] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2uiv(program.uni_sem_map[&u.sem_index], 1, &self as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [u32; 3] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3uiv(program.uni_sem_map[&u.sem_index], 1, &self as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [u32; 4] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4uiv(program.uni_sem_map[&u.sem_index], 1, &self as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [u32] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1uiv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[u32; 2]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2uiv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[u32; 3]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3uiv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[u32; 4]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4uiv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for f32 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1f(program.uni_sem_map[&u.sem_index], self) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [f32; 2] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2fv(program.uni_sem_map[&u.sem_index], 1, &self as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [f32; 3] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3fv(program.uni_sem_map[&u.sem_index], 1, &self as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [f32; 4] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4fv(program.uni_sem_map[&u.sem_index], 1, &self as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [f32] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[f32; 2]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform2fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[f32; 3]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform3fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[f32; 4]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform4fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for M22 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self];
    unsafe { gl::UniformMatrix2fv(program.uni_sem_map[&u.sem_index], 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl Uniformable for M33 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self];
    unsafe { gl::UniformMatrix3fv(program.uni_sem_map[&u.sem_index], 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl Uniformable for M44 {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self];
    unsafe { gl::UniformMatrix4fv(program.uni_sem_map[&u.sem_index], 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<'a> Uniformable for &'a [M22] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::UniformMatrix2fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<'a> Uniformable for &'a [M33] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::UniformMatrix3fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<'a> Uniformable for &'a [M44] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::UniformMatrix4fv(program.uni_sem_map[&u.sem_index], self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl Uniformable for bool {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1ui(program.uni_sem_map[&u.sem_index], self as GLuint) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [bool; 2] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self[0] as u32, self[1] as u32];
    unsafe { gl::Uniform2uiv(program.uni_sem_map[&u.sem_index], 1, &v as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [bool; 3] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self[0] as u32, self[1] as u32, self[2] as u32];
    unsafe { gl::Uniform3uiv(program.uni_sem_map[&u.sem_index], 1, &v as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [bool; 4] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v = [self[0] as u32, self[1] as u32, self[2] as u32, self[3] as u32];
    unsafe { gl::Uniform4uiv(program.uni_sem_map[&u.sem_index], 1,  &v as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [bool] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v: Vec<_> = self.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(program.uni_sem_map[&u.sem_index], v.len() as GLsizei, v.as_ptr()) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[bool; 2]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(program.uni_sem_map[&u.sem_index], v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[bool; 3]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32]).collect();
    unsafe { gl::Uniform3uiv(program.uni_sem_map[&u.sem_index], v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[bool; 4]] {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32]).collect();
    unsafe { gl::Uniform4uiv(program.uni_sem_map[&u.sem_index], v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for Unit {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.uni_sem_map.len());
    unsafe { gl::Uniform1i(program.uni_sem_map[&u.sem_index], *self as GLint) }
  }

  fn reify_type() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for Binding {
  fn update(self, program: &Program, u: &Uniform<Self>) {
    assert!((u.sem_index as usize) < program.ubo_sem_map.len());
    unsafe { gl::UniformBlockBinding(program.handle(), program.ubo_sem_map[&u.sem_index] as GLuint, *self as GLuint) }
  }

  fn reify_type() -> Type { Type::BufferBinding }

  fn dim() -> Dim { Dim::Dim1 }
}

enum Location {
  Uniform(GLint),
  UniformBlock(GLint),
}

// Retrieve the uniform location.
fn get_uniform_location(program: GLuint, name: &str, ty: Type, dim: Dim) -> (Location, ::std::result::Result<(), UniformWarning>) {
  let c_name = CString::new(name.as_bytes()).unwrap();
  let location = if ty == Type::BufferBinding {
    let index = unsafe { gl::GetUniformBlockIndex(program, c_name.as_ptr() as *const GLchar) };

    if index == gl::INVALID_INDEX {
      return (Location::UniformBlock(-1), Err(UniformWarning::Inactive(name.to_owned())));
    }

    Location::UniformBlock(index as GLint)
  } else {
    let location = unsafe { gl::GetUniformLocation(program, c_name.as_ptr() as *const GLchar) };

    if location == -1 {
      return (Location::Uniform(-1), Err(UniformWarning::Inactive(name.to_owned())));
    }

    Location::Uniform(location)
  };

  if let Some(err) = uniform_type_match(program, name, ty, dim) {
    return (location, Err(UniformWarning::TypeMismatch(name.to_owned(), err)));
  }

  (location, Ok(()))
}

// Return something if no match can be established.
fn uniform_type_match(program: GLuint, name: &str, ty: Type, dim: Dim) -> Option<String> {
  let mut size: GLint = 0;
  let mut typ: GLuint = 0;

  unsafe {
    // get the index of the uniform
    let mut index = 0;
    gl::GetUniformIndices(program, 1, [name.as_ptr() as *const i8].as_ptr(), &mut index);
    // get its size and type
    gl::GetActiveUniform(program, index, 0, null_mut(), &mut size, &mut typ, null_mut());
  }

  // FIXME
  // early-return if array – we don’t support them yet
  if size != 1 {
    return None;
  }

  match (ty, dim) {
    (Type::Integral, Dim::Dim1) if typ != gl::INT => Some("requested int doesn't match".to_owned()),
    (Type::Integral, Dim::Dim2) if typ != gl::INT_VEC2 => Some("requested ivec2 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim3) if typ != gl::INT_VEC3 => Some("requested ivec3 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim4) if typ != gl::INT_VEC4 => Some("requested ivec4 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim1) if typ != gl::UNSIGNED_INT => Some("requested uint doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim2) if typ != gl::UNSIGNED_INT_VEC2 => Some("requested uvec2 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim3) if typ != gl::UNSIGNED_INT_VEC3 => Some("requested uvec3 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim4) if typ != gl::UNSIGNED_INT_VEC4 => Some("requested uvec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim1) if typ != gl::FLOAT => Some("requested float doesn't match".to_owned()),
    (Type::Floating, Dim::Dim2) if typ != gl::FLOAT_VEC2 => Some("requested vec2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim3) if typ != gl::FLOAT_VEC3 => Some("requested vec3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim4) if typ != gl::FLOAT_VEC4 => Some("requested vec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim22) if typ != gl::FLOAT_MAT2 => Some("requested mat2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim33) if typ != gl::FLOAT_MAT3 => Some("requested mat3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim44) if typ != gl::FLOAT_MAT4 => Some("requested mat4 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim1) if typ != gl::BOOL => Some("requested bool doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim2) if typ != gl::BOOL_VEC2 => Some("requested bvec2 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim3) if typ != gl::BOOL_VEC3 => Some("requested bvec3 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim4) if typ != gl::BOOL_VEC4 => Some("requested bvec4 doesn't match".to_owned()),
    _ => None
  }
}
