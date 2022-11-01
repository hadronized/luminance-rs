use std::marker::PhantomData;

use crate::{
  backend::{PipelineBackend, PipelineError, ShaderError, TextureBackend, TextureError},
  dim::Dimensionable,
  pixel::Pixel,
  primitive::Primitive,
  render_slots::{CompatibleRenderSlots, RenderSlots},
  render_state::RenderState,
  scissor::Scissor,
  shader::{Program, ProgramUpdate, Uniforms},
  texture::{InUseTexture, Texture},
  vertex::{CompatibleVertex, Vertex},
  vertex_entity::VertexEntityView,
};

/// The viewport being part of the [`PipelineState`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Viewport {
  /// The whole viewport is used. The position and dimension of the viewport rectangle are
  /// extracted from the framebuffer.
  Whole,

  /// The viewport is specific and the rectangle area is user-defined.
  Specific {
    /// The lower position on the X axis to start the viewport rectangle at.
    x: u32,
    /// The lower position on the Y axis to start the viewport rectangle at.
    y: u32,
    /// The width of the viewport.
    width: u32,
    /// The height of the viewport.
    height: u32,
  },
}

/// Various customization options for pipelines.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct PipelineState {
  /// Color to use when clearing color buffers.
  ///
  /// Set this to `Some(color)` to use that color to clear the [`Framebuffer`] when running a [`PipelineGate`]. Set it
  /// to `None` not to clear the framebuffer when running the [`PipelineGate`].
  ///
  /// An example of not setting the clear color is if you want to accumulate renders in a [`Framebuffer`] (for instance
  /// for a paint-like application).
  pub clear_color: Option<[f32; 4]>,

  /// Depth value to use when clearing the depth buffer.
  ///
  /// Set this to `Some(depth)` to use that depth to clear the [`Framebuffer`] depth buffer.
  pub clear_depth: Option<f32>,

  /// Stencil value to use when clearing the stencil buffer.
  ///
  /// Set this to `Some(stencil)` to use that stencil to clear the [`Framebuffer`] stencil buffer.
  pub clear_stencil: Option<i32>,

  /// Viewport to use when rendering.
  pub viewport: Viewport,

  /// Whether [sRGB](https://en.wikipedia.org/wiki/SRGB) support should be enabled.
  ///
  /// When this is set to `true`, shader outputs that go in [`Framebuffer`] for each of the color slots have sRGB pixel
  /// formats are assumed to be in the linear RGB color space. The pipeline will then convert that linear color outputs
  /// to sRGB to be stored in the [`Framebuffer`].
  ///
  /// Typical examples are when you are rendering into an image that is to be displayed to on screen: the
  /// [`Framebuffer`] can use sRGB color pixel formats and the shader doesn’t have to worry about converting from linear
  /// color space into sRGB color space, as the pipeline will do that for you.
  pub srgb_enabled: bool,

  /// Whether to use scissor test when clearing buffers.
  pub clear_scissor: Scissor,
}

impl Default for PipelineState {
  /// Default [`PipelineState`]:
  ///
  /// - Clear color is `Some([0., 0., 0., 1.])`.
  /// - Depth value is `Some(1.)`.
  /// - Stencil value is `Some(0)`.
  /// - The viewport uses the whole framebuffer’s.
  /// - sRGB encoding is disabled.
  /// - No scissor test is performed.
  fn default() -> Self {
    PipelineState {
      clear_color: Some([0., 0., 0., 1.]),
      clear_depth: Some(1.),
      clear_stencil: Some(0),
      viewport: Viewport::Whole,
      srgb_enabled: false,
      clear_scissor: Scissor::Off,
    }
  }
}

impl PipelineState {
  /// Create a default [`PipelineState`].
  ///
  /// See the documentation of the [`Default`] for further details.
  pub fn new() -> Self {
    Self::default()
  }

  /// Get the clear color, if any.
  pub fn clear_color(&self) -> Option<&[f32; 4]> {
    self.clear_color.as_ref()
  }

  /// Set the clear color.
  pub fn set_clear_color(self, clear_color: impl Into<Option<[f32; 4]>>) -> Self {
    Self {
      clear_color: clear_color.into(),
      ..self
    }
  }

  /// Get the clear depth, if any.
  pub fn clear_depth(&self) -> Option<f32> {
    self.clear_depth
  }

  /// Set the clear depth.
  pub fn set_clear_depth(self, clear_depth: impl Into<Option<f32>>) -> Self {
    Self {
      clear_depth: clear_depth.into(),
      ..self
    }
  }

  /// Get the clear stencil, if any.
  pub fn clear_stencil(&self) -> Option<i32> {
    self.clear_stencil
  }

  /// Set the clear stencil.
  pub fn set_clear_stencil(self, clear_stencil: impl Into<Option<i32>>) -> Self {
    Self {
      clear_stencil: clear_stencil.into(),
      ..self
    }
  }

  /// Get the viewport.
  pub fn viewport(&self) -> Viewport {
    self.viewport
  }

  /// Set the viewport.
  pub fn set_viewport(self, viewport: Viewport) -> Self {
    Self { viewport, ..self }
  }

  /// Check whether sRGB linearization is enabled.
  pub fn is_srgb_enabled(&self) -> bool {
    self.srgb_enabled
  }

  /// Enable sRGB linearization.
  pub fn enable_srgb(self, srgb_enabled: bool) -> Self {
    Self {
      srgb_enabled,
      ..self
    }
  }

  /// Get the scissor configuration, if any.
  pub fn scissor(&self) -> &Scissor {
    &self.clear_scissor
  }

  /// Set the scissor configuration.
  pub fn set_scissor(self, clear_scissor: Scissor) -> Self {
    Self {
      clear_scissor,
      ..self
    }
  }
}

#[derive(Debug)]
pub struct WithFramebuffer<'a, B, S>
where
  B: 'a + ?Sized,
{
  backend: &'a mut B,
  _phantom: PhantomData<*const S>,
}

impl<'a, B, S> WithFramebuffer<'a, B, S>
where
  B: 'a + PipelineBackend,
{
  pub unsafe fn new(backend: &'a mut B) -> Self {
    Self {
      backend,
      _phantom: PhantomData,
    }
  }

  pub fn with_program<V, P, T, E, Err>(
    &mut self,
    program: &Program<V, P, T, E>,
    f: impl for<'b> FnOnce(WithProgram<'b, B, V, P, T, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    V: Vertex,
    P: Primitive,
    S: CompatibleRenderSlots<T>,
    T: RenderSlots,
    E: Uniforms,
    Err: From<PipelineError>,
  {
    unsafe { self.backend.with_program(program, f) }
  }

  pub fn use_texture<D, P>(
    &mut self,
    texture: &Texture<D, P>,
  ) -> Result<InUseTexture<D, P>, TextureError>
  where
    B: TextureBackend,
    D: Dimensionable,
    P: Pixel,
  {
    unsafe { self.backend.use_texture(texture.handle()) }
  }
}

pub struct WithProgram<'a, B, V, P, S, E>
where
  B: 'a + ?Sized,
{
  backend: &'a mut B,
  program: &'a Program<V, P, S, E>,
  _phantom: PhantomData<*const (V, P, S, E)>,
}

impl<'a, B, V, P, S, E> WithProgram<'a, B, V, P, S, E>
where
  B: 'a + PipelineBackend,
  V: Vertex,
  P: Primitive,
  S: RenderSlots,
{
  pub unsafe fn new(backend: &'a mut B, program: &'a Program<V, P, S, E>) -> Self {
    Self {
      backend,
      program,
      _phantom: PhantomData,
    }
  }

  pub fn with_render_state<Err>(
    &mut self,
    render_state: &RenderState,
    f: impl for<'b> FnOnce(WithRenderState<'b, B, V, P>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Err: From<PipelineError>,
  {
    unsafe { self.backend.with_render_state(render_state, f) }
  }

  pub fn update(
    &mut self,
    f: impl for<'b> FnOnce(ProgramUpdate<'b, B>, &E) -> Result<(), ShaderError>,
  ) -> Result<(), ShaderError> {
    let program_update = ProgramUpdate {
      backend: self.backend,
      program_handle: self.program.handle(),
    };

    f(program_update, &self.program.uniforms)
  }

  pub fn use_texture<D, Px>(
    &mut self,
    texture: &Texture<D, Px>,
  ) -> Result<InUseTexture<D, Px>, TextureError>
  where
    B: TextureBackend,
    D: Dimensionable,
    Px: Pixel,
  {
    unsafe { self.backend.use_texture(texture.handle()) }
  }
}

#[derive(Debug)]
pub struct WithRenderState<'a, B, V, P>
where
  B: 'a + ?Sized,
{
  backend: &'a mut B,
  _phantom: PhantomData<*const (V, P)>,
}

impl<'a, B, V, P> WithRenderState<'a, B, V, P>
where
  B: 'a + PipelineBackend,
  V: Vertex,
  P: Primitive,
{
  pub unsafe fn new(backend: &'a mut B) -> Self {
    Self {
      backend,
      _phantom: PhantomData,
    }
  }

  pub fn render_vertex_entity<W>(
    &mut self,
    view: VertexEntityView<W, P>,
  ) -> Result<(), PipelineError>
  where
    V: CompatibleVertex<W>,
    W: Vertex,
  {
    unsafe { self.backend.render_vertex_entity(view) }
  }

  pub fn use_texture<D, Px>(
    &mut self,
    texture: &Texture<D, Px>,
  ) -> Result<InUseTexture<D, Px>, TextureError>
  where
    B: TextureBackend,
    D: Dimensionable,
    Px: Pixel,
  {
    unsafe { self.backend.use_texture(texture.handle()) }
  }
}
