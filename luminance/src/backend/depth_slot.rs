//! Depth slot backend interface.
//!
//! This interface defines the low-level API depth slots must implement to be usable.

use crate::backend::framebuffer::Framebuffer;
use crate::backend::texture::Texture as TextureBackend;
use crate::context::GraphicsContext;
use crate::framebuffer::FramebufferError;
use crate::pixel::{DepthPixel, PixelFormat};
use crate::texture::{Dimensionable, Sampler};

use crate::texture::Texture;

pub trait DepthSlot<B, D>
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  /// Texture associated with this color slot.
  type DepthTexture;

  /// Turn a depth slot into a pixel format.
  fn depth_format() -> Option<PixelFormat>;

  /// Reify a raw textures into a depth slot.
  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>;
}

impl<B, D> DepthSlot<B, D> for ()
where
  B: ?Sized + Framebuffer<D>,
  D::Size: Copy,
  D: Dimensionable,
{
  type DepthTexture = ();

  fn depth_format() -> Option<PixelFormat> {
    None
  }

  fn reify_depth_texture<C>(
    _: &mut C,
    _: D::Size,
    _: usize,
    _: &Sampler,
    _: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    Ok(())
  }
}

impl<B, D, P> DepthSlot<B, D> for P
where
  B: ?Sized + Framebuffer<D> + TextureBackend<D, P>,
  D: Dimensionable,
  D::Size: Copy,
  P: DepthPixel,
{
  type DepthTexture = Texture<B, D, P>;

  fn depth_format() -> Option<PixelFormat> {
    Some(P::pixel_format())
  }

  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let texture = Texture::new(ctx, size, mipmaps, *sampler)?;
    unsafe { B::attach_depth_texture(framebuffer, &texture.repr)? };

    Ok(texture)
  }
}
