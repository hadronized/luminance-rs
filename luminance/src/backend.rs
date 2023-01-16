use crate::{
  dim::Dimensionable,
  framebuffer::{Back, Framebuffer},
  pipeline::{PipelineState, WithFramebuffer, WithProgram, WithRenderState},
  pixel::{Pixel, PixelFormat, PixelType},
  primitive::Primitive,
  render_slots::{DepthChannel, DepthRenderSlot, RenderChannel, RenderSlots},
  render_state::RenderState,
  shader::{MemoryLayout, Program, Uni, Uniform, UniformBuffer, Uniforms},
  texture::{InUseTexture, Mipmaps, Texture, TextureSampling},
  vertex::Vertex,
  vertex_entity::{VertexEntity, VertexEntityView},
  vertex_storage::AsVertexStorage,
};
use std::{collections::HashMap, error::Error as ErrorTrait, fmt};

/// Cached value.
///
/// A cached value is used to prevent issuing costy GPU commands if we know the target value is
/// already set to what the command tries to set. For instance, if you ask to use a texture ID
/// `34` once, that value will be set on the GPU and cached on our side. Later, if no other texture
/// setting has occurred, if you ask to use the texture ID `34` again, because the value is cached,
/// we know the GPU is already using it, so we don’t have to perform anything GPU-wise.
///
/// This optimization has limits and sometimes, because of side-effects, it is not possible to cache
/// something correctly.
#[derive(Debug)]
pub struct Cached<T>(Option<T>);

impl<T> Cached<T>
where
  T: PartialEq,
{
  /// Start with no value.
  pub fn empty() -> Self {
    Cached(None)
  }

  /// Explicitly invalidate a value.
  ///
  /// This is necessary when we want to be able to force a GPU command to run.
  pub fn invalidate(&mut self) {
    self.0 = None;
  }

  pub fn set(&mut self, value: T) -> Option<T> {
    self.0.replace(value)
  }

  // Set the value if invalid or never set (and call the function).
  //
  // If the value was still valid, returns `true`.
  pub fn set_if_invalid(&mut self, value: T, f: impl FnOnce()) -> bool {
    match self.0 {
      Some(ref x) if x == &value => false,

      _ => {
        self.0 = Some(value);
        f();
        true
      }
    }
  }

  /// Check whether the cached value is invalid regarding a value.
  ///
  /// A non-cached value (i.e. empty) is always invalid whatever compared value. If a value is already cached, then it’s
  /// invalid if it’s not equal ([`PartialEq`]) to the input value.
  pub fn is_invalid(&self, new_val: &T) -> bool {
    match &self.0 {
      Some(ref t) => t != new_val,
      _ => true,
    }
  }
}

/// Map a resource handle to a given “binding” name.
///
/// The mapper will always try to exhaust the max number of concurrent bound resources. Once it’s exhausted, it will
/// reused the ones that are marked as “idle.” An “idle” resource might reuse it soon, but the ownership is not removed
/// just yet so that if the mapper doesn’t ask the resource back, the resource can rebind it without having to go
/// through a new lookup procedure again, removing back-and-forth interactions.
#[derive(Debug)]
pub struct ResourceMapper {
  current_binding: Cached<usize>,
  next_binding: usize,
  max_binding: usize,
  idling_bindings: HashMap<usize, usize>, // binding -> resource handle
}

#[derive(Debug)]
pub enum ResourceMapperError {
  NotEnoughBindings { max: usize },
}

impl ResourceMapper {
  pub fn new(max_binding: usize) -> Self {
    Self {
      current_binding: Cached::empty(),
      next_binding: 0,
      max_binding,
      idling_bindings: HashMap::new(),
    }
  }

  pub fn current_binding(&mut self) -> &mut Cached<usize> {
    &mut self.current_binding
  }

  /// Get a new binding.
  ///
  /// We always try to get a fresh binding, and if we can’t, we will try to reuse an idling one.
  pub fn get_binding(&mut self) -> Result<(usize, Option<usize>), ResourceMapperError> {
    if self.next_binding < self.max_binding {
      // we still can use a fresh unit
      let unit = self.next_binding;
      self.next_binding += 1;

      Ok((unit, None))
    } else {
      // we have exhausted the hardware bindings; try to reuse an idling one and if we cannot, then it’s an error
      self
        .reuse_binding()
        .ok_or_else(|| ResourceMapperError::NotEnoughBindings {
          max: self.max_binding,
        })
    }
  }

  /// Try to reuse a binding. Return `None` if no binding is available, and `Some((unit, old_resource_handle))`
  /// otherwise.
  fn reuse_binding(&mut self) -> Option<(usize, Option<usize>)> {
    let unit = self.idling_bindings.keys().next().cloned()?;
    let old_resource_handle = self.idling_bindings.remove(&unit)?;

    Some((unit, Some(old_resource_handle)))
  }

  /// Mark a binding as idle.
  pub fn mark_idle(&mut self, unit: usize, handle: usize) {
    self.idling_bindings.insert(unit, handle);
  }

  /// Mark a binding as non-idle.
  pub fn mark_nonidle(&mut self, unit: usize) {
    self.idling_bindings.remove(&unit);
  }
}

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
  RenderLayerUsage { cause: Option<Box<dyn ErrorTrait>> },
  ReadRenderLayer { cause: Option<Box<dyn ErrorTrait>> },
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

      FramebufferError::RenderLayerUsage { cause } => write!(
        f,
        "error when using a render layer: {}",
        cause
          .as_ref()
          .map(|cause| cause.to_string())
          .unwrap_or_else(|| "unknown cause".to_string())
      ),

      FramebufferError::ReadRenderLayer { cause } => write!(
        f,
        "error when reading a render layer: {}",
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

  FramebufferError(FramebufferError),

  TextureError(TextureError),
}

impl From<ShaderError> for PipelineError {
  fn from(e: ShaderError) -> Self {
    PipelineError::ShaderError(e)
  }
}

impl From<FramebufferError> for PipelineError {
  fn from(e: FramebufferError) -> Self {
    PipelineError::FramebufferError(e)
  }
}

impl From<TextureError> for PipelineError {
  fn from(e: TextureError) -> Self {
    PipelineError::TextureError(e)
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

      PipelineError::FramebufferError(e) => write!(f, "framebuffer error in pipeline: {}", e),

      PipelineError::TextureError(e) => write!(f, "texture error in pipeline: {}", e),
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

impl From<ResourceMapperError> for TextureError {
  fn from(value: ResourceMapperError) -> Self {
    match value {
      ResourceMapperError::NotEnoughBindings { max } => TextureError::NotEnoughTextureUnits { max },
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

pub unsafe trait Backend:
  VertexEntityBackend
  + FramebufferBackend
  + ShaderBackend
  + TextureBackend
  + PipelineBackend
  + QueryBackend
{
  unsafe fn unload(&mut self);
}

pub unsafe trait VertexEntityBackend {
  unsafe fn new_vertex_entity<V, P, S, I, W, WS>(
    &mut self,
    vertices: S,
    indices: I,
    instance_data: WS,
  ) -> Result<VertexEntity<V, P, S, W, WS>, VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
    I: Into<Vec<u32>>,
    W: Vertex,
    WS: AsVertexStorage<W>;

  unsafe fn vertex_entity_render<V, P>(
    &self,
    handle: usize,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
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

  unsafe fn vertex_entity_update_instance_data<W, WS>(
    &mut self,
    handle: usize,
    storage: &mut WS,
  ) -> Result<(), VertexEntityError>
  where
    W: Vertex,
    WS: AsVertexStorage<W>;
}

pub unsafe trait FramebufferBackend {
  unsafe fn new_render_layer<D, P>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
    index: usize,
  ) -> Result<Texture<D, P>, FramebufferError>
  where
    D: Dimensionable,
    P: RenderChannel;

  unsafe fn new_depth_render_layer<D, P>(
    &mut self,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
  ) -> Result<Texture<D, P>, FramebufferError>
  where
    D: Dimensionable,
    P: DepthChannel;

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
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

  ($( as_ref $name:ident, $t:ty, )*) => {
    $( fn $name<T>(&mut self, uni: &Uni<T>, value: &$t) -> Result<(), ShaderError> where T: AsRef<$t>; )*
  };

  ($( array $name:ident, $t:ty, )*) => {
    $( fn $name<const N: usize>(&mut self, uni: &Uni<[$t; N]>, value: &[$t; N]) -> Result<(), ShaderError>; )*
  };
}

pub unsafe trait ShaderBackend {
  unsafe fn new_program<V, W, P, S, E>(
    &mut self,
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
  ) -> Result<Program<V, W, P, S, E>, ShaderError>
  where
    V: Vertex,
    W: Vertex,
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
    uni: &Uni<T>,
    value: &T::Value,
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
    as_ref visit_ivec2, [i32; 2],
    as_ref visit_uvec2, [u32; 2],
    as_ref visit_vec2, [f32; 2],
    as_ref visit_bvec2, [bool; 2],

    as_ref visit_ivec3, [i32; 3],
    as_ref visit_uvec3, [u32; 3],
    as_ref visit_vec3, [f32; 3],
    as_ref visit_bvec3, [bool; 3],

    as_ref visit_ivec4, [i32; 4],
    as_ref visit_uvec4, [u32; 4],
    as_ref visit_vec4, [f32; 4],
    as_ref visit_bvec4, [bool; 4],

    as_ref visit_mat22, [[f32; 2]; 2],
    as_ref visit_mat33, [[f32; 3]; 3],
    as_ref visit_mat44, [[f32; 4]; 4],
  }

  fn visit_texture<D, P>(
    &mut self,
    uni: &Uni<InUseTexture<D, P>>,
    value: &InUseTexture<D, P>,
  ) -> Result<(), ShaderError>
  where
    D: Dimensionable,
    P: PixelType;

  unsafe fn new_uniform_buffer<T, Scheme>(
    &mut self,
    value: T::Aligned,
  ) -> Result<UniformBuffer<T, Scheme>, ShaderError>
  where
    T: MemoryLayout<Scheme>;
}

pub unsafe trait TextureBackend {
  unsafe fn reserve_texture<D, P>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn new_texture<D, P>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
    texels: &[P::RawEncoding],
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn resize_texture<D, P>(
    &mut self,
    handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn set_texture_data<D, P>(
    &mut self,
    handle: usize,
    offset: D::Offset,
    size: D::Size,
    gen_mipmaps: bool,
    texels: &[P::RawEncoding],
    level: usize,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn clear_texture_data<D, P>(
    &mut self,
    handle: usize,
    offset: D::Offset,
    size: D::Size,
    gen_mimaps: bool,
    clear_value: P::RawEncoding,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn read_texture<D, P>(
    &mut self,
    handle: usize,
  ) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    D: Dimensionable,
    P: Pixel;

  unsafe fn use_texture<D, P>(&mut self, handle: usize) -> Result<InUseTexture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: PixelType;
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

  unsafe fn with_program<V, W, P, S, E, Err>(
    &mut self,
    program: &Program<V, W, P, S, E>,
    f: impl for<'a> FnOnce(WithProgram<'a, Self, V, W, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    V: Vertex,
    W: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: Uniforms,
    Err: From<PipelineError>;

  unsafe fn with_render_state<V, W, P, Err>(
    &mut self,
    render_state: &RenderState,
    f: impl for<'a> FnOnce(WithRenderState<'a, Self, V, W, P>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    V: Vertex,
    W: Vertex,
    P: Primitive,
    Err: From<PipelineError>;

  unsafe fn render_vertex_entity<V, W, P>(
    &mut self,
    view: VertexEntityView<V, W, P>,
  ) -> Result<(), PipelineError>
  where
    V: Vertex,
    W: Vertex,
    P: Primitive;
}

pub unsafe trait QueryBackend {
  fn backend_author(&self) -> Result<String, QueryError>;

  fn backend_name(&self) -> Result<String, QueryError>;

  fn backend_version(&self) -> Result<String, QueryError>;

  fn backend_shading_lang_version(&self) -> Result<String, QueryError>;
}
