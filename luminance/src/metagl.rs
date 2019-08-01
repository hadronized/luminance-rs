//! OpenGL module provider.
//!
//! This module provides OpenGL types and functions that are used to implement the rest of this
//! crate.

#[cfg(feature = "std")]
mod meta {
  pub(crate) use gl;
  pub(crate) use gl::types::*;
}

#[cfg(not(feature = "std"))]
mod meta {
  use alloc::vec::Vec;

  // types
  pub type GLenum = u32;
  pub type GLboolean = u8;
  pub type GLbitfield = u32;
  // pub type GLvoid = ();
  // pub type GLbyte = i8;
  // pub type GLshort = i16;
  pub type GLint = i32;
  // pub type GLubyte = u8;
  // pub type GLushort = u16;
  pub type GLuint = u32;
  pub type GLsizei = i32;
  pub type GLfloat = f32;
  // pub type GLclampf = f32;
  // pub type GLdouble = f64;
  // pub type GLclampd = f64;
  pub type GLchar = i8;
  pub type GLsizeiptr = isize;

  // c_void, taken from the libc crate
  // Use repr(u8) as LLVM expects `void*` to be the same as `i8*` to help enable
  // more optimization opportunities around it recognizing things like
  // malloc/free.
  #[repr(u8)]
  #[allow(non_camel_case_types)]
  pub enum c_void {
    // Two dummy variants so the #[repr] attribute can be used.
    #[doc(hidden)]
    __variant1,
    #[doc(hidden)]
    __variant2,
  }

  // a little helper to build GL-compatible strings (akin to CString, but since we don’t have
  // access to libffi… and that GLchar = i8… yeah fuck it)
  #[derive(Debug)]
  pub struct NulError;

  #[inline(always)]
  pub unsafe fn with_cstring<F, A>(s: &str, f: F) -> Result<A, NulError>
  where F: FnOnce(*const GLchar) -> A {
    let bytes = s.as_bytes();

    if bytes.contains(&b'\0') {
      Err(NulError)
    } else {
      let mut marshalled = Vec::with_capacity(s.len() + 1); // +1 for the NUL byte
      marshalled.extend(bytes);
      marshalled.push(b'\0'); // hello you dancing byte
      Ok(f(marshalled.as_ptr() as *const GLchar))
    }
  }

  pub mod gl {
    use super::*;

    // constants
    pub const ACTIVE_TEXTURE: GLenum = 34016;
    pub const ACTIVE_UNIFORM_MAX_LENGTH: GLenum = 35719;
    pub const ALWAYS: GLenum = 519;
    pub const ARRAY_BUFFER: GLenum = 34962;
    pub const BACK: GLenum = 1029;
    pub const BLEND: GLenum = 3042;
    pub const BLEND_DST_RGB: GLenum = 32968;
    pub const BLEND_EQUATION_RGB: GLenum = 32777;
    pub const BLEND_SRC_RGB: GLenum = 32969;
    pub const BOOL: GLenum = 35670;
    pub const BOOL_VEC2: GLenum = 35671;
    pub const BOOL_VEC3: GLenum = 35672;
    pub const BOOL_VEC4: GLenum = 35673;
    pub const BYTE: GLenum = 5120;
    pub const CCW: GLenum = 2305;
    pub const CLAMP_TO_EDGE: GLenum = 33071;
    pub const COLOR_ATTACHMENT0: GLenum = 36064;
    pub const COLOR_BUFFER_BIT: GLenum = 16384;
    pub const COMPARE_REF_TO_TEXTURE: GLenum = 34894;
    pub const COMPILE_STATUS: GLenum = 35713;
    pub const CULL_FACE: GLenum = 2884;
    pub const CULL_FACE_MODE: GLenum = 2885;
    pub const CURRENT_PROGRAM: GLenum = 35725;
    pub const CW: GLenum = 2304;
    pub const DEPTH_ATTACHMENT: GLenum = 36096;
    pub const DEPTH_COMPONENT: GLenum = 6402;
    pub const DEPTH_COMPONENT32F: GLenum = 36012;
    pub const DEPTH_BUFFER_BIT: GLenum = 256;
    pub const DEPTH_TEST: GLenum = 2929;
    pub const DRAW_FRAMEBUFFER: GLenum = 36009;
    pub const DRAW_FRAMEBUFFER_BINDING: GLenum = 36006;
    pub const DST_ALPHA: GLenum = 772;
    pub const DST_COLOR: GLenum = 774;
    pub const ELEMENT_ARRAY_BUFFER: GLenum = 34963;
    pub const EQUAL: GLenum = 514;
    pub const FLOAT: GLenum = 5126;
    pub const FLOAT_MAT2: GLenum = 35674;
    pub const FLOAT_MAT3: GLenum = 35675;
    pub const FLOAT_MAT4: GLenum = 35676;
    pub const FLOAT_VEC2: GLenum = 35664;
    pub const FLOAT_VEC3: GLenum = 35665;
    pub const FLOAT_VEC4: GLenum = 35666;
    pub const FRAGMENT_SHADER: GLenum = 35632;
    pub const FRAMEBUFFER: GLenum = 36160;
    pub const FRAMEBUFFER_COMPLETE: GLenum = 36053;
    pub const FRAMEBUFFER_INCOMPLETE_ATTACHMENT: GLenum = 36054;
    pub const FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER: GLenum = 36059;
    pub const FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS: GLenum = 36264;
    pub const FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: GLenum = 36055;
    pub const FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: GLenum = 36182;
    pub const FRAMEBUFFER_INCOMPLETE_READ_BUFFER: GLenum = 36060;
    pub const FRAMEBUFFER_UNDEFINED: GLenum = 33305;
    pub const FRAMEBUFFER_UNSUPPORTED: GLenum = 36061;
    pub const FRONT: GLenum = 1028;
    pub const FRONT_AND_BACK: GLenum = 1032;
    pub const FRONT_FACE: GLenum = 2886;
    pub const FUNC_ADD: GLenum = 32774;
    pub const FUNC_REVERSE_SUBTRACT: GLenum = 32779;
    pub const FUNC_SUBTRACT: GLenum = 32778;
    pub const GEOMETRY_SHADER: GLenum = 36313;
    pub const GEQUAL: GLenum = 518;
    pub const GREATER: GLenum = 516;
    pub const INFO_LOG_LENGTH: GLenum = 35716;
    pub const INT: GLenum = 5124;
    pub const INT_SAMPLER_1D: GLenum = 36297;
    pub const INT_SAMPLER_2D: GLenum = 36298;
    pub const INT_SAMPLER_3D: GLenum = 36299;
    pub const INT_SAMPLER_CUBE: GLenum = 36300;
    pub const INT_VEC2: GLenum = 35667;
    pub const INT_VEC3: GLenum = 35668;
    pub const INT_VEC4: GLenum = 35669;
    pub const INVALID_INDEX: GLenum = 4294967295;
    pub const LEQUAL: GLenum = 515;
    pub const LESS: GLenum = 513;
    pub const LINE_STRIP: GLenum = 3;
    pub const LINEAR: GLenum = 9729;
    pub const LINEAR_MIPMAP_LINEAR: GLenum = 9987;
    pub const LINEAR_MIPMAP_NEAREST: GLenum = 9985;
    pub const LINES: GLenum = 1;
    pub const LINK_STATUS: GLenum = 35714;
    pub const MAX: GLenum = 32776;
    pub const MIN: GLenum = 32775;
    pub const MIRRORED_REPEAT: GLenum = 33648;
    pub const NEAREST: GLenum = 9728;
    pub const NEAREST_MIPMAP_LINEAR: GLenum = 9986;
    pub const NEAREST_MIPMAP_NEAREST: GLenum = 9984;
    pub const NEVER: GLenum = 512;
    pub const NONE: GLenum = 0;
    pub const NOTEQUAL: GLenum = 517;
    pub const ONE: GLenum = 1;
    pub const ONE_MINUS_DST_ALPHA: GLenum = 773;
    pub const ONE_MINUS_DST_COLOR: GLenum = 775;
    pub const ONE_MINUS_SRC_ALPHA: GLenum = 771;
    pub const ONE_MINUS_SRC_COLOR: GLenum = 769;
    pub const POINTS: GLenum = 0;
    pub const READ_ONLY: GLenum = 35000;
    pub const READ_WRITE: GLenum = 35002;
    pub const RENDERBUFFER: GLenum = 36161;
    pub const RED: GLenum = 6403;
    pub const RED_INTEGER: GLenum = 36244;
    pub const R8I: GLenum = 33329;
    pub const R8UI: GLenum = 33330;
    pub const R16I: GLenum = 33331;
    pub const R16UI: GLenum = 33332;
    pub const R32F: GLenum = 33326;
    pub const R32I: GLenum = 33333;
    pub const R32UI: GLenum = 33334;
    pub const REPEAT: GLenum = 10497;
    pub const RG: GLenum = 33319;
    pub const RG_INTEGER: GLenum = 33320;
    pub const RG8I: GLenum = 33335;
    pub const RG8UI: GLenum = 33336;
    pub const RG16I: GLenum = 33337;
    pub const RG16UI: GLenum = 33338;
    pub const RG32F: GLenum = 33328;
    pub const RG32I: GLenum = 33339;
    pub const RG32UI: GLenum = 33340;
    pub const RGB: GLenum = 6407;
    pub const RGB_INTEGER: GLenum = 36248;
    pub const RGB8I: GLenum = 36239;
    pub const RGB8UI: GLenum = 36221;
    pub const RGB16I: GLenum = 36233;
    pub const RGB16UI: GLenum = 36215;
    pub const RGB32F: GLenum = 34837;
    pub const RGB32I: GLenum = 36227;
    pub const RGB32UI: GLenum = 36209;
    pub const RGBA: GLenum = 6408;
    pub const RGBA_INTEGER: GLenum = 36249;
    pub const RGBA8I: GLenum = 36238;
    pub const RGBA8UI: GLenum = 36220;
    pub const RGBA16I: GLenum = 36232;
    pub const RGBA16UI: GLenum = 36214;
    pub const RGBA32F: GLenum = 34836;
    pub const RGBA32I: GLenum = 36226;
    pub const RGBA32UI: GLenum = 36208;
    pub const SHORT: GLenum = 5122;
    pub const SAMPLER_1D: GLenum = 35677;
    pub const SAMPLER_2D: GLenum = 35678;
    pub const SAMPLER_3D: GLenum = 35679;
    pub const SAMPLER_CUBE: GLenum = 35680;
    pub const SRC_ALPHA: GLenum = 770;
    pub const SRC_ALPHA_SATURATE: GLenum = 776;
    pub const SRC_COLOR: GLenum = 768;
    pub const STREAM_DRAW: GLenum = 35040;
    pub const TESS_CONTROL_SHADER: GLenum = 36488;
    pub const TESS_EVALUATION_SHADER: GLenum = 36487;
    pub const TEXTURE0: GLenum = 33984;
    pub const TEXTURE_1D: GLenum = 3552;
    pub const TEXTURE_1D_ARRAY: GLenum = 35864;
    pub const TEXTURE_2D: GLenum = 3553;
    pub const TEXTURE_2D_ARRAY: GLenum = 35866;
    pub const TEXTURE_3D: GLenum = 32879;
    pub const TEXTURE_BASE_LEVEL: GLenum = 33084;
    pub const TEXTURE_COMPARE_FUNC: GLenum = 34893;
    pub const TEXTURE_COMPARE_MODE: GLenum = 34892;
    pub const TEXTURE_CUBE_MAP: GLenum = 34067;
    pub const TEXTURE_CUBE_MAP_ARRAY: GLenum = 36873;
    pub const TEXTURE_CUBE_MAP_POSITIVE_X: GLenum = 34069;
    pub const TEXTURE_HEIGHT: GLenum = 4097;
    pub const TEXTURE_MAG_FILTER: GLenum = 10240;
    pub const TEXTURE_MAX_LEVEL: GLenum = 33085;
    pub const TEXTURE_MIN_FILTER: GLenum = 10241;
    pub const TEXTURE_WIDTH: GLenum = 4096;
    pub const TEXTURE_WRAP_R: GLenum = 32882;
    pub const TEXTURE_WRAP_S: GLenum = 10242;
    pub const TEXTURE_WRAP_T: GLenum = 10243;
    pub const TRIANGLE_FAN: GLenum = 6;
    pub const TRIANGLE_STRIP: GLenum = 5;
    pub const TRIANGLES: GLenum = 4;
    pub const UNIFORM_BUFFER: GLenum = 35345;
    pub const UNSIGNED_BYTE: GLenum = 5121;
    pub const UNSIGNED_INT: GLenum = 5125;
    pub const UNSIGNED_INT_SAMPLER_1D: GLenum = 36305;
    pub const UNSIGNED_INT_SAMPLER_2D: GLenum = 36306;
    pub const UNSIGNED_INT_SAMPLER_3D: GLenum = 36307;
    pub const UNSIGNED_INT_SAMPLER_CUBE: GLenum = 36308;
    pub const UNSIGNED_INT_VEC2: GLenum = 35667;
    pub const UNSIGNED_INT_VEC3: GLenum = 35668;
    pub const UNSIGNED_INT_VEC4: GLenum = 35669;
    pub const UNSIGNED_SHORT: GLenum = 5123;
    pub const VERTEX_ARRAY_BINDING: GLenum = 34229;
    pub const VERTEX_SHADER: GLenum = 35633;
    pub const WRITE_ONLY: GLenum = 35001;
    pub const ZERO: GLenum = 0;

    pub const FALSE: GLboolean = 0;
    pub const TRUE: GLboolean = 1;

    // functions
    extern "system" {
      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glActiveTexture"]
      pub fn ActiveTexture(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glAttachShader"]
      pub fn AttachShader(_: GLuint, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindBuffer"]
      pub fn BindBuffer(_: GLenum, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindBufferBase"]
      pub fn BindBufferBase(_: GLenum, _: GLuint, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindFramebuffer"]
      pub fn BindFramebuffer(_: GLenum, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindRenderbuffer"]
      pub fn BindRenderbuffer(_: GLenum, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindTexture"]
      pub fn BindTexture(_: GLenum, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBindVertexArray"]
      pub fn BindVertexArray(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBlendEquation"]
      pub fn BlendEquation(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBlendFunc"]
      pub fn BlendFunc(_: GLenum, _: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glBufferData"]
      pub fn BufferData(_: GLenum, _: GLsizeiptr, _: *const c_void, _: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glCheckFramebufferStatus"]
      pub fn CheckFramebufferStatus(_: GLenum) -> GLenum;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glClear"]
      pub fn Clear(_: GLbitfield);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glClearColor"]
      pub fn ClearColor(_: GLfloat, _: GLfloat, _: GLfloat, _: GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glCompileShader"]
      pub fn CompileShader(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glCreateProgram"]
      pub fn CreateProgram() -> GLuint;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glCreateShader"]
      pub fn CreateShader(_: GLenum) -> GLuint;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glCullFace"]
      pub fn CullFace(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteBuffers"]
      pub fn DeleteBuffers(_: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteFramebuffers"]
      pub fn DeleteFramebuffers(_: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteProgram"]
      pub fn DeleteProgram(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteRenderbuffers"]
      pub fn DeleteRenderbuffers(_: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteShader"]
      pub fn DeleteShader(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteTextures"]
      pub fn DeleteTextures(_: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDeleteVertexArrays"]
      pub fn DeleteVertexArrays(_: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDisable"]
      pub fn Disable(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawArrays"]
      pub fn DrawArrays(_: GLenum, _: GLint, _: GLsizei);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawArraysInstanced"]
      pub fn DrawArraysInstanced(_: GLenum, _: GLint, _: GLsizei, _: GLsizei);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawElements"]
      pub fn DrawElements(_: GLenum, _: GLsizei, _: GLenum, _: *const c_void);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawElementsInstanced"]
      pub fn DrawElementsInstanced(_: GLenum, _: GLsizei, _: GLenum, _: *const c_void, _: GLsizei);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawBuffer"]
      pub fn DrawBuffer(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glDrawBuffers"]
      pub fn DrawBuffers(_: GLsizei, _: *const GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glEnable"]
      pub fn Enable(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glEnableVertexAttribArray"]
      pub fn EnableVertexAttribArray(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glFramebufferRenderbuffer"]
      pub fn FramebufferRenderbuffer(_: GLenum, _: GLenum, _: GLenum, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glFramebufferTexture"]
      pub fn FramebufferTexture(_: GLenum, _: GLenum, _: GLuint, _: GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glFrontFace"]
      pub fn FrontFace(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenBuffers"]
      pub fn GenBuffers(_: GLsizei, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenFramebuffers"]
      pub fn GenFramebuffers(_: GLsizei, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenRenderbuffers"]
      pub fn GenRenderbuffers(_: GLsizei, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenTextures"]
      pub fn GenTextures(_: GLsizei, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenVertexArrays"]
      pub fn GenVertexArrays(_: GLsizei, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGenerateMipmap"]
      pub fn GenerateMipmap(_: GLenum);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetActiveUniform"]
      pub fn GetActiveUniform(
        _: GLuint,
        _: GLuint,
        _: GLsizei,
        _: *mut GLsizei,
        _: *mut GLint,
        _: *mut GLenum,
        _: *mut GLchar,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetIntegerv"]
      pub fn GetIntegerv(_: GLenum, _: *mut GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetProgramInfoLog"]
      pub fn GetProgramInfoLog(_: GLuint, _: GLsizei, _: *mut GLsizei, _: *mut GLchar);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetProgramiv"]
      pub fn GetProgramiv(_: GLuint, _: GLenum, _: *mut GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetShaderInfoLog"]
      pub fn GetShaderInfoLog(_: GLuint, _: GLsizei, _: *mut GLsizei, _: *mut GLchar);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetShaderiv"]
      pub fn GetShaderiv(_: GLuint, _: GLenum, _: *mut GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetTexImage"]
      pub fn GetTexImage(_: GLenum, _: GLint, _: GLenum, _: GLenum, _: *mut c_void);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetTexLevelParameteriv"]
      pub fn GetTexLevelParameteriv(_: GLenum, _: GLint, _: GLenum, _: *mut GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetUniformBlockIndex"]
      pub fn GetUniformBlockIndex(_: GLuint, _: *const GLchar) -> GLuint;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetUniformLocation"]
      pub fn GetUniformLocation(_: GLuint, _: *const GLchar) -> GLint;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glGetUniformIndices"]
      pub fn GetUniformIndices(_: GLuint, _: GLsizei, _: *const *const GLchar, _: *mut GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glIsEnabled"]
      pub fn IsEnabled(_: GLenum) -> GLboolean;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glLinkProgram"]
      pub fn LinkProgram(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glMapBuffer"]
      pub fn MapBuffer(_: GLenum, _: GLenum) -> *mut c_void;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glRenderbufferStorage"]
      pub fn RenderbufferStorage(_: GLenum, _: GLenum, _: GLsizei, _: GLsizei);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glShaderSource"]
      pub fn ShaderSource(_: GLuint, _: GLsizei, _: *const *const GLchar, _: *const GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexImage1D"]
      pub fn TexImage1D(
        _: GLenum,
        _: GLint,
        _: GLint,
        _: GLsizei,
        _: GLint,
        _: GLenum,
        _: GLenum,
        _: *const c_void,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexImage2D"]
      pub fn TexImage2D(
        _: GLenum,
        _: GLint,
        _: GLint,
        _: GLsizei,
        _: GLsizei,
        _: GLint,
        _: GLenum,
        _: GLenum,
        _: *const c_void,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexImage3D"]
      pub fn TexImage3D(
        _: GLenum,
        _: GLint,
        _: GLint,
        _: GLsizei,
        _: GLsizei,
        _: GLsizei,
        _: GLint,
        _: GLenum,
        _: GLenum,
        _: *const c_void,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexParameteri"]
      pub fn TexParameteri(_: GLenum, _: GLenum, _: GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexSubImage1D"]
      pub fn TexSubImage1D(_: GLenum, _: GLint, _: GLint, _: GLsizei, _: GLenum, _: GLenum, _: *const c_void);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexSubImage2D"]
      pub fn TexSubImage2D(
        _: GLenum,
        _: GLint,
        _: GLint,
        _: GLint,
        _: GLsizei,
        _: GLsizei,
        _: GLenum,
        _: GLenum,
        _: *const c_void,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glTexSubImage3D"]
      pub fn TexSubImage3D(
        _: GLenum,
        _: GLint,
        _: GLint,
        _: GLint,
        _: GLint,
        _: GLsizei,
        _: GLsizei,
        _: GLsizei,
        _: GLenum,
        _: GLenum,
        _: *const c_void,
      );

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1f"]
      pub fn Uniform1f(_: GLint, _: GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1i"]
      pub fn Uniform1i(_: GLint, _: GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1ui"]
      pub fn Uniform1ui(_: GLint, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1iv"]
      pub fn Uniform1iv(_: GLint, _: GLsizei, _: *const GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1fv"]
      pub fn Uniform1fv(_: GLint, _: GLsizei, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform1uiv"]
      pub fn Uniform1uiv(_: GLint, _: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform2fv"]
      pub fn Uniform2fv(_: GLint, _: GLsizei, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform2iv"]
      pub fn Uniform2iv(_: GLint, _: GLsizei, _: *const GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform2uiv"]
      pub fn Uniform2uiv(_: GLint, _: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform3fv"]
      pub fn Uniform3fv(_: GLint, _: GLsizei, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform3iv"]
      pub fn Uniform3iv(_: GLint, _: GLsizei, _: *const GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform3uiv"]
      pub fn Uniform3uiv(_: GLint, _: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform4fv"]
      pub fn Uniform4fv(_: GLint, _: GLsizei, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform4iv"]
      pub fn Uniform4iv(_: GLint, _: GLsizei, _: *const GLint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniform4uiv"]
      pub fn Uniform4uiv(_: GLint, _: GLsizei, _: *const GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniformBlockBinding"]
      pub fn UniformBlockBinding(_: GLuint, _: GLuint, _: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniformMatrix2fv"]
      pub fn UniformMatrix2fv(_: GLint, _: GLsizei, _: GLboolean, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniformMatrix3fv"]
      pub fn UniformMatrix3fv(_: GLint, _: GLsizei, _: GLboolean, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUniformMatrix4fv"]
      pub fn UniformMatrix4fv(_: GLint, _: GLsizei, _: GLboolean, _: *const GLfloat);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUnmapBuffer"]
      pub fn UnmapBuffer(_: GLenum) -> GLboolean;

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glUseProgram"]
      pub fn UseProgram(_: GLuint);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glVertexAttribPointer"]
      pub fn VertexAttribPointer(_: GLuint, _: GLint, _: GLenum, _: GLboolean, _: GLsizei, _: *const c_void);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glVertexAttribIPointer"]
      pub fn VertexAttribIPointer(_: GLuint, _: GLint, _: GLenum, _: GLsizei, _: *const c_void);

      #[allow(non_snake_case)]
      #[inline]
      #[link_name = "glViewport"]
      pub fn Viewport(_: GLint, _: GLint, _: GLsizei, _: GLsizei);
    }
  }
}

pub(crate) use self::meta::*;
