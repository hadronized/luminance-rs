use gl;
use gl::types::*;
use luminance::shader::program2::{
  ProgramInterface as ProgramInterfaceBackend, TessellationStages, Type as UniformType,
  UniformBuild, UniformBuilder as UniformBuilderBackend, UniformInterface, Uniformable,
};
use luminance::shader::stage::Type as StageType;
use luminance::vertex::Semantics;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::null_mut;

use crate::shader::stage::{Stage, StageError};

/// Errors that a `Program` can generate.
#[derive(Debug)]
pub enum ProgramError {
  /// A shader stage failed to compile or validate its state.
  StageError(StageError),
  /// Program link failed. You can inspect the reason by looking at the contained `String`.
  LinkFailed(String),
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
  UniformWarning(UniformWarning),
  /// Some vertex attribute is ill-formed.
  VertexAttribWarning(VertexAttribWarning),
}

impl fmt::Display for ProgramError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ProgramError::StageError(ref e) => write!(f, "shader program has stage error: {}", e),

      ProgramError::LinkFailed(ref s) => write!(f, "shader program failed to link: {}", s),

      ProgramError::UniformWarning(ref e) => {
        write!(f, "shader program contains uniform warning(s): {}", e)
      }
      ProgramError::VertexAttribWarning(ref e) => write!(
        f,
        "shader program contains vertex attribute warning(s): {}",
        e
      ),
    }
  }
}

/// Warnings related to vertex attributes issues.
#[derive(Debug)]
pub enum VertexAttribWarning {
  /// Inactive vertex attribute (not read).
  Inactive(String),
}

impl fmt::Display for VertexAttribWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      VertexAttribWarning::Inactive(ref s) => write!(f, "inactive {} vertex attribute", s),
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

/// A raw shader program.
///
/// This is a type-erased version of a `Program`.
#[derive(Debug)]
pub struct RawProgram {
  handle: GLuint,
}

impl RawProgram {
  /// Create a new program by attaching shader stages.
  fn new<'a, T, G>(
    vertex: &'a Stage,
    tess: T,
    geometry: G,
    fragment: &'a Stage,
  ) -> Result<Self, ProgramError>
  where
    T: Into<Option<TessellationStages<'a, Stage>>>,
    G: Into<Option<&'a Stage>>,
  {
    unsafe {
      let handle = gl::CreateProgram();

      gl::AttachShader(handle, vertex.handle());

      if let Some(TessellationStages {
        control,
        evaluation,
      }) = tess.into()
      {
        gl::AttachShader(handle, control.handle());
        gl::AttachShader(handle, evaluation.handle());
      }

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

      let mut linked: GLint = gl::FALSE.into();
      gl::GetProgramiv(handle, gl::LINK_STATUS, &mut linked);

      if linked == gl::TRUE.into() {
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

pub struct Program<S, Out, Uni> {
  raw: RawProgram,
  uni_iface: Uni,
  _in: PhantomData<*const S>,
  _out: PhantomData<*const Out>,
}

impl<S, Out, Uni> Program<S, Out, Uni>
where
  S: Semantics,
{
  /// Create a new program by consuming `Stage`s.
  pub fn from_stages<'a, T, G>(
    tess: T,
    vertex: &'a Stage,
    geometry: G,
    fragment: &'a Stage,
  ) -> Result<BuiltProgram<S, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface,
    T: Into<Option<TessellationStages<'a, Stage>>>,
    G: Into<Option<&'a Stage>>,
  {
    Self::from_stages_env(vertex, tess, geometry, fragment, ())
  }

  /// Create a new program by consuming strings.
  pub fn from_strings<'a, T, G>(
    tess: T,
    vertex: &'a str,
    geometry: G,
    fragment: &'a str,
  ) -> Result<BuiltProgram<S, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface,
    T: Into<Option<TessellationStages<'a, str>>>,
    G: Into<Option<&'a str>>,
  {
    Self::from_strings_env(vertex, tess, geometry, fragment, ())
  }

  /// Create a new program by consuming `Stage`s and by looking up an environment.
  pub fn from_stages_env<'a, E, T, G>(
    vertex: &'a Stage,
    tess: T,
    geometry: G,
    fragment: &'a Stage,
    env: E,
  ) -> Result<BuiltProgram<S, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<E>,
    T: Into<Option<TessellationStages<'a, Stage>>>,
    G: Into<Option<&'a Stage>>,
  {
    let raw = RawProgram::new(vertex, tess, geometry, fragment)?;

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

    Ok(BuiltProgram { program, warnings })
  }

  /// Create a new program by consuming strings.
  pub fn from_strings_env<'a, E, T, G>(
    vertex: &'a str,
    tess: T,
    geometry: G,
    fragment: &'a str,
    env: E,
  ) -> Result<BuiltProgram<S, Out, Uni>, ProgramError>
  where
    Uni: UniformInterface<E>,
    T: Into<Option<TessellationStages<'a, str>>>,
    G: Into<Option<&'a str>>,
  {
    let vs = Stage::new(StageType::VertexShader, vertex).map_err(ProgramError::StageError)?;

    let tess = match tess.into() {
      Some(TessellationStages {
        control,
        evaluation,
      }) => {
        let tcs = Stage::new(StageType::TessellationControlShader, control)
          .map_err(ProgramError::StageError)?;
        let tes = Stage::new(StageType::TessellationControlShader, evaluation)
          .map_err(ProgramError::StageError)?;

        Some((tcs, tes))
      }
      None => None,
    };

    let gs = match geometry.into() {
      Some(gs_str) => {
        Some(Stage::new(StageType::GeometryShader, gs_str).map_err(ProgramError::StageError)?)
      }
      None => None,
    };

    let fs = Stage::new(StageType::FragmentShader, fragment).map_err(ProgramError::StageError)?;

    Self::from_stages_env(
      &vs,
      tess
        .as_ref()
        .map(|(ref control, ref evaluation)| TessellationStages {
          control,
          evaluation,
        }),
      gs.as_ref(),
      &fs,
      env,
    )
  }

  /// Get the program interface associated with this program.
  pub(crate) fn interface(&self) -> ProgramInterface<Uni> {
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
  pub fn adapt<Q>(self) -> Result<BuiltProgram<S, Out, Q>, AdaptationFailure<S, Out, Uni>>
  where
    Q: UniformInterface,
  {
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
  ) -> Result<BuiltProgram<S, Out, Q>, AdaptationFailure<S, Out, Uni>>
  where
    Q: UniformInterface<E>,
  {
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
        let warnings = warnings.into_iter().map(ProgramWarning::Uniform).collect();

        Ok(BuiltProgram { program, warnings })
      }

      Err(iface_err) => {
        // we couldn’t generate the new uniform interface; return the error(s) that occurred and the
        // the untouched former program
        let failure = AdaptationFailure {
          program: self,
          error: iface_err,
        };
        Err(failure)
      }
    }
  }

  /// A version of [`Program::adapt_env`] that doesn’t change the uniform interface type.
  ///
  /// This function might be needed for when you want to update the uniform interface but still
  /// enforce that the type must remain the same.
  pub fn readapt_env<E>(
    self,
    env: E,
  ) -> Result<BuiltProgram<S, Out, Uni>, AdaptationFailure<S, Out, Uni>>
  where
    Uni: UniformInterface<E>,
  {
    self.adapt_env(env)
  }
}

impl<S, Out, Uni> Deref for Program<S, Out, Uni> {
  type Target = RawProgram;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

/// A built program with potential warnings.
///
/// The sole purpose of this type is to be destructured when a program is built.
pub struct BuiltProgram<S, Out, Uni> {
  /// Built program.
  pub program: Program<S, Out, Uni>,
  /// Potential warnings.
  pub warnings: Vec<ProgramWarning>,
}

impl<S, Out, Uni> BuiltProgram<S, Out, Uni> {
  /// Get the program and ignore the warnings.
  pub fn ignore_warnings(self) -> Program<S, Out, Uni> {
    self.program
  }
}

/// A [`Program`] uniform adaptation that has failed.
pub struct AdaptationFailure<S, Out, Uni> {
  /// Program used before trying to adapt.
  pub program: Program<S, Out, Uni>,
  /// Program error that prevented to adapt.
  pub error: ProgramError,
}

impl<S, Out, Uni> AdaptationFailure<S, Out, Uni> {
  /// Get the program and ignore the error.
  pub fn ignore_error(self) -> Program<S, Out, Uni> {
    self.program
  }
}

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

  fn ask<T>(&mut self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    Uniform<T>: Uniformable<T>,
  {
    let uniform = match Uniform::<T>::TY {
      UniformType::BufferBinding => self.ask_uniform_block(name)?,
      _ => self.ask_uniform(name)?,
    };

    uniform_type_match(self.raw.handle, name, Uniform::<T>::TY)?;

    Ok(uniform)
  }

  /// Get an unbound [`Uniform`].
  ///
  /// Unbound [`Uniform`]s are not any different from typical [`Uniform`]s but when resolving
  /// mapping in the _shader program_, if the [`Uniform`] is found inactive or doesn’t exist,
  /// instead of returning an error, this function will return an _unbound uniform_, which is a
  /// uniform that does nothing interesting.
  ///
  /// That function is useful if you don’t really care about silently sending values down a shader
  /// program and getting them ignored. It might be the case for optional uniforms, for instance.
  fn ask_unbound<T>(&mut self, name: &str) -> Uniform<T>
  where
    Uniform<T>: Uniformable<T>,
  {
    match self.ask(name) {
      Ok(uniform) => uniform,
      Err(warning) => {
        self.warnings.push(warning);
        self.unbound()
      }
    }
  }

  fn ask_uniform<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    Uniform<T>: Uniformable<T>,
  {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetUniformLocation(self.raw.handle, c_name.as_ptr() as *const GLchar) }
    };

    if location < 0 {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location))
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    Uniform<T>: Uniformable<T>,
  {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetUniformBlockIndex(self.raw.handle, c_name.as_ptr() as *const GLchar) }
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
  fn unbound<T>(&self) -> Uniform<T>
  where
    Uniform<T>: Uniformable<T>,
  {
    Uniform::unbound(self.raw.handle)
  }
}

impl<'a> UniformBuilderBackend for UniformBuilder<'a> {
  type Err = UniformWarning;
}

impl<'a, T> UniformBuild<T> for UniformBuilder<'a>
where
  Uniform<T>: Uniformable<T>,
{
  type Uniform = Uniform<T>;

  fn ask_specific<S>(&mut self, name: S) -> Result<Self::Uniform, Self::Err>
  where
    S: AsRef<str>,
  {
    UniformBuilder::ask(self, name.as_ref())
  }

  fn ask_unbound_specific<S>(&mut self, name: S) -> Self::Uniform
  where
    S: AsRef<str>,
  {
    UniformBuilder::ask_unbound(self, name.as_ref())
  }

  fn unbound_specific(&mut self) -> Self::Uniform {
    UniformBuilder::unbound(self)
  }
}

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
  /// Get a [`UniformBuilder`] in order to perform dynamic uniform lookup.
  pub fn query(&'a self) -> UniformBuilder<'a> {
    UniformBuilder::new(self.raw_program)
  }
}

//impl<'a, Uni> ProgramInterfaceBackend<'a, Uni> for ProgramInterface<'a, Uni> {
//  type UniformBuilder = UniformBuilder<'a>;
//
//  fn query(&'a self) -> Self::UniformBuilder {
//    ProgramInterface::query(self)
//  }
//}

/// Warnings related to uniform issues.
#[derive(Debug)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),
  /// Type mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, UniformType),
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
}

impl fmt::Display for UniformWarning {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      UniformWarning::Inactive(ref s) => write!(f, "inactive {} uniform", s),

      UniformWarning::TypeMismatch(ref n, ref t) => {
        write!(f, "type mismatch for uniform {}: {}", n, t)
      }
    }
  }
}

// Check whether a shader program’s uniform type matches the type we have chosen.
fn uniform_type_match(program: GLuint, name: &str, ty: UniformType) -> Result<(), UniformWarning> {
  let mut size: GLint = 0;
  let mut glty: GLuint = 0;

  unsafe {
    // get the max length of the returned names
    let mut max_len = 0;
    gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_len);

    // get the index of the uniform
    let mut index = 0;

    let c_name = CString::new(name.as_bytes()).unwrap();
    gl::GetUniformIndices(
      program,
      1,
      [c_name.as_ptr() as *const GLchar].as_ptr(),
      &mut index,
    );

    // get its size and type
    let mut name_ = Vec::<GLchar>::with_capacity(max_len as usize);
    gl::GetActiveUniform(
      program,
      index,
      max_len,
      null_mut(),
      &mut size,
      &mut glty,
      name_.as_mut_ptr(),
    );
  }

  // early-return if array – we don’t support them yet
  if size != 1 {
    return Ok(());
  }

  check_types_match(name, ty, glty)
}

// Check if a type matches the OpenGL counterpart.
#[allow(clippy::cognitive_complexity)]
fn check_types_match(name: &str, ty: UniformType, glty: GLuint) -> Result<(), UniformWarning> {
  match ty {
    // scalars
    UniformType::Int if glty != gl::INT => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::UInt if glty != gl::UNSIGNED_INT => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::Float if glty != gl::FLOAT => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::Bool if glty != gl::BOOL => Err(UniformWarning::type_mismatch(name, ty)),
    // vectors
    UniformType::IVec2 if glty != gl::INT_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::IVec3 if glty != gl::INT_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::IVec4 if glty != gl::INT_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::UIVec2 if glty != gl::UNSIGNED_INT_VEC2 => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UIVec3 if glty != gl::UNSIGNED_INT_VEC3 => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UIVec4 if glty != gl::UNSIGNED_INT_VEC4 => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::Vec2 if glty != gl::FLOAT_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::Vec3 if glty != gl::FLOAT_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::Vec4 if glty != gl::FLOAT_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::BVec2 if glty != gl::BOOL_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::BVec3 if glty != gl::BOOL_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::BVec4 if glty != gl::BOOL_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    // matrices
    UniformType::M22 if glty != gl::FLOAT_MAT2 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::M33 if glty != gl::FLOAT_MAT3 => Err(UniformWarning::type_mismatch(name, ty)),
    UniformType::M44 if glty != gl::FLOAT_MAT4 => Err(UniformWarning::type_mismatch(name, ty)),
    // textures
    UniformType::ISampler1D if glty != gl::INT_SAMPLER_1D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::ISampler2D if glty != gl::INT_SAMPLER_2D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::ISampler3D if glty != gl::INT_SAMPLER_3D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UISampler1D if glty != gl::UNSIGNED_INT_SAMPLER_1D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UISampler2D if glty != gl::UNSIGNED_INT_SAMPLER_2D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UISampler3D if glty != gl::UNSIGNED_INT_SAMPLER_3D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::Sampler1D if glty != gl::SAMPLER_1D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::Sampler2D if glty != gl::SAMPLER_2D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::Sampler3D if glty != gl::SAMPLER_3D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::ICubemap if glty != gl::INT_SAMPLER_CUBE => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::UICubemap if glty != gl::UNSIGNED_INT_SAMPLER_CUBE => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    UniformType::Cubemap if glty != gl::SAMPLER_CUBE => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    _ => Ok(()),
  }
}

// Generate a uniform interface and collect warnings.
fn create_uniform_interface<Uni, E>(
  raw: &RawProgram,
  env: E,
) -> Result<(Uni, Vec<UniformWarning>), ProgramError>
where
  Uni: UniformInterface<E>,
{
  let mut builder = UniformBuilder::new(raw);
  let iface = Uni::uniform_interface(&mut builder, env).map_err(ProgramError::UniformWarning)?;

  Ok((iface, builder.warnings))
}

fn bind_vertex_attribs_locations<S>(raw: &RawProgram) -> Vec<ProgramWarning>
where
  S: Semantics,
{
  let mut warnings = Vec::new();

  for desc in S::semantics_set() {
    match get_vertex_attrib_location(raw, &desc.name) {
      Ok(_) => {
        let index = desc.index as GLuint;

        // we are not interested in the location as we’re about to change it to what we’ve
        // decided in the semantics
        let c_name = CString::new(desc.name.as_bytes()).unwrap();
        unsafe { gl::BindAttribLocation(raw.handle, index, c_name.as_ptr() as *const GLchar) };
      }

      Err(warning) => warnings.push(ProgramWarning::VertexAttrib(warning)),
    }
  }

  warnings
}

fn get_vertex_attrib_location(raw: &RawProgram, name: &str) -> Result<GLuint, VertexAttribWarning> {
  let location = {
    let c_name = CString::new(name.as_bytes()).unwrap();
    unsafe { gl::GetAttribLocation(raw.handle, c_name.as_ptr() as *const GLchar) }
  };

  if location < 0 {
    Err(VertexAttribWarning::Inactive(name.to_owned()))
  } else {
    Ok(location as _)
  }
}

#[derive(Debug)]
pub struct Uniform<T> {
  program: GLuint,
  index: GLint,
  _t: PhantomData<*const T>,
}

impl<T> Uniform<T> {
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
}

macro_rules! impl_uniformable {
  // slices
  (& [[$t:ty; $n:expr]], $ut:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<&'a [[$t; $n]]> for Uniform<&'a [[$t; $n]]> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: &[[$t; $n]]) {
        unsafe {
          gl::$f(
            self.index,
            value.len() as GLsizei,
            value.as_ptr() as *const $t,
          )
        };
      }
    }
  };

  // matrix slices
  (& [[$t:ty; $n:expr]; $m:expr], $ut:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<&'a [[$t; $n]; $m]> for Uniform<&'a [[$t; $n]; $m]> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: &[[$t; $n]; $m]) {
        unsafe {
          gl::$f(
            self.index,
            value.len() as GLsizei,
            gl::FALSE,
            value.as_ptr() as *const $t,
          )
        };
      }
    }
  };

  (& [$t:ty], $ut:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<&'a [$t]> for Uniform<&'a [$t]> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: &[$t]) {
        unsafe { gl::$f(self.index, value.len() as GLsizei, value.as_ptr()) };
      }
    }
  };

  // matrices
  ([[$t:ty; $n:expr]; $m:expr], $ut:tt, $f:tt) => {
    unsafe impl Uniformable<[[$t; $n]; $m]> for Uniform<[[$t; $n]; $m]> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: [[$t; $n]; $m]) {
        let v = [value];
        unsafe { gl::$f(self.index, 1, gl::FALSE, v.as_ptr() as *const $t) };
      }
    }
  };

  // arrays
  ([$t:ty; $n:expr], $ut:tt, $f:tt) => {
    unsafe impl Uniformable<[$t; $n]> for Uniform<[$t; $n]> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: [$t; $n]) {
        unsafe { gl::$f(self.index, 1, &value as *const $t) };
      }
    }
  };

  // scalars
  ($t:ty, $ut:tt, $f:tt) => {
    unsafe impl Uniformable<$t> for Uniform<$t> {
      const TY: UniformType = UniformType::$ut;

      fn update(self, value: $t) {
        unsafe { gl::$f(self.index, value) };
      }
    }
  };
}

// i32
impl_uniformable!(i32, Int, Uniform1i);
impl_uniformable!([i32; 2], IVec2, Uniform2iv);
impl_uniformable!([i32; 3], IVec3, Uniform3iv);
impl_uniformable!([i32; 4], IVec4, Uniform4iv);
impl_uniformable!(&[i32], Int, Uniform1iv);
impl_uniformable!(&[[i32; 2]], IVec2, Uniform2iv);
impl_uniformable!(&[[i32; 3]], IVec3, Uniform3iv);
impl_uniformable!(&[[i32; 4]], IVec4, Uniform4iv);

// u32
impl_uniformable!(u32, UInt, Uniform1ui);
impl_uniformable!([u32; 2], UIVec2, Uniform2uiv);
impl_uniformable!([u32; 3], UIVec3, Uniform3uiv);
impl_uniformable!([u32; 4], UIVec4, Uniform4uiv);
impl_uniformable!(&[u32], UInt, Uniform1uiv);
impl_uniformable!(&[[u32; 2]], UIVec2, Uniform2uiv);
impl_uniformable!(&[[u32; 3]], UIVec3, Uniform3uiv);
impl_uniformable!(&[[u32; 4]], UIVec4, Uniform4uiv);

// f32
impl_uniformable!(f32, Float, Uniform1f);
impl_uniformable!([f32; 2], Vec2, Uniform2fv);
impl_uniformable!([f32; 3], Vec3, Uniform3fv);
impl_uniformable!([f32; 4], Vec4, Uniform4fv);
impl_uniformable!(&[f32], Float, Uniform1fv);
impl_uniformable!(&[[f32; 2]], Vec2, Uniform2fv);
impl_uniformable!(&[[f32; 3]], Vec3, Uniform3fv);
impl_uniformable!(&[[f32; 4]], Vec4, Uniform4fv);
impl_uniformable!([[f32; 2]; 2], M22, UniformMatrix2fv);
impl_uniformable!([[f32; 3]; 3], M33, UniformMatrix3fv);
impl_uniformable!([[f32; 4]; 4], M44, UniformMatrix4fv);
impl_uniformable!(&[[f32; 2]; 2], M22, UniformMatrix2fv);
impl_uniformable!(&[[f32; 3]; 3], M33, UniformMatrix3fv);
impl_uniformable!(&[[f32; 4]; 4], M44, UniformMatrix4fv);

// bool
unsafe impl Uniformable<bool> for Uniform<bool> {
  const TY: UniformType = UniformType::Bool;

  fn update(self, value: bool) {
    unsafe { gl::Uniform1ui(self.index, value as GLuint) }
  }
}

unsafe impl Uniformable<[bool; 2]> for Uniform<[bool; 2]> {
  const TY: UniformType = UniformType::BVec2;

  fn update(self, value: [bool; 2]) {
    let v = [value[0] as u32, value[1] as u32];
    unsafe { gl::Uniform2uiv(self.index, 1, &v as *const u32) }
  }
}

unsafe impl Uniformable<[bool; 3]> for Uniform<[bool; 3]> {
  const TY: UniformType = UniformType::BVec3;

  fn update(self, value: [bool; 3]) {
    let v = [value[0] as u32, value[1] as u32, value[2] as u32];
    unsafe { gl::Uniform3uiv(self.index, 1, &v as *const u32) }
  }
}

unsafe impl Uniformable<[bool; 4]> for Uniform<[bool; 4]> {
  const TY: UniformType = UniformType::BVec4;

  fn update(self, value: [bool; 4]) {
    let v = [
      value[0] as u32,
      value[1] as u32,
      value[2] as u32,
      value[3] as u32,
    ];
    unsafe { gl::Uniform4uiv(self.index, 1, &v as *const u32) }
  }
}

unsafe impl<'a> Uniformable<&'a [bool]> for Uniform<&'a [bool]> {
  const TY: UniformType = UniformType::Bool;

  fn update(self, value: &[bool]) {
    let v: Vec<_> = value.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(self.index, v.len() as GLsizei, v.as_ptr()) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 2]]> for Uniform<&'a [[bool; 2]]> {
  const TY: UniformType = UniformType::BVec2;

  fn update(self, value: &[[bool; 2]]) {
    let v: Vec<_> = value.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 3]]> for Uniform<&'a [[bool; 3]]> {
  const TY: UniformType = UniformType::BVec3;

  fn update(self, value: &[[bool; 3]]) {
    let v: Vec<_> = value
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32])
      .collect();
    unsafe { gl::Uniform3uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 4]]> for Uniform<&'a [[bool; 4]]> {
  const TY: UniformType = UniformType::BVec4;

  fn update(self, value: &[[bool; 4]]) {
    let v: Vec<_> = value
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32])
      .collect();
    unsafe { gl::Uniform4uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}
