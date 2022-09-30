use std::marker::PhantomData;

use crate::{
  backend::{FramebufferBackend, FramebufferError},
  dim::Dimensionable,
  render_channel::{DepthChannel, DepthChannelType},
};

/// Render slots.
///
/// Render slots are used to represent the “structure” of render layer. For instance, a render layer might have a color
/// channel and a depth channel. For more complex examples, it could have a diffuse, specular, normal and shininess
/// channel.
pub trait RenderSlots {
  type RenderLayers;

  unsafe fn new_render_layers<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
  ) -> Result<Self::RenderLayers, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable;
}

impl RenderSlots for () {
  type RenderLayers = ();

  unsafe fn new_render_layers<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
  ) -> Result<Self::RenderLayers, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

pub trait CompatibleRenderSlots<S> {}

#[derive(Debug)]
pub struct RenderLayer<RC> {
  handle: usize,
  _phantom: PhantomData<*const RC>,
}

impl<RC> RenderLayer<RC> {
  pub unsafe fn new(handle: usize) -> Self {
    Self {
      handle,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

pub trait DepthRenderSlot {
  type DepthRenderLayer;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType>;

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
  ) -> Result<Self::DepthRenderLayer, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable;
}

impl DepthRenderSlot for () {
  type DepthRenderLayer = ();

  const DEPTH_CHANNEL_TY: Option<DepthChannelType> = None;

  unsafe fn new_depth_render_layer<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
  ) -> Result<Self::DepthRenderLayer, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

impl<RC> DepthRenderSlot for RC
where
  RC: DepthChannel,
{
  type DepthRenderLayer = RenderLayer<RC>;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType> = Some(RC::CHANNEL_TY);

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
  ) -> Result<Self::DepthRenderLayer, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(backend.new_depth_render_layer::<D, _>(framebuffer_handle, size)?)
  }
}
