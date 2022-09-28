use std::fmt;

use crate::{depth_stencil::Comparison, pixel::PixelFormat};

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

// pub struct Texture<B, D, P>
// where
//   B: ?Sized + TextureBackend<D, P>,
//   D: Dimensionable,
//   P: Pixel,
// {
//   pub(crate) repr: B::TextureRepr,
//   size: D::Size,
//   _phantom: PhantomData<*const P>,
// }
//
// impl<B, D, P> Texture<B, D, P>
// where
//   B: ?Sized + TextureBackend<D, P>,
//   D: Dimensionable,
//   P: Pixel,
// {
//   /// Create a new [`Texture`].
//   ///
//   /// `size` is the desired size of the [`Texture`].
//   ///
//   /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
//   /// shader, for instance.
//   ///
//   /// `gen_mipmaps` determines whether mipmaps should be generated automatically.
//   ///
//   /// `texels` is a [`TexelUpload`] of texels to put into the texture store.
//   ///
//   /// # Notes
//   ///
//   /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture`] for a
//   /// simpler interface.
//   pub fn new<C>(
//     ctx: &mut C,
//     size: D::Size,
//     sampler: Sampler,
//     texels: TexelUpload<[P::Encoding]>,
//   ) -> Result<Self, TextureError>
//   where
//     C: GraphicsContext<Backend = B>,
//   {
//     unsafe {
//       ctx
//         .backend()
//         .new_texture(size, sampler, texels)
//         .map(|repr| Texture {
//           repr,
//           size,
//           _phantom: PhantomData,
//         })
//     }
//   }
//
//   /// Create a new [`Texture`] with raw texels.
//   ///
//   /// `size` is the wished size of the [`Texture`].
//   ///
//   /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
//   /// shader, for instance.
//   ///
//   /// `texels` is a [`TexelUpload`] of raw texels to put into the texture store.
//   ///
//   /// # Notes
//   ///
//   /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture_raw`] for a
//   /// simpler interface.
//   pub fn new_raw<C>(
//     ctx: &mut C,
//     size: D::Size,
//     sampler: Sampler,
//     texels: TexelUpload<[P::RawEncoding]>,
//   ) -> Result<Self, TextureError>
//   where
//     C: GraphicsContext<Backend = B>,
//   {
//     unsafe {
//       ctx
//         .backend()
//         .new_texture_raw(size, sampler, texels)
//         .map(|repr| Texture {
//           repr,
//           size,
//           _phantom: PhantomData,
//         })
//     }
//   }
//
//   /// Return the number of mipmaps.
//   pub fn mipmaps(&self) -> usize {
//     unsafe { B::mipmaps(&self.repr) }
//   }
//
//   /// Return the size of the texture.
//   pub fn size(&self) -> D::Size {
//     self.size
//   }
//
//   /// Resize the texture by providing a new size and texels by reusing its GPU resources.
//   ///
//   /// This function works similarly to [`Texture::new`] but instead of creating a brand new texture, reuses the texture
//   /// resources on the GPU.
//   pub fn resize(
//     &mut self,
//     size: D::Size,
//     texels: TexelUpload<[P::Encoding]>,
//   ) -> Result<(), TextureError> {
//     self.size = size;
//     unsafe { B::resize(&mut self.repr, size, texels) }
//   }
//
//   /// Resize the texture by providing a new size and raw texels by reusing its GPU resources.
//   ///
//   /// This function works similarly to [`Texture::new_raw`] but instead of creating a brand new texture, reuses the texture
//   /// resources on the GPU.
//   pub fn resize_raw(
//     &mut self,
//     size: D::Size,
//     texels: TexelUpload<[P::RawEncoding]>,
//   ) -> Result<(), TextureError> {
//     self.size = size;
//     unsafe { B::resize_raw(&mut self.repr, size, texels) }
//   }
//
//   /// Upload pixels to a region of the texture described by the rectangle made with `size` and
//   /// `offset`.
//   pub fn upload_part(
//     &mut self,
//     offset: D::Offset,
//     size: D::Size,
//     texels: TexelUpload<[P::Encoding]>,
//   ) -> Result<(), TextureError> {
//     unsafe { B::upload_part(&mut self.repr, offset, size, texels) }
//   }
//
//   /// Upload pixels to the whole texture.
//   pub fn upload(&mut self, texels: TexelUpload<[P::Encoding]>) -> Result<(), TextureError> {
//     unsafe { B::upload(&mut self.repr, self.size, texels) }
//   }
//
//   /// Upload raw data to a region of the texture described by the rectangle made with `size` and
//   /// `offset`.
//   pub fn upload_part_raw(
//     &mut self,
//     offset: D::Offset,
//     size: D::Size,
//     texels: TexelUpload<[P::RawEncoding]>,
//   ) -> Result<(), TextureError> {
//     unsafe { B::upload_part_raw(&mut self.repr, offset, size, texels) }
//   }
//
//   /// Upload raw data to the whole texture.
//   pub fn upload_raw(&mut self, texels: TexelUpload<[P::RawEncoding]>) -> Result<(), TextureError> {
//     unsafe { B::upload_raw(&mut self.repr, self.size, texels) }
//   }
//
//   /// Get a copy of all the pixels from the texture.
//   pub fn get_raw_texels(&self) -> Result<Vec<P::RawEncoding>, TextureError>
//   where
//     P::RawEncoding: Copy + Default,
//   {
//     unsafe { B::get_raw_texels(&self.repr, self.size) }
//   }
// }
