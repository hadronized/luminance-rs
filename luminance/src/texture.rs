use crate::{depth_stencil::Comparison, dim::Dimensionable};
use std::marker::PhantomData;

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wrap {
  /// If textures coordinates lay outside of `[0;1]`, they will be clamped to either `0` or `1` for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of `[0;1]`. Picture this as:
  ///
  /// ```ignore
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between `[0;1]`
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between `[0;1]` and `[1;0]`.
  MirroredRepeat,
}

/// Minification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinFilter {
  /// Nearest interpolation.
  Nearest,
  /// Linear interpolation between surrounding pixels.
  Linear,
  /// This filter will select the nearest mipmap between two samples and will perform a nearest
  /// interpolation afterwards.
  NearestMipmapNearest,
  /// This filter will select the nearest mipmap between two samples and will perform a linear
  /// interpolation afterwards.
  NearestMipmapLinear,
  /// This filter will linearly interpolate between two mipmaps, which selected texels would have
  /// been interpolated with a nearest filter.
  LinearMipmapNearest,
  /// This filter will linearly interpolate between two mipmaps, which selected texels would have
  /// been linarily interpolated as well.
  LinearMipmapLinear,
}

/// Magnification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MagFilter {
  /// Nearest interpolation.
  Nearest,
  /// Linear interpolation between surrounding pixels.
  Linear,
}

/// A [`Sampler`] object gives hint on how a [`Texture`] should be sampled.
#[derive(Clone, Copy, Debug)]
pub struct TextureSampling {
  /// How should we wrap around the *r* sampling coordinate?
  pub wrap_r: Wrap,

  /// How should we wrap around the *s* sampling coordinate?
  pub wrap_s: Wrap,

  /// How should we wrap around the *t* sampling coordinate?
  pub wrap_t: Wrap,

  /// Minification filter.
  pub min_filter: MinFilter,

  /// Magnification filter.
  pub mag_filter: MagFilter,

  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<Comparison>,
}

impl Default for TextureSampling {
  fn default() -> Self {
    TextureSampling {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      min_filter: MinFilter::Linear,
      mag_filter: MagFilter::Linear,
      depth_comparison: None,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mipmaps {
  No,
  Yes { count: usize },
}

impl Mipmaps {
  pub fn count(count: usize) -> Self {
    Mipmaps::Yes { count }
  }
}

pub struct Texture<D, P>
where
  D: Dimensionable,
{
  handle: usize,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (D, P)>,
}

impl<D, P> Texture<D, P>
where
  D: Dimensionable,
{
  pub unsafe fn new(handle: usize, dropper: Box<dyn FnMut(usize)>) -> Self {
    Self {
      handle,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

impl<D, P> Drop for Texture<D, P>
where
  D: Dimensionable,
{
  fn drop(&mut self) {
    (self.dropper)(self.handle)
  }
}

pub struct InUseTexture<D, S> {
  handle: usize,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (D, S)>,
}

impl<D, S> InUseTexture<D, S> {
  pub unsafe fn new(handle: usize, dropper: Box<dyn FnMut(usize)>) -> Self {
    Self {
      handle,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}

impl<D, S> Drop for InUseTexture<D, S> {
  fn drop(&mut self) {
    (self.dropper)(self.handle)
  }
}
