//! Color slot backend interface.
//!
//! This interface defines the low-level API color slots must implement to be usable.
//!
//! # Note to backend contributors
//!
//! If you are implementing a backend, there is nothing here to implement, but you will likely want to use some traits /
//! types from this module to use as constraint.

use crate::{
  backend::{framebuffer::Framebuffer, texture::Texture as TextureBackend},
  context::GraphicsContext,
  framebuffer::FramebufferError,
  texture::{TexelUpload, Texture},
};
use crate::{
  pixel::{ColorPixel, PixelFormat, RenderablePixel},
  texture::{Dimensionable, Sampler},
};

/// A color slot.
///
/// A color slot represents the associated _color data_ within a [`Framebuffer`]. This type is entirely constructed at
/// compile-time to ensure type safety. Even though this trait lives on the backend side of luminance, no backend is
/// supposed to implement it, but instead use it.
///
/// Three types of color slots exist:
///
/// - None, represented by the `()` implementor.
/// - A single color [`Texture`]. This type of color slot is often suitable for renderable framebuffer.
/// - A tuple of different color [`Texture`]. This situation is mostly used for _multi render target_, allowing to
///   render (via a fragment shader) into different part of the color slot.
///
/// For color slots that have color textures, the pixel type must be a [`RenderablePixel`] as well as a [`ColorPixel`].
///
/// Feel free to have a look at the list of implementors of this trait to know which types you can use as color slots.
pub trait ColorSlot<B, D>
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  /// The associated data.
  ///
  /// This type represents the available, constructed object that will be usable outside of the [`Framebuffer`]. There
  /// is no trait to implement on this type but you mostly want to map a single [`Texture`] or a tuple of texture by
  /// using this as a type family for your implementor type.
  type ColorTextures;

  /// Pixel format representing the color slot.
  ///
  /// Those [`PixelFormat`] represent the format of each part of the color slot.
  fn color_formats() -> Vec<PixelFormat>;

  /// Reify the color slots into 0, 1 or several textures.
  ///
  /// This function must construct and initialize all the required textures.
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
    let texture = Texture::new(ctx, size, sampler.clone(), TexelUpload::reserve(mipmaps))?;

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
