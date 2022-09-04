//! Graphics context.
//!
//! # Graphics context and backends
//!
//! A graphics context is an external type typically implemented by other crates and which provides
//! support for backends. Its main scope is to unify all possible implementations of backends
//! behind a single trait: [`GraphicsContext`]. A [`GraphicsContext`] really only requires two items
//! to be implemented:
//!
//! - The type of the backend to use — [`GraphicsContext::Backend`]. That type will often be used
//!   to access the GPU, cache costly operations, etc.
//! - A method to get a mutable access to the underlying backend — [`GraphicsContext::backend`].
//!
//! Most of the time, if you want to work with _any_ windowing implementation, you will want to
//! use a type variable such as `C: GraphicsContext`. If you want to work with any context
//! supporting a specific backend, use `C: GraphicsContext<Backend = YourBackendType`. Etc.
//!
//! This crate doesn’t provide you with creating such contexts. Instead, you must do it yourself
//! or rely on crates doing it for you.
//!
//! # Default implementation of helper functions
//!
//! By default, graphics contexts automatically get several methods implemented on them. Those
//! methods are helper functions available to write code in a more elegant and compact way than
//! passing around mutable references on the context. Often, it will help you not having to
//! use type ascription, too, since the [`GraphicsContext::Backend`] type is known when calling
//! those functions.
//!
//! Instead of:
//!
//! ```ignore
//! use luminance::context::GraphicsContext as _;
//! use luminance::buffer::Buffer;
//!
//! let buffer: Buffer<SomeBackendType, u8> = Buffer::from_slice(&mut context, slice).unwrap();
//! ```
//!
//! You can simply do:
//!
//! ```ignore
//! use luminance::context::GraphicsContext as _;
//!
//! let buffer = context.new_buffer_from_slice(slice).unwrap();
//! ```

use crate::{
  backend::{
    color_slot::ColorSlot,
    depth_stencil_slot::DepthStencilSlot,
    framebuffer::Framebuffer as FramebufferBackend,
    query::Query as QueryBackend,
    shader::{Shader, ShaderData as ShaderDataBackend},
    tess::Tess as TessBackend,
    texture::Texture as TextureBackend,
  },
  framebuffer::{Framebuffer, FramebufferError},
  pipeline::PipelineGate,
  pixel::Pixel,
  query::Query,
  shader::{ProgramBuilder, ShaderData, ShaderDataError, Stage, StageError, StageType},
  tess::{Deinterleaved, Interleaved, TessBuilder, TessVertexData},
  texture::{Dimensionable, Sampler, TexelUpload, Texture, TextureError},
  vertex::Vertex,
};

/// Class of graphics context.
///
/// Graphics context must implement this trait to be able to be used throughout the rest of the
/// crate.
pub unsafe trait GraphicsContext: Sized {
  /// Internal type used by the backend to cache, optimize and store data. This roughly represents
  /// the GPU data / context a backend implementation needs to work correctly.
  type Backend;

  /// Access the underlying backend.
  fn backend(&mut self) -> &mut Self::Backend;

  /// Access the query API.
  fn query(&mut self) -> Query<Self::Backend>
  where
    Self::Backend: QueryBackend,
  {
    Query::new(self)
  }

  /// Create a new pipeline gate
  fn new_pipeline_gate(&mut self) -> PipelineGate<Self::Backend> {
    PipelineGate::new(self)
  }

  /// Create a new framebuffer.
  ///
  /// See the documentation of [`Framebuffer::new`] for further details.
  fn new_framebuffer<D, CS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: usize,
    sampler: Sampler,
  ) -> Result<Framebuffer<Self::Backend, D, CS, DS>, FramebufferError>
  where
    Self::Backend: FramebufferBackend<D>,
    D: Dimensionable,
    CS: ColorSlot<Self::Backend, D>,
    DS: DepthStencilSlot<Self::Backend, D>,
  {
    Framebuffer::new(self, size, mipmaps, sampler)
  }

  /// Create a new shader stage.
  ///
  /// See the documentation of [`Stage::new`] for further details.
  fn new_shader_stage<R>(
    &mut self,
    ty: StageType,
    src: R,
  ) -> Result<Stage<Self::Backend>, StageError>
  where
    Self::Backend: Shader,
    R: AsRef<str>,
  {
    Stage::new(self, ty, src)
  }

  /// Create a new shader program.
  ///
  /// See the documentation of [`ProgramBuilder::new`] for further details.
  fn new_shader_program<V, Out, Uni>(&mut self) -> ProgramBuilder<Self, V, Out, Uni>
  where
    Self::Backend: Shader,
    V: Vertex,
  {
    ProgramBuilder::new(self)
  }

  /// Create a new shader data.
  ///
  /// See the documentation of [`ShaderData::new`] for further details.
  fn new_shader_data<T>(
    &mut self,
    values: impl IntoIterator<Item = T>,
  ) -> Result<ShaderData<Self::Backend, T>, ShaderDataError>
  where
    Self::Backend: ShaderDataBackend<T>,
  {
    ShaderData::new(self, values)
  }

  /// Create a [`TessBuilder`].
  ///
  /// See the documentation of [`TessBuilder::new`] for further details.
  fn new_tess(&mut self) -> TessBuilder<Self::Backend, (), (), (), Interleaved>
  where
    Self::Backend: TessBackend<(), (), (), Interleaved>,
  {
    TessBuilder::new(self)
  }

  /// Create a [`TessBuilder`] with deinterleaved memory.
  ///
  /// See the documentation of [`TessBuilder::new`] for further details.
  fn new_deinterleaved_tess<V, W>(&mut self) -> TessBuilder<Self::Backend, V, (), W, Deinterleaved>
  where
    Self::Backend: TessBackend<V, (), W, Deinterleaved>,
    V: TessVertexData<Deinterleaved>,
    W: TessVertexData<Deinterleaved>,
  {
    TessBuilder::new(self)
  }

  /// Create a new texture from texels.
  ///
  /// Feel free to have a look at the documentation of [`Texture::new`] for further details.
  fn new_texture<D, P>(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<Texture<Self::Backend, D, P>, TextureError>
  where
    Self::Backend: TextureBackend<D, P>,
    D: Dimensionable,
    P: Pixel,
  {
    Texture::new(self, size, sampler, texels)
  }

  /// Create a new texture from raw texels.
  ///
  /// Feel free to have a look at the documentation of [`Texture::new_raw`] for further details.
  fn new_texture_raw<D, P>(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<Texture<Self::Backend, D, P>, TextureError>
  where
    Self::Backend: TextureBackend<D, P>,
    D: Dimensionable,
    P: Pixel,
  {
    Texture::new_raw(self, size, sampler, texels)
  }
}
