//! This module provides texture features.

use std::marker::PhantomData;
use pixel::Pixel;

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug)]
pub enum Wrap {
  /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
  ///
  /// ```
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between [0;1]
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
  MirroredRepeat
}

/// Minification and magnification filter.
#[derive(Clone, Copy, Debug)]
pub enum Filter {
  /// Clamp to nearest pixel.
  Nearest,
  /// Linear interpolation with surrounding pixels.
  Linear
}

/// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
/// fragment’s depth that is already stored.
#[derive(Clone, Copy, Debug)]
pub enum DepthComparison {
  /// Depth test never succeeds.
  Never,
  /// Depth test always succeeds.
  Always,
  /// Depth test succeeds if `a == b`.
  Equal,
  /// Depth test succeeds if `a != b`.
  NotEqual,
  /// Depth test succeeds if `a < b`.
  Less,
  /// Depth test succeeds if `a <= b`.
  LessOrEqual,
  /// Depth test succeeds if `a > b`.
  Greater,
  /// Depth test succeeds if `a >= b`.
  GreaterOrEqual
}

/// Reify a type into a `Dim`.
pub trait Dimensionable {
  type Size;
  type Offset;

  /// Dimension.
  fn dim() -> Dim;
  /// Width of the associated `Size`.
  fn width(size: Self::Size) -> u32 where Self::Size: Copy;
  /// Height of the associated `Size`. If it doesn’t have one, set it to 1.
  fn height(_: Self::Size) -> u32 where Self::Size: Copy { 1 }
  /// Depth of the associated `Size`. If it doesn’t have one, set it to 1.
  fn depth(_: Self::Size) -> u32 where Self::Size: Copy { 1 }
  /// X offset.
  fn x_offset(offset: Self::Offset) -> u32 where Self::Offset: Copy;
  /// Y offset. If it doesn’t have one, set it to 0.
  fn y_offset(_: Self::Offset) -> u32 where Self::Offset: Copy { 1 }
  /// Z offset. If it doesn’t have one, set it to 0.
  fn z_offset(_: Self::Offset) -> u32 where Self::Offset: Copy { 1 }
  /// Zero offset.
  fn zero_offset() -> Self::Offset;
}

pub fn dim_capacity<D>(size: D::Size) -> u32 where D: Dimensionable, D::Size: Copy {
  D::width(size) * D::height(size) * D::depth(size)
}

/// Dimension of a texture.
#[derive(Clone, Copy, Debug)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Cubemap
}

#[derive(Clone, Copy, Debug)]
pub struct Dim1;

impl Dimensionable for Dim1 {
  type Size = u32;
  type Offset = u32;

  fn dim() -> Dim { Dim::Dim1 }

  fn width(w: Self::Size) -> u32 { w }

  fn x_offset(off: Self::Offset) -> u32 { off }

  fn zero_offset() -> Self::Offset { 0 }
}

#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Size = (u32, u32);
  type Offset = (u32, u32);

  fn dim() -> Dim { Dim::Dim2 }

  fn width(size: Self::Size) -> u32 { size.0 }

  fn height(size: Self::Size) -> u32 { size.1 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn zero_offset() -> Self::Offset { (0, 0) }
}

#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Size = (u32, u32, u32);
  type Offset = (u32, u32, u32);

  fn dim() -> Dim { Dim::Dim3 }

  fn width(size: Self::Size) -> u32 { size.0 }

  fn height(size: Self::Size) -> u32 { size.1 }

  fn depth(size: Self::Size) -> u32 { size.2 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn z_offset(off: Self::Offset) -> u32 { off.2 }

  fn zero_offset() -> Self::Offset { (0, 0, 0) }
}

#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Size = u32;
  type Offset = (u32, u32, CubeFace);

  fn dim() -> Dim { Dim::Cubemap }

  fn width(s: Self::Size) -> u32 { s }

  fn height(s: Self::Size) -> u32 { s }

  fn depth(_: Self::Size) -> u32 { 6 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn z_offset(off: Self::Offset) -> u32 {
    match off.2 {
      CubeFace::PositiveX => 0,
      CubeFace::NegativeX => 1,
      CubeFace::PositiveY => 2,
      CubeFace::NegativeY => 3,
      CubeFace::PositiveZ => 4,
      CubeFace::NegativeZ => 5
    }
  }

  fn zero_offset() -> Self::Offset { (0, 0, CubeFace::PositiveX) }
}

/// Faces of a cubemap.
#[derive(Clone, Copy, Debug)]
pub enum CubeFace {
  PositiveX,
  NegativeX,
  PositiveY,
  NegativeY,
  PositiveZ,
  NegativeZ
}

/// Reify a type into a `Layering`.
pub trait Layerable {
  fn layering() -> Layering;
}

/// Texture layering. If a texture is layered, it has an extra coordinates to access the layer.
#[derive(Clone, Copy, Debug)]
pub enum Layering {
  /// Non-layered.
  Flat,
  /// Layered.
  Layered
}

#[derive(Clone, Copy, Debug)]
pub struct Flat;

impl Layerable for Flat { fn layering() -> Layering { Layering::Flat } }

#[derive(Clone, Copy, Debug)]
pub struct Layered;

impl Layerable for Layered { fn layering() -> Layering { Layering::Layered } }

/// Trait to implement to provide texture features.
pub trait HasTexture {
  type ATexture;

  /// Create a new texture.
  ///
  /// `size` is a value used to specify the dimension of the texture. `mipmaps` is the number of
  /// extra *mipmaps* you want to have. If you set this value to `0`, you end up with only one level
  /// (the base level) of texture storage.
  fn new_texture<L, D, P>(size: D::Size, mipmaps: usize, sampler: &Sampler) -> Self::ATexture
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel;
  /// Destroy a texture.
  fn free(tex: &mut Self::ATexture);
  /// Clear the texture’s texels by setting them all to the same value.
  fn clear_part<L, D, P>(tex: &Self::ATexture, gen_mimpmaps: bool, offset: D::Offset, size: D::Size, pixel: P::Encoding)
    where L: Layerable, D: Dimensionable, D::Offset: Copy, D::Size: Copy, P: Pixel, P::Encoding: Copy;
  /// Upload texels to the texture’s memory.
  fn upload_part<L, D, P>(tex: &Self::ATexture, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::Encoding])
    where L: Layerable, D::Offset: Copy, D::Size: Copy, D: Dimensionable, P: Pixel;
  /// Upload raw texels to the texture’s memory.
  fn upload_part_raw<L, D, P>(tex: &Self::ATexture, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::RawEncoding])
    where L: Layerable, D::Offset: Copy, D::Size: Copy, D: Dimensionable, P: Pixel;
  /// Retrieve the texels as a collection of P::RawEncoding.
  fn get_raw_texels<P>(tex: &Self::ATexture) -> Vec<P::RawEncoding> where P: Pixel, P::RawEncoding: Copy;
}

/// Texture.
///
/// `L` refers to the layering type; `D` refers to the dimension; `P` is the pixel format for the
/// texels.
#[derive(Debug)]
pub struct Texture<C, L, D, P> where C: HasTexture, L: Layerable, D: Dimensionable, P: Pixel {
  pub repr: C::ATexture,
  pub size: D::Size,
  pub mipmaps: usize,
  _l: PhantomData<L>,
  _c: PhantomData<C>,
  _p: PhantomData<P>
}

impl<C, L, D, P> Drop for Texture<C, L, D, P> where C: HasTexture, L: Layerable, D: Dimensionable, P: Pixel {
  fn drop(&mut self) {
    C::free(&mut self.repr)
  }
}

impl<C, L, D, P> Texture<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  pub fn new(size: D::Size, mipmaps: usize, sampler: &Sampler) -> Self {
    let mipmaps = mipmaps + 1; // + 1 prevent having 0 mipmaps
    let tex = C::new_texture::<L, D, P>(size, mipmaps, sampler);

    Texture {
      repr: tex,
      size: size,
      mipmaps: mipmaps,
      _c: PhantomData,
      _l: PhantomData,
      _p: PhantomData
    }
  }

  pub fn from_raw(texture: C::ATexture, size: D::Size, mipmaps: usize) -> Self {
    Texture {
      repr: texture,
      size: size,
      mipmaps: mipmaps + 1,
      _c: PhantomData,
      _l: PhantomData,
      _p: PhantomData
    }
  }

  pub fn clear_part(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, pixel: P::Encoding)
      where D::Offset: Copy,
            D::Size: Copy,
            P::Encoding: Copy {
    C::clear_part::<L, D, P>(&self.repr, gen_mipmaps, offset, size, pixel)
  }

  pub fn clear(&self, gen_mipmaps: bool, pixel: P::Encoding)
      where D::Offset: Copy,
            D::Size: Copy,
            P::Encoding: Copy {
    self.clear_part(gen_mipmaps, D::zero_offset(), self.size, pixel)
  }

  pub fn upload_part(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::Encoding])
      where D::Offset: Copy,
            D::Size: Copy {
    C::upload_part::<L, D, P>(&self.repr, gen_mipmaps, offset, size, texels)
  }

  pub fn upload(&self, gen_mipmaps: bool, texels: &[P::Encoding])
      where D::Offset: Copy,
            D::Size: Copy {
    self.upload_part(gen_mipmaps, D::zero_offset(), self.size, texels)
  }

  pub fn upload_part_raw(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::RawEncoding])
      where D::Offset: Copy,
            D::Size: Copy {
    C::upload_part_raw::<L, D, P>(&self.repr, gen_mipmaps, offset, size, texels)
  }

  pub fn upload_raw(&self, gen_mipmaps: bool, texels: &[P::RawEncoding])
      where D::Offset: Copy,
            D::Size: Copy {
    self.upload_part_raw(gen_mipmaps, D::zero_offset(), self.size, texels)
  }

  pub fn get_raw_texels(&self) -> Vec<P::RawEncoding> where P: Pixel, P::RawEncoding: Copy {
    C::get_raw_texels::<P>(&self.repr)
  }
}

/// A `Sampler` object gives hint on how a `Texture` should be sampled.
#[derive(Clone, Copy, Debug)]
pub struct Sampler {
  /// How should we wrap around the *r* sampling coordinate?
  pub wrap_r: Wrap,
  /// How should we wrap around the *s* sampling coordinate?
  pub wrap_s: Wrap,
  /// How should we wrap around the *t* sampling coordinate?
  pub wrap_t: Wrap,
  /// Minification filter.
  pub minification: Filter,
  /// Magnification filter.
  pub magnification: Filter,
  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<DepthComparison>
}

/// Default value is as following:
///
/// ```
/// Sampler {
///   wrap_r: Wrap::ClampToEdge,
///   wrap_s: Wrap::ClampToEdge,
///   wrap_t: Wrap::ClampToEdge,
///   minification: Filter::Linear,
///   magnification: Filter::Linear,
///   depth_comparison: None
/// }
/// ```
impl Default for Sampler {
  fn default() -> Self {
    Sampler {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      minification: Filter::Linear,
      magnification: Filter::Linear,
      depth_comparison: None
    }
  }
}
