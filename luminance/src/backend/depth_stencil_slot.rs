//! Depth & stencil slot backend interface.
//!
//! This interface defines the low-level API depth slots must implement to be usable. This kind of slot also shares
//! stencil information, so depending on the kind of depth slot, you will either get a depth slot or a depth/stencil
//! slot, as both are tightly related.
//!
//! # Note to backend contributors
//!
//! If you are implementing a backend, there is nothing here to implement, but you will likely want to use some traits /
//! types from this module to use as constraint.

use crate::{
  backend::{framebuffer::Framebuffer, texture::Texture as TextureBackend},
  context::GraphicsContext,
  framebuffer::FramebufferError,
  pixel::{Depth32F, Depth32FStencil8, Pixel as _, PixelFormat},
  texture::{Dimensionable, Sampler, TexelUpload, Texture},
};

/// A depth/stencil slot.
///
/// A depth/stencil slot represents the associated _depth/stencil data_ within a [`Framebuffer`]. This type is entirely
/// constructed at compile-time to ensure type safety. Even though this trait lives on the backend side of luminance,
/// no backend is supposed to implement it, but instead use it.
///
/// Several types of depth/stencil slots exist:
///
/// - None, represented by the `()` implementor. This means that no depth and no stencil information will be available
///   for the framebuffer.
/// - A single depth [`Texture`]. This type of depth/stenccil slot is often suitable for renderable framebuffer. The
///   pixel format must implement [`DepthPixel`].
/// - A combined depth/stencil [`Texture`], allowing to use a depth buffer along with a stencil buffer.
///
/// Feel free to have a look at the list of implementors of this trait to know which types you can use as depth and
/// stencil slots.
pub trait DepthStencilSlot<B, D>
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  /// Depth data associated with this slot.
  type DepthStencilTexture;

  /// Turn a depth/stencil slot into a pixel format representing the depth information.
  fn depth_format() -> Option<PixelFormat>;

  /// Reify a raw texture into a depth slot.
  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthStencilTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>;
}

impl<B, D> DepthStencilSlot<B, D> for ()
where
  B: ?Sized + Framebuffer<D>,
  D::Size: Copy,
  D: Dimensionable,
{
  type DepthStencilTexture = ();

  fn depth_format() -> Option<PixelFormat> {
    None
  }

  fn reify_depth_texture<C>(
    _: &mut C,
    _: D::Size,
    _: usize,
    _: &Sampler,
    _: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthStencilTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    Ok(())
  }
}

impl<B, D> DepthStencilSlot<B, D> for Depth32F
where
  B: ?Sized + Framebuffer<D> + TextureBackend<D, Depth32F>,
  D: Dimensionable,
  D::Size: Copy,
{
  type DepthStencilTexture = Texture<B, D, Depth32F>;

  fn depth_format() -> Option<PixelFormat> {
    Some(Depth32F::pixel_format())
  }

  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthStencilTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let texture = Texture::new(ctx, size, sampler.clone(), TexelUpload::reserve(mipmaps))?;
    unsafe { B::attach_depth_texture(framebuffer, &texture.repr)? };

    Ok(texture)
  }
}

impl<B, D> DepthStencilSlot<B, D> for Depth32FStencil8
where
  B: ?Sized + Framebuffer<D> + TextureBackend<D, Depth32FStencil8>,
  D: Dimensionable,
  D::Size: Copy,
{
  type DepthStencilTexture = Texture<B, D, Depth32FStencil8>;

  fn depth_format() -> Option<PixelFormat> {
    Some(Depth32F::pixel_format())
  }

  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthStencilTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let texture = Texture::new(ctx, size, sampler.clone(), TexelUpload::reserve(mipmaps))?;
    unsafe { B::attach_depth_texture(framebuffer, &texture.repr)? };

    Ok(texture)
  }
}
