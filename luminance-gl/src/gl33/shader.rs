use super::buffer::Buffer;
use crate::gl33::GL33;
use gl::{self, types::*};
use luminance::{
  backend::shader::{Shader, ShaderData, Uniformable},
  pipeline::{ShaderDataBinding, TextureBinding},
  pixel::{SamplerType, Type as PixelType},
  shader::{
    types::{Arr, Mat22, Mat33, Mat44, Vec2, Vec3, Vec4},
    ProgramError, ShaderDataError, StageError, StageType, TessellationStages, Uniform, UniformType,
    UniformWarning, VertexAttribWarning,
  },
  texture::{Dim, Dimensionable},
  vertex::Semantics,
};
use luminance_std140::{ArrElem, Std140};
use std::{
  ffi::CString,
  mem,
  ptr::{null, null_mut},
};

#[derive(Debug)]
pub struct Stage {
  handle: GLuint,
}

impl Drop for Stage {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteShader(self.handle);
    }
  }
}

#[derive(Debug)]
pub struct Program {
  pub(crate) handle: GLuint,
}

impl Drop for Program {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.handle);
    }
  }
}

impl Program {
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

        log.set_len(log_len as usize);

        Err(ProgramError::link_failed(String::from_utf8(log).unwrap()))
      }
    }
  }
}

pub struct UniformBuilder {
  handle: GLuint,
}

impl UniformBuilder {
  fn new(program: &Program) -> Self {
    UniformBuilder {
      handle: program.handle,
    }
  }

  fn ask_uniform<T>(
    &self,
    name: &str,
    ty: UniformType,
    size: usize,
  ) -> Result<Uniform<T>, UniformWarning>
  where
    GL33: for<'u> Uniformable<'u, T>,
  {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetUniformLocation(self.handle, c_name.as_ptr() as *const GLchar) }
    };

    // ensure the location smells good
    if location < 0 {
      return Err(UniformWarning::inactive(name));
    }

    // ensure the type is correct regarding what we have in the type-system
    uniform_type_match(self.handle, name, ty, size)?;

    Ok(unsafe { Uniform::new(location) })
  }

  fn ask_uniform_block<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    GL33: for<'u> Uniformable<'u, T>,
  {
    let location = {
      let c_name = CString::new(name.as_bytes()).unwrap();
      unsafe { gl::GetUniformBlockIndex(self.handle, c_name.as_ptr() as *const GLchar) }
    };

    // ensure the location smells good
    if location == gl::INVALID_INDEX {
      return Err(UniformWarning::inactive(name));
    }

    Ok(unsafe { Uniform::new(location as _) })
  }
}

unsafe impl Shader for GL33 {
  type StageRepr = Stage;

  type ProgramRepr = Program;

  type UniformBuilderRepr = UniformBuilder;

  unsafe fn new_stage(&mut self, ty: StageType, src: &str) -> Result<Self::StageRepr, StageError> {
    let handle = gl::CreateShader(opengl_shader_type(ty));

    if handle == 0 {
      return Err(StageError::compilation_failed(
        ty,
        "unable to create shader stage",
      ));
    }

    let c_src = CString::new(glsl_pragma_src(src).as_bytes()).unwrap();
    gl::ShaderSource(handle, 1, [c_src.as_ptr()].as_ptr(), null());
    gl::CompileShader(handle);

    let mut compiled: GLint = gl::FALSE.into();
    gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut compiled);

    if compiled == gl::TRUE.into() {
      Ok(Stage { handle })
    } else {
      let mut log_len: GLint = 0;
      gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut log_len);

      let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
      gl::GetShaderInfoLog(handle, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

      gl::DeleteShader(handle);

      log.set_len(log_len as usize);

      Err(StageError::compilation_failed(
        ty,
        String::from_utf8(log).unwrap(),
      ))
    }
  }

  unsafe fn new_program(
    &mut self,
    vertex: &Self::StageRepr,
    tess: Option<TessellationStages<Self::StageRepr>>,
    geometry: Option<&Self::StageRepr>,
    fragment: &Self::StageRepr,
  ) -> Result<Self::ProgramRepr, ProgramError> {
    let handle = gl::CreateProgram();

    if let Some(TessellationStages {
      control,
      evaluation,
    }) = tess
    {
      gl::AttachShader(handle, control.handle);
      gl::AttachShader(handle, evaluation.handle);
    }

    gl::AttachShader(handle, vertex.handle);

    if let Some(geometry) = geometry {
      gl::AttachShader(handle, geometry.handle);
    }

    gl::AttachShader(handle, fragment.handle);

    let program = Program { handle };
    program.link().map(move |_| program)
  }

  unsafe fn apply_semantics<Sem>(
    program: &mut Self::ProgramRepr,
  ) -> Result<Vec<VertexAttribWarning>, ProgramError>
  where
    Sem: Semantics,
  {
    let warnings = bind_vertex_attribs_locations::<Sem>(program);

    program.link()?;

    Ok(warnings)
  }

  unsafe fn new_uniform_builder(
    program: &mut Self::ProgramRepr,
  ) -> Result<Self::UniformBuilderRepr, ProgramError> {
    Ok(UniformBuilder::new(&program))
  }

  unsafe fn ask_uniform<T>(
    uniform_builder: &mut Self::UniformBuilderRepr,
    name: &str,
  ) -> Result<Uniform<T>, UniformWarning>
  where
    Self: for<'u> Uniformable<'u, T>,
  {
    let uniform = match Self::ty() {
      UniformType::ShaderDataBinding => uniform_builder.ask_uniform_block(name)?,
      _ => uniform_builder.ask_uniform(name, Self::ty(), Self::SIZE)?,
    };

    Ok(uniform)
  }

  unsafe fn unbound<T>(_: &mut Self::UniformBuilderRepr) -> Uniform<T>
  where
    Self: for<'u> Uniformable<'u, T>,
  {
    Uniform::new(-1)
  }
}

fn opengl_shader_type(t: StageType) -> GLenum {
  match t {
    StageType::TessellationControlShader => gl::TESS_CONTROL_SHADER,
    StageType::TessellationEvaluationShader => gl::TESS_EVALUATION_SHADER,
    StageType::VertexShader => gl::VERTEX_SHADER,
    StageType::GeometryShader => gl::GEOMETRY_SHADER,
    StageType::FragmentShader => gl::FRAGMENT_SHADER,
  }
}

#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
const GLSL_PRAGMA: &str = "#version 330 core\n\
                           #extension GL_ARB_separate_shader_objects : require\n\
                           #extension GL_ARB_gpu_shader_fp64 : require\n\
                           layout(std140) uniform;\n\
                           #line 1\n";
#[cfg(not(feature = "GL_ARB_gpu_shader_fp64"))]
const GLSL_PRAGMA: &str = "#version 330 core\n\
                           #extension GL_ARB_separate_shader_objects : require\n\
                           layout(std140) uniform;\n\
                           #line 1\n";

fn glsl_pragma_src(src: &str) -> String {
  let mut pragma = String::from(GLSL_PRAGMA);
  pragma.push_str(src);
  pragma
}

fn uniform_type_match(
  program: GLuint,
  name: &str,
  ty: UniformType,
  size: usize,
) -> Result<(), UniformWarning> {
  let mut glty: GLuint = 0;
  let mut found_size: GLint = 0;

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
      &mut found_size,
      &mut glty,
      name_.as_mut_ptr(),
    );
  }

  let found_size = found_size as usize;
  if size > 0 && found_size != size {
    return Err(UniformWarning::size_mismatch(name, size, found_size));
  }

  check_uniform_type_match(name, ty, glty)
}

#[allow(clippy::cognitive_complexity)]
fn check_uniform_type_match(
  name: &str,
  ty: UniformType,
  glty: GLuint,
) -> Result<(), UniformWarning> {
  // helper macro to check type mismatch for each variant
  macro_rules! milkcheck {
    ($ty:expr, $( ( $v:tt, $t:tt ) ),* $(,)?) => {
      match $ty {
        $(
          UniformType::$v => {
            if glty == gl::$t {
              Ok(())
            } else {
              Err(UniformWarning::type_mismatch(name, ty))
            }
          }
        )*

        _ => Err(UniformWarning::unsupported_type(name, ty))
      }
    }
  }

  milkcheck!(
    ty,
    // scalars
    (Int, INT),
    (UInt, UNSIGNED_INT),
    (Float, FLOAT),
    (Double, DOUBLE),
    (Bool, BOOL),
    // vectors
    (IVec2, INT_VEC2),
    (IVec3, INT_VEC3),
    (IVec4, INT_VEC4),
    (UIVec2, UNSIGNED_INT_VEC2),
    (UIVec3, UNSIGNED_INT_VEC3),
    (UIVec4, UNSIGNED_INT_VEC4),
    (Vec2, FLOAT_VEC2),
    (Vec3, FLOAT_VEC3),
    (Vec4, FLOAT_VEC4),
    (DVec2, DOUBLE_VEC2),
    (DVec3, DOUBLE_VEC3),
    (DVec4, DOUBLE_VEC4),
    (BVec2, BOOL_VEC2),
    (BVec3, BOOL_VEC3),
    (BVec4, BOOL_VEC4),
    // matrices
    (M22, FLOAT_MAT2),
    (M33, FLOAT_MAT3),
    (M44, FLOAT_MAT4),
    (DM22, DOUBLE_MAT2),
    (DM33, DOUBLE_MAT3),
    (DM44, DOUBLE_MAT4),
    // textures
    (ISampler1D, INT_SAMPLER_1D),
    (ISampler2D, INT_SAMPLER_2D),
    (ISampler3D, INT_SAMPLER_3D),
    (ISampler1DArray, INT_SAMPLER_1D_ARRAY),
    (ISampler2DArray, INT_SAMPLER_2D_ARRAY),
    (UISampler1D, UNSIGNED_INT_SAMPLER_1D),
    (UISampler2D, UNSIGNED_INT_SAMPLER_2D),
    (UISampler3D, UNSIGNED_INT_SAMPLER_3D),
    (UISampler1DArray, UNSIGNED_INT_SAMPLER_1D_ARRAY),
    (UISampler2DArray, UNSIGNED_INT_SAMPLER_2D_ARRAY),
    (Sampler1D, SAMPLER_1D),
    (Sampler2D, SAMPLER_2D),
    (Sampler3D, SAMPLER_3D),
    (Sampler1DArray, SAMPLER_1D_ARRAY),
    (Sampler2DArray, SAMPLER_2D_ARRAY),
    (ICubemap, INT_SAMPLER_CUBE),
    (UICubemap, UNSIGNED_INT_SAMPLER_CUBE),
    (Cubemap, SAMPLER_CUBE),
  )
}

fn bind_vertex_attribs_locations<Sem>(program: &Program) -> Vec<VertexAttribWarning>
where
  Sem: Semantics,
{
  let mut warnings = Vec::new();

  for desc in Sem::semantics_set() {
    match get_vertex_attrib_location(program, &desc.name) {
      Ok(_) => {
        let index = desc.index as GLuint;

        // we are not interested in the location as we’re about to change it to what we’ve
        // decided in the semantics
        let c_name = CString::new(desc.name.as_bytes()).unwrap();
        unsafe { gl::BindAttribLocation(program.handle, index, c_name.as_ptr() as *const GLchar) };
      }

      Err(warning) => warnings.push(warning),
    }
  }

  warnings
}

fn get_vertex_attrib_location(
  program: &Program,
  name: &str,
) -> Result<GLuint, VertexAttribWarning> {
  let location = {
    let c_name = CString::new(name.as_bytes()).unwrap();
    unsafe { gl::GetAttribLocation(program.handle, c_name.as_ptr() as *const GLchar) }
  };

  if location < 0 {
    Err(VertexAttribWarning::inactive(name))
  } else {
    Ok(location as _)
  }
}

macro_rules! impl_Uniformable {
  (Arr<$t:ty>, $uty:tt, $f:tt) => {
    unsafe impl<'a, const N: usize> Uniformable<'a, Arr<$t, N>> for GL33 {
      type Target = &'a [$t; N];

      const SIZE: usize = N;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<$t, N>>, value: Self::Target) {
        gl::$f(uniform.index(), N as GLsizei, value.as_ptr() as _);
      }
    }
  };

  (vec $t:ty, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $t> for GL33 {
      type Target = $t;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(_: &mut Program, uniform: &'a Uniform<$t>, value: Self::Target) {
        gl::$f(uniform.index(), 1, value.as_ptr());
      }
    }
  };

  ($t:ty, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $t> for GL33 {
      type Target = $t;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(_: &mut Program, uniform: &'a Uniform<$t>, value: Self::Target) {
        gl::$f(uniform.index(), value);
      }
    }
  };

  // matrix notation
  (mat Arr<$t:ty>, $uty:tt, $f:tt) => {
    unsafe impl<'a, const N: usize> Uniformable<'a, Arr<$t, N>> for GL33 {
      type Target = &'a [$t; N];

      const SIZE: usize = N;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<$t, N>>, value: Self::Target) {
        gl::$f(
          uniform.index(),
          N as GLsizei,
          gl::FALSE,
          value.as_ptr() as _,
        );
      }
    }
  };

  (mat $t:ty, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $t> for GL33 {
      type Target = $t;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(_: &mut Program, uniform: &'a Uniform<$t>, value: Self::Target) {
        gl::$f(uniform.index(), 1, gl::FALSE, value.as_ptr() as _);
      }
    }
  };
}

impl_Uniformable!(i32, Int, Uniform1i);
impl_Uniformable!(vec Vec2<i32>, IVec2, Uniform2iv);
impl_Uniformable!(vec Vec3<i32>, IVec3, Uniform3iv);
impl_Uniformable!(vec Vec4<i32>, IVec4, Uniform4iv);

impl_Uniformable!(Arr<i32>, Int, Uniform1iv);
impl_Uniformable!(Arr<Vec2<i32>>, IVec2, Uniform2iv);
impl_Uniformable!(Arr<Vec3<i32>>, IVec3, Uniform3iv);
impl_Uniformable!(Arr<Vec4<i32>>, IVec4, Uniform4iv);

impl_Uniformable!(u32, UInt, Uniform1ui);
impl_Uniformable!(vec Vec2<u32>, UIVec2, Uniform2uiv);
impl_Uniformable!(vec Vec3<u32>, UIVec3, Uniform3uiv);
impl_Uniformable!(vec Vec4<u32>, UIVec4, Uniform4uiv);
impl_Uniformable!(Arr<u32>, UInt, Uniform1uiv);
impl_Uniformable!(Arr<Vec2<u32>>, UIVec2, Uniform2uiv);
impl_Uniformable!(Arr<Vec3<u32>>, UIVec3, Uniform3uiv);
impl_Uniformable!(Arr<Vec4<u32>>, UIVec4, Uniform4uiv);

impl_Uniformable!(f32, Float, Uniform1f);
impl_Uniformable!(vec Vec2<f32>, Vec2, Uniform2fv);
impl_Uniformable!(vec Vec3<f32>, Vec3, Uniform3fv);
impl_Uniformable!(vec Vec4<f32>, Vec4, Uniform4fv);
impl_Uniformable!(Arr<f32>, Float, Uniform1fv);
impl_Uniformable!(Arr<Vec2<f32>>, Vec2, Uniform2fv);
impl_Uniformable!(Arr<Vec3<f32>>, Vec3, Uniform3fv);
impl_Uniformable!(Arr<Vec4<f32>>, Vec4, Uniform4fv);

#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(f64, Double, Uniform1d);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(vec Vec2<f64>, DVec2, Uniform2dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(vec Vec3<f64>, DVec3, Uniform3dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(vec Vec4<f64>, DVec4, Uniform4dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(Arr<f64>, Double, Uniform1dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(Arr<Vec2<f64>>, DVec2, Uniform2dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(Arr<Vec3<f64>>, DVec3, Uniform3dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(Arr<Vec4<f64>>, DVec4, Uniform4dv);

impl_Uniformable!(mat Mat22<f32>, M22, UniformMatrix2fv);
impl_Uniformable!(mat Arr<Mat22<f32>>, M22, UniformMatrix2fv);

impl_Uniformable!(mat Mat33<f32>, M33, UniformMatrix3fv);
impl_Uniformable!(mat Arr<Mat33<f32>>, M33, UniformMatrix3fv);

impl_Uniformable!(mat Mat44<f32>, M44, UniformMatrix4fv);
impl_Uniformable!(mat Arr<Mat44<f32>>, M44, UniformMatrix4fv);

#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Mat22<f64>, DM22, UniformMatrix2dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Arr<Mat22<f64>>, DM22, UniformMatrix2dv);

#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Mat33<f64>, DM33, UniformMatrix3dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Arr<Mat33<f64>>, DM33, UniformMatrix3dv);

#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Mat44<f64>, DM44, UniformMatrix4dv);
#[cfg(feature = "GL_ARB_gpu_shader_fp64")]
impl_Uniformable!(mat Arr<Mat44<f64>>, DM44, UniformMatrix4dv);

unsafe impl<'a> Uniformable<'a, bool> for GL33 {
  type Target = bool;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<bool>, value: Self::Target) {
    gl::Uniform1ui(uniform.index(), value as u32);
  }
}

unsafe impl<'a> Uniformable<'a, Vec2<bool>> for GL33 {
  type Target = Vec2<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Vec2<bool>>, value: Self::Target) {
    let v = [value[0] as u32, value[1] as u32];
    gl::Uniform2uiv(uniform.index(), 1, v.as_ptr() as _);
  }
}

unsafe impl<'a> Uniformable<'a, Vec3<bool>> for GL33 {
  type Target = Vec3<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Vec3<bool>>, value: Self::Target) {
    let v = [value[0] as u32, value[1] as u32, value[2] as u32];
    gl::Uniform3uiv(uniform.index(), 1, v.as_ptr() as _);
  }
}

unsafe impl<'a> Uniformable<'a, Vec4<bool>> for GL33 {
  type Target = Vec4<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Vec4<bool>>, value: Self::Target) {
    let v = [
      value[0] as u32,
      value[1] as u32,
      value[2] as u32,
      value[3] as u32,
    ];
    gl::Uniform4uiv(uniform.index(), 1, v.as_ptr() as _);
  }
}

// a cache for implementors needing to switch from [bool; N] to [u32; N]
static mut BOOL_CACHE: Vec<u32> = Vec::new();

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<bool, N>> for GL33 {
  type Target = &'a [bool; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<bool, N>>, value: Self::Target) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(value.iter().map(|x| *x as u32));

    gl::Uniform1uiv(uniform.index(), N as GLsizei, BOOL_CACHE.as_ptr() as _);
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec2<bool>, N>> for GL33 {
  type Target = &'a [Vec2<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<Vec2<bool>, N>>, value: Self::Target) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(value.iter().flat_map(|x| [x[0] as u32, x[1] as u32]));

    gl::Uniform2uiv(uniform.index(), N as GLsizei, BOOL_CACHE.as_ptr() as _);
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec3<bool>, N>> for GL33 {
  type Target = &'a [Vec3<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<Vec3<bool>, N>>, value: Self::Target) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(
      value
        .iter()
        .flat_map(|x| [x[0] as u32, x[1] as u32, x[2] as u32]),
    );

    gl::Uniform3uiv(uniform.index(), N as GLsizei, BOOL_CACHE.as_ptr() as _);
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec4<bool>, N>> for GL33 {
  type Target = &'a [Vec4<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(_: &mut Program, uniform: &'a Uniform<Arr<Vec4<bool>, N>>, value: Self::Target) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(
      value
        .iter()
        .flat_map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32]),
    );

    gl::Uniform4uiv(uniform.index(), N as GLsizei, BOOL_CACHE.as_ptr() as _);
  }
}

unsafe impl<'a, T> Uniformable<'a, ShaderDataBinding<T>> for GL33
where
  T: 'a,
{
  type Target = ShaderDataBinding<T>;

  const SIZE: usize = 0;

  unsafe fn ty() -> UniformType {
    UniformType::ShaderDataBinding
  }

  unsafe fn update(
    program: &mut Program,
    uniform: &'a Uniform<ShaderDataBinding<T>>,
    value: Self::Target,
  ) {
    gl::UniformBlockBinding(
      program.handle,
      uniform.index() as GLuint,
      value.binding() as GLuint,
    )
  }
}

unsafe impl<'a, D, S> Uniformable<'a, TextureBinding<D, S>> for GL33
where
  D: 'a + Dimensionable,
  S: 'a + SamplerType,
{
  type Target = TextureBinding<D, S>;

  const SIZE: usize = 0;

  unsafe fn ty() -> UniformType {
    match (S::sample_type(), D::dim()) {
      (PixelType::NormIntegral, Dim::Dim1) => UniformType::Sampler1D,
      (PixelType::NormUnsigned, Dim::Dim1) => UniformType::Sampler1D,
      (PixelType::Integral, Dim::Dim1) => UniformType::ISampler1D,
      (PixelType::Unsigned, Dim::Dim1) => UniformType::UISampler1D,
      (PixelType::Floating, Dim::Dim1) => UniformType::Sampler1D,

      (PixelType::NormIntegral, Dim::Dim2) => UniformType::Sampler2D,
      (PixelType::NormUnsigned, Dim::Dim2) => UniformType::Sampler2D,
      (PixelType::Integral, Dim::Dim2) => UniformType::ISampler2D,
      (PixelType::Unsigned, Dim::Dim2) => UniformType::UISampler2D,
      (PixelType::Floating, Dim::Dim2) => UniformType::Sampler2D,

      (PixelType::NormIntegral, Dim::Dim3) => UniformType::Sampler3D,
      (PixelType::NormUnsigned, Dim::Dim3) => UniformType::Sampler3D,
      (PixelType::Integral, Dim::Dim3) => UniformType::ISampler3D,
      (PixelType::Unsigned, Dim::Dim3) => UniformType::UISampler3D,
      (PixelType::Floating, Dim::Dim3) => UniformType::Sampler3D,

      (PixelType::NormIntegral, Dim::Cubemap) => UniformType::Cubemap,
      (PixelType::NormUnsigned, Dim::Cubemap) => UniformType::Cubemap,
      (PixelType::Integral, Dim::Cubemap) => UniformType::ICubemap,
      (PixelType::Unsigned, Dim::Cubemap) => UniformType::UICubemap,
      (PixelType::Floating, Dim::Cubemap) => UniformType::Cubemap,

      (PixelType::NormIntegral, Dim::Dim1Array) => UniformType::Sampler1DArray,
      (PixelType::NormUnsigned, Dim::Dim1Array) => UniformType::Sampler1DArray,
      (PixelType::Integral, Dim::Dim1Array) => UniformType::ISampler1DArray,
      (PixelType::Unsigned, Dim::Dim1Array) => UniformType::UISampler1DArray,
      (PixelType::Floating, Dim::Dim1Array) => UniformType::Sampler1DArray,

      (PixelType::NormIntegral, Dim::Dim2Array) => UniformType::Sampler2DArray,
      (PixelType::NormUnsigned, Dim::Dim2Array) => UniformType::Sampler2DArray,
      (PixelType::Integral, Dim::Dim2Array) => UniformType::ISampler2DArray,
      (PixelType::Unsigned, Dim::Dim2Array) => UniformType::UISampler2DArray,
      (PixelType::Floating, Dim::Dim2Array) => UniformType::Sampler2DArray,
    }
  }

  unsafe fn update(
    _: &mut Program,
    uniform: &'a Uniform<TextureBinding<D, S>>,
    value: Self::Target,
  ) {
    gl::Uniform1i(uniform.index(), value.binding() as GLint)
  }
}

unsafe impl<T> ShaderData<T> for GL33
where
  T: Std140,
{
  type ShaderDataRepr = Buffer<<ArrElem<T> as Std140>::Encoded>;

  unsafe fn new_shader_data(
    &mut self,
    values: impl Iterator<Item = T>,
  ) -> Result<Self::ShaderDataRepr, ShaderDataError> {
    Ok(Buffer::from_vec(
      self,
      values
        .into_iter()
        .map(|x| ArrElem(x).std140_encode())
        .collect(),
    ))
  }

  unsafe fn get_shader_data_at(
    shader_data: &Self::ShaderDataRepr,
    i: usize,
  ) -> Result<T, ShaderDataError> {
    shader_data
      .buf
      .get(i)
      .map(|&x| <ArrElem<T> as Std140>::std140_decode(x).0)
      .ok_or_else(|| ShaderDataError::OutOfBounds { index: i })
  }

  unsafe fn set_shader_data_at(
    shader_data: &mut Self::ShaderDataRepr,
    i: usize,
    x: T,
  ) -> Result<T, ShaderDataError> {
    let prev = mem::replace(
      &mut shader_data
        .slice_buffer_mut()
        .map_err(|_| ShaderDataError::CannotSetData { index: i })?[i],
      ArrElem(x).std140_encode(),
    );

    Ok(<ArrElem<T> as Std140>::std140_decode(prev).0)
  }

  unsafe fn set_shader_data_values(
    shader_data: &mut Self::ShaderDataRepr,
    values: impl Iterator<Item = T>,
  ) -> Result<(), ShaderDataError> {
    let mut slice = shader_data
      .slice_buffer_mut()
      .map_err(|_| ShaderDataError::CannotReplaceData)?;

    for (item, value) in slice.iter_mut().zip(values) {
      *item = ArrElem(value).std140_encode();
    }

    Ok(())
  }
}
