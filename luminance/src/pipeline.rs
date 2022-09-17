use std::{
  error, fmt,
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use crate::{
  backend::{
    color_slot::ColorSlot,
    depth_stencil_slot::DepthStencilSlot,
    framebuffer::Framebuffer as FramebufferBackend,
    pipeline::{Pipeline as PipelineBackend, PipelineBase, PipelineShaderData, PipelineTexture},
  },
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pixel::Pixel,
  scissor::{Scissor, ScissorRegion},
  shader::ShaderData,
  shading_gate::ShadingGate,
  texture::{Dimensionable, Texture},
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
  pub fn scissor(&self) -> &Option<ScissorRegion> {
    &self.clear_scissor
  }

  /// Set the scissor configuration.
  pub fn set_scissor(self, scissor: impl Into<Option<ScissorRegion>>) -> Self {
    Self {
      clear_scissor: scissor.into(),
      ..self
    }
  }
}

#[derive(Debug)]
pub struct WithFramebuffer<'a, B, S>
where
  B: ?Sized,
{
  backend: &'a mut B,
  _phantom: PhantomData<*const S>,
}

#[derive(Debug)]
pub struct WithProgram<'a, B, V, P, S, E>
where
  B: ?Sized,
{
  backend: &'a mut B,
  _phantom: PhantomData<*const (V, P, S, E)>,
}

#[derive(Debug)]
pub struct WithRenderState<'a, B, V>
where
  B: ?Sized,
{
  backend: &'a mut B,
  _phantom: PhantomData<*const V>,
}

// /// A GPU pipeline handle.
// ///
// /// A [`Pipeline`] is a special object that is provided as soon as one enters a [`PipelineGate`].
// /// It is used to dynamically modify the behavior of the running graphics pipeline. That includes,
// /// for instance, obtaining _bound resources_, like buffers and textures, for subsequent uses in
// /// shader stages.
// ///
// /// # Parametricity
// ///
// /// - `B` is the backend type. It must implement [`PipelineBase`].
// pub struct Pipeline<'a, B>
// where
//   B: ?Sized + PipelineBase,
// {
//   repr: B::PipelineRepr,
//   _phantom: PhantomData<&'a mut ()>,
// }
//
// impl<'a, B> Pipeline<'a, B>
// where
//   B: PipelineBase,
// {
//   /// Bind a texture.
//   ///
//   /// Once the texture is bound, the [`BoundTexture`] object has to be dropped / die in order to bind the texture again.
//   pub fn bind_texture<D, P>(
//     &'a self,
//     texture: &'a mut Texture<B, D, P>,
//   ) -> Result<BoundTexture<'a, B, D, P>, PipelineError>
//   where
//     B: PipelineTexture<D, P>,
//     D: Dimensionable,
//     P: Pixel,
//   {
//     unsafe {
//       B::bind_texture(&self.repr, &texture.repr).map(|repr| BoundTexture {
//         repr,
//         _phantom: PhantomData,
//       })
//     }
//   }
//
//   /// Bind a shader data.
//   ///
//   /// Once the shader data is bound, the [`BoundShaderData`] object has to be dropped / die in order to bind the shader
//   /// data again.
//   pub fn bind_shader_data<T>(
//     &'a self,
//     shader_data: &'a mut ShaderData<B, T>,
//   ) -> Result<BoundShaderData<'a, B, T>, PipelineError>
//   where
//     B: PipelineShaderData<T>,
//   {
//     unsafe {
//       B::bind_shader_data(&self.repr, &shader_data.repr).map(|repr| BoundShaderData {
//         repr,
//         _phantom: PhantomData,
//       })
//     }
//   }
// }
//
// /// Top-most node in a graphics pipeline.
// ///
// /// [`PipelineGate`] nodes represent the “entry-points” of graphics pipelines. They are used
// /// with a [`Framebuffer`] to render to and a [`PipelineState`] to customize the overall behavior
// /// of the pipeline.
// ///
// /// # Parametricity
// ///
// /// - `B`, the backend type.
// pub struct PipelineGate<'a, B> {
//   backend: &'a mut B,
// }
//
// impl<'a, B> PipelineGate<'a, B> {
//   /// Create a new [`PipelineGate`].
//   pub fn new<C>(ctx: &'a mut C) -> Self
//   where
//     C: GraphicsContext<Backend = B>,
//   {
//     PipelineGate {
//       backend: ctx.backend(),
//     }
//   }
//
//   /// Enter a pipeline node.
//   ///
//   /// This method is the entry-point in a graphics pipeline. It takes a [`Framebuffer`] and a
//   /// [`PipelineState`] and a closure that allows to go deeper in the pipeline (i.e. resource
//   /// graph). The closure is passed a [`Pipeline`] for you to dynamically alter the pipeline and a
//   /// [`ShadingGate`] to enter shading nodes.
//   ///
//   /// # Errors
//   ///
//   /// [`PipelineError`] might be thrown for various reasons, depending on the backend you use.
//   /// However, this method doesn’t return [`PipelineError`] directly: instead, it returns
//   /// `E: From<PipelineError>`. This allows you to inject your own error type in the argument
//   /// closure, allowing for a grainer control of errors inside the pipeline.
//   pub fn pipeline<E, D, CS, DS, F>(
//     &mut self,
//     framebuffer: &Framebuffer<B, D, CS, DS>,
//     pipeline_state: &PipelineState,
//     f: F,
//   ) -> Render<E>
//   where
//     B: FramebufferBackend<D> + PipelineBackend<D>,
//     D: Dimensionable,
//     CS: ColorSlot<B, D>,
//     DS: DepthStencilSlot<B, D>,
//     F: for<'b> FnOnce(Pipeline<'b, B>, ShadingGate<'b, B>) -> Result<(), E>,
//     E: From<PipelineError>,
//   {
//     let render = || {
//       unsafe {
//         self
//           .backend
//           .start_pipeline(&framebuffer.repr, pipeline_state);
//       }
//
//       let pipeline = unsafe {
//         self.backend.new_pipeline().map(|repr| Pipeline {
//           repr,
//           _phantom: PhantomData,
//         })?
//       };
//
//       let shading_gate = ShadingGate {
//         backend: self.backend,
//       };
//
//       f(pipeline, shading_gate)
//     };
//
//     Render(render())
//   }
// }
//
// /// Output of a [`PipelineGate`].
// ///
// /// This type is used as a proxy over `Result<(), E>`, which it defers to. It is needed so that
// /// you can seamlessly call the [`assume`] method
// ///
// /// [`assume`]: crate::pipeline::Render::assume
// pub struct Render<E>(Result<(), E>);
//
// impl<E> Render<E> {
//   /// Turn a [`Render`] into a [`Result`].
//   #[inline]
//   pub fn into_result(self) -> Result<(), E> {
//     self.0
//   }
// }
//
// impl Render<PipelineError> {
//   /// Assume the error type is [`PipelineError`].
//   ///
//   /// Most of the time, users will not provide their own error types for pipelines. Rust doesn’t
//   /// have default type parameters for methods, so this function is needed to inform the type
//   /// system to default the error type to [`PipelineError`].
//   #[inline]
//   pub fn assume(self) -> Self {
//     self
//   }
// }
//
// impl<E> From<Render<E>> for Result<(), E> {
//   fn from(render: Render<E>) -> Self {
//     render.0
//   }
// }
//
// impl<E> Deref for Render<E> {
//   type Target = Result<(), E>;
//
//   fn deref(&self) -> &Self::Target {
//     &self.0
//   }
// }
//
// impl<E> DerefMut for Render<E> {
//   fn deref_mut(&mut self) -> &mut Self::Target {
//     &mut self.0
//   }
// }
//
// /// Opaque shader data binding.
// ///
// /// This type represents a bound [`ShaderData`] via [`BoundShaderData`]. It can be used along with a [`Uniform`] to
// /// customize a shader’s behavior.
// ///
// /// # Parametricity
// ///
// /// - `T` is the type of the carried item by the [`ShaderData`].
// ///
// /// # Notes
// ///
// /// You shouldn’t try to do store / cache or do anything special with that value. Consider it an opaque object.
// ///
// /// [`Uniform`]: crate::shader::Uniform
// #[derive(Debug)]
// pub struct ShaderDataBinding<T> {
//   binding: u32,
//   _phantom: PhantomData<*const T>,
// }
//
// impl<T> ShaderDataBinding<T> {
//   /// Access the underlying binding value.
//   ///
//   /// # Notes
//   ///
//   /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
//   pub fn binding(self) -> u32 {
//     self.binding
//   }
// }
//
// /// A _bound_ [`ShaderData`].
// ///
// /// # Parametricity
// ///
// /// - `B` is the backend type. It must implement [`ShaderData`](crate::backend::shader::ShaderData).
// /// - `T` is the carried item type.
// ///
// /// # Notes
// ///
// /// Once a [`ShaderData`] is bound, it can be used and passed around to shaders. In order to do so, you will need to
// /// pass a [`ShaderDataBinding`] to your [`ProgramInterface`]. That value is unique to each [`BoundShaderData`] and
// /// should always be asked — you shouldn’t cache them, for instance.
// ///
// /// Getting a [`ShaderDataBinding`] is a cheap operation and is performed via the [`BoundShaderData::binding`] method.
// ///
// /// [`ProgramInterface`]: crate::shader::ProgramInterface
// pub struct BoundShaderData<'a, B, T>
// where
//   B: PipelineShaderData<T>,
// {
//   pub(crate) repr: B::BoundShaderDataRepr,
//   _phantom: PhantomData<&'a ()>,
// }
//
// impl<'a, B, T> BoundShaderData<'a, B, T>
// where
//   B: PipelineShaderData<T>,
// {
//   /// Obtain a [`ShaderDataBinding`] object that can be used to refer to this bound shader data in shader stages.
//   ///
//   /// # Notes
//   ///
//   /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
//   /// an opaque object.
//   pub fn binding(&self) -> ShaderDataBinding<T> {
//     let binding = unsafe { B::shader_data_binding(&self.repr) };
//     ShaderDataBinding {
//       binding,
//       _phantom: PhantomData,
//     }
//   }
// }
//
// /// Opaque texture binding.
// ///
// /// This type represents a bound [`Texture`] via [`BoundTexture`]. It can be used along with a
// /// [`Uniform`] to customize a shader’s behavior.
// ///
// /// # Parametricity
// ///
// /// - `D` is the dimension of the original texture. It must implement [`Dimensionable`] in most
// ///   useful methods.
// /// - `S` is the sampler type. It must implement [`SamplerType`] in most useful methods.
// ///
// /// # Notes
// ///
// /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
// /// an opaque object.
// ///
// /// [`Uniform`]: crate::shader::Uniform
// /// [`SamplerType`]: crate::pixel::SamplerType
// #[derive(Debug)]
// pub struct TextureBinding<D, S> {
//   binding: u32,
//   _phantom: PhantomData<*const (D, S)>,
// }
//
// impl<D, S> TextureBinding<D, S> {
//   /// Access the underlying binding value.
//   ///
//   /// # Notes
//   ///
//   /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
//   pub fn binding(self) -> u32 {
//     self.binding
//   }
// }
//
// /// A _bound_ [`Texture`].
// ///
// /// # Parametricity
// ///
// /// - `B` is the backend type. It must implement [`PipelineTexture`].
// /// - `D` is the dimension. It must implement [`Dimensionable`].
// /// - `P` is the pixel type. It must implement [`Pixel`].
// ///
// /// # Notes
// ///
// /// Once a [`Texture`] is bound, it can be used and passed around to shaders. In order to do so,
// /// you will need to pass a [`TextureBinding`] to your [`ProgramInterface`]. That value is unique
// /// to each [`BoundTexture`] and should always be asked — you shouldn’t cache them, for instance.
// ///
// /// Getting a [`TextureBinding`] is a cheap operation and is performed via the
// /// [`BoundTexture::binding`] method.
// ///
// /// [`ProgramInterface`]: crate::shader::ProgramInterface
// pub struct BoundTexture<'a, B, D, P>
// where
//   B: PipelineTexture<D, P>,
//   D: Dimensionable,
//   P: Pixel,
// {
//   pub(crate) repr: B::BoundTextureRepr,
//   _phantom: PhantomData<&'a ()>,
// }
//
// impl<'a, B, D, P> BoundTexture<'a, B, D, P>
// where
//   B: PipelineTexture<D, P>,
//   D: Dimensionable,
//   P: Pixel,
// {
//   /// Obtain a [`TextureBinding`] object that can be used to refer to this bound texture in shader
//   /// stages.
//   ///
//   /// # Notes
//   ///
//   /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
//   /// an opaque object.
//   pub fn binding(&self) -> TextureBinding<D, P::SamplerType> {
//     let binding = unsafe { B::texture_binding(&self.repr) };
//     TextureBinding {
//       binding,
//       _phantom: PhantomData,
//     }
//   }
// }
