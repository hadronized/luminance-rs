use std::marker::PhantomData;

use crate::{
  backend::{FramebufferBackend, FramebufferError},
  dim::Dimensionable,
  framebuffer::Back,
  render_channel::{DepthChannel, DepthChannelType, RenderChannelDesc},
  texture::Mipmaps,
};

/// Render slots.
///
/// Render slots are used to represent the “structure” of render layer. For instance, a render layer might have a color
/// channel and a depth channel. For more complex examples, it could have a diffuse, specular, normal and shininess
/// channel.
pub trait RenderSlots {
  type RenderLayers<D>
  where
    D: Dimensionable;

  fn color_channel_descs() -> &'static [RenderChannelDesc];

  unsafe fn new_render_layers<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<Self::RenderLayers<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable;
}

impl RenderSlots for () {
  type RenderLayers<D> = () where D: Dimensionable;

  fn color_channel_descs() -> &'static [RenderChannelDesc] {
    &[]
  }

  unsafe fn new_render_layers<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
  ) -> Result<Self::RenderLayers<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

impl<RS> RenderSlots for Back<RS>
where
  RS: RenderSlots,
{
  type RenderLayers<D> = () where D: Dimensionable;

  fn color_channel_descs() -> &'static [RenderChannelDesc] {
    &[]
  }

  unsafe fn new_render_layers<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
  ) -> Result<Self::RenderLayers<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

pub trait CompatibleRenderSlots<S> {}

#[derive(Debug)]
pub struct RenderLayer<D, RC> {
  handle: usize,
  _phantom: PhantomData<*const (D, RC)>,
}

impl<D, RC> RenderLayer<D, RC> {
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
  type DepthRenderLayer<D>
  where
    D: Dimensionable;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType>;

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable;
}

impl DepthRenderSlot for () {
  type DepthRenderLayer<D> = () where D: Dimensionable;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType> = None;

  unsafe fn new_depth_render_layer<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

impl<DS> DepthRenderSlot for Back<DS>
where
  DS: DepthRenderSlot,
{
  type DepthRenderLayer<D> = () where D: Dimensionable;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType> = None;

  unsafe fn new_depth_render_layer<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
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
  type DepthRenderLayer<D> = RenderLayer<D, RC> where D: Dimensionable;

  const DEPTH_CHANNEL_TY: Option<DepthChannelType> = Some(RC::CHANNEL_TY);

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(backend.new_depth_render_layer(framebuffer_handle, size, mipmaps)?)
  }
}
