//! Shader support for WebGL2.

use super::buffer::{Buffer, BufferError};
use crate::webgl2::{state::WebGL2State, WebGL2};
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
use std::{cell::RefCell, collections::HashMap, mem, rc::Rc};
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

#[derive(Debug)]
pub struct Stage {
  handle: WebGlShader,
  ty: StageType,
  state: Rc<RefCell<WebGL2State>>,
}

impl Drop for Stage {
  fn drop(&mut self) {
    self.state.borrow().ctx.delete_shader(Some(&self.handle));
  }
}

impl Stage {
  fn new(webgl2: &mut WebGL2, ty: StageType, src: &str) -> Result<Self, StageError> {
    let state = webgl2.state.borrow();

    let shader_ty = webgl_shader_type(ty)
      .ok_or_else(|| StageError::CompilationFailed(ty, "unsupported shader type".to_owned()))?;

    let handle = state.ctx.create_shader(shader_ty).ok_or_else(|| {
      StageError::CompilationFailed(ty, "unable to create shader stage".to_owned())
    })?;

    state.ctx.shader_source(&handle, &patch_shader_src(src));
    state.ctx.compile_shader(&handle);

    let compiled = state
      .ctx
      .get_shader_parameter(&handle, WebGl2RenderingContext::COMPILE_STATUS)
      .as_bool()
      .ok_or_else(|| {
        StageError::CompilationFailed(ty, "cannot determine compilation status".to_owned())
      })?;

    if compiled {
      Ok(Stage {
        handle,
        ty,
        state: webgl2.state.clone(),
      })
    } else {
      let log = state
        .ctx
        .get_shader_info_log(&handle)
        .ok_or_else(|| StageError::CompilationFailed(ty, "no compilation error".to_owned()))?;

      state.ctx.delete_shader(Some(&handle));

      Err(StageError::compilation_failed(ty, log))
    }
  }

  fn handle(&self) -> &WebGlShader {
    &self.handle
  }
}

/// A type used to map [`i32`] (uniform locations) to [`WebGlUniformLocation`].
///
/// It is typically shared with internal mutation (Rc + RefCell) so that it can add the location
/// mappings in the associated [`Program`].
type LocationMap = HashMap<i32, WebGlUniformLocation>;

#[derive(Debug)]
pub struct Program {
  pub(crate) handle: WebGlProgram,
  location_map: Rc<RefCell<LocationMap>>,
  state: Rc<RefCell<WebGL2State>>,
}

impl Drop for Program {
  fn drop(&mut self) {
    self.state.borrow().ctx.delete_program(Some(&self.handle));
  }
}

impl Program {
  fn new(
    webgl2: &mut WebGL2,
    vertex: &Stage,
    tess: Option<TessellationStages<Stage>>,
    geometry: Option<&Stage>,
    fragment: &Stage,
  ) -> Result<Self, ProgramError> {
    let state = webgl2.state.borrow();

    let handle = state.ctx.create_program().ok_or_else(|| {
      ProgramError::CreationFailed("unable to allocate GPU shader program".to_owned())
    })?;

    if let Some(TessellationStages {
      control,
      evaluation,
    }) = tess
    {
      state.ctx.attach_shader(&handle, control.handle());
      state.ctx.attach_shader(&handle, evaluation.handle());
    }

    state.ctx.attach_shader(&handle, vertex.handle());

    if let Some(geometry) = geometry {
      state.ctx.attach_shader(&handle, geometry.handle());
    }

    state.ctx.attach_shader(&handle, fragment.handle());

    let location_map = Rc::new(RefCell::new(HashMap::new()));
    let state = webgl2.state.clone();
    let program = Program {
      handle,
      location_map,
      state,
    };

    program.link().map(move |_| program)
  }

  fn link(&self) -> Result<(), ProgramError> {
    let handle = &self.handle;
    let state = self.state.borrow();

    state.ctx.link_program(handle);

    let linked = state
      .ctx
      .get_program_parameter(handle, WebGl2RenderingContext::LINK_STATUS)
      .as_bool()
      .ok_or_else(|| ProgramError::LinkFailed("unknown link status".to_owned()))?;

    if linked {
      Ok(())
    } else {
      let log = state
        .ctx
        .get_program_info_log(handle)
        .unwrap_or("unknown link error".to_owned());
      Err(ProgramError::link_failed(log))
    }
  }

  fn handle(&self) -> &WebGlProgram {
    &self.handle
  }
}

pub struct UniformBuilder {
  handle: WebGlProgram,
  location_map: Rc<RefCell<LocationMap>>,
  state: Rc<RefCell<WebGL2State>>,
}

impl UniformBuilder {
  fn new(program: &Program) -> Self {
    UniformBuilder {
      handle: program.handle.clone(),
      location_map: program.location_map.clone(),
      state: program.state.clone(),
    }
  }

  fn ask_uniform<T>(
    &mut self,
    name: &str,
    ty: UniformType,
    size: usize,
  ) -> Result<Uniform<T>, UniformWarning>
  where
    WebGL2: for<'a> Uniformable<'a, T>,
  {
    let location = self
      .state
      .borrow()
      .ctx
      .get_uniform_location(&self.handle, name);

    match location {
      Some(location) => {
        // if we correctly map the uniform, we generate a new ID by using the length of the current
        // location map — we never remove items, so it will always grow — and insert the indirection
        let mut location_map = self.location_map.borrow_mut();
        let idx = location_map.len() as i32;
        location_map.insert(idx, location);

        // check the type
        uniform_type_match(
          &self.state.borrow(),
          &self.handle,
          name,
          idx as u32,
          ty,
          size,
        )?;

        Ok(unsafe { Uniform::new(idx) })
      }

      None => Err(UniformWarning::inactive(name)),
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    WebGL2: for<'a> Uniformable<'a, T>,
  {
    let location = self
      .state
      .borrow()
      .ctx
      .get_uniform_block_index(&self.handle, name);

    if location == WebGl2RenderingContext::INVALID_INDEX {
      Err(UniformWarning::inactive(name))
    } else {
      Ok(unsafe { Uniform::new(location as _) })
    }
  }
}

unsafe impl Shader for WebGL2 {
  type StageRepr = Stage;

  type ProgramRepr = Program;

  type UniformBuilderRepr = UniformBuilder;

  unsafe fn new_stage(&mut self, ty: StageType, src: &str) -> Result<Self::StageRepr, StageError> {
    Stage::new(self, ty, src)
  }

  unsafe fn new_program(
    &mut self,
    vertex: &Self::StageRepr,
    tess: Option<TessellationStages<Self::StageRepr>>,
    geometry: Option<&Self::StageRepr>,
    fragment: &Self::StageRepr,
  ) -> Result<Self::ProgramRepr, ProgramError> {
    Program::new(self, vertex, tess, geometry, fragment)
  }

  unsafe fn apply_semantics<Sem>(
    program: &mut Self::ProgramRepr,
  ) -> Result<Vec<VertexAttribWarning>, ProgramError>
  where
    Sem: Semantics,
  {
    let warnings = {
      let state = program.state.borrow();
      bind_vertex_attribs_locations::<Sem>(&state, program)
    };

    // we need to link again to make the location mappings a thing
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
    Self: for<'a> Uniformable<'a, T>,
  {
    let ty = Self::ty();
    let uniform = match ty {
      UniformType::ShaderDataBinding => uniform_builder.ask_uniform_block(name)?,
      _ => uniform_builder.ask_uniform(name, ty, Self::SIZE)?,
    };

    Ok(uniform)
  }

  unsafe fn unbound<T>(_: &mut Self::UniformBuilderRepr) -> Uniform<T>
  where
    Self: for<'a> Uniformable<'a, T>,
  {
    Uniform::new(-1)
  }
}

fn webgl_shader_type(ty: StageType) -> Option<u32> {
  match ty {
    StageType::VertexShader => Some(WebGl2RenderingContext::VERTEX_SHADER),
    StageType::FragmentShader => Some(WebGl2RenderingContext::FRAGMENT_SHADER),
    _ => None,
  }
}

const GLSL_PRAGMA: &str = "#version 300 es\n\
                           precision highp float;\n\
                           precision highp int;\n\
                           layout(std140) uniform;\n\
                           #line 1\n";

fn patch_shader_src(src: &str) -> String {
  let mut pragma = String::from(GLSL_PRAGMA);
  pragma.push_str(src);
  pragma
}

fn uniform_type_match(
  state: &WebGL2State,
  program: &WebGlProgram,
  name: &str,
  location: u32,
  ty: UniformType,
  size: usize,
) -> Result<(), UniformWarning> {
  // uniform blocks are not handled the same way as regular uniforms, so we can already re-use the previously queried
  // location, which is an index for them
  let index = if ty == UniformType::ShaderDataBinding {
    location
  } else {
    // create a 1-item array to hold the name of the uniform we’d like to get information from
    let name_array = js_sys::Array::new();
    name_array.push(&name.into()); // push the name as a JsValue

    // get the index of the uniform; it’s represented as an array of a single element, since our
    // input has only one element
    state
      .ctx
      .get_uniform_indices(program, name_array.as_ref())
      .ok_or_else(|| UniformWarning::TypeMismatch("cannot retrieve uniform index".to_owned(), ty))?
      .get(0)
      .as_f64()
      .map(|x| x as u32)
      .ok_or_else(|| {
        UniformWarning::TypeMismatch("wrong type when retrieving uniform".to_owned(), ty)
      })?
  };

  // get its size and type
  let info = state
    .ctx
    .get_active_uniform(program, index)
    .ok_or_else(|| UniformWarning::TypeMismatch("cannot retrieve active uniform".to_owned(), ty))?;

  let found_size = info.size() as usize;
  if size > 0 && found_size != size {
    return Err(UniformWarning::size_mismatch(name, size, found_size));
  }

  check_types_match(name, ty, info.type_())
}

#[allow(clippy::cognitive_complexity)]
fn check_types_match(name: &str, ty: UniformType, glty: u32) -> Result<(), UniformWarning> {
  // helper macro to check type mismatch for each variant
  macro_rules! milkcheck {
    ($ty:expr, $( ( $v:tt, $t:tt ) ),* $(,)?) => {
      match $ty {
        $(
          UniformType::$v => {
            if glty == WebGl2RenderingContext::$t {
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
    (BVec2, BOOL_VEC2),
    (BVec3, BOOL_VEC3),
    (BVec4, BOOL_VEC4),
    // matrices
    (M22, FLOAT_MAT2),
    (M33, FLOAT_MAT3),
    (M44, FLOAT_MAT4),
    // textures
    (ISampler2D, INT_SAMPLER_2D),
    (ISampler3D, INT_SAMPLER_3D),
    (ISampler2DArray, INT_SAMPLER_2D_ARRAY),
    (UISampler2D, UNSIGNED_INT_SAMPLER_2D),
    (UISampler3D, UNSIGNED_INT_SAMPLER_3D),
    (UISampler2DArray, UNSIGNED_INT_SAMPLER_2D_ARRAY),
    (Sampler2D, SAMPLER_2D),
    (Sampler3D, SAMPLER_3D),
    (Sampler2DArray, SAMPLER_2D_ARRAY),
    (ICubemap, INT_SAMPLER_CUBE),
    (UICubemap, UNSIGNED_INT_SAMPLER_CUBE),
    (Cubemap, SAMPLER_CUBE),
  )
}

fn bind_vertex_attribs_locations<Sem>(
  state: &WebGL2State,
  program: &Program,
) -> Vec<VertexAttribWarning>
where
  Sem: Semantics,
{
  let mut warnings = Vec::new();

  for desc in Sem::semantics_set() {
    match get_vertex_attrib_location(state, program, &desc.name) {
      Ok(_) => {
        let index = desc.index as u32;
        state
          .ctx
          .bind_attrib_location(program.handle(), index, &desc.name);
      }

      Err(warning) => warnings.push(warning),
    }
  }

  warnings
}

fn get_vertex_attrib_location(
  state: &WebGL2State,
  program: &Program,
  name: &str,
) -> Result<i32, VertexAttribWarning> {
  let location = state.ctx.get_attrib_location(program.handle(), name);

  if location < 0 {
    Err(VertexAttribWarning::inactive(name))
  } else {
    Ok(location)
  }
}

// A helper macro used to implement Uniformable for very similar function types. Several forms
// exist: mostly, slice form (like &[_]) will have to compute a length and pass the length down the
// WebGL function along with a flatten version of a slice, while scalar versions will simply
// forward the arguments.
macro_rules! impl_Uniformable {
  (vec arr $q:ident $t:ty, $size:expr, $uty:tt, $f:tt) => {
    unsafe impl<'a, const N: usize> Uniformable<'a, Arr<$q<$t>, N>> for WebGL2 {
      type Target = &'a [$q<$t>; N];

      const SIZE: usize = N;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(
        program: &mut Program,
        uniform: &'a Uniform<Arr<$q<$t>, N>>,
        value: Self::Target,
      ) {
        let len = value.len();
        let data = flatten_slice!(value: $t, len = $size * N);

        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          data,
          0, // offset
          len as _,
        );
      }
    }
  };

  (arr $t:ty , $uty:tt, $f:tt) => {
    unsafe impl<'a, const N: usize> Uniformable<'a, Arr<$t, N>> for WebGL2 {
      type Target = &'a [$t; N];

      const SIZE: usize = N;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(
        program: &mut Program,
        uniform: &'a Uniform<Arr<$t, N>>,
        value: Self::Target,
      ) {
        program
          .state
          .borrow()
          .ctx
          .$f(program.location_map.borrow().get(&uniform.index()), value);
      }
    }
  };

  (vec $t:ty, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $t> for WebGL2 {
      type Target = $t;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(program: &mut Program, uniform: &'a Uniform<$t>, value: Self::Target) {
        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          value.as_ref(),
        );
      }
    }
  };

  ($t:ty, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $t> for WebGL2 {
      type Target = $t;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(program: &mut Program, uniform: &'a Uniform<$t>, value: Self::Target) {
        program
          .state
          .borrow()
          .ctx
          .$f(program.location_map.borrow().get(&uniform.index()), value);
      }
    }
  };

  // matrix notation
  (mat arr $q:ident $t:ty, $size:expr, $uty:tt, $f:tt) => {
    unsafe impl<'a, const N: usize> Uniformable<'a, Arr<$q<$t>, N>> for WebGL2 {
      type Target = &'a [$q<$t>; N];

      const SIZE: usize = N;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(
        program: &mut Program,
        uniform: &'a Uniform<Arr<$q<$t>, N>>,
        value: Self::Target,
      ) {
        let data = flatten_slice!(value: $t, len = $size * N);

        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          false,
          data,
          0,
          value.len() as u32,
        );
      }
    }
  };

  (mat $q:ident $t:ty, $size:expr, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<'a, $q<$t>> for WebGL2 {
      type Target = $q<$t>;

      const SIZE: usize = 1;

      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(program: &mut Program, uniform: &'a Uniform<$q<$t>>, value: Self::Target) {
        let data = flatten_slice!(value: $t, len = $size);

        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          false,
          data,
        );
      }
    }
  };
}

// here we go in deep mud
impl_Uniformable!(i32, Int, uniform1i);
impl_Uniformable!(vec Vec2<i32>, IVec2, uniform2iv_with_i32_array);
impl_Uniformable!(vec Vec3<i32>, IVec3, uniform3iv_with_i32_array);
impl_Uniformable!(vec Vec4<i32>, IVec4, uniform4iv_with_i32_array);
impl_Uniformable!(arr i32, Int, uniform1iv_with_i32_array);
impl_Uniformable!(
  vec arr Vec2 i32,
  2,
  IVec2,
  uniform2iv_with_i32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec3 i32,
  3,
  IVec3,
  uniform3iv_with_i32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec4 i32,
  4,
  IVec4,
  uniform4iv_with_i32_array_and_src_offset_and_src_length
);

impl_Uniformable!(u32, UInt, uniform1ui);
impl_Uniformable!(vec Vec2<u32>, UIVec2, uniform2uiv_with_u32_array);
impl_Uniformable!(vec Vec3<u32>, UIVec3, uniform3uiv_with_u32_array);
impl_Uniformable!(vec Vec4<u32>, UIVec4, uniform4uiv_with_u32_array);
impl_Uniformable!(arr u32, UInt, uniform1uiv_with_u32_array);
impl_Uniformable!(
  vec arr Vec2 u32,
  2,
  UIVec2,
  uniform2uiv_with_u32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec3 u32,
  3,
  UIVec3,
  uniform3uiv_with_u32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec4 u32,
  4,
  UIVec4,
  uniform4uiv_with_u32_array_and_src_offset_and_src_length
);

impl_Uniformable!(f32, Float, uniform1f);
impl_Uniformable!(vec Vec2<f32>, Vec2, uniform2fv_with_f32_array);
impl_Uniformable!(vec Vec3<f32>, Vec3, uniform3fv_with_f32_array);
impl_Uniformable!(vec Vec4<f32>, Vec4, uniform4fv_with_f32_array);
impl_Uniformable!(arr f32, Float, uniform1fv_with_f32_array);
impl_Uniformable!(
  vec arr Vec2 f32,
  2,
  Vec2,
  uniform2fv_with_f32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec3 f32,
  3,
  Vec3,
  uniform3fv_with_f32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  vec arr Vec4 f32,
  4,
  Vec4,
  uniform4fv_with_f32_array_and_src_offset_and_src_length
);

// please don’t judge me
impl_Uniformable!(mat Mat22 f32, 4, M22, uniform_matrix2fv_with_f32_array);
impl_Uniformable!(mat arr Mat22 f32, 4, M22, uniform_matrix2fv_with_f32_array_and_src_offset_and_src_length);

impl_Uniformable!(mat Mat33 f32, 9, M33, uniform_matrix3fv_with_f32_array);
impl_Uniformable!(mat arr Mat33 f32, 9, M33, uniform_matrix3fv_with_f32_array_and_src_offset_and_src_length);

impl_Uniformable!(mat Mat44 f32, 16, M44, uniform_matrix4fv_with_f32_array);
impl_Uniformable!(mat arr Mat44 f32, 16, M44, uniform_matrix4fv_with_f32_array_and_src_offset_and_src_length);

// Special exception for booleans: because we cannot simply send the bool Rust type down to the
// GPU, we have to convert them to 32-bit integer (unsigned), which is a total fuck up and waste of
// memory bandwidth, but well, WebGL / OpenGL, whatcha wanna do. Also, for slice versions… we have
// to allocate short-living, temporary vectors to hold the converted data.
//
// All this makes me so sad that I want a corgi by my side. Please. And chimken nuggets.
unsafe impl<'a> Uniformable<'a, bool> for WebGL2 {
  type Target = bool;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(program: &mut Program, uniform: &'a Uniform<bool>, value: Self::Target) {
    program.state.borrow().ctx.uniform1ui(
      program.location_map.borrow().get(&uniform.index()),
      value as u32,
    );
  }
}

unsafe impl<'a> Uniformable<'a, Vec2<bool>> for WebGL2 {
  type Target = Vec2<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(program: &mut Program, uniform: &'a Uniform<Vec2<bool>>, value: Self::Target) {
    let v = [value[0] as u32, value[1] as u32];

    program
      .state
      .borrow()
      .ctx
      .uniform2uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl<'a> Uniformable<'a, Vec3<bool>> for WebGL2 {
  type Target = Vec3<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(program: &mut Program, uniform: &'a Uniform<Vec3<bool>>, value: Self::Target) {
    let v = [value[0] as u32, value[1] as u32, value[2] as u32];

    program
      .state
      .borrow()
      .ctx
      .uniform3uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl<'a> Uniformable<'a, Vec4<bool>> for WebGL2 {
  type Target = Vec4<bool>;

  const SIZE: usize = 1;

  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(program: &mut Program, uniform: &'a Uniform<Vec4<bool>>, value: Self::Target) {
    let v = [
      value[0] as u32,
      value[1] as u32,
      value[2] as u32,
      value[3] as u32,
    ];

    program
      .state
      .borrow()
      .ctx
      .uniform4uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

// bool cache for optimizing [bool; N] -> [u32; N]
static mut BOOL_CACHE: Vec<u32> = Vec::new();

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<bool, N>> for WebGL2 {
  type Target = &'a [bool; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(program: &mut Program, uniform: &'a Uniform<Arr<bool, N>>, value: Self::Target) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(value.iter().map(|x| *x as u32));

    program.state.borrow().ctx.uniform1uiv_with_u32_array(
      program.location_map.borrow().get(&uniform.index()),
      &BOOL_CACHE,
    );
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec2<bool>, N>> for WebGL2 {
  type Target = &'a [Vec2<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(
    program: &mut Program,
    uniform: &'a Uniform<Arr<Vec2<bool>, N>>,
    value: Self::Target,
  ) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(value.iter().flat_map(|x| [x[0] as u32, x[1] as u32]));

    program.state.borrow().ctx.uniform2uiv_with_u32_array(
      program.location_map.borrow().get(&uniform.index()),
      &BOOL_CACHE,
    );
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec3<bool>, N>> for WebGL2 {
  type Target = &'a [Vec3<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(
    program: &mut Program,
    uniform: &'a Uniform<Arr<Vec3<bool>, N>>,
    value: Self::Target,
  ) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(
      value
        .iter()
        .flat_map(|x| [x[0] as u32, x[1] as u32, x[2] as u32]),
    );

    program.state.borrow().ctx.uniform3uiv_with_u32_array(
      program.location_map.borrow().get(&uniform.index()),
      &BOOL_CACHE,
    );
  }
}

unsafe impl<'a, const N: usize> Uniformable<'a, Arr<Vec4<bool>, N>> for WebGL2 {
  type Target = &'a [Vec4<bool>; N];

  const SIZE: usize = N;

  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(
    program: &mut Program,
    uniform: &'a Uniform<Arr<Vec4<bool>, N>>,
    value: Self::Target,
  ) {
    BOOL_CACHE.clear();
    BOOL_CACHE.extend(
      value
        .iter()
        .flat_map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32]),
    );

    program.state.borrow().ctx.uniform4uiv_with_u32_array(
      program.location_map.borrow().get(&uniform.index()),
      &BOOL_CACHE,
    );
  }
}

unsafe impl<'a, T> Uniformable<'a, ShaderDataBinding<T>> for WebGL2
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
    program.state.borrow().ctx.uniform_block_binding(
      &program.handle,
      uniform.index() as u32,
      value.binding(),
    );
  }
}

unsafe impl<'a, D, S> Uniformable<'a, TextureBinding<D, S>> for WebGL2
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
    program: &mut Program,
    uniform: &'a Uniform<TextureBinding<D, S>>,
    value: Self::Target,
  ) {
    program.state.borrow().ctx.uniform1i(
      program.location_map.borrow().get(&uniform.index()),
      value.binding() as i32,
    );
  }
}

unsafe impl<T> ShaderData<T> for WebGL2
where
  T: Std140,
{
  type ShaderDataRepr =
    Buffer<<ArrElem<T> as Std140>::Encoded, { WebGl2RenderingContext::UNIFORM_BUFFER }>;

  unsafe fn new_shader_data(
    &mut self,
    values: impl Iterator<Item = T>,
  ) -> Result<Self::ShaderDataRepr, ShaderDataError> {
    Buffer::from_vec(
      self,
      values
        .into_iter()
        .map(|x| ArrElem(x).std140_encode())
        .collect(),
    )
    .map_err(|BufferError::CannotCreate| ShaderDataError::CannotCreate)
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
      &mut shader_data.slice_buffer_mut()[i],
      ArrElem(x).std140_encode(),
    );

    Ok(<ArrElem<T> as Std140>::std140_decode(prev).0)
  }

  unsafe fn set_shader_data_values(
    shader_data: &mut Self::ShaderDataRepr,
    values: impl Iterator<Item = T>,
  ) -> Result<(), ShaderDataError> {
    let mut slice = shader_data.slice_buffer_mut();

    for (item, value) in slice.iter_mut().zip(values) {
      *item = ArrElem(value).std140_encode();
    }

    Ok(())
  }
}
