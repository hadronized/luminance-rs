use gl;
use gl::types::*;
use luminance::shader::program2::{Type, Uniformable};
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::null_mut;

/// Warnings related to uniform issues.
#[derive(Debug)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),
  /// Type mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, Type),
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
  pub fn type_mismatch<N>(name: N, ty: Type) -> Self
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
fn uniform_type_match(program: GLuint, name: &str, ty: Type) -> Result<(), UniformWarning> {
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

// Check if a [`Type`] matches the OpenGL counterpart.
#[allow(clippy::cognitive_complexity)]
fn check_types_match(name: &str, ty: Type, glty: GLuint) -> Result<(), UniformWarning> {
  match ty {
    // scalars
    Type::Int if glty != gl::INT => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UInt if glty != gl::UNSIGNED_INT => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Float if glty != gl::FLOAT => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Bool if glty != gl::BOOL => Err(UniformWarning::type_mismatch(name, ty)),
    // vectors
    Type::IVec2 if glty != gl::INT_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::IVec3 if glty != gl::INT_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::IVec4 if glty != gl::INT_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UIVec2 if glty != gl::UNSIGNED_INT_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UIVec3 if glty != gl::UNSIGNED_INT_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UIVec4 if glty != gl::UNSIGNED_INT_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Vec2 if glty != gl::FLOAT_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Vec3 if glty != gl::FLOAT_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Vec4 if glty != gl::FLOAT_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::BVec2 if glty != gl::BOOL_VEC2 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::BVec3 if glty != gl::BOOL_VEC3 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::BVec4 if glty != gl::BOOL_VEC4 => Err(UniformWarning::type_mismatch(name, ty)),
    // matrices
    Type::M22 if glty != gl::FLOAT_MAT2 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::M33 if glty != gl::FLOAT_MAT3 => Err(UniformWarning::type_mismatch(name, ty)),
    Type::M44 if glty != gl::FLOAT_MAT4 => Err(UniformWarning::type_mismatch(name, ty)),
    // textures
    Type::ISampler1D if glty != gl::INT_SAMPLER_1D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::ISampler2D if glty != gl::INT_SAMPLER_2D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::ISampler3D if glty != gl::INT_SAMPLER_3D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UISampler1D if glty != gl::UNSIGNED_INT_SAMPLER_1D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    Type::UISampler2D if glty != gl::UNSIGNED_INT_SAMPLER_2D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    Type::UISampler3D if glty != gl::UNSIGNED_INT_SAMPLER_3D => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    Type::Sampler1D if glty != gl::SAMPLER_1D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Sampler2D if glty != gl::SAMPLER_2D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::Sampler3D if glty != gl::SAMPLER_3D => Err(UniformWarning::type_mismatch(name, ty)),
    Type::ICubemap if glty != gl::INT_SAMPLER_CUBE => Err(UniformWarning::type_mismatch(name, ty)),
    Type::UICubemap if glty != gl::UNSIGNED_INT_SAMPLER_CUBE => {
      Err(UniformWarning::type_mismatch(name, ty))
    }
    Type::Cubemap if glty != gl::SAMPLER_CUBE => Err(UniformWarning::type_mismatch(name, ty)),
    _ => Ok(()),
  }
}

#[derive(Debug)]
pub struct Uniform<T> {
  program: GLuint,
  index: GLint,
  _t: PhantomData<*const T>,
}

macro_rules! impl_uniformable {
  // slices
  (& [[$t:ty; $n:expr]], $ut:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<&'a [[$t; $n]]> for Uniform<&'a [[$t; $n]]> {
      const TY: Type = Type::$ut;

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
      const TY: Type = Type::$ut;

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
      const TY: Type = Type::$ut;

      fn update(self, value: &[$t]) {
        unsafe { gl::$f(self.index, value.len() as GLsizei, value.as_ptr()) };
      }
    }
  };

  // matrices
  ([[$t:ty; $n:expr]; $m:expr], $ut:tt, $f:tt) => {
    unsafe impl Uniformable<[[$t; $n]; $m]> for Uniform<[[$t; $n]; $m]> {
      const TY: Type = Type::$ut;

      fn update(self, value: [[$t; $n]; $m]) {
        let v = [value];
        unsafe { gl::$f(self.index, 1, gl::FALSE, v.as_ptr() as *const $t) };
      }
    }
  };

  // arrays
  ([$t:ty; $n:expr], $ut:tt, $f:tt) => {
    unsafe impl Uniformable<[$t; $n]> for Uniform<[$t; $n]> {
      const TY: Type = Type::$ut;

      fn update(self, value: [$t; $n]) {
        unsafe { gl::$f(self.index, 1, &value as *const $t) };
      }
    }
  };

  // scalars
  ($t:ty, $ut:tt, $f:tt) => {
    unsafe impl Uniformable<$t> for Uniform<$t> {
      const TY: Type = Type::$ut;

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
  const TY: Type = Type::Bool;

  fn update(self, value: bool) {
    unsafe { gl::Uniform1ui(self.index, value as GLuint) }
  }
}

unsafe impl Uniformable<[bool; 2]> for Uniform<[bool; 2]> {
  const TY: Type = Type::BVec2;

  fn update(self, value: [bool; 2]) {
    let v = [value[0] as u32, value[1] as u32];
    unsafe { gl::Uniform2uiv(self.index, 1, &v as *const u32) }
  }
}

unsafe impl Uniformable<[bool; 3]> for Uniform<[bool; 3]> {
  const TY: Type = Type::BVec3;

  fn update(self, value: [bool; 3]) {
    let v = [value[0] as u32, value[1] as u32, value[2] as u32];
    unsafe { gl::Uniform3uiv(self.index, 1, &v as *const u32) }
  }
}

unsafe impl Uniformable<[bool; 4]> for Uniform<[bool; 4]> {
  const TY: Type = Type::BVec4;

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
  const TY: Type = Type::Bool;

  fn update(self, value: &[bool]) {
    let v: Vec<_> = value.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(self.index, v.len() as GLsizei, v.as_ptr()) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 2]]> for Uniform<&'a [[bool; 2]]> {
  const TY: Type = Type::BVec2;

  fn update(self, value: &[[bool; 2]]) {
    let v: Vec<_> = value.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 3]]> for Uniform<&'a [[bool; 3]]> {
  const TY: Type = Type::BVec3;

  fn update(self, value: &[[bool; 3]]) {
    let v: Vec<_> = value
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32])
      .collect();
    unsafe { gl::Uniform3uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}

unsafe impl<'a> Uniformable<&'a [[bool; 4]]> for Uniform<&'a [[bool; 4]]> {
  const TY: Type = Type::BVec4;

  fn update(self, value: &[[bool; 4]]) {
    let v: Vec<_> = value
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32])
      .collect();
    unsafe { gl::Uniform4uiv(self.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }
}
