use crate::gl33::{
  depth_stencil::comparison_to_glenum, pixel::opengl_pixel_format, state::GLState, GL33,
};
use gl::{self, types::*};
use luminance::{
  backend::texture::{Texture as TextureBackend, TextureBase},
  pixel::{Pixel, PixelFormat},
  texture::{Dim, Dimensionable, MagFilter, MinFilter, Sampler, TexelUpload, TextureError, Wrap},
};
use std::{cell::RefCell, mem, os::raw::c_void, ptr, rc::Rc};

pub struct Texture {
  pub(crate) handle: GLuint, // handle to the GPU texture object
  pub(crate) target: GLenum, // “type” of the texture; used for bindings
  mipmaps: usize,
  state: Rc<RefCell<GLState>>,
}

impl Drop for Texture {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteTextures(1, &self.handle);
    }
  }
}

unsafe impl TextureBase for GL33 {
  type TextureRepr = Texture;
}

unsafe impl<D, P> TextureBackend<D, P> for GL33
where
  D: Dimensionable,
  P: Pixel,
{
  unsafe fn new_texture(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<Self::TextureRepr, TextureError> {
    generic_new_texture::<D, P, P::Encoding>(self, size, sampler, texels)
  }

  unsafe fn new_texture_raw(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<Self::TextureRepr, TextureError> {
    generic_new_texture::<D, P, P::RawEncoding>(self, size, sampler, texels)
  }

  unsafe fn mipmaps(texture: &Self::TextureRepr) -> usize {
    texture.mipmaps
  }

  unsafe fn upload_part(
    texture: &mut Self::TextureRepr,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError> {
    let mut gfx_state = texture.state.borrow_mut();

    gfx_state.bind_texture(texture.target, texture.handle);

    upload_texels::<D, P, P::Encoding>(texture.target, offset, size, texels)?;

    gfx_state.bind_texture(texture.target, 0);

    Ok(())
  }

  unsafe fn upload(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError> {
    <Self as TextureBackend<D, P>>::upload_part(texture, D::ZERO_OFFSET, size, texels)
  }

  unsafe fn upload_part_raw(
    texture: &mut Self::TextureRepr,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError> {
    let mut gfx_state = texture.state.borrow_mut();

    gfx_state.bind_texture(texture.target, texture.handle);

    upload_texels::<D, P, P::RawEncoding>(texture.target, offset, size, texels)?;

    gfx_state.bind_texture(texture.target, 0);

    Ok(())
  }

  unsafe fn upload_raw(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError> {
    <Self as TextureBackend<D, P>>::upload_part_raw(texture, D::ZERO_OFFSET, size, texels)
  }

  unsafe fn get_raw_texels(
    texture: &Self::TextureRepr,
    _: D::Size,
  ) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    P::RawEncoding: Copy + Default,
  {
    let pf = P::pixel_format();
    let (format, _, ty) = opengl_pixel_format(pf).unwrap();

    let mut w = 0;
    let mut h = 0;

    let mut gfx_state = texture.state.borrow_mut();
    gfx_state.bind_texture(texture.target, texture.handle);

    // retrieve the size of the texture (w and h)
    gl::GetTexLevelParameteriv(texture.target, 0, gl::TEXTURE_WIDTH, &mut w);
    gl::GetTexLevelParameteriv(texture.target, 0, gl::TEXTURE_HEIGHT, &mut h);

    // set the packing alignment based on the number of bytes to skip
    let skip_bytes = (pf.format.bytes_len() * w as usize) % 8;
    set_pack_alignment(skip_bytes);

    // resize the vec to allocate enough space to host the returned texels
    let mut texels = vec![Default::default(); (w * h) as usize * pf.channels_len()];

    gl::GetTexImage(
      texture.target,
      0,
      format,
      ty,
      texels.as_mut_ptr() as *mut c_void,
    );

    gfx_state.bind_texture(texture.target, 0);

    Ok(texels)
  }

  unsafe fn resize(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError> {
    let mipmaps = texels.mipmaps();
    let mut state = texture.state.borrow_mut();

    state.bind_texture(texture.target, texture.handle);
    create_texture_storage::<D>(size, 1 + mipmaps, P::pixel_format())?;
    upload_texels::<D, P, P::Encoding>(texture.target, D::ZERO_OFFSET, size, texels)
  }

  unsafe fn resize_raw(
    texture: &mut <Self as TextureBase>::TextureRepr,
    size: <D as Dimensionable>::Size,
    texels: TexelUpload<'_, [<P as Pixel>::RawEncoding]>,
  ) -> Result<(), luminance::texture::TextureError> {
    let mipmaps = texels.mipmaps();
    let mut state = texture.state.borrow_mut();

    state.bind_texture(texture.target, texture.handle);
    create_texture_storage::<D>(size, 1 + mipmaps, P::pixel_format())?;
    upload_texels::<D, P, P::RawEncoding>(texture.target, D::ZERO_OFFSET, size, texels)
  }
}

pub(crate) fn opengl_target(d: Dim) -> GLenum {
  match d {
    Dim::Dim1 => gl::TEXTURE_1D,
    Dim::Dim2 => gl::TEXTURE_2D,
    Dim::Dim3 => gl::TEXTURE_3D,
    Dim::Cubemap => gl::TEXTURE_CUBE_MAP,
    Dim::Dim1Array => gl::TEXTURE_1D_ARRAY,
    Dim::Dim2Array => gl::TEXTURE_2D_ARRAY,
  }
}

pub(crate) unsafe fn create_texture<D>(
  target: GLenum,
  size: D::Size,
  mipmaps: usize,
  pf: PixelFormat,
  sampler: Sampler,
) -> Result<(), TextureError>
where
  D: Dimensionable,
{
  set_texture_levels(target, mipmaps);
  apply_sampler_to_texture(target, sampler);
  create_texture_storage::<D>(size, 1 + mipmaps, pf)
}

fn set_texture_levels(target: GLenum, mipmaps: usize) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, mipmaps as GLint);
  }
}

fn apply_sampler_to_texture(target: GLenum, sampler: Sampler) {
  unsafe {
    gl::TexParameteri(
      target,
      gl::TEXTURE_WRAP_R,
      opengl_wrap(sampler.wrap_r) as GLint,
    );
    gl::TexParameteri(
      target,
      gl::TEXTURE_WRAP_S,
      opengl_wrap(sampler.wrap_s) as GLint,
    );
    gl::TexParameteri(
      target,
      gl::TEXTURE_WRAP_T,
      opengl_wrap(sampler.wrap_t) as GLint,
    );
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
          comparison_to_glenum(fun) as GLint,
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

unsafe fn generic_new_texture<D, P, Px>(
  gl33: &mut GL33,
  size: D::Size,
  sampler: Sampler,
  texels: TexelUpload<[Px]>,
) -> Result<Texture, TextureError>
where
  D: Dimensionable,
  P: Pixel,
{
  let mut state = gl33.state.borrow_mut();
  let mipmaps = texels.mipmaps();
  let target = opengl_target(D::dim());

  let handle = state.create_texture();
  state.bind_texture(target, handle);

  create_texture::<D>(target, size, mipmaps, P::pixel_format(), sampler)?;
  upload_texels::<D, P, Px>(target, D::ZERO_OFFSET, size, texels)?;

  let texture = Texture {
    handle,
    target,
    mipmaps,
    state: gl33.state.clone(),
  };

  Ok(texture)
}

fn create_texture_storage<D>(
  size: D::Size,
  levels: usize,
  pf: PixelFormat,
) -> Result<(), TextureError>
where
  D: Dimensionable,
{
  match opengl_pixel_format(pf) {
    Some(glf) => {
      let (format, iformat, encoding) = glf;

      match D::dim() {
        // 1D texture
        Dim::Dim1 => {
          create_texture_1d_storage(format, iformat, encoding, D::width(size), levels);
          Ok(())
        }

        // 2D texture
        Dim::Dim2 => {
          create_texture_2d_storage(
            gl::TEXTURE_2D,
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            levels,
          );
          Ok(())
        }

        // 3D texture
        Dim::Dim3 => {
          create_texture_3d_storage(
            gl::TEXTURE_3D,
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            D::depth(size),
            levels,
          );
          Ok(())
        }

        // cubemap
        Dim::Cubemap => {
          create_cubemap_storage(format, iformat, encoding, D::width(size), levels);
          Ok(())
        }

        // 1D array texture
        Dim::Dim1Array => {
          create_texture_2d_storage(
            gl::TEXTURE_1D_ARRAY,
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            levels,
          );
          Ok(())
        }

        // 2D array texture
        Dim::Dim2Array => {
          create_texture_3d_storage(
            gl::TEXTURE_2D_ARRAY,
            format,
            iformat,
            encoding,
            D::width(size),
            D::height(size),
            D::depth(size),
            levels,
          );
          Ok(())
        }
      }
    }

    None => Err(TextureError::texture_storage_creation_failed(format!(
      "unsupported texture pixel format: {:?}",
      pf
    ))),
  }
}

fn create_texture_1d_storage(
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  levels: usize,
) {
  for level in 0..levels {
    let w = w / (1 << level as u32);

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
  target: GLenum,
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  h: u32,
  levels: usize,
) {
  for level in 0..levels {
    let div = 1 << level as u32;
    let w = w / div;
    let h = h / div;

    unsafe {
      gl::TexImage2D(
        target,
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
  target: GLenum,
  format: GLenum,
  iformat: GLenum,
  encoding: GLenum,
  w: u32,
  h: u32,
  d: u32,
  levels: usize,
) {
  for level in 0..levels {
    let div = 1 << level as u32;
    let w = w / div;
    let h = h / div;
    let d = d / div;

    unsafe {
      gl::TexImage3D(
        target,
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
  levels: usize,
) {
  for level in 0..levels {
    let s = s / (1 << level as u32);

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
    }
  }
}

// set the unpack alignment for uploading aligned texels
fn set_unpack_alignment(skip_bytes: usize) {
  let unpack_alignment = match skip_bytes {
    0 => 8,
    2 => 2,
    4 => 4,
    _ => 1,
  };

  unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, unpack_alignment) };
}

// set the pack alignment for downloading aligned texels
fn set_pack_alignment(skip_bytes: usize) {
  let pack_alignment = match skip_bytes {
    0 => 8,
    2 => 2,
    4 => 4,
    _ => 1,
  };

  unsafe { gl::PixelStorei(gl::PACK_ALIGNMENT, pack_alignment) };
}

// Upload texels into the texture’s memory.
fn upload_texels<D, P, T>(
  target: GLenum,
  off: D::Offset,
  size: D::Size,
  texels: TexelUpload<[T]>,
) -> Result<(), TextureError>
where
  D: Dimensionable,
  P: Pixel,
{
  let pf = P::pixel_format();
  let pf_size = pf.format.bytes_len();
  let expected_bytes = D::count(size) * pf_size;

  if let Some(base_level_texels) = texels.get_base_level() {
    // number of bytes in the input texels argument
    let input_bytes = base_level_texels.len() * mem::size_of::<T>();

    if input_bytes < expected_bytes {
      // potential segfault / overflow; abort
      return Err(TextureError::not_enough_pixels(expected_bytes, input_bytes));
    }
  }

  // set the pixel row alignment to the required value for uploading data according to the width
  // of the texture and the size of a single pixel; here, skip_bytes represents the number of bytes
  // that will be skipped
  let skip_bytes = (D::width(size) as usize * pf_size) % 8;
  set_unpack_alignment(skip_bytes);

  // handle mipmaps
  match texels {
    TexelUpload::BaseLevel { texels, mipmaps } => {
      set_texels::<D, _>(target, pf, 0, size, off, texels)?;

      if mipmaps > 0 {
        unsafe { gl::GenerateMipmap(target) };
      }
    }

    TexelUpload::Levels(levels) => {
      for (i, &texels) in levels.into_iter().enumerate() {
        set_texels::<D, _>(target, pf, i as _, size, off, texels)?;
      }
    }

    TexelUpload::Reserve { mipmaps } => {
      if mipmaps > 0 {
        unsafe { gl::GenerateMipmap(target) };
      }
    }
  }

  Ok(())
}

// Set texels for a texture.
fn set_texels<D, T>(
  target: GLenum,
  pf: PixelFormat,
  level: GLint,
  size: D::Size,
  off: D::Offset,
  texels: &[T],
) -> Result<(), TextureError>
where
  D: Dimensionable,
{
  match opengl_pixel_format(pf) {
    Some((format, _, encoding)) => match D::dim() {
      Dim::Dim1 => unsafe {
        gl::TexSubImage1D(
          target,
          level,
          D::x_offset(off) as GLint,
          D::width(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },

      Dim::Dim2 => unsafe {
        gl::TexSubImage2D(
          target,
          level,
          D::x_offset(off) as GLint,
          D::y_offset(off) as GLint,
          D::width(size) as GLsizei,
          D::height(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },

      Dim::Dim3 => unsafe {
        gl::TexSubImage3D(
          target,
          level,
          D::x_offset(off) as GLint,
          D::y_offset(off) as GLint,
          D::z_offset(off) as GLint,
          D::width(size) as GLsizei,
          D::height(size) as GLsizei,
          D::depth(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },

      Dim::Cubemap => unsafe {
        gl::TexSubImage2D(
          gl::TEXTURE_CUBE_MAP_POSITIVE_X + D::z_offset(off),
          level,
          D::x_offset(off) as GLint,
          D::y_offset(off) as GLint,
          D::width(size) as GLsizei,
          D::width(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },

      Dim::Dim1Array => unsafe {
        gl::TexSubImage2D(
          target,
          level,
          D::x_offset(off) as GLint,
          D::y_offset(off) as GLint,
          D::width(size) as GLsizei,
          D::height(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },

      Dim::Dim2Array => unsafe {
        gl::TexSubImage3D(
          target,
          level,
          D::x_offset(off) as GLint,
          D::y_offset(off) as GLint,
          D::z_offset(off) as GLint,
          D::width(size) as GLsizei,
          D::height(size) as GLsizei,
          D::depth(size) as GLsizei,
          format,
          encoding,
          texels.as_ptr() as *const c_void,
        );
      },
    },

    None => return Err(TextureError::unsupported_pixel_format(pf)),
  }

  Ok(())
}
