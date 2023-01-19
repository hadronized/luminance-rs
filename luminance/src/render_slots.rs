use crate::{
  backend::{FramebufferBackend, FramebufferError},
  dim::Dimensionable,
  framebuffer::Back,
  pixel::{
    Depth32F, Depth32FStencil8, NormR16I, NormR16UI, NormR32I, NormR32UI, NormR8I, NormR8UI,
    NormRG16I, NormRG16UI, NormRG32I, NormRG32UI, NormRG8I, NormRG8UI, NormRGB16I, NormRGB16UI,
    NormRGB32I, NormRGB32UI, NormRGB8I, NormRGB8UI, NormRGBA16I, NormRGBA16UI, NormRGBA32I,
    NormRGBA32UI, NormRGBA8I, NormRGBA8UI, Pixel, PixelFormat, R16I, R16UI, R32F, R32I, R32UI, R8I,
    R8UI, RG16I, RG16UI, RG32F, RG32I, RG32UI, RG8I, RG8UI, RGB16I, RGB16UI, RGB32F, RGB32I,
    RGB32UI, RGB8I, RGB8UI, RGBA16I, RGBA16UI, RGBA32F, RGBA32I, RGBA32UI, RGBA8I, RGBA8UI,
  },
  texture::{Mipmaps, Texture, TextureSampling},
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
    sampling: &TextureSampling,
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
    _: &TextureSampling,
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
    _: &TextureSampling,
  ) -> Result<Self::RenderLayers<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

pub trait CompatibleRenderSlots<S> {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderChannelDesc {
  pub name: &'static str,
  pub fmt: PixelFormat,
}

pub trait RenderChannel: Pixel {}

impl RenderChannel for R8I {}
impl RenderChannel for R8UI {}
impl RenderChannel for RG8I {}
impl RenderChannel for RG8UI {}
impl RenderChannel for RGB8I {}
impl RenderChannel for RGB8UI {}
impl RenderChannel for RGBA8I {}
impl RenderChannel for RGBA8UI {}
impl RenderChannel for NormR8I {}
impl RenderChannel for NormR8UI {}
impl RenderChannel for NormRG8I {}
impl RenderChannel for NormRG8UI {}
impl RenderChannel for NormRGB8I {}
impl RenderChannel for NormRGB8UI {}
impl RenderChannel for NormRGBA8I {}
impl RenderChannel for NormRGBA8UI {}

impl RenderChannel for R16I {}
impl RenderChannel for R16UI {}
impl RenderChannel for RG16I {}
impl RenderChannel for RG16UI {}
impl RenderChannel for RGB16I {}
impl RenderChannel for RGB16UI {}
impl RenderChannel for RGBA16I {}
impl RenderChannel for RGBA16UI {}
impl RenderChannel for NormR16I {}
impl RenderChannel for NormR16UI {}
impl RenderChannel for NormRG16I {}
impl RenderChannel for NormRG16UI {}
impl RenderChannel for NormRGB16I {}
impl RenderChannel for NormRGB16UI {}
impl RenderChannel for NormRGBA16I {}
impl RenderChannel for NormRGBA16UI {}

impl RenderChannel for R32I {}
impl RenderChannel for R32UI {}
impl RenderChannel for R32F {}
impl RenderChannel for RG32I {}
impl RenderChannel for RG32UI {}
impl RenderChannel for RG32F {}
impl RenderChannel for RGB32I {}
impl RenderChannel for RGB32UI {}
impl RenderChannel for RGB32F {}
impl RenderChannel for RGBA32I {}
impl RenderChannel for RGBA32UI {}
impl RenderChannel for RGBA32F {}
impl RenderChannel for NormR32I {}
impl RenderChannel for NormR32UI {}
impl RenderChannel for NormRG32I {}
impl RenderChannel for NormRG32UI {}
impl RenderChannel for NormRGB32I {}
impl RenderChannel for NormRGB32UI {}
impl RenderChannel for NormRGBA32I {}
impl RenderChannel for NormRGBA32UI {}

pub trait DepthRenderSlot {
  type DepthRenderLayer<D>
  where
    D: Dimensionable;

  const DEPTH_CHANNEL_FMT: Option<PixelFormat>;

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable;
}

impl DepthRenderSlot for () {
  type DepthRenderLayer<D> = () where D: Dimensionable;

  const DEPTH_CHANNEL_FMT: Option<PixelFormat> = None;

  unsafe fn new_depth_render_layer<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
    _: &TextureSampling,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

impl<RS> DepthRenderSlot for Back<RS>
where
  RS: DepthRenderSlot,
{
  type DepthRenderLayer<D> = () where D: Dimensionable;

  const DEPTH_CHANNEL_FMT: Option<PixelFormat> = None;

  unsafe fn new_depth_render_layer<B, D>(
    _: &mut B,
    _: usize,
    _: D::Size,
    _: Mipmaps,
    _: &TextureSampling,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(())
  }
}

impl<P> DepthRenderSlot for P
where
  P: DepthChannel,
{
  type DepthRenderLayer<D> = Texture<D, P> where D: Dimensionable;

  const DEPTH_CHANNEL_FMT: Option<PixelFormat> = Some(P::PIXEL_FMT);

  unsafe fn new_depth_render_layer<B, D>(
    backend: &mut B,
    framebuffer_handle: usize,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: &TextureSampling,
  ) -> Result<Self::DepthRenderLayer<D>, FramebufferError>
  where
    B: FramebufferBackend,
    D: Dimensionable,
  {
    Ok(backend.new_depth_render_layer(framebuffer_handle, size, mipmaps, sampling)?)
  }
}
pub trait DepthChannel: Pixel {}

impl DepthChannel for Depth32F {}
impl DepthChannel for Depth32FStencil8 {}
