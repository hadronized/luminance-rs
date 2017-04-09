//! Framebuffers and utility types and functions.
//!
//! Framebuffers are at the core of rendering. They’re the support of rendering operation and can
//! be used to highly enhance the visual aspect of a render. You’re always provided with at least
//! one framebuffer, `default_framebuffer()`. That function returns a framebuffer that represents –
//! for short – your screen’s back framebuffer. You can render to that framebuffer and when you
//! *swap* the window’s buffers, your render appears at the screen.
//!
//! # Framebuffers
//!
//! A framebuffer is an object maintaining the required GPU state to hold images your render to.
//! It gathers two important concepts:
//!
//! - *color buffers*;
//! - *depth buffers*.
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

use gl;
use gl::types::*;
use std::marker::PhantomData;

use chain::Chain;
use pixel::{ColorPixel, DepthPixel, PixelFormat, RenderablePixel};
use texture::{Dim2, Dimensionable, Flat, Layerable, RawTexture, Texture, TextureError,
              create_texture, opengl_target};

/// Framebuffer error.
///
/// `Incomplete(reason)` occurs at framebuffer creation and `reason` gives a `String` explaination
/// of the failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FramebufferError {
  TextureError(TextureError),
  Incomplete(IncompleteReason)
}

/// Reason a framebuffer is incomplete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
  Undefined,
  IncompleteAttachment,
  MissingAttachment,
  IncompleteDrawBuffer,
  IncompleteReadBuffer,
  Unsupported,
  IncompleteMultisample,
  IncompleteLayerTargets,
}

pub type Result<T> = ::std::result::Result<T, FramebufferError>;

/// Framebuffer with static layering, dimension, access and slots formats.
///
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
#[derive(Debug)]
pub struct Framebuffer<L, D, CS, DS>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<L, D>,
          DS: DepthSlot<L, D> {
  handle: GLuint,
  renderbuffer: Option<GLuint>,
  w: u32,
  h: u32,
  pub color_slot: CS,
  pub depth_slot: DS,
  _l: PhantomData<L>,
  _d: PhantomData<D>,
}

impl Framebuffer<Flat, Dim2, (), ()> {
  pub fn default(size: <Dim2 as Dimensionable>::Size) -> Self {
    Framebuffer {
      handle: 0,
      renderbuffer: None,
      w: size.0,
      h: size.1,
      color_slot: (),
      depth_slot: (),
      _l: PhantomData,
      _d: PhantomData,
    }
  }
}

impl<L, D, CS, DS> Drop for Framebuffer<L, D, CS, DS>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<L, D>,
          DS: DepthSlot<L, D> {
  fn drop(&mut self) {
    self.destroy();
  }
}

impl<L, D, CS, DS> Framebuffer<L, D, CS, DS>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<L, D>,
          DS: DepthSlot<L, D> {
  // FIXME: use LinkedList instead of Vec for textures
  /// Create a new farmebuffer.
  pub fn new(size: D::Size, mipmaps: usize) -> Result<Framebuffer<L, D, CS, DS>> {
    let mipmaps = mipmaps + 1;
    let mut handle: GLuint = 0;
    let color_formats = CS::color_formats();
    let depth_format = DS::depth_format();
    let target = opengl_target(L::layering(), D::dim());
    let mut textures: Vec<GLuint> = vec![0; (color_formats.len() + if depth_format.is_some() { 1 } else { 0 })]; // FIXME: remove that (inference)
    let mut depth_texture: Option<GLuint> = None;
    let mut depth_renderbuffer: Option<GLuint> = None;

    unsafe {
      gl::GenFramebuffers(1, &mut handle);

      gl::BindFramebuffer(gl::FRAMEBUFFER, handle);

      // generate all the required textures once; the textures vec will be reduced and dispatched
      // into other containers afterwards (in ColorSlot::reify_textures)
      gl::GenTextures((textures.len()) as GLint, textures.as_mut_ptr());

      // color textures
      if color_formats.is_empty() {
        gl::DrawBuffer(gl::NONE);
      } else {
        for (i, (format, texture)) in color_formats.iter().zip(&textures).enumerate() {
          gl::BindTexture(target, *texture);
          create_texture::<L, D>(target, size, mipmaps, *format, &Default::default()).map_err(FramebufferError::TextureError)?;
          gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as GLenum, *texture, 0);
        }

        // specify the list of color buffers to draw to
        let color_buf_nb = color_formats.len() as GLsizei;
        let color_buffers: Vec<_> = (gl::COLOR_ATTACHMENT0..gl::COLOR_ATTACHMENT0 + color_buf_nb as GLenum).collect();

        gl::DrawBuffers(color_buf_nb, color_buffers.as_ptr());
      }

      // depth texture, if exists
      if let Some(format) = depth_format {
        let texture = textures.pop().unwrap();

        gl::BindTexture(target, texture);
        create_texture::<L, D>(target, size, mipmaps, format, &Default::default()).map_err(FramebufferError::TextureError)?;
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, texture, 0);

        depth_texture = Some(texture);
      } else {
        let mut renderbuffer: GLuint = 0;

        gl::GenRenderbuffers(1, &mut renderbuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT32F, D::width(size) as GLsizei, D::height(size) as GLsizei);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, renderbuffer);

        depth_renderbuffer = Some(renderbuffer);
      }

      gl::BindTexture(target, 0);

      let framebuffer = Framebuffer {
        handle: handle,
        renderbuffer: depth_renderbuffer,
        w: D::width(size),
        h: D::height(size),
        color_slot: CS::reify_textures(size, mipmaps, &mut textures),
        depth_slot: DS::reify_texture(size, mipmaps, depth_texture),
        _l: PhantomData,
        _d: PhantomData,
      };

      match get_status() {
        Ok(_) => {
          gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

          Ok(framebuffer)
        },
        Err(reason) => {
          gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

          Self::destroy(&framebuffer);

          Err(FramebufferError::Incomplete(reason))
        }
      }
    }
  }

  // Destroy OpenGL-side stuff.
  fn destroy(&self) {
    unsafe {
      if let Some(renderbuffer) = self.renderbuffer {
        gl::DeleteRenderbuffers(1, &renderbuffer);
      }

      if self.handle != 0 {
        gl::DeleteFramebuffers(1, &self.handle);
      }
    }
  }

  #[inline]
  pub unsafe fn handle(&self) -> GLuint {
    self.handle
  }

  #[inline]
  pub fn width(&self) -> u32 {
    self.w
  }

  #[inline]
  pub fn height(&self) -> u32 {
    self.h
  }
}

fn get_status() -> ::std::result::Result<(), IncompleteReason> {
  let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };

  match status {
    gl::FRAMEBUFFER_COMPLETE => Ok(()),
    gl::FRAMEBUFFER_UNDEFINED => Err(IncompleteReason::Undefined),
    gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err(IncompleteReason::IncompleteAttachment),
    gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Err(IncompleteReason::MissingAttachment),
    gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => Err(IncompleteReason::IncompleteDrawBuffer),
    gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => Err(IncompleteReason::IncompleteReadBuffer),
    gl::FRAMEBUFFER_UNSUPPORTED => Err(IncompleteReason::Unsupported),
    gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => Err(IncompleteReason::IncompleteMultisample),
    gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => Err(IncompleteReason::IncompleteLayerTargets),
    _ => panic!("unknown OpenGL framebuffer incomplete status! status={}", status)
  }
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub trait ColorSlot<L, D> where L: Layerable, D: Dimensionable, D::Size: Copy {
  /// Turn a color slot into a list of pixel formats.
  fn color_formats() -> Vec<PixelFormat>;
  /// Reify a list of raw textures into a color slot.
  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self;
}

impl<L, D> ColorSlot<L, D> for () where L: Layerable, D: Dimensionable, D::Size: Copy {
  fn color_formats() -> Vec<PixelFormat> { Vec::new() }

  fn reify_textures(_: D::Size, _: usize, _: &mut Vec<GLuint>) -> Self { () }
}

impl<L, D, P> ColorSlot<L, D> for Texture<L, D, P>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> { vec![P::pixel_format()] }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let color_texture = textures.swap_remove(0);

    unsafe {
      let raw = RawTexture::new(color_texture, opengl_target(L::layering(), D::dim()));
      Texture::from_raw(raw, size, mipmaps)
    }
  }
}

impl<L, D, P, B> ColorSlot<L, D> for Chain<Texture<L, D, P>, B>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel + RenderablePixel,
          B: ColorSlot<L, D> {
  fn color_formats() -> Vec<PixelFormat> {
    let mut a = Texture::<L, D, P>::color_formats();
    a.extend(B::color_formats());
    a
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let a = Texture::<L, D, P>::reify_textures(size, mipmaps, textures);
    let b = B::reify_textures(size, mipmaps, textures);

    Chain(a, b)
  }
}

impl<L, D, P0, P1> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Texture<L, D, P1>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, b) = Chain::<Texture<L, D, P0>, Texture<L, D, P1>>::reify_textures(size, mipmaps, textures);
    (a, b)
  }
}

impl<L, D, P0, P1, P2> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Texture<L, D, P2>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, c)) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Texture<L, D, P2>>>::reify_textures(size, mipmaps, textures);
    (a, b, c)
  }
}

impl<L, D, P0, P1, P2, P3> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Texture<L, D, P3>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, d))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Texture<L, D, P3>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d)
  }
}

impl<L, D, P0, P1, P2, P3, P4> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Texture<L, D, P4>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, e)))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Texture<L, D, P4>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e)
  }
}

impl<L, D, P0, P1, P2, P3, P4, P5> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>, Texture<L, D, P5>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Texture<L, D, P5>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, f))))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Texture<L, D, P5>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f)
  }
}

impl<L, D, P0, P1, P2, P3, P4, P5, P6> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>, Texture<L, D, P5>, Texture<L, D, P6>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Texture<L, D, P6>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, g)))))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Texture<L, D, P6>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g)
  }
}

impl<L, D, P0, P1, P2, P3, P4, P5, P6, P7> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>, Texture<L, D, P5>, Texture<L, D, P6>, Texture<L, D, P7>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Texture<L, D, P7>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, h))))))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Texture<L, D, P7>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h)
  }
}

impl<L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>, Texture<L, D, P5>, Texture<L, D, P6>, Texture<L, D, P7>, Texture<L, D, P8>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel,
          P8: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Chain<Texture<L, D, P7>, Texture<L, D, P8>>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, i)))))))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Chain<Texture<L, D, P7>, Texture< L, D, P8>>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h, i)
  }
}

impl<L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8, P9> ColorSlot<L, D> for (Texture<L, D, P0>, Texture<L, D, P1>, Texture<L, D, P2>, Texture<L, D, P3>, Texture<L, D, P4>, Texture<L, D, P5>, Texture<L, D, P6>, Texture<L, D, P7>, Texture<L, D, P8>, Texture<L, D, P9>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel,
          P8: ColorPixel + RenderablePixel,
          P9: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Chain<Texture<L, D, P7>, Chain<Texture<L, D, P8>, Texture<L, D, P9>>>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<GLuint>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, Chain(i, j))))))))) = Chain::<Texture<L, D, P0>, Chain<Texture<L, D, P1>, Chain<Texture<L, D, P2>, Chain<Texture<L, D, P3>, Chain<Texture<L, D, P4>, Chain<Texture<L, D, P5>, Chain<Texture<L, D, P6>, Chain<Texture<L, D, P7>, Chain<Texture<L, D, P8>, Texture<L, D, P9>>>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h, i, j)
  }
}

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub trait DepthSlot<L, D> where L: Layerable, D: Dimensionable, D::Size: Copy {
  /// Turn a depth slot into a pixel format.
  fn depth_format() -> Option<PixelFormat>;
  /// Reify a raw textures into a depth slot.
  fn reify_texture<T>(size: D::Size, mipmaps: usize, texture: T) -> Self where T: Into<Option<GLuint>>; }

impl<L, D> DepthSlot<L, D> for () where L: Layerable, D: Dimensionable, D::Size: Copy {
  fn depth_format() -> Option<PixelFormat> { None }

  fn reify_texture<T>(_: D::Size, _: usize, _: T) -> Self where T: Into<Option<GLuint>> { () }
}

impl<L, D, P> DepthSlot<L, D> for Texture<L, D, P>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: DepthPixel {
  fn depth_format() -> Option<PixelFormat> { Some(P::pixel_format()) }

  fn reify_texture<T>(size: D::Size, mipmaps: usize, texture: T) -> Self where T: Into<Option<GLuint>> {
    unsafe {
      let raw = RawTexture::new(texture.into().unwrap(), opengl_target(L::layering(), D::dim()));
      Texture::from_raw(raw, size, mipmaps)
    }
  }
}
