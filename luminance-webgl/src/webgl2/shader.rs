//! Shader support for WebGL2.

use luminance::backend::shader::{Shader, Uniformable};
use luminance::pipeline::{BufferBinding, TextureBinding};
use luminance::pixel::{SamplerType, Type as PixelType};
use luminance::shader::{
  ProgramError, StageError, StageType, TessellationStages, Uniform, UniformType, UniformWarning,
  VertexAttribWarning,
};
use luminance::texture::{Dim, Dimensionable};
use luminance::vertex::Semantics;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::webgl2::state::WebGL2State;
use crate::webgl2::WebGL2;

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
        .unwrap_or_else(|| "unknown link error".to_owned());
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

  fn ask_uniform<T>(&mut self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    T: Uniformable<WebGL2>,
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

        Ok(unsafe { Uniform::new(idx) })
      }

      None => Err(UniformWarning::inactive(name)),
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> Result<Uniform<T>, UniformWarning>
  where
    T: Uniformable<WebGL2>,
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
    T: Uniformable<Self>,
  {
    let ty = T::ty();
    let uniform = match ty {
      UniformType::BufferBinding => uniform_builder.ask_uniform_block(name)?,
      _ => uniform_builder.ask_uniform(name)?,
    };

    let state = uniform_builder.state.borrow();
    uniform_type_match(&state, &uniform_builder.handle, name, ty)?;

    Ok(uniform)
  }

  unsafe fn unbound<T>(_: &mut Self::UniformBuilderRepr) -> Uniform<T>
  where
    T: Uniformable<Self>,
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
                           precision highp int;\n";

fn patch_shader_src(src: &str) -> String {
  let mut pragma = String::from(GLSL_PRAGMA);
  pragma.push_str(src);
  pragma
}

fn uniform_type_match(
  state: &WebGL2State,
  program: &WebGlProgram,
  name: &str,
  ty: UniformType,
) -> Result<(), UniformWarning> {
  // create a 1-item array to hold the name of the uniform we’d like to get information from
  let name_array = js_sys::Array::new();
  name_array.push(&name.into()); // push the name as a JsValue

  // get the index of the uniform; it’s represented as an array of a single element, since our
  // input has only one element
  let index = state
    .ctx
    .get_uniform_indices(program, name_array.as_ref())
    .ok_or_else(|| UniformWarning::TypeMismatch("cannot retrieve uniform index".to_owned(), ty))?
    .get(0)
    .as_f64()
    .map(|x| x as u32)
    .ok_or_else(|| {
      UniformWarning::TypeMismatch("wrong type when retrieving uniform".to_owned(), ty)
    })?;

  // get its size and type
  let info = state
    .ctx
    .get_active_uniform(program, index)
    .ok_or_else(|| UniformWarning::TypeMismatch("cannot retrieve active uniform".to_owned(), ty))?;

  // early-return if array – we don’t support them yet
  if info.size() != 1 {
    return Ok(());
  }

  check_types_match(name, ty, info.type_())
}

#[allow(clippy::cognitive_complexity)]
fn check_types_match(name: &str, ty: UniformType, glty: u32) -> Result<(), UniformWarning> {
  // helper macro to check type mismatch for each variant
  macro_rules! milkcheck {
    ($ty:expr, $( ( $v:tt, $t:tt ) ),*) => {
      match $ty {
        $( UniformType::$v if glty != WebGl2RenderingContext::$t => Err(UniformWarning::type_mismatch(name, ty)), )*
        _ => Ok(())
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
    (Cubemap, SAMPLER_CUBE)
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
//
// The matrix form is fucking ugly: the mat T ; N form is simply akin to [[T; N]; N] while the
// mat & T ; N is akin to &[[[T; N]; N]] (i.e. a slice of matrices).
//
// Also, ideally, the ty() function should live in the luminance crate, because other backends will
// have to copy the same implementation over and over.
//
// I’m so sorry.
macro_rules! impl_Uniformable {
  (&[[$t:ty; $dim:expr]], $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<WebGL2> for &'a [[$t; $dim]] {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        let len = self.len();
        let data = flatten_slice!(self: [$t; $dim]);

        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          data,
          0, // offset
          len as _,
        );
      }
    }
  };

  (&[$t:ty], $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<WebGL2> for &'a [$t] {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        program
          .state
          .borrow()
          .ctx
          .$f(program.location_map.borrow().get(&uniform.index()), self);
      }
    }
  };

  ([$t:ty; $dim:expr], $uty:tt, $f:tt) => {
    unsafe impl Uniformable<WebGL2> for [$t; $dim] {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        program
          .state
          .borrow()
          .ctx
          .$f(program.location_map.borrow().get(&uniform.index()), &self);
      }
    }
  };

  ($t:ty, $uty:tt, $f:tt) => {
    unsafe impl Uniformable<WebGL2> for $t {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        program
          .state
          .borrow()
          .ctx
          .$f(program.location_map.borrow().get(&uniform.index()), self);
      }
    }
  };

  // matrix notation
  (mat & $t:ty ; $dim:expr, $uty:tt, $f:tt) => {
    unsafe impl<'a> Uniformable<WebGL2> for &'a &[[[$t; $dim]; $dim]] {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        let data = flatten_slice!(self: [$t; $dim * $dim]);

        program.state.borrow().ctx.$f(
          program.location_map.borrow().get(&uniform.index()),
          false,
          data,
        );
      }
    }
  };

  (mat $t:ty ; $dim:expr, $uty:tt, $f:tt) => {
    unsafe impl Uniformable<WebGL2> for [[$t; $dim]; $dim] {
      unsafe fn ty() -> UniformType {
        UniformType::$uty
      }

      unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
        let data = flatten_slice!(self: [$t; $dim * $dim]);

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
impl_Uniformable!([i32; 2], IVec2, uniform2iv_with_i32_array);
impl_Uniformable!([i32; 3], IVec3, uniform3iv_with_i32_array);
impl_Uniformable!([i32; 4], IVec4, uniform4iv_with_i32_array);
impl_Uniformable!(&[i32], Int, uniform1iv_with_i32_array);
impl_Uniformable!(
  &[[i32; 2]],
  IVec2,
  uniform2iv_with_i32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[i32; 3]],
  IVec3,
  uniform3iv_with_i32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[i32; 4]],
  IVec4,
  uniform4iv_with_i32_array_and_src_offset_and_src_length
);

impl_Uniformable!(u32, UInt, uniform1ui);
impl_Uniformable!([u32; 2], UIVec2, uniform2uiv_with_u32_array);
impl_Uniformable!([u32; 3], UIVec3, uniform3uiv_with_u32_array);
impl_Uniformable!([u32; 4], UIVec4, uniform4uiv_with_u32_array);
impl_Uniformable!(&[u32], UInt, uniform1uiv_with_u32_array);
impl_Uniformable!(
  &[[u32; 2]],
  UIVec2,
  uniform2uiv_with_u32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[u32; 3]],
  UIVec3,
  uniform3uiv_with_u32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[u32; 4]],
  UIVec4,
  uniform4uiv_with_u32_array_and_src_offset_and_src_length
);

impl_Uniformable!(f32, Float, uniform1f);
impl_Uniformable!([f32; 2], Vec2, uniform2fv_with_f32_array);
impl_Uniformable!([f32; 3], Vec3, uniform3fv_with_f32_array);
impl_Uniformable!([f32; 4], Vec4, uniform4fv_with_f32_array);
impl_Uniformable!(&[f32], Float, uniform1fv_with_f32_array);
impl_Uniformable!(
  &[[f32; 2]],
  Vec2,
  uniform2fv_with_f32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[f32; 3]],
  Vec3,
  uniform3fv_with_f32_array_and_src_offset_and_src_length
);
impl_Uniformable!(
  &[[f32; 4]],
  Vec4,
  uniform4fv_with_f32_array_and_src_offset_and_src_length
);

// please don’t judge me
impl_Uniformable!(mat f32; 2, M22, uniform_matrix2fv_with_f32_array);
impl_Uniformable!(mat & f32; 2, M22, uniform_matrix2fv_with_f32_array);

impl_Uniformable!(mat f32; 3, M33, uniform_matrix3fv_with_f32_array);
impl_Uniformable!(mat & f32; 3, M33, uniform_matrix3fv_with_f32_array);

impl_Uniformable!(mat f32; 4, M44, uniform_matrix4fv_with_f32_array);
impl_Uniformable!(mat & f32; 4, M44, uniform_matrix4fv_with_f32_array);

// Special exception for booleans: because we cannot simply send the bool Rust type down to the
// GPU, we have to convert them to 32-bit integer (unsigned), which is a total fuck up and waste of
// memory bandwidth, but well, WebGL / OpenGL, whatcha wanna do. Also, for slice versions… we have
// to allocate short-living, temporary vectors to hold the converted data.
//
// All this makes me so sad that I want a corgi by my side. Please. And chimken nuggets.
unsafe impl Uniformable<WebGL2> for bool {
  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    program.state.borrow().ctx.uniform1ui(
      program.location_map.borrow().get(&uniform.index()),
      self as u32,
    );
  }
}

unsafe impl Uniformable<WebGL2> for [bool; 2] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32];

    program
      .state
      .borrow()
      .ctx
      .uniform2uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl Uniformable<WebGL2> for [bool; 3] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32];

    program
      .state
      .borrow()
      .ctx
      .uniform3uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl Uniformable<WebGL2> for [bool; 4] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v = [
      self[0] as u32,
      self[1] as u32,
      self[2] as u32,
      self[3] as u32,
    ];

    program
      .state
      .borrow()
      .ctx
      .uniform4uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl<'a> Uniformable<WebGL2> for &'a [bool] {
  unsafe fn ty() -> UniformType {
    UniformType::Bool
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| *x as u32).collect();

    program
      .state
      .borrow()
      .ctx
      .uniform1uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), &v);
  }
}

unsafe impl<'a> Uniformable<WebGL2> for &'a [[bool; 2]] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec2
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    let data = flatten_slice!(v: [u32; 2]);

    program
      .state
      .borrow()
      .ctx
      .uniform2uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), data);
  }
}

unsafe impl<'a> Uniformable<WebGL2> for &'a [[bool; 3]] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec3
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v: Vec<_> = self
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32])
      .collect();
    let data = flatten_slice!(v: [u32; 3]);

    program
      .state
      .borrow()
      .ctx
      .uniform3uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), data);
  }
}

unsafe impl<'a> Uniformable<WebGL2> for &'a [[bool; 4]] {
  unsafe fn ty() -> UniformType {
    UniformType::BVec4
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    let v: Vec<_> = self
      .iter()
      .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32])
      .collect();
    let data = flatten_slice!(v: [u32; 4]);

    program
      .state
      .borrow()
      .ctx
      .uniform4uiv_with_u32_array(program.location_map.borrow().get(&uniform.index()), data);
  }
}

unsafe impl<T> Uniformable<WebGL2> for BufferBinding<T> {
  unsafe fn ty() -> UniformType {
    UniformType::BufferBinding
  }

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    program.state.borrow().ctx.uniform_block_binding(
      &program.handle,
      uniform.index() as u32,
      self.binding(),
    );
  }
}

unsafe impl<D, S> Uniformable<WebGL2> for TextureBinding<D, S>
where
  D: Dimensionable,
  S: SamplerType,
{
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

  unsafe fn update(self, program: &mut Program, uniform: &Uniform<Self>) {
    program.state.borrow().ctx.uniform1i(
      program.location_map.borrow().get(&uniform.index()),
      self.binding() as i32,
    );
  }
}
