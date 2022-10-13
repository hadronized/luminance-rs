use crate::{
  dim::Dimensionable,
  framebuffer::{Back, Framebuffer},
  pipeline::{PipelineState, WithFramebuffer, WithProgram, WithRenderState},
  pixel::{Pixel, PixelFormat},
  primitive::Primitive,
  render_channel::{DepthChannel, RenderChannel},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  render_state::RenderState,
  shader::{Program, Uni, Uniform, Uniforms},
  texture::{Sampler, Texture},
  vertex::Vertex,
  vertex_entity::{VertexEntity, VertexEntityView},
  vertex_storage::AsVertexStorage,
};
use std::{error::Error as ErrorTrait, fmt};

#[derive(Debug)]
#[non_exhaustive]
pub enum VertexEntityError {
  Creation { cause: Option<Box<dyn ErrorTrait>> },
  Render { cause: Option<Box<dyn ErrorTrait>> },
  UpdateVertexStorage { cause: Option<Box<dyn ErrorTrait>> },
  UpdateIndices { cause: Option<Box<dyn ErrorTrait>> },
}

impl fmt::Display for VertexEntityError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VertexEntityError::Creation { cause } => write!(
        f,
        "cannot create vertex entity: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      VertexEntityError::Render { cause } => write!(
        f,
        "cannot render vertex entity: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),
      VertexEntityError::UpdateVertexStorage { cause } => {
        write!(
          f,
          "cannot update vertex storage: {}",
          cause
            .as_ref()
            .map(|cause| cause.to_string())
            .unwrap_or_else(|| "unknown cause".to_string())
        )
      }

      VertexEntityError::UpdateIndices { cause } => write!(
        f,
        "cannot update indices: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),
    }
  }
}

impl ErrorTrait for VertexEntityError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum FramebufferError {
  Creation { cause: Option<Box<dyn ErrorTrait>> },
  RenderLayerCreation { cause: Option<Box<dyn ErrorTrait>> },
  DepthRenderLayerCreation { cause: Option<Box<dyn ErrorTrait>> },
  RetrieveBackBuffer { cause: Option<Box<dyn ErrorTrait>> },
}

impl fmt::Display for FramebufferError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      FramebufferError::Creation { cause } => write!(
        f,
        "cannot create framebuffer: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      FramebufferError::RenderLayerCreation { cause } => write!(
        f,
        "cannot create render layer: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      FramebufferError::DepthRenderLayerCreation { cause } => write!(
        f,
        "cannot create depth render layer: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      FramebufferError::RetrieveBackBuffer { cause } => write!(
        f,
        "cannot retrieve back buffer {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),
    }
  }
}

#[derive(Debug)]
pub enum ShaderError {
  Creation {
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniCreation {
    name: String,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniSet {
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniBufferCreation {
    name: String,
    cause: Option<Box<dyn ErrorTrait>>,
  },
}

impl fmt::Display for ShaderError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ShaderError::Creation { cause } => writeln!(
        f,
        "cannot create shader program: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      ShaderError::UniCreation { name, cause } => write!(
        f,
        "cannot create uniform variable (\"{}\"): {}",
        name,
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      ShaderError::UniSet { cause } => write!(
        f,
        "cannot set uniform variable: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      ShaderError::UniBufferCreation { name, cause } => write!(
        f,
        "cannot create uniform buffer (\"{}\"): {}",
        name,
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),
    }
  }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PipelineError {
  WithFramebuffer {
    pipeline_state: PipelineState,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  WithProgram {
    cause: Option<Box<dyn ErrorTrait>>,
  },

  WithRenderState {
    render_state: RenderState,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  RenderVertexEntity {
    start_vertex: usize,
    vertex_count: usize,
    instance_count: usize,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  ShaderError(ShaderError),
}

impl From<ShaderError> for PipelineError {
  fn from(e: ShaderError) -> Self {
    PipelineError::ShaderError(e)
  }
}

impl fmt::Display for PipelineError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PipelineError::WithFramebuffer {
        pipeline_state,
        cause,
      } => write!(
         f,
        "error in framebuffer pipeline: {}; state:\n{:#?}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string()),
        pipeline_state
      ),

      PipelineError::WithProgram { cause } => write!(
        f,
        "error in shader program pipeline: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      PipelineError::WithRenderState {
        render_state,
        cause,
      } => write!(
        f,
        "error in render state pipeline: {}; state:\n{:#?}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string()),
        render_state
      ),

      PipelineError::RenderVertexEntity {
        start_vertex,
        vertex_count,
        instance_count,
        cause,
      } => write!(
        f,
        "error in render vertex entity pipeline: {}; start_vertex={}, vertex_count={}, instance_count={}",
        cause.as_ref().map(|cause| cause.to_string()).unwrap_or_else(|| "unknown cause".to_string()),
        start_vertex, vertex_count, instance_count,
      ),

      PipelineError::ShaderError(e) => write!(f, "shader error in pipeline: {}", e),
    }
  }
}

/// Errors that might happen when working with textures.
#[non_exhaustive]
#[derive(Debug)]
pub enum TextureError {
  /// The texture handle has no data associated with.
  NoData { handle: usize },

  /// Not enough texture units.
  NotEnoughTextureUnits { max: usize },

  /// A texture’s storage failed to be created.
  ///
  /// The carried [`String`] gives the reason of the failure.
  TextureStorageCreationFailed { cause: Option<Box<dyn ErrorTrait>> },

  /// Not enough pixel data provided for the given area asked.
  ///
  /// You must provide at least as many pixels as expected by the area in the texture you’re
  /// uploading to.
  NotEnoughPixels {
    /// Expected number of pixels in bytes.
    expected_bytes: usize,

    /// Provided number of pixels in bytes.
    provided_bytes: usize,

    cause: Option<Box<dyn ErrorTrait>>,
  },

  /// Unsupported pixel format.
  ///
  /// Sometimes, some hardware might not support a given pixel format (or the format exists on
  /// the interface side but doesn’t in the implementation). That error represents such a case.
  UnsupportedPixelFormat(PixelFormat),

  /// Cannot retrieve texels from a texture.
  ///
  /// That error might happen on some hardware implementations if the user tries to retrieve
  /// texels from a texture that doesn’t support getting its texels retrieved.
  CannotRetrieveTexels { cause: Option<Box<dyn ErrorTrait>> },

  /// Failed to upload texels.
  CannotUploadTexels { cause: Option<Box<dyn ErrorTrait>> },
}

impl fmt::Display for TextureError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      TextureError::NoData { handle } => {
        write!(f, "texture {} has no data associated with", handle)
      }

      TextureError::NotEnoughTextureUnits { max } => {
        write!(f, "not enough texture units (max = {})", max)
      }

      TextureError::TextureStorageCreationFailed { cause } => {
        write!(
          f,
          "texture storage creation failed; cause: {}",
          cause
            .as_ref()
            .map(|cause| cause.to_string())
            .unwrap_or("unknown cause".to_string())
        )
      }

      TextureError::NotEnoughPixels {
        expected_bytes,
        provided_bytes,
        cause,
      } => write!(
        f,
        "not enough texels provided: expected {} bytes, provided {} bytes; cause: {}",
        expected_bytes,
        provided_bytes,
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown".to_owned())
      ),

      TextureError::UnsupportedPixelFormat(fmt) => {
        write!(f, "unsupported pixel format: {:?}", fmt)
      }

      TextureError::CannotRetrieveTexels { cause } => {
        write!(
          f,
          "cannot retrieve texture’s texels; cause: {}",
          cause
            .as_ref()
            .map(|cause| cause.to_string())
            .unwrap_or_else(|| "unknown".to_owned())
        )
      }

      TextureError::CannotUploadTexels { cause } => {
        write!(
          f,
          "cannot upload texels to texture; cause: {}",
          cause
            .as_ref()
            .map(|cause| cause.to_string())
            .unwrap_or_else(|| "unknown".to_owned())
        )
      }
    }
  }
}

impl std::error::Error for TextureError {}

/// Query error.
#[derive(Debug)]
pub enum QueryError {
  NoBackendAuthor,
  NoBackendName,
  NoBackendVersion,
  NoBackendShadingLanguageVersion,
  NoMaxTextureArrayElements,
}

impl fmt::Display for QueryError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      QueryError::NoBackendAuthor => f.write_str("no backend author available"),
      QueryError::NoBackendName => f.write_str("no backend name available"),
      QueryError::NoBackendVersion => f.write_str("no backend version available"),
      QueryError::NoBackendShadingLanguageVersion => {
        f.write_str("no backend shading language version available")
      }
      QueryError::NoMaxTextureArrayElements => {
        f.write_str("no maximum number of elements for texture arrays available")
      }
    }
  }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
  VertexEntity(VertexEntityError),
  Framebuffer(FramebufferError),
  Shader(ShaderError),
  Texture(TextureError),
  Pipeline(PipelineError),
  Query(QueryError),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::VertexEntity(e) => write!(f, "vertex entity error: {}", e),
      Error::Framebuffer(e) => write!(f, "framebuffer error: {}", e),
      Error::Shader(e) => write!(f, "shader error: {}", e),
      Error::Pipeline(e) => write!(f, "pipeline error: {}", e),
      Error::Texture(e) => write!(f, "texture error: {}", e),
      Error::Query(e) => write!(f, "query error: {}", e),
    }
  }
}

impl From<VertexEntityError> for Error {
  fn from(e: VertexEntityError) -> Self {
    Error::VertexEntity(e)
  }
}

impl From<FramebufferError> for Error {
  fn from(e: FramebufferError) -> Self {
    Error::Framebuffer(e)
  }
}

impl From<ShaderError> for Error {
  fn from(e: ShaderError) -> Self {
    Error::Shader(e)
  }
}

impl From<TextureError> for Error {
  fn from(e: TextureError) -> Self {
    Error::Texture(e)
  }
}

impl From<PipelineError> for Error {
  fn from(e: PipelineError) -> Self {
    Error::Pipeline(e)
  }
}

pub trait Backend:
  VertexEntityBackend
  + FramebufferBackend
  + ShaderBackend
  + TextureBackend
  + PipelineBackend
  + QueryBackend
{
}

impl<B> Backend for B where
  B: VertexEntityBackend
    + FramebufferBackend
    + ShaderBackend
    + TextureBackend
    + PipelineBackend
    + QueryBackend
{
}

pub unsafe trait VertexEntityBackend {
  unsafe fn new_vertex_entity<V, P, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, P, S>, VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
    I: Into<Vec<u32>>;

  unsafe fn vertex_entity_render<V, P>(
    &self,
    handle: usize,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
    primitive_restart: bool,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    P: Primitive;

  unsafe fn vertex_entity_update_vertices<V, S>(
    &mut self,
    handle: usize,
    storage: &mut S,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: AsVertexStorage<V>;

  unsafe fn vertex_entity_update_indices(
    &mut self,
    handle: usize,
    indices: &mut Vec<u32>,
  ) -> Result<(), VertexEntityError>;
}

pub unsafe trait FramebufferBackend {
  unsafe fn new_render_layer<D, RC>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: usize,
    index: usize,
  ) -> Result<RenderLayer<RC>, FramebufferError>
  where
    D: Dimensionable,
    RC: RenderChannel;

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: usize,
  ) -> Result<RenderLayer<DC>, FramebufferError>
  where
    D: Dimensionable,
    DC: DepthChannel;

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: usize,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, Back<RS>, Back<DS>>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;
}

macro_rules! mk_uniform_visit {
  ($( $name:ident, $t:ty, )*) => {
    $( fn $name(&mut self, uni: &Uni<$t>, value: &$t) -> Result<(), ShaderError>; )*
  };

  ($( array $name:ident, $t:ty, )*) => {
    $( fn $name<const N: usize>(&mut self, uni: &Uni<[$t; N]>, value: &[$t; N]) -> Result<(), ShaderError>; )*
  };
}

pub unsafe trait ShaderBackend {
  unsafe fn new_program<V, P, S, E>(
    &mut self,
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
  ) -> Result<Program<V, P, S, E>, ShaderError>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: Uniforms;

  unsafe fn new_shader_uni<T>(&mut self, handle: usize, name: &str) -> Result<Uni<T>, ShaderError>
  where
    T: Uniform;

  unsafe fn new_shader_uni_unbound<T>(&mut self, handle: usize) -> Result<Uni<T>, ShaderError>
  where
    T: Uniform;

  unsafe fn set_shader_uni<T>(
    &mut self,
    handle: usize,
    uni: &Uni<T::UniType>,
    value: T,
  ) -> Result<(), ShaderError>
  where
    T: Uniform;

  mk_uniform_visit! {
    visit_i32, i32,
    visit_u32, u32,
    visit_f32, f32,
    visit_bool, bool,
  }

  mk_uniform_visit! {
    array visit_i32_array, i32,
    array visit_u32_array, u32,
    array visit_f32_array, f32,
    array visit_bool_array, bool,
  }

  mk_uniform_visit! {
    visit_ivec2, [i32; 2],
    visit_uvec2, [u32; 2],
    visit_vec2, [f32; 2],
    visit_bvec2, [bool; 2],

    visit_ivec3, [i32; 3],
    visit_uvec3, [u32; 3],
    visit_vec3, [f32; 3],
    visit_bvec3, [bool; 3],

    visit_ivec4, [i32; 4],
    visit_uvec4, [u32; 4],
    visit_vec4, [f32; 4],
    visit_bvec4, [bool; 4],

    visit_mat22, [[f32; 2]; 2],
    visit_mat33, [[f32; 3]; 3],
    visit_mat44, [[f32; 4]; 4],
  }
}

pub unsafe trait TextureBackend {
  unsafe fn new_texture<D, P>(
    &mut self,
    size: D::Size,
    sampler: Sampler,
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn resize_texture<D>(&mut self, handle: usize, size: D::Size) -> Result<(), TextureError>
  where
    D: Dimensionable;

  unsafe fn set_texture_data<D, P>(
    &mut self,
    handle: usize,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn clear_texture_data<P>(
    &mut self,
    handle: usize,
    clear_value: P::RawEncoding,
  ) -> Result<(), TextureError>
  where
    P: Pixel;

  unsafe fn get_texels<P>(&mut self, handle: usize) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    P: Pixel;
}

pub unsafe trait PipelineBackend:
  FramebufferBackend + ShaderBackend + VertexEntityBackend
{
  unsafe fn with_framebuffer<D, CS, DS, Err>(
    &mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    pipeline_state: &PipelineState,
    f: impl for<'a> FnOnce(WithFramebuffer<'a, Self, CS>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<PipelineError>;

  unsafe fn with_program<V, P, S, E, Err>(
    &mut self,
    program: &Program<V, P, S, E>,
    f: impl for<'a> FnOnce(WithProgram<'a, Self, V, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: Uniforms,
    Err: From<PipelineError>;

  unsafe fn with_render_state<V, P, Err>(
    &mut self,
    render_state: &RenderState,
    f: impl for<'a> FnOnce(WithRenderState<'a, Self, V, P>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    V: Vertex,
    P: Primitive,
    Err: From<PipelineError>;

  unsafe fn render_vertex_entity<V, P>(
    &mut self,
    view: VertexEntityView<V, P>,
  ) -> Result<(), PipelineError>
  where
    V: Vertex,
    P: Primitive;
}

pub unsafe trait QueryBackend {
  fn backend_author(&self) -> Result<String, QueryError>;

  fn backend_name(&self) -> Result<String, QueryError>;

  fn backend_version(&self) -> Result<String, QueryError>;

  fn backend_shading_lang_version(&self) -> Result<String, QueryError>;
}
