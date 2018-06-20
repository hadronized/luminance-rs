use gl;
use gl::types::*;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::ptr::{null, null_mut};

type Result<A> = ::std::result::Result<A, StageError>;

/// A shader stage type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  TessellationControlShader,
  TessellationEvaluationShader,
  VertexShader,
  GeometryShader,
  FragmentShader
}

/// A shader stage.
#[derive(Debug)]
pub struct Stage {
  handle: GLuint,
  ty: Type
}

impl Stage {
  /// Create a new shader stage.
  pub fn new(ty: Type, src: &str) -> Result<Self> {
    unsafe {

      let src = CString::new(glsl_pragma_src(src).as_bytes()).unwrap();
      let handle = gl::CreateShader(opengl_shader_type(ty));

      if handle == 0 {
        return Err(StageError::CompilationFailed(ty, String::from("unable to create shader stage")));
      }

      gl::ShaderSource(handle, 1, [src.as_ptr()].as_ptr(), null());
      gl::CompileShader(handle);

      let mut compiled: GLint = gl::FALSE as GLint;
      gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut compiled);

      if compiled == (gl::TRUE as GLint) {
        Ok(Stage {
          handle: handle,
          ty: ty
        })
      } else {
        let mut log_len: GLint = 0;
        gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetShaderInfoLog(handle, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

        gl::DeleteShader(handle);

        log.set_len(log_len as usize);

        Err(StageError::CompilationFailed(ty, String::from_utf8(log).unwrap()))
      }
    }
  }

  #[inline]
  pub(crate) fn handle(&self) -> GLuint {
    self.handle
  }
}

impl Drop for Stage {
  fn drop(&mut self) {
    unsafe { gl::DeleteShader(self.handle) }
  }
}

/// Errors that shader stages can emit.
#[derive(Clone, Debug)]
pub enum StageError {
  /// Occurs when a shader fails to compile.
  CompilationFailed(Type, String),
  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(Type)
}

impl fmt::Display for StageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for StageError {
  fn description(&self) -> &str {
    match *self {
      StageError::CompilationFailed(..) => "compilation failed",
      StageError::UnsupportedType(_) => "unsupported type"
    }
  }
}

fn glsl_pragma_src(src: &str) -> String {
  // Naive method that miss some cases with the extensions
  // and is not memory efficient.
  // Do not think is is probelamtic for a functio ncalled so few times
  let mut lines = src.lines().collect::<Vec<_>>();

  // remove blanks lines
  while lines.len() > 0 && lines[0].is_empty() {
      lines.remove(0);
  }

  // Check if version is given
  if lines.len()>0 && !lines[0].starts_with("#version") {
    lines.insert(0, GLSL_PRAGMA_VERSION);
  }

  // Insert in all cases the extension.
  if lines.iter().any(|s| -> bool {s.starts_with("#extension") && s.contains("GL_ARB_separate_shader_objects")}) {
    lines.insert(1, GLSL_PRAGMA_EXTENSION);
  }

  lines.join("\n")
}

const GLSL_PRAGMA_VERSION: &'static str = "\
#version 330 core\n";
const GLSL_PRAGMA_EXTENSION: &'static str = "\
#extension GL_ARB_separate_shader_objects : require\n";

fn opengl_shader_type(t: Type) -> GLenum {
  match t {
    Type::TessellationControlShader => gl::TESS_CONTROL_SHADER,
    Type::TessellationEvaluationShader => gl::TESS_EVALUATION_SHADER,
    Type::VertexShader => gl::VERTEX_SHADER,
    Type::GeometryShader => gl::GEOMETRY_SHADER,
    Type::FragmentShader => gl::FRAGMENT_SHADER
  }
}
