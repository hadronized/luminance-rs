//! Framebuffers and utility types and functions.
//!
//! Framebuffers are at the core of rendering. They’re the support of rendering operations and can
//! be used to highly enhance the visual aspect of a render. You’re always provided with at least
//! one framebuffer, `Framebuffer::back_buffer`. That function returns a framebuffer that represents –
//! for short – the current back framebuffer. You can render to that framebuffer and when you
//! *swap* the buffers, your render appears in the front framebuffer (likely your screen).
//!
//! # Framebuffers
//!
//! A framebuffer is an object maintaining the required GPU state to hold images you render to. It
//! gathers two important concepts:
//!
//!   - *Color buffers*.
//!   - *Depth buffers*.
//!
//! The *color buffers* hold the color images you render to. A framebuffer can hold several of them
//! with different color formats. The *depth buffers* hold the depth images you render to.
//! Framebuffers can hold only one depth buffer.
//!
//! # Framebuffer slots
//!
//! A framebuffer slot contains either its color buffers or its depth buffer. Sometimes, you might
//! find it handy to have no slot at all for a given type of buffer. In that case, we use `()`.
//!
//! The slots are a way to convert the different formats you use for your framebuffers’ buffers into
//! their respective texture representation so that you can handle the corresponding texels.
//!
//! Color buffers are abstracted by `ColorSlot` and the depth buffer by `DepthSlot`.

use crate::context::GraphicsContext;
use crate::pixel::{ColorPixel, DepthPixel, PixelFormat, RenderablePixel};
use crate::texture::{Dim2, Dimensionable, Layerable};

/// Framebuffer with static layering, dimension, access and slots formats.
//
/// A `Framebuffer` is a *GPU* special object used to render to. Because framebuffers have a
/// *layering* property, it’s possible to have regular render and *layered rendering*. The dimension
/// of a framebuffer makes it possible to render to 1D, 2D, 3D and cubemaps.
///
/// A framebuffer has two kind of slots:
///
/// - **color slot** ;
/// - **depth slot**.
///
/// A framebuffer can have zero or several color slots and it can have zero or one depth slot. If
/// you use several color slots, you’ll be performing what’s called *MRT* (*M* ultiple *R* ender
/// *T* argets), enabling to render to several textures at once.
pub trait Framebuffer<S, L, D>: Sized
where
  L: Layerable,
  D: Dimensionable,
{
  type BackBuffer;

  type Textures;

  type ColorSlot: ColorSlot<S, L, D, Self::Textures>;

  type DepthSlot: DepthSlot<S, L, D, Self::Textures>;

  type Err;

  /// Get the back buffer with the given dimension.
  fn back_buffer<C>(
    ctx: &mut C,
    size: <Dim2 as Dimensionable>::Size,
  ) -> Result<Self::BackBuffer, Self::Err>
  where
    C: GraphicsContext<State = S>;

  /// Create a new framebuffer.
  ///
  /// You’re always handed at least the base level of the texture. If you require any *additional*
  /// levels, you can pass the number via the `mipmaps` parameter.
  fn new<C>(ctx: &mut C, size: D::Size, mipmaps: usize) -> Result<Self, Self::Err>
  where
    C: GraphicsContext<State = S>;

  /// Dimension of the framebuffer.
  fn dimension(&self) -> D::Size;

  /// Access the underlying color slot.
  fn color_slot(&self) -> &Self::ColorSlot;

  /// Access the underlying depth slot.
  fn depth_slot(&self) -> &Self::DepthSlot;
}

pub trait ColorSlot<S, L, D, I>
where
  L: Layerable,
  D: Dimensionable,
{
  type ColorTextures;

  const COLOR_FORMATS: &'static [PixelFormat];

  /// Reify a list of raw textures.
  fn reify_textures<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    textures: &mut I,
  ) -> Self::ColorTextures
  where
    C: GraphicsContext<State = S>;
}

impl<S, L, D, I> ColorSlot<S, L, D, I> for ()
where
  L: Layerable,
  D: Dimensionable,
{
  type ColorTextures = ();

  const COLOR_FORMATS: &'static [PixelFormat] = &[];

  fn reify_textures<C>(_: &mut C, _: D::Size, _: usize, _: &mut I) -> Self::ColorTextures
  where
    C: GraphicsContext<State = S>,
  {
    ()
  }
}

impl<S, L, D, I, P> ColorSlot<S, L, D, I> for P
where
  L: Layerable,
  D: Dimensionable,
  Self: ColorPixel + RenderablePixel + ReifyTexture<S, L, D, I>,
{
  type ColorTextures = <Self as ReifyTexture<S, L, D, I>>::Texture;

  const COLOR_FORMATS: &'static [PixelFormat] = &[Self::PIXEL_FORMAT];

  fn reify_textures<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    textures: &mut I,
  ) -> Self::ColorTextures
  where
    C: GraphicsContext<State = S>,
  {
    Self::reify_texture(ctx, size, mipmaps, textures)
  }
}

macro_rules! impl_color_slot_tuple {
  ($($pf:ident),*) => {
    impl<S, L, D, I, $($pf),*> ColorSlot<S, L, D, I> for ($($pf),*)
    where
      L: Layerable,
      D: Dimensionable,
      $(
        $pf: ColorPixel + RenderablePixel + ReifyTexture<S, L, D, I>
      ),* {
      type ColorTextures = ($(<$pf as ReifyTexture<S, L, D, I>>::Texture),*);

      const COLOR_FORMATS: &'static [PixelFormat] = &[$($pf::PIXEL_FORMAT),*];

      fn reify_textures<C>(
        ctx: &mut C,
        size: D::Size,
        mipmaps: usize,
        textures: &mut I,
      ) -> Self::ColorTextures
      where C: GraphicsContext<State = S> {
        ( $( $pf::reify_texture(ctx, size, mipmaps, textures) ),* )
      }
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

impl_color_slot_tuples!(PF, PE, PD, PC, PB, PA, P9, P8, P7, P6, P5, P4, P3, P2, P1, P0);

pub trait DepthSlot<S, L, D, I>
where
  L: Layerable,
  D: Dimensionable,
{
  type DepthTexture;

  const DEPTH_FORMAT: Option<PixelFormat>;
}

impl<S, L, D, I> DepthSlot<S, L, D, I> for ()
where
  L: Layerable,
  D: Dimensionable,
{
  type DepthTexture = ();

  const DEPTH_FORMAT: Option<PixelFormat> = None;
}

impl<S, L, D, I, P> DepthSlot<S, L, D, I> for P
where
  L: Layerable,
  D: Dimensionable,
  Self: DepthPixel + ReifyTexture<S, L, D, I>,
{
  type DepthTexture = <Self as ReifyTexture<S, L, D, I>>::Texture;

  const DEPTH_FORMAT: Option<PixelFormat> = Some(Self::PIXEL_FORMAT);
}

pub trait ReifyTexture<S, L, D, I>
where
  L: Layerable,
  D: Dimensionable,
{
  type Texture;

  fn reify_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    textures: &mut I,
  ) -> Self::Texture
  where
    C: GraphicsContext<State = S>;
}
