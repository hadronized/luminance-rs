use crate::{
  dim::Dimensionable,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, WithFramebuffer, WithProgram, WithRenderState},
  primitive::Primitive,
  render_channel::{IsDepthChannelType, IsRenderChannelType},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  render_state::RenderState,
  shader::{FromEnv, IsUniBuffer, Program, Uni, UniBuffer, Uniform},
  vertex::Vertex,
  vertex_entity::{Indices, VertexEntity, VertexEntityView, Vertices},
  vertex_storage::VertexStorage,
};
use std::{error::Error as ErrorTrait, fmt};

pub mod state;

#[derive(Debug)]
pub enum VertexEntityError {
  Creation { cause: Option<Box<dyn ErrorTrait>> },
  Render { cause: Option<Box<dyn ErrorTrait>> },
  RetrieveVertexStorage { cause: Option<Box<dyn ErrorTrait>> },
  UpdateVertexStorage { cause: Option<Box<dyn ErrorTrait>> },
  RetrieveIndices { cause: Option<Box<dyn ErrorTrait>> },
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
      VertexEntityError::RetrieveVertexStorage { cause } => write!(
        f,
        "cannot retrieve vertex storage: {}",
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

      VertexEntityError::RetrieveIndices { cause } => write!(
        f,
        "cannot retrieve indices: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

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

#[derive(Debug)]
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
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
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
      ShaderError::Creation {
        vertex_code,
        primitive_code,
        shading_code,
        cause,
      } => {
        writeln!(
          f,
          "cannot create shader program: {}",
          cause
            .as_ref()
            .map(|cause| cause.to_string())
            .unwrap_or_else(|| "unknown cause".to_string())
        )?;
        writeln!(f, "vertex stage:\n{}", vertex_code)?;
        writeln!(f, "primitive stage:\n{}", primitive_code)?;
        writeln!(f, "shading stage:\n{}", shading_code)
      }

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
    )
    }
  }
}

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
  Pipeline(PipelineError),
  Query(QueryError),
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

impl From<PipelineError> for Error {
  fn from(e: PipelineError) -> Self {
    Error::Pipeline(e)
  }
}

pub trait Backend:
  VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend + QueryBackend
{
}

impl<B> Backend for B where
  B: VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend + QueryBackend
{
}

pub unsafe trait VertexEntityBackend {
  unsafe fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, S>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>,
    I: Into<Vec<u32>>;

  unsafe fn vertex_entity_render<V, S>(
    &self,
    entity: &VertexEntity<V, S>,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Vertices<'a, V, S>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    vertices: Vertices<'a, V, S>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Indices<'a>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    indices: Indices<'a>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;
}

pub unsafe trait FramebufferBackend {
  unsafe fn new_render_layer<D, RC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<RC>, FramebufferError>
  where
    D: Dimensionable,
    RC: IsRenderChannelType;

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<DC>, FramebufferError>
  where
    D: Dimensionable,
    DC: IsDepthChannelType;

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;
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
    E: FromEnv;

  unsafe fn new_shader_uni<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<Uni<T>, ShaderError>
  where
    T: Uniform;

  unsafe fn set_program_env<T>(
    &mut self,
    program_handle: usize,
    env_handle: usize,
    value: T,
  ) -> Result<(), ShaderError>
  where
    T: Uniform;

  unsafe fn new_shader_uni_buffer<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<UniBuffer<T>, ShaderError>
  where
    T: IsUniBuffer;
}

pub unsafe trait PipelineBackend {
  unsafe fn with_framebuffer<'a, D, CS, DS, Err>(
    &mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    state: &PipelineState,
    f: impl FnOnce(WithFramebuffer<'a, Self, CS>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<PipelineError>;

  unsafe fn with_program<'a, V, P, S, E, Err>(
    &mut self,
    program: &Program<V, P, S, E>,
    f: impl FnOnce(WithProgram<'a, Self, V, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: FromEnv,
    Err: From<PipelineError>;

  unsafe fn with_render_state<'a, V, Err>(
    &mut self,
    render_state: &RenderState,
    f: impl FnOnce(WithRenderState<'a, Self, V>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    Err: From<PipelineError>;

  unsafe fn render_vertex_entity<V>(
    &mut self,
    view: VertexEntityView<V>,
  ) -> Result<(), PipelineError>
  where
    V: Vertex;
}

pub unsafe trait QueryBackend {
  fn backend_author(&self) -> Result<String, QueryError>;

  fn backend_name(&self) -> Result<String, QueryError>;

  fn backend_version(&self) -> Result<String, QueryError>;

  fn backend_shading_lang_version(&self) -> Result<String, QueryError>;

  fn max_texture_array_elements(&self) -> Result<usize, QueryError>;
}
