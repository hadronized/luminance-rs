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
//! Rust code. The size won’t matter, only the type. Here’s an exhaustive type of which sampler type
//! you must use according to the type of pixel format ([`PixelFormat`]) you use:
//!
//! > The `*` must be replaced by the dimension you use for your texture. If you use `Dim2` for
//! > instance, replace with `2`, as in `sampler*D -> sampler2D`.
//!
//! | `PixelFormat` | GLSL sampler type |
//! |---------------|-------------------|
//! | `R8I`         | `isampler*D`      |
//! | `R8UI`        | `usampler*D`      |
//! | `R16I`        | `isampler*D`      |
//! | `R16UI`       | `usampler*D`      |
//! | `R32I`        | `isampler*D`      |
//! | `R32UI`       | `usampler*D`      |
//! | `R32F`        | `sampler*D`       |
//! | `RG8I`        | `isampler*D`      |
//! | `RG8UI`       | `usampler*D`      |
//! | `RG16I`       | `isampler*D`      |
//! | `RG16UI`      | `usampler*D`      |
//! | `RG32I`       | `isampler*D`      |
//! | `RG32UI`      | `usampler*D`      |
//! | `RG32F`       | `sampler*D`       |
//! | `RGB8I`       | `isampler*D`      |
//! | `RGB8UI`      | `usampler*D`      |
//! | `RGB16I`      | `isampler*D`      |
//! | `RGB16UI`     | `usampler*D`      |
//! | `RGB32I`      | `isampler*D`      |
//! | `RGB32UI`     | `usampler*D`      |
//! | `RGB32F`      | `sampler*D`       |
//! | `RGBA8I`      | `isampler*D`      |
//! | `RGBA8UI`     | `usampler*D`      |
//! | `RGBA16I`     | `isampler*D`      |
//! | `RGBA16UI`    | `usampler*D`      |
//! | `RGBA32I`     | `isampler*D`      |
//! | `RGBA32UI`    | `usampler*D`      |
//! | `RGBA32F`     | `sampler*D`       |
//! | `Depth32F`    | `sampler1D`       |
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

#[cfg(feature = "std")]
use std::cell::RefCell;
#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::marker::PhantomData;
#[cfg(feature = "std")]
use std::mem;
#[cfg(feature = "std")]
use std::ops::{Deref, DerefMut};
#[cfg(feature = "std")]
use std::os::raw::c_void;
#[cfg(feature = "std")]
use std::ptr;
#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::cell::RefCell;
#[cfg(not(feature = "std"))]
use core::fmt::{self, Write};
#[cfg(not(feature = "std"))]
use core::marker::PhantomData;
#[cfg(not(feature = "std"))]
use core::mem;
#[cfg(not(feature = "std"))]
use core::ops::{Deref, DerefMut};
#[cfg(not(feature = "std"))]
use core::ptr;

use crate::context::GraphicsContext;
use crate::metagl::*;
use crate::pixel::{opengl_pixel_format, pixel_components, Pixel, PixelFormat};
use crate::state::GraphicsState;

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
  GreaterOrEqual,
}

/// Reify a type into a `Dim`.
pub trait Dimensionable {
  /// Size type of a dimension (used to caracterize dimensions’ areas).
  type Size: Copy;

  /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
  type Offset: Copy;

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

  /// Zero offset.
  const ZERO_OFFSET: Self::Offset;
}

// Capacity of the dimension, which is the product of the width, height and depth.
fn dim_capacity<D>(size: D::Size) -> u32 where D: Dimensionable {
  D::width(size) * D::height(size) * D::depth(size)
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

  fn dim() -> Dim {
    Dim::Dim1
  }

  fn width(w: Self::Size) -> u32 {
    w
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off
  }

  const ZERO_OFFSET: Self::Offset = 0;
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Offset = [u32; 2];
  type Size = [u32; 2];

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

  const ZERO_OFFSET: Self::Offset = [0, 0];
}

/// 3D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Offset = [u32; 3];
  type Size = [u32; 3];

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

  const ZERO_OFFSET: Self::Offset = [0, 0, 0];
}

/// Cubemap dimension.
#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Offset = ([u32; 2], CubeFace);
  type Size = u32;

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

  const ZERO_OFFSET: Self::Offset = ([0, 0], CubeFace::PositiveX);
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

/// Raw buffer. Any buffer can be converted to that type. However, keep in mind that even though
/// type erasure is safe, creating a buffer from a raw buffer is not.
pub struct RawTexture {
  handle: GLuint, // handle to the GPU texture object
  target: GLenum, // “type” of the texture; used for bindings
  state: Rc<RefCell<GraphicsState>>,
}

impl RawTexture {
  pub(crate) unsafe fn new(
    state: Rc<RefCell<GraphicsState>>,
    handle: GLuint,
    target: GLenum
  ) -> Self {
    RawTexture {
      handle,
      target,
      state,
    }
  }

  #[inline]
  pub(crate) fn handle(&self) -> GLuint {
    self.handle
  }

  #[inline]
  pub(crate) fn target(&self) -> GLenum {
    self.target
  }
}

/// Texture.
///
/// `L` refers to the layering type; `D` refers to the dimension; `P` is the pixel format for the
/// texels.
pub struct Texture<L, D, P>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  raw: RawTexture,
  size: D::Size,
  mipmaps: usize, // number of mipmaps
  _l: PhantomData<L>,
  _p: PhantomData<P>,
}

impl<L, D, P> Deref for Texture<L, D, P>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  type Target = RawTexture;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

impl<L, D, P> DerefMut for Texture<L, D, P>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.raw
  }
}

impl<L, D, P> Drop for Texture<L, D, P>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  fn drop(&mut self) {
    unsafe { gl::DeleteTextures(1, &self.handle) }
  }
}

impl<L, D, P> Texture<L, D, P>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  /// Create a new texture.
  ///
  ///   - The `mipmaps` parameter must be set to `0` if you want only one “layer of texels”.
  ///     creating a texture without any layer wouldn’t make any sense, so if you want three layers,
  ///     you will want the _base_ layer plus two mipmaps layers: you will then pass `2` as value
  ///     here.
  ///   - The `sampler` parameter allows to customize the way the texture will be sampled in
  ///     shader stages. Refer to the documentation of [`Sampler`] for further details.
  pub fn new<C>(ctx: &mut C, size: D::Size, mipmaps: usize, sampler: &Sampler) -> Result<Self, TextureError>
  where C: GraphicsContext {
    let mipmaps = mipmaps + 1; // + 1 prevent having 0 mipmaps
    let mut texture = 0;
    let target = opengl_target(L::layering(), D::dim());

    unsafe {
      gl::GenTextures(1, &mut texture);
      ctx.state().borrow_mut().bind_texture(target, texture);

      create_texture::<L, D>(target, size, mipmaps, P::pixel_format(), sampler)?;

      let raw = RawTexture::new(ctx.state().clone(), texture, target);

      Ok(Texture {
        raw,
        size,
        mipmaps,
        _l: PhantomData,
        _p: PhantomData,
      })
    }
  }

  /// Create a texture from its backend representation.
  pub(crate) unsafe fn from_raw(raw: RawTexture, size: D::Size, mipmaps: usize) -> Self {
    Texture {
      raw,
      size,
      mipmaps: mipmaps + 1,
      _l: PhantomData,
      _p: PhantomData,
    }
  }

  /// Convert a texture to its raw representation.
  pub fn to_raw(self) -> RawTexture {
    let raw = unsafe { ptr::read(&self.raw) };

    // forget self so that we don’t call drop on it after the function has returned
    mem::forget(self);
    raw
  }

  /// Number of mipmaps in the texture.
  #[inline(always)]
  pub fn mipmaps(&self) -> usize {
    self.mipmaps
  }

  /// Clear a part of a texture.
  ///
  /// The part being cleared is defined by a rectangle in which the `offset` represents the
  /// left-upper corner and the `size` gives the dimension of the rectangle. All the covered texels
  /// by this rectangle will be cleared to the `pixel` value.
  pub fn clear_part(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    pixel: P::Encoding
  )
  where P::Encoding: Copy {
    self.upload_part(
      gen_mipmaps,
      offset,
      size,
      &vec![pixel; dim_capacity::<D>(size) as usize],
    )
  }

  /// Clear a whole texture with a `pixel` value.
  pub fn clear(&self, gen_mipmaps: GenMipmaps, pixel: P::Encoding)
  where P::Encoding: Copy {
    self.clear_part(gen_mipmaps, D::ZERO_OFFSET, self.size, pixel)
  }

  /// Upload texels to a part of a texture.
  ///
  /// The part being updated is defined by a rectangle in which the `offset` represents the
  /// left-upper corner and the `size` gives the dimension of the rectangle. All the covered texels
  /// by this rectangle will be updated by the `texels` slice.
  pub fn upload_part(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::Encoding],
  ) {
    unsafe {
      let mut gfx_state = self.state.borrow_mut();

      gfx_state.bind_texture(self.target, self.handle);

      upload_texels::<L, D, P, P::Encoding>(self.target, offset, size, texels);

      if gen_mipmaps == GenMipmaps::Yes {
        gl::GenerateMipmap(self.target);
      }

      gfx_state.bind_texture(self.target, 0);
    }
  }

  /// Upload `texels` to the whole texture.
  pub fn upload(
    &self,
    gen_mipmaps: GenMipmaps,
    texels: &[P::Encoding],
  ) {
    self.upload_part(gen_mipmaps, D::ZERO_OFFSET, self.size, texels)
  }

  /// Upload raw `texels` to a part of a texture.
  ///
  /// This function is similar to `upload_part` but it works on `P::RawEncoding` instead of
  /// `P::Encoding`. This useful when the texels are represented as a contiguous array of raw
  /// components of the texels.
  pub fn upload_part_raw(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
  ) {
    unsafe {
      let mut gfx_state = self.state.borrow_mut();

      gfx_state.bind_texture(self.target, self.handle);

      upload_texels::<L, D, P, P::RawEncoding>(self.target, offset, size, texels);

      if gen_mipmaps == GenMipmaps::Yes {
        gl::GenerateMipmap(self.target);
      }

      gfx_state.bind_texture(self.target, 0);
    }
  }

  /// Upload raw `texels` to the whole texture.
  pub fn upload_raw(&self, gen_mipmaps: GenMipmaps, texels: &[P::RawEncoding]) {
    self.upload_part_raw(gen_mipmaps, D::ZERO_OFFSET, self.size, texels)
  }

  // FIXME: cubemaps?
  /// Get the raw texels associated with this texture.
  pub fn get_raw_texels(
    &self
  ) -> Vec<P::RawEncoding> where P: Pixel, P::RawEncoding: Copy + Default {
    let mut texels = Vec::new();
    let pf = P::pixel_format();
    let (format, _, ty) = opengl_pixel_format(pf).unwrap();

    unsafe {
      let mut w = 0;
      let mut h = 0;

      let mut gfx_state = self.state.borrow_mut();
      gfx_state.bind_texture(self.target, self.handle);

      // retrieve the size of the texture (w and h)
      gl::GetTexLevelParameteriv(self.target, 0, gl::TEXTURE_WIDTH, &mut w);
      gl::GetTexLevelParameteriv(self.target, 0, gl::TEXTURE_HEIGHT, &mut h);

      // resize the vec to allocate enough space to host the returned texels
      texels.resize_with((w * h) as usize * pixel_components(pf), Default::default);

      gl::GetTexImage(self.target, 0, format, ty, texels.as_mut_ptr() as *mut c_void);

      gfx_state.bind_texture(self.target, 0);
    }

    texels
  }

  /// Get the inner size of the texture.
  ///
  /// That value represents the _dimension_ of the texture. Depending on the type of texture, its
  /// dimensionality varies. For instance:
  ///
  ///   - 1D textures have a single value, giving the length of the texture.
  ///   - 2D textures have two values for their _width_ and _height_.
  ///   - 3D textures have three values: _width_, _height_ and _depth_.
  ///   - Etc. etc.
  pub fn size(&self) -> D::Size {
    self.size
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

pub(crate) fn opengl_target(l: Layering, d: Dim) -> GLenum {
  match l {
    Layering::Flat => match d {
      Dim::Dim1 => gl::TEXTURE_1D,
      Dim::Dim2 => gl::TEXTURE_2D,
      Dim::Dim3 => gl::TEXTURE_3D,
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP,
    },
    Layering::Layered => match d {
      Dim::Dim1 => gl::TEXTURE_1D_ARRAY,
      Dim::Dim2 => gl::TEXTURE_2D_ARRAY,
      Dim::Dim3 => panic!("3D textures array not supported"),
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP_ARRAY,
    },
  }
}

pub(crate) unsafe fn create_texture<L, D>(
  target: GLenum,
  size: D::Size,
  mipmaps: usize,
  pf: PixelFormat,
  sampler: &Sampler,
) -> Result<(), TextureError>
where L: Layerable,
      D: Dimensionable {
  set_texture_levels(target, mipmaps);
  apply_sampler_to_texture(target, sampler);
  create_texture_storage::<L, D>(size, mipmaps, pf)
}

fn create_texture_storage<L, D>(size: D::Size, mipmaps: usize, pf: PixelFormat) -> Result<(), TextureError>
where L: Layerable,
      D: Dimensionable {
  match opengl_pixel_format(pf) {
    Some(glf) => {
      let (format, iformat, encoding) = glf;

      match (L::layering(), D::dim()) {
        // 1D texture
        (Layering::Flat, Dim::Dim1) => {
          create_texture_1d_storage(format, iformat, encoding, D::width(size), mipmaps);
          Ok(())
        }

        // 2D texture
        (Layering::Flat, Dim::Dim2) => {
          create_texture_2d_storage(
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            mipmaps,
          );
          Ok(())
        }

        // 3D texture
        (Layering::Flat, Dim::Dim3) => {
          create_texture_3d_storage(
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            D::depth(size),
            mipmaps,
          );
          Ok(())
        }

        // cubemap
        (Layering::Flat, Dim::Cubemap) => {
          create_cubemap_storage(format, iformat, encoding, D::width(size), mipmaps);
          Ok(())
        }

        _ => {
          #[cfg(feature = "std")]
          {
            Err(TextureError::TextureStorageCreationFailed(format!(
              "unsupported texture OpenGL pixel format: {:?}",
              glf
            )))
          }

          #[cfg(not(feature = "std"))]
          {
            let mut reason = String::new();
            let _ = write!(&mut reason, "unsupported texture OpenGL pixel format: {:?}", glf);
            Err(TextureError::TextureStorageCreationFailed(reason))
          }
        }
      }
    }

    None => {
      #[cfg(feature = "std")]
      {
        Err(TextureError::TextureStorageCreationFailed(format!(
          "unsupported texture pixel format: {:?}",
          pf
        )))
      }

      #[cfg(not(feature = "std"))]
      {
        let mut reason = String::new();
        let _ = write!(&mut reason, "unsupported texture pixel format: {:?}", pf);
        Err(TextureError::TextureStorageCreationFailed(reason))
      }
    }
  }
}

fn create_texture_1d_storage(
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  mipmaps: usize
) {
  for level in 0 .. mipmaps {
    let w = w / 2u32.pow(level as u32);

    unsafe {
      gl::TexImage1D(
        gl::TEXTURE_1D,
        level as GLint,
        iformat as GLint,
        w as GLsizei,
        0,
        format,
        encoding,
        ptr::null(),
      )
    };
  }
}

fn create_texture_2d_storage(
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  h: u32,
  mipmaps: usize,
) {
  for level in 0..mipmaps {
    let div = 2u32.pow(level as u32);
    let w = w / div;
    let h = h / div;

    unsafe {
      gl::TexImage2D(
        gl::TEXTURE_2D,
        level as GLint,
        iformat as GLint,
        w as GLsizei,
        h as GLsizei,
        0,
        format,
        encoding,
        ptr::null(),
      )
    };
  }
}

fn create_texture_3d_storage(
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  h: u32,
  d: u32,
  mipmaps: usize,
) {
  for level in 0..mipmaps {
    let div = 2u32.pow(level as u32);
    let w = w / div;
    let h = h / div;
    let d = d / div;

    unsafe {
      gl::TexImage3D(
        gl::TEXTURE_3D,
        level as GLint,
        iformat as GLint,
        w as GLsizei,
        h as GLsizei,
        d as GLsizei,
        0,
        format,
        encoding,
        ptr::null(),
      )
    };
  }
}

fn create_cubemap_storage(
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  s: u32,
  mipmaps: usize
) {
  for level in 0..mipmaps {
    let s = s / 2u32.pow(level as u32);

    unsafe {
      gl::TexImage2D(
        gl::TEXTURE_CUBE_MAP,
        level as GLint,
        iformat as GLint,
        s as GLsizei,
        s as GLsizei,
        0,
        format,
        encoding,
        ptr::null(),
      )
    };
  }
}

fn set_texture_levels(target: GLenum, mipmaps: usize) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, mipmaps as GLint - 1);
  }
}

fn apply_sampler_to_texture(target: GLenum, sampler: &Sampler) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_WRAP_R, opengl_wrap(sampler.wrap_r) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_WRAP_S, opengl_wrap(sampler.wrap_s) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_WRAP_T, opengl_wrap(sampler.wrap_t) as GLint);
    gl::TexParameteri(
      target,
      gl::TEXTURE_MIN_FILTER,
      opengl_min_filter(sampler.min_filter) as GLint,
    );
    gl::TexParameteri(
      target,
      gl::TEXTURE_MAG_FILTER,
      opengl_mag_filter(sampler.mag_filter) as GLint,
    );
    match sampler.depth_comparison {
      Some(fun) => {
        gl::TexParameteri(
          target,
          gl::TEXTURE_COMPARE_FUNC,
          opengl_depth_comparison(fun) as GLint,
        );
        gl::TexParameteri(
          target,
          gl::TEXTURE_COMPARE_MODE,
          gl::COMPARE_REF_TO_TEXTURE as GLint,
        );
      }
      None => {
        gl::TexParameteri(target, gl::TEXTURE_COMPARE_MODE, gl::NONE as GLint);
      }
    }
  }
}

fn opengl_wrap(wrap: Wrap) -> GLenum {
  match wrap {
    Wrap::ClampToEdge => gl::CLAMP_TO_EDGE,
    Wrap::Repeat => gl::REPEAT,
    Wrap::MirroredRepeat => gl::MIRRORED_REPEAT,
  }
}

fn opengl_min_filter(filter: MinFilter) -> GLenum {
  match filter {
    MinFilter::Nearest => gl::NEAREST,
    MinFilter::Linear => gl::LINEAR,
    MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
    MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
    MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
    MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
  }
}

fn opengl_mag_filter(filter: MagFilter) -> GLenum {
  match filter {
    MagFilter::Nearest => gl::NEAREST,
    MagFilter::Linear => gl::LINEAR,
  }
}

fn opengl_depth_comparison(fun: DepthComparison) -> GLenum {
  match fun {
    DepthComparison::Never => gl::NEVER,
    DepthComparison::Always => gl::ALWAYS,
    DepthComparison::Equal => gl::EQUAL,
    DepthComparison::NotEqual => gl::NOTEQUAL,
    DepthComparison::Less => gl::LESS,
    DepthComparison::LessOrEqual => gl::LEQUAL,
    DepthComparison::Greater => gl::GREATER,
    DepthComparison::GreaterOrEqual => gl::GEQUAL,
  }
}

// Upload texels into the texture’s memory. Becareful of the type of texels you send down.
fn upload_texels<L, D, P, T>(target: GLenum, off: D::Offset, size: D::Size, texels: &[T])
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  let pf = P::pixel_format();

  match opengl_pixel_format(pf) {
    Some((format, _, encoding)) => match L::layering() {
      Layering::Flat => match D::dim() {
        Dim::Dim1 => unsafe {
          gl::TexSubImage1D(
            target,
            0,
            D::x_offset(off) as GLint,
            D::width(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          )
        },
        Dim::Dim2 => unsafe {
          gl::TexSubImage2D(
            target,
            0,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          )
        },
        Dim::Dim3 => unsafe {
          gl::TexSubImage3D(
            target,
            0,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::z_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::height(size) as GLsizei,
            D::depth(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          )
        },
        Dim::Cubemap => unsafe {
          gl::TexSubImage3D(
            target,
            0,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            (gl::TEXTURE_CUBE_MAP_POSITIVE_X + D::z_offset(off)) as GLint,
            D::width(size) as GLsizei,
            D::width(size) as GLsizei,
            1,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          )
        },
      },
      Layering::Layered => panic!("Layering::Layered not implemented yet"),
    },
    None => panic!("unknown pixel format"),
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

/// Errors that might happen when working with textures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureError {
  /// A texture’s storage failed to be created.
  ///
  /// The carried [`String`] gives the reason of the failure.
  TextureStorageCreationFailed(String),
}

impl fmt::Display for TextureError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TextureError::TextureStorageCreationFailed(ref e) => {
        write!(f, "texture storage creation failed: {}", e)
      }
    }
  }
}
