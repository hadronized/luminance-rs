use gl;
use gl::types::*;
use luminance::context::GraphicsContext;
use luminance::pixel::{Pixel, PixelFormat};
use luminance::texture::{Dim, Dimensionable, GenMipmaps, Layerable, Layering, MagFilter, MinFilter, Sampler, Texture as TextureBackend, Wrap};
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::rc::Rc;
use std::ptr;

use crate::depth_test::depth_comparison_to_glenum;
use crate::pixel::opengl_pixel_format;
use crate::state::GraphicsState;

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
  pub fn new<C>(ctx: &mut C, size: D::Size, mipmaps: usize, sampler: Sampler) -> Result<Self, TextureError>
  where C: GraphicsContext<State = GraphicsState> {
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
  pub fn into_raw(self) -> RawTexture {
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
  ) -> Result<(), TextureError>
  where P::Encoding: Copy {
    self.upload_part(
      gen_mipmaps,
      offset,
      size,
      &vec![pixel; dim_capacity::<D>(size) as usize],
    )
  }

  /// Clear a whole texture with a `pixel` value.
  pub fn clear(&self, gen_mipmaps: GenMipmaps, pixel: P::Encoding) -> Result<(), TextureError>
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
  ) -> Result<(), TextureError> {
    unsafe {
      let mut gfx_state = self.state.borrow_mut();

      gfx_state.bind_texture(self.target, self.handle);

      upload_texels::<L, D, P, P::Encoding>(self.target, offset, size, texels)?;

      if gen_mipmaps == GenMipmaps::Yes {
        gl::GenerateMipmap(self.target);
      }

      gfx_state.bind_texture(self.target, 0);
    }

    Ok(())
  }

  /// Upload `texels` to the whole texture.
  pub fn upload(
    &self,
    gen_mipmaps: GenMipmaps,
    texels: &[P::Encoding],
  ) -> Result<(), TextureError> {
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
  ) -> Result<(), TextureError> {
    unsafe {
      let mut gfx_state = self.state.borrow_mut();

      gfx_state.bind_texture(self.target, self.handle);

      upload_texels::<L, D, P, P::RawEncoding>(self.target, offset, size, texels)?;

      if gen_mipmaps == GenMipmaps::Yes {
        gl::GenerateMipmap(self.target);
      }

      gfx_state.bind_texture(self.target, 0);
    }

    Ok(())
  }

  /// Upload raw `texels` to the whole texture.
  pub fn upload_raw(
    &self,
    gen_mipmaps: GenMipmaps,
    texels: &[P::RawEncoding]
  ) -> Result<(), TextureError> {
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

      // set the packing alignment based on the number of bytes to skip
      let skip_bytes = (pf.format.size() * w as usize) % 8;
      set_pack_alignment(skip_bytes);

      // resize the vec to allocate enough space to host the returned texels
      texels.resize_with((w * h) as usize * pf.canals_len(), Default::default);

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
      Dim::Dim3 => unimplemented!("3D textures array not supported"),
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP_ARRAY,
    },
  }
}

pub(crate) unsafe fn create_texture<L, D>(
  target: GLenum,
  size: D::Size,
  mipmaps: usize,
  pf: PixelFormat,
  sampler: Sampler,
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
          Err(TextureError::TextureStorageCreationFailed(format!(
                "unsupported texture OpenGL pixel format: {:?}",
                glf
          )))
        }
      }
    }

    None => {
      Err(TextureError::TextureStorageCreationFailed(format!(
            "unsupported texture pixel format: {:?}",
            pf
      )))
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

    for face in 0..6 {
      unsafe {
        gl::TexImage2D(
          gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
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
    };
  }
}

fn set_texture_levels(target: GLenum, mipmaps: usize) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, mipmaps as GLint - 1);
  }
}

fn apply_sampler_to_texture(target: GLenum, sampler: Sampler) {
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
          depth_comparison_to_glenum(fun) as GLint,
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

// set the unpack alignment for uploading aligned texels
fn set_unpack_alignment(skip_bytes: usize) {
  let unpack_alignment = match skip_bytes {
    0 => 8,
    2 => 2,
    4 => 4,
    _ => 1
  };

  unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, unpack_alignment) };
}

// set the pack alignment for downloading aligned texels
fn set_pack_alignment(skip_bytes: usize) {
  let pack_alignment = match skip_bytes {
    0 => 8,
    2 => 2,
    4 => 4,
    _ => 1
  };

  unsafe { gl::PixelStorei(gl::PACK_ALIGNMENT, pack_alignment) };
}

// Upload texels into the texture’s memory. Becareful of the type of texels you send down.
fn upload_texels<L, D, P, T>(
  target: GLenum,
  off: D::Offset,
  size: D::Size,
  texels: &[T]
) -> Result<(), TextureError>
where L: Layerable,
      D: Dimensionable,
      P: Pixel {
  // number of bytes in the input texels argument
  let input_bytes = texels.len() * mem::size_of::<T>();
  let pf = P::pixel_format();
  let pf_size = pf.format.size();
  let expected_bytes = D::count(size) * pf_size;

  if input_bytes < expected_bytes {
    // potential segfault / overflow; abort
    return Err(TextureError::NotEnoughPixels(expected_bytes, input_bytes));
  }

  // set the pixel row alignment to the required value for uploading data according to the width
  // of the texture and the size of a single pixel; here, skip_bytes represents the number of bytes
  // that will be skipped
  let skip_bytes = (D::width(size) as usize * pf_size) % 8;
  set_unpack_alignment(skip_bytes);

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
        }

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
        }

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
        }

        Dim::Cubemap => unsafe {
          gl::TexSubImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + D::z_offset(off),
            0,
            D::x_offset(off) as GLint,
            D::y_offset(off) as GLint,
            D::width(size) as GLsizei,
            D::width(size) as GLsizei,
            format,
            encoding,
            texels.as_ptr() as *const c_void,
          )
        }
      }

      Layering::Layered => unimplemented!("Layering::Layered not implemented yet"),
    }

    None => return Err(TextureError::UnsupportedPixelFormat(pf))
  }

  Ok(())
}

/// Errors that might happen when working with textures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureError {
  /// A texture’s storage failed to be created.
  ///
  /// The carried [`String`] gives the reason of the failure.
  TextureStorageCreationFailed(String),
  /// Not enough pixel data provided for the given area asked.
  ///
  /// The first [`usize`] is the number of expected bytes to be uploaded and the second [`usize`] is
  /// the number you provided. You must provide at least as many pixels as expected by the area in
  /// the texture you’re uploading to.
  NotEnoughPixels(usize, usize),
  /// Unsupported pixel format.
  ///
  /// Sometimes, some hardware might not support a given pixel format (or the format exists on
  /// the interface side but doesn’t in the implementation). That error represents such a case.
  UnsupportedPixelFormat(PixelFormat)
}

impl fmt::Display for TextureError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TextureError::TextureStorageCreationFailed(ref e) => {
        write!(f, "texture storage creation failed: {}", e)
      }

      TextureError::NotEnoughPixels(expected, provided) => {
        write!(f, "not enough texels provided: expected {} bytes, provided {} bytes", expected, provided)
      }

      TextureError::UnsupportedPixelFormat(fmt) => {
        write!(f, "unsupported pixel format: {:?}", fmt)
      }
    }
  }
}

// Capacity of the dimension, which is the product of the width, height and depth.
fn dim_capacity<D>(size: D::Size) -> u32 where D: Dimensionable {
  D::width(size) * D::height(size) * D::depth(size)
}

impl<C, L, D, P> TextureBackend<C, L, D, P> for Texture<L, D, P>
where C:GraphicsContext<State = GraphicsState>,
      L: Layerable,
      D: Dimensionable,
      P: Pixel {
  type Sampler = Sampler;

  type Err = TextureError;

  fn new(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: Self::Sampler
  ) -> Result<Self, Self::Err> {
    Texture::new(ctx, size, mipmaps, sampler)
  }

  fn mipmaps(&self) -> usize {
    Texture::mipmaps(self)
  }

  fn clear_part(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    pixel: P::Encoding
  ) -> Result<(), Self::Err>
  where P::Encoding: Copy {
    Texture::clear_part(self, gen_mipmaps, offset, size, pixel)
  }

  fn upload_part(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::Encoding],
  ) -> Result<(), Self::Err> {
    Texture::upload_part(self, gen_mipmaps, offset, size, texels)
  }

  fn upload_part_raw(
    &self,
    gen_mipmaps: GenMipmaps,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
  ) -> Result<(), Self::Err> {
    Texture::upload_part_raw(self, gen_mipmaps, offset, size, texels)
  }

  /// Get the raw texels associated with this texture.
  fn get_raw_texels(&self) -> Vec<P::RawEncoding> where P: Pixel, P::RawEncoding: Copy + Default {
    Texture::get_raw_texels(self)
  }

  fn size(&self) -> D::Size {
    Texture::size(self)
  }
}
