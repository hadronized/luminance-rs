//! GPU textures.
//!
//! A GPU texture is an object that can be perceived as an array on the GPU. It holds several items
//! and is a dimensional object. Textures are often associated with _images_, even though their
//! usage in the graphics world can be much larger than that.
//!
//! Textures ([`Texture`]) come in several flavors and the differences lie in:
//!
//! - The dimensionality: textures can be 1D, 2D, 3D, layered, cube maps, etc. Dimension is encoded
//!   via the [`Dim`] type, and indexed in the type-system with the [`Dimensionable`] and its
//!   implementors.
//! - The encoding: the _items_ inside textures are called _pixels_ or _texels_. Those can be
//!   encoded in several ways. They are represented by various types, which are implementors of the
//!   [`Pixel`] trait.
//! - The usage: some textures will be used as images, others will be used to pass arbitrary data
//!   around in shaders, etc.
//!
//! Whatever the flavor, textures have few uses outside of shaders. When a shader wants
//! to use a texture, it can do it directly, by accessing each pixel by their position inside the
//! texture (using a normalized coordinates system) or via the use of a [`Sampler`]. A [`Sampler`],
//! as the name implies, is an object that tells the GPU how fetching (sampling) a texture should
//! occur. Several options there too:
//!
//! - The GPU can fetch a pixel without sampling. It means that you have to pass the exact
//!   coordinates of the pixel you want to access. This is useful when you store arbitrary
//!   information, like UUID, velocities, etc.
//! - The GPU can fetch a pixel with a floating-point coordinates system. How that system works
//!   depends on the settings of [`Sampler`] you choose. For instance, you can always fetch the
//!   _nearest_ pixel to where you sample, or you can ask the GPU to perform a linear
//!   interpolation between all neighboring pixels, etc. [`Sampler`] allows way more than that, so
//!   feel free to read their documentation.

use crate::{
  backend::texture::Texture as TextureBackend,
  context::GraphicsContext,
  depth_stencil::Comparison,
  pixel::{Pixel, PixelFormat},
};
use std::{error, fmt, marker::PhantomData};

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

/// Class of [`Texture`] dimensions.
///
/// This trait provides a simple mapping between the implementor and the [`Dim`] type, which represents a [`Texture`]
/// dimension. This allows to heavily type [`Texture`] so that their dimension is indexed-tracked in the type-system.
/// This trait is then used to reify the implementors into [`Dim`] so that the runtime can work with them.
pub trait Dimensionable {
  /// Size type of a dimension (used to caracterize dimensions’ areas).
  type Size: Copy;

  /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
  type Offset: Copy;

  /// Zero offset.
  ///
  /// Any size added with `ZERO_OFFSET` must remain the size itself.
  const ZERO_OFFSET: Self::Offset;

  /// Reified [`Dim`].
  ///
  /// Implementors must ensure they map to the right variant of [`Dim`].
  fn dim() -> Dim;

  /// Width of the associated [`Dimensionable::Size`].
  fn width(size: Self::Size) -> u32;

  /// Height of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
  fn height(_: Self::Size) -> u32 {
    1
  }

  /// Depth of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dim {
  /// 1D.
  Dim1,
  /// 2D.
  Dim2,
  /// 3D.
  Dim3,
  /// Cubemap (i.e. a cube defining 6 faces — akin to 4D).
  Cubemap,
  /// 1D array.
  ///
  /// This corresponds to _layered_ 1D textures, i.e. a 1D texture with an extra parameter to tap into the corresponding
  /// layer. Using in a [`Texture`] allows to perform _layered rendering_.
  Dim1Array,
  /// 2D array.
  ///
  /// This corresponds to _layered_ 2D textures, i.e. a 2D texture with an extra parameter to tap into the corresponding
  /// layer. Using in a [`Texture`] allows to perform _layered rendering_.
  Dim2Array,
}

impl fmt::Display for Dim {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Dim::Dim1 => f.write_str("1D"),
      Dim::Dim2 => f.write_str("2D"),
      Dim::Dim3 => f.write_str("3D"),
      Dim::Cubemap => f.write_str("cubemap"),
      Dim::Dim1Array => f.write_str("1D array"),
      Dim::Dim2Array => f.write_str("2D array"),
    }
  }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// 1D array dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1Array;

impl Dimensionable for Dim1Array {
  type Offset = (u32, u32);
  type Size = (u32, u32);

  const ZERO_OFFSET: Self::Offset = (0, 0);

  fn dim() -> Dim {
    Dim::Dim1Array
  }

  fn width(size: Self::Size) -> u32 {
    size.0
  }

  fn height(size: Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.1
  }

  fn count((width, layer): Self::Size) -> usize {
    width as usize * layer as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2Array;

impl Dimensionable for Dim2Array {
  type Offset = ([u32; 2], u32);
  type Size = ([u32; 2], u32);

  const ZERO_OFFSET: Self::Offset = ([0, 0], 0);

  fn dim() -> Dim {
    Dim::Dim2Array
  }

  fn width(size: Self::Size) -> u32 {
    size.0[0]
  }

  fn height(size: Self::Size) -> u32 {
    size.0[1]
  }

  fn depth(size: Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: Self::Offset) -> u32 {
    off.0[0]
  }

  fn y_offset(off: Self::Offset) -> u32 {
    off.0[1]
  }

  fn z_offset(off: Self::Offset) -> u32 {
    off.1
  }

  fn count(([width, height], layer): Self::Size) -> usize {
    width as usize * height as usize * layer as usize
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
  pub depth_comparison: Option<Comparison>,
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

/// Texel upload.
///
/// You have the choice between different options regarding mipmaps.:
///
/// - You can upload texels and let mipmaps being automatically created for you.
/// - You can upload texels and disable mipmap creation.
/// - You can upload texels by manually providing all the mipmap levels.
#[derive(Debug)]
pub enum TexelUpload<'a, T>
where
  T: ?Sized,
{
  /// Provide the base level and whether mipmaps should be generated.
  BaseLevel {
    /// Texels list to upload.
    texels: &'a T,

    /// Whether mipmap levels should be automatically created.
    ///
    /// Use `0` if you don’t want any mipmap.
    mipmaps: usize,
  },

  /// Provide all the levels at once.
  ///
  /// The number of elements in the outer slice represents the number of mipmaps; each inner slice represents the texels
  /// to be uploaded to the mipmap level.
  Levels(&'a [&'a T]),

  /// Reserve only the base level and optional mipmap levels.
  ///
  /// This variant allows you not to pass any texel data and ask the backend to just reserve the memory for the texture.
  /// This will allow to pass data later, and let the backend fill the data for you. The texture texels will be set in a
  /// vendor-specific way, so you should assume that the texture will be filled with garbage.
  Reserve {
    /// Number of mipmap levels to allocate.
    ///
    /// Use `0` if you don’t want any.
    mipmaps: usize,
  },
}

impl<'a, T> TexelUpload<'a, T>
where
  T: ?Sized,
{
  /// Create a texel upload for the base level of a texture and let mipmap levels be automatically created.
  pub fn base_level(texels: &'a T, mipmaps: usize) -> Self {
    Self::BaseLevel { texels, mipmaps }
  }

  /// Create a texel upload by reserving memory and let mipmap levels be reserved as well.
  pub fn reserve(mipmaps: usize) -> Self {
    Self::Reserve { mipmaps }
  }

  /// Create a texel upload by manually providing all base + mipmap levels.
  pub fn levels(texels: &'a [&'a T]) -> Self {
    Self::Levels(texels)
  }

  /// Number of mipmaps.
  pub fn mipmaps(&self) -> usize {
    match self {
      TexelUpload::BaseLevel { mipmaps, .. } => *mipmaps,
      TexelUpload::Levels(levels) => levels.len(),
      TexelUpload::Reserve { mipmaps } => *mipmaps,
    }
  }

  /// Get the base level texels.
  pub fn get_base_level(&self) -> Option<&'a T> {
    match self {
      TexelUpload::BaseLevel { texels, .. } => Some(*texels),
      TexelUpload::Levels(levels) => levels.get(0).map(|base_level| *base_level),
      TexelUpload::Reserve { .. } => None,
    }
  }
}

/// Errors that might happen when working with textures.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureError {
  /// A texture’s storage failed to be created.
  ///
  /// The carried [`String`] gives the reason of the failure.
  TextureStorageCreationFailed(String),

  /// Not enough pixel data provided for the given area asked.
  ///
  /// You must provide at least as many pixels as expected by the area in the texture you’re
  /// uploading to.
  NotEnoughPixels {
    /// Expected number of pixels in bytes.
    expected_bytes: usize,
    /// Provided number of pixels in bytes.
    provided_bytes: usize,
  },

  /// Unsupported pixel format.
  ///
  /// Sometimes, some hardware might not support a given pixel format (or the format exists on
  /// the interface side but doesn’t in the implementation). That error represents such a case.
  UnsupportedPixelFormat(PixelFormat),

  /// Cannot retrieve texels from a texture.
  ///
  /// That error might happen on some hardware implementations if the user tries to retrieve
  /// texels from a texture that doesn’t support getting its texels retrieved.
  CannotRetrieveTexels(String),

  /// Failed to upload texels.
  CannotUploadTexels(String),
}

impl TextureError {
  /// A texture’s storage failed to be created.
  pub fn texture_storage_creation_failed(reason: impl Into<String>) -> Self {
    TextureError::TextureStorageCreationFailed(reason.into())
  }

  /// Not enough pixel data provided for the given area asked.
  pub fn not_enough_pixels(expected_bytes: usize, provided_bytes: usize) -> Self {
    TextureError::NotEnoughPixels {
      expected_bytes,
      provided_bytes,
    }
  }

  /// Unsupported pixel format.
  pub fn unsupported_pixel_format(pf: PixelFormat) -> Self {
    TextureError::UnsupportedPixelFormat(pf)
  }

  /// Cannot retrieve texels from a texture.
  pub fn cannot_retrieve_texels(reason: impl Into<String>) -> Self {
    TextureError::CannotRetrieveTexels(reason.into())
  }

  /// Failed to upload texels.
  pub fn cannot_upload_texels(reason: impl Into<String>) -> Self {
    TextureError::CannotUploadTexels(reason.into())
  }
}

impl fmt::Display for TextureError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TextureError::TextureStorageCreationFailed(ref e) => {
        write!(f, "texture storage creation failed: {}", e)
      }

      TextureError::NotEnoughPixels {
        ref expected_bytes,
        ref provided_bytes,
      } => write!(
        f,
        "not enough texels provided: expected {} bytes, provided {} bytes",
        expected_bytes, provided_bytes
      ),

      TextureError::UnsupportedPixelFormat(ref fmt) => {
        write!(f, "unsupported pixel format: {:?}", fmt)
      }

      TextureError::CannotRetrieveTexels(ref e) => {
        write!(f, "cannot retrieve texture’s texels: {}", e)
      }

      TextureError::CannotUploadTexels(ref e) => {
        write!(f, "cannot upload texels to texture: {}", e)
      }
    }
  }
}

impl error::Error for TextureError {}

/// Textures.
///
/// Textures allow mainly two use cases:
///
/// - Passing data (in the form of images or regular data) to shaders.
/// - Offscreen rendering.
///
/// In the former case, you will want to uplad _images_ to [`Texture`]. This can be done with the various upload functions:
///
/// - [`Texture::upload_part`]
/// - [`Texture::upload`]
/// - [`Texture::upload_part_raw`]
/// - [`Texture::upload_raw`]
///
/// In the second case, a [`Texture`] can be used as part of a [`ColorSlot`] or [`DepthStencilSlot`]
/// of a [`Framebuffer`]. This allows to create graphics pipeline that will output into the
/// [`Texture`], that you can use in another graphics pipeline later.
///
/// # Parametricity
///
/// Textures have three type parameters:
///
/// - `B`, which is the backend type. It must implement [`TextureBackend`].
/// - `D`, the dimension of the texture. It must implement [`Dimensionable`].
/// - `P`, the pixel type. It must implement [`Pixel`].
///
/// # Dimension
///
/// The dimension of the texture gives its flavor: 2D, 3D, cubemap, etc. You have access to a bunch of different types
/// here, which all are implementors of the [`Dimensionable`] trait.
///
/// # Pixel format
///
/// The internal representation of the [`Texture`] will be derived from the `P` type, which is the [`Pixel`] type. Lots
/// of types are available, but you have to know that depending on the use you want to make of the texture, not all
/// types are compatible. For instance, if you access a [`Texture`] as part of a [`ColorSlot`] of a [`Framebuffer`], the
/// [`Pixel`] type must be [`RenderablePixel`]. The compiler will always tells you if you are trying to use a pixel type
/// that is not compatible, but you should check the implementors of [`Pixel`], [`RenderablePixel`] and [`DepthPixel`]
/// before starting using them.
///
/// [`Framebuffer`]: crate::framebuffer::Framebuffer
/// [`ColorSlot`]: crate::backend::color_slot::ColorSlot
/// [`DepthStencilSlot`]: crate::backend::depth_stencil_slot::DepthStencilSlot
/// [`RenderablePixel`]: crate::pixel::RenderablePixel
/// [`DepthPixel`]: crate::pixel::DepthPixel
/// [`TextureBackend`]: crate::backend::texture::Texture
pub struct Texture<B, D, P>
where
  B: ?Sized + TextureBackend<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  pub(crate) repr: B::TextureRepr,
  size: D::Size,
  _phantom: PhantomData<*const P>,
}

impl<B, D, P> Texture<B, D, P>
where
  B: ?Sized + TextureBackend<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  /// Create a new [`Texture`].
  ///
  /// `size` is the desired size of the [`Texture`].
  ///
  /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
  /// shader, for instance.
  ///
  /// `gen_mipmaps` determines whether mipmaps should be generated automatically.
  ///
  /// `texels` is a [`TexelUpload`] of texels to put into the texture store.
  ///
  /// # Notes
  ///
  /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture`] for a
  /// simpler interface.
  pub fn new<C>(
    ctx: &mut C,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<Self, TextureError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe {
      ctx
        .backend()
        .new_texture(size, sampler, texels)
        .map(|repr| Texture {
          repr,
          size,
          _phantom: PhantomData,
        })
    }
  }

  /// Create a new [`Texture`] with raw texels.
  ///
  /// `size` is the wished size of the [`Texture`].
  ///
  /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
  /// shader, for instance.
  ///
  /// `texels` is a [`TexelUpload`] of raw texels to put into the texture store.
  ///
  /// # Notes
  ///
  /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture_raw`] for a
  /// simpler interface.
  pub fn new_raw<C>(
    ctx: &mut C,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<Self, TextureError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe {
      ctx
        .backend()
        .new_texture_raw(size, sampler, texels)
        .map(|repr| Texture {
          repr,
          size,
          _phantom: PhantomData,
        })
    }
  }

  /// Return the number of mipmaps.
  pub fn mipmaps(&self) -> usize {
    unsafe { B::mipmaps(&self.repr) }
  }

  /// Return the size of the texture.
  pub fn size(&self) -> D::Size {
    self.size
  }

  /// Resize the texture by providing a new size and texels by reusing its GPU resources.
  ///
  /// This function works similarly to [`Texture::new`] but instead of creating a brand new texture, reuses the texture
  /// resources on the GPU.
  pub fn resize(
    &mut self,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError> {
    self.size = size;
    unsafe { B::resize(&mut self.repr, size, texels) }
  }

  /// Resize the texture by providing a new size and raw texels by reusing its GPU resources.
  ///
  /// This function works similarly to [`Texture::new_raw`] but instead of creating a brand new texture, reuses the texture
  /// resources on the GPU.
  pub fn resize_raw(
    &mut self,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError> {
    self.size = size;
    unsafe { B::resize_raw(&mut self.repr, size, texels) }
  }

  /// Upload pixels to a region of the texture described by the rectangle made with `size` and
  /// `offset`.
  pub fn upload_part(
    &mut self,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError> {
    unsafe { B::upload_part(&mut self.repr, offset, size, texels) }
  }

  /// Upload pixels to the whole texture.
  pub fn upload(&mut self, texels: TexelUpload<[P::Encoding]>) -> Result<(), TextureError> {
    unsafe { B::upload(&mut self.repr, self.size, texels) }
  }

  /// Upload raw data to a region of the texture described by the rectangle made with `size` and
  /// `offset`.
  pub fn upload_part_raw(
    &mut self,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError> {
    unsafe { B::upload_part_raw(&mut self.repr, offset, size, texels) }
  }

  /// Upload raw data to the whole texture.
  pub fn upload_raw(&mut self, texels: TexelUpload<[P::RawEncoding]>) -> Result<(), TextureError> {
    unsafe { B::upload_raw(&mut self.repr, self.size, texels) }
  }

  /// Get a copy of all the pixels from the texture.
  pub fn get_raw_texels(&self) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    P::RawEncoding: Copy + Default,
  {
    unsafe { B::get_raw_texels(&self.repr, self.size) }
  }
}
