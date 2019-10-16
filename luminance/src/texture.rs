//! This module provides texture features.
//!
//! # Introduction to textures
//!
//! Textures are used intensively in graphics programs as they tend to be the *de facto* memory area
//! to store data. You use them typically when you want to customize a render, hold a render’s
//! texels or even store arbritrary data.
//!
//! Currently, the following textures are supported:
//!
//! - 1D, 2D and 3D textures
//! - cubemaps
//! - array of textures (any of the types above)
//!
//! Those combinations are encoded by several types. First of all, `Texture<L, D, P>` is the
//! polymorphic type used to represent textures. The `L` type variable is the *layering type* of
//! the texture. It can either be `Flat` or `Layered`. The `D` type variable is the dimension of the
//! texture. It can either be `Dim1`, `Dim2`, `Dim3` or `Cubemap`. Finally, the `P` type variable
//! is the pixel format the texture follows. See the `pixel` module for further details about pixel
//! formats.
//!
//! Additionally, all textures have between 0 or several *mipmaps*. Mipmaps are additional layers of
//! texels used to perform trilinear filtering in most applications. Those are low-definition images
//! of the the base image used to smoothly interpolate texels when a projection kicks in. See
//! [this](https://en.wikipedia.org/wiki/Mipmap) for more insight.
//!
//! # Creating textures
//!
//! Textures are created by providing a size, the number of mipmaps that should be used and a
//! reference to a `Sampler` object. Up to now, textures and samplers form the same object – but
//! that might change in the future. Samplers are just a way to describe how texels will be fetched
//! from a shader.
//!
//! ## Associated types
//!
//! Because textures might have different shapes, the types of their sizes and offsets vary. You
//! have to look at the implementation of `Dimensionable::Size` and `Dimensionable::Offset` to know
//! which type you have to pass. For instance, for a 2D texture – e.g. `Texture<Flat, Dim2, _>`, you
//! have to pass a pair `(width, height)`.
//!
//! ## Samplers
//!
//! Samplers gather filters – i.e. how a shader should interpolate texels while fetching them,
//! wrap rules – i.e. how a shader should behave when leaving the normalized UV coordinates? and
//! a depth comparison, for depth textures only. See the documentation of `Sampler` for further
//! explanations.
//!
//! Samplers must be declared in the shader code according to the type of the texture used in the
//! Rust code. The size won’t matter, only the type.
//!
//! # Uploading data to textures
//!
//! One of the primary use of textures is to store images so that they can be used in your
//! application mapped on objects in your scene, for instance. In order to do so, you have to load
//! the image from the disk – see the awesome [image](https://crates.io/crates/image) – and then
//! upload the data to the texture. You have several functions to do so:
//!
//! - `Texture::upload`: this function takes a slice of texels and upload them to the whole texture memory
//! - `Texture::upload_part`: this function does the same thing as `Texture::upload`, but gives you the extra
//!   control on where in the texture you want to upload and with which size
//! - `Texture::upload_raw`: this function takes a slice of raw encoding data and upload them to the whole
//!   texture memory. This is especially handy when your texture has several channels but the data you have
//!   don’t take channels into account and are just *raw* data.
//! - `Texture::upload_part_raw`: same thing as above, but with offset and size control.
//!
//! Alternatively, you can clear the texture with `Texture::clear` and `Texture::clear_part`.
//!
//! # Retrieving texels
//!
//! The function `Texel::get_raw_texels` must be used to retreive texels out of a texture. This
//! function allocates memory, so be careful when using it.
//!
//! [`PixelFormat`]: crate::pixel::PixelFormat

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

pub use crate::depth_test::DepthComparison;
use crate::pixel::{Pixel, PixelFormat};

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug)]
pub enum Wrap {
  /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
  ///
  /// ```ignore
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between [0;1]
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
  MirroredRepeat,
}

/// Minification filter.
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub enum MagFilter {
  /// Nearest interpolation.
  Nearest,
  /// Linear interpolation between surrounding pixels.
  Linear,
}

/// Reify a type into a `Dim`.
pub trait Dimensionable {
  /// Size type of a dimension (used to caracterize dimensions’ areas).
  type Size: Copy;

  /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
  type Offset: Copy;

  /// Zero offset.
  const ZERO_OFFSET: Self::Offset;

  /// Dimension.
  fn dim() -> Dim;

  /// Width of the associated `Size`.
  fn width(size: Self::Size) -> u32;

  /// Height of the associated `Size`. If it doesn’t have one, set it to 1.
  fn height(_: Self::Size) -> u32 {
    1
  }

  /// Depth of the associated `Size`. If it doesn’t have one, set it to 1.
  fn depth(_: Self::Size) -> u32 {
    1
  }

  /// X offset.
  fn x_offset(offset: Self::Offset) -> u32;

  /// Y offset. If it doesn’t have one, set it to 0.
  fn y_offset(_: Self::Offset) -> u32 {
    1
  }

  /// Z offset. If it doesn’t have one, set it to 0.
  fn z_offset(_: Self::Offset) -> u32 {
    1
  }

  /// Amount of pixels this size represents.
  ///
  /// For 2D sizes, it represents the area; for 3D sizes, the volume; etc.
  /// For cubemaps, it represents the side length of the cube.
  fn count(size: Self::Size) -> usize;
}

/// Dimension of a texture.
#[derive(Clone, Copy, Debug)]
pub enum Dim {
  /// 1D.
  Dim1,
  /// 2D.
  Dim2,
  /// 3D.
  Dim3,
  /// Cubemap (i.e. a cube defining 6 faces — akin to 4D).
  Cubemap,
}

/// 1D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1;

impl Dimensionable for Dim1 {
  type Offset = u32;
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = 0;

  fn dim() -> Dim {
    Dim::Dim1
  }

  fn width(w: Self::Size) -> u32 {
    w
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off
  }

  fn count(size: Self::Size) -> usize {
    size as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Offset = [u32; 2];
  type Size = [u32; 2];

  const ZERO_OFFSET: Self::Offset = [0, 0];

  fn dim() -> Dim {
    Dim::Dim2
  }

  fn width(size: Self::Size) -> u32 {
    size[0]
  }

  fn height(size: Self::Size) -> u32 {
    size[1]
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off[1]
  }

  fn count([width, height]: Self::Size) -> usize {
    width as usize * height as usize
  }
}

/// 3D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Offset = [u32; 3];
  type Size = [u32; 3];

  const ZERO_OFFSET: Self::Offset = [0, 0, 0];

  fn dim() -> Dim {
    Dim::Dim3
  }

  fn width(size: Self::Size) -> u32 {
    size[0]
  }

  fn height(size: Self::Size) -> u32 {
    size[1]
  }

  fn depth(size: Self::Size) -> u32 {
    size[2]
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    off[2]
  }

  fn count([width, height, depth]: Self::Size) -> usize {
    width as usize * height as usize * depth as usize
  }
}

/// Cubemap dimension.
#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Offset = ([u32; 2], CubeFace);
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = ([0, 0], CubeFace::PositiveX);

  fn dim() -> Dim {
    Dim::Cubemap
  }

  fn width(s: Self::Size) -> u32 {
    s
  }

  fn height(s: Self::Size) -> u32 {
    s
  }

  fn depth(_: Self::Size) -> u32 {
    6
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.0[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    match off.1 {
      CubeFace::PositiveX => 0,
      CubeFace::NegativeX => 1,
      CubeFace::PositiveY => 2,
      CubeFace::NegativeY => 3,
      CubeFace::PositiveZ => 4,
      CubeFace::NegativeZ => 5,
    }
  }

  fn count(size: Self::Size) -> usize {
    let size = size as usize;
    size * size
  }
}

/// Faces of a cubemap.
#[derive(Clone, Copy, Debug)]
pub enum CubeFace {
  /// The +X face of the cube.
  PositiveX,
  /// The -X face of the cube.
  NegativeX,
  /// The +Y face of the cube.
  PositiveY,
  /// The -Y face of the cube.
  NegativeY,
  /// The +Z face of the cube.
  PositiveZ,
  /// The -Z face of the cube.
  NegativeZ,
}

/// Trait used to reify a type into a `Layering`.
pub trait Layerable {
  /// Reify to `Layering`.
  fn layering() -> Layering;
}

/// Texture layering. If a texture is layered, it has an extra coordinate to access the layer.
#[derive(Clone, Copy, Debug)]
pub enum Layering {
  /// Non-layered.
  Flat,
  /// Layered.
  Layered,
}

/// Flat texture hint.
///
/// A flat texture means it doesn’t have the concept of layers.
#[derive(Clone, Copy, Debug)]
pub struct Flat;

impl Layerable for Flat {
  fn layering() -> Layering {
    Layering::Flat
  }
}

/// Layered texture hint.
///
/// A layered texture has an extra coordinate to access the layer and can be thought of as an array
/// of textures.
#[derive(Clone, Copy, Debug)]
pub struct Layered;

impl Layerable for Layered {
  fn layering() -> Layering {
    Layering::Layered
  }
}


/// Whether mipmaps should be generated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GenMipmaps {
  /// Mipmaps should be generated.
  ///
  /// Mipmaps are generated when creating textures but also when uploading texels, clearing, etc.
  Yes,
  /// Never generate mipmaps.
  No
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
  pub min_filter: MinFilter,
  /// Magnification filter.
  pub mag_filter: MagFilter,
  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<DepthComparison>,
}

/// Default value is as following:
impl Default for Sampler {
  fn default() -> Self {
    Sampler {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      min_filter: MinFilter::NearestMipmapLinear,
      mag_filter: MagFilter::Linear,
      depth_comparison: None,
    }
  }
}
