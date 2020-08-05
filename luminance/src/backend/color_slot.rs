//! Color slot backend interface.
//!
//! This interface defines the low-level API color slots must implement to be usable.

use crate::backend::framebuffer::Framebuffer;
use crate::backend::texture::Texture as TextureBackend;
use crate::context::GraphicsContext;
use crate::framebuffer::FramebufferError;
use crate::pixel::{ColorPixel, PixelFormat, RenderablePixel};
use crate::texture::{Dimensionable, Sampler};

use crate::texture::Texture;

pub trait ColorSlot<B, D>
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  type ColorTextures;

  fn color_formats() -> Vec<PixelFormat>;

  fn reify_color_textures<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
    attachment_index: usize,
  ) -> Result<Self::ColorTextures, FramebufferError>
  where
    C: GraphicsContext<Backend = B>;
}

impl<B, D> ColorSlot<B, D> for ()
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  type ColorTextures = ();

  fn color_formats() -> Vec<PixelFormat> {
    Vec::new()
  }

  fn reify_color_textures<C>(
    _: &mut C,
    _: D::Size,
    _: usize,
    _: &Sampler,
    _: &mut B::FramebufferRepr,
    _: usize,
  ) -> Result<Self::ColorTextures, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    Ok(())
  }
}

impl<B, D, P> ColorSlot<B, D> for P
where
  B: ?Sized + Framebuffer<D> + TextureBackend<D, P>,
  D: Dimensionable,
  D::Size: Copy,
  Self: ColorPixel + RenderablePixel,
{
  type ColorTextures = Texture<B, D, P>;

  fn color_formats() -> Vec<PixelFormat> {
    vec![P::pixel_format()]
  }

  fn reify_color_textures<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
    attachment_index: usize,
  ) -> Result<Self::ColorTextures, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let texture = Texture::new(ctx, size, mipmaps, *sampler)?;
    unsafe { B::attach_color_texture(framebuffer, &texture.repr, attachment_index)? };

    Ok(texture)
  }
}

macro_rules! impl_color_slot_tuple {
  ($($pf:ident),*) => {
    impl<B, D, $($pf),*> ColorSlot<B, D> for ($($pf),*)
    where
      B: ?Sized + Framebuffer<D> + $(TextureBackend<D, $pf> +)*,
      D: Dimensionable,
      D::Size: Copy,
      $(
        $pf: ColorPixel + RenderablePixel
      ),*
    {
      type ColorTextures = ($(Texture<B, D, $pf>),*);

      fn color_formats() -> Vec<PixelFormat> {
        vec![$($pf::pixel_format()),*]

      }

      impl_reify_color_textures!{ $($pf),* }
    }
  }
}

// A small helper macro to implement reify_color_textures in impl_color_slot_tuple!.
//
// We need this macro so that we can implement the increment logic without having to do weird
// arithmetic at runtime or have dead code.
macro_rules! impl_reify_color_textures {
  ($pf:ident , $($pfr:ident),*) => {
    #[allow(clippy::eval_order_dependence)]
    fn reify_color_textures<C>(
      ctx: &mut C,
      size: D::Size,
      mipmaps: usize,
      sampler: &Sampler,
      framebuffer: &mut B::FramebufferRepr,
      mut attachment_index: usize,
    ) -> Result<Self::ColorTextures, FramebufferError>
    where
      C: GraphicsContext<Backend = B>,
    {
      let textures = (
        // first element of the tuple
        <$pf as ColorSlot<B, D>>::reify_color_textures(
          ctx,
          size,
          mipmaps,
          sampler,
          framebuffer,
          attachment_index,
        )?,
        // rest of the tuple
        $({
          attachment_index += 1;
          let texture = <$pfr as ColorSlot<B, D>>::reify_color_textures(
            ctx,
            size,
            mipmaps,
            sampler,
            framebuffer,
            attachment_index,
          )?;

          texture
        }),*
      );

      Ok(textures)
    }
  }
}

macro_rules! impl_color_slot_tuples {
  ($first:ident , $second:ident) => {
    // stop at pairs
    impl_color_slot_tuple!($first, $second);
  };

  ($first:ident , $($pf:ident),*) => {
    // implement the same list without the first type (reduced by one)
    impl_color_slot_tuples!($($pf),*);
    // implement the current list
    impl_color_slot_tuple!($first, $($pf),*);
  };
}

impl_color_slot_tuples!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
