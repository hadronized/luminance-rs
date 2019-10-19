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

use gl;
use gl::types::*;
use luminance::context::GraphicsContext;
use luminance::framebuffer::{ColorSlot, DepthSlot};
use luminance::pixel::{ColorPixel, DepthPixel, PixelFormat, RenderablePixel};
use luminance::texture::{Dim2, Dimensionable, Flat, Layerable};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::marker::PhantomData;

use crate::state::{Bind, GraphicsState};
use crate::texture::{create_texture, opengl_target, RawTexture, Texture, TextureError};

/// Framebuffer error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FramebufferError {
  /// Texture error.
  ///
  /// This happen while creating / associating the color / depth slots.
  TextureError(TextureError),
  /// Incomplete error.
  ///
  /// This happens when finalizing the construction of the framebuffer.
  Incomplete(IncompleteReason),
}

impl fmt::Display for FramebufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      FramebufferError::TextureError(ref e) => write!(f, "framebuffer texture error: {}", e),

      FramebufferError::Incomplete(ref e) => write!(f, "incomplete framebuffer: {}", e),
    }
  }
}

/// Reason a framebuffer is incomplete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
  /// Incomplete framebuffer.
  Undefined,
  /// Incomplete attachment (color / depth).
  IncompleteAttachment,
  /// An attachment was missing.
  MissingAttachment,
  /// Incomplete draw buffer.
  IncompleteDrawBuffer,
  /// Incomplete read buffer.
  IncompleteReadBuffer,
  /// Unsupported.
  Unsupported,
  /// Incomplete multisample configuration.
  IncompleteMultisample,
  /// Incomplete layer targets.
  IncompleteLayerTargets,
}

impl fmt::Display for IncompleteReason {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      IncompleteReason::Undefined => write!(f, "incomplete reason"),
      IncompleteReason::IncompleteAttachment => write!(f, "incomplete attachment"),
      IncompleteReason::MissingAttachment => write!(f, "missing attachment"),
      IncompleteReason::IncompleteDrawBuffer => write!(f, "incomplete draw buffer"),
      IncompleteReason::IncompleteReadBuffer => write!(f, "incomplete read buffer"),
      IncompleteReason::Unsupported => write!(f, "unsupported"),
      IncompleteReason::IncompleteMultisample => write!(f, "incomplete multisample"),
      IncompleteReason::IncompleteLayerTargets => write!(f, "incomplete layer targets"),
    }
  }
}

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
pub struct Framebuffer<L, D, CS, DS>
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy,
      CS: ColorSlot,
      DS: DepthSlot {
  handle: GLuint,
  renderbuffer: Option<GLuint>,
  w: u32,
  h: u32,
  color_slot: CS::ColorTextures,
  depth_slot: DS::DepthTexture,
  state: Rc<RefCell<GraphicsState>>,
  _l: PhantomData<L>,
  _d: PhantomData<D>,
}

impl Framebuffer<Flat, Dim2, (), ()> {
  /// Get the back buffer with the given dimension.
  pub fn back_buffer<C>(
    ctx: &mut C,
    size: <Dim2 as Dimensionable>::Size
  ) -> Self
  where C: GraphicsContext {
    Framebuffer {
      handle: 0,
      renderbuffer: None,
      w: size[0],
      h: size[1],
      color_slot: (),
      depth_slot: (),
      state: ctx.state().clone(),
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
  /// Create a new farmebuffer.
  ///
  /// You’re always handed at least the base level of the texture. If you require any *additional*
  /// levels, you can pass the number via the `mipmaps` parameter.
  pub fn new<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
  ) -> Result<Framebuffer<L, D, CS, DS>, FramebufferError>
  where C: GraphicsContext {
    let mipmaps = mipmaps + 1;
    let mut handle: GLuint = 0;
    let color_formats = CS::color_formats();
    let depth_format = DS::depth_format();
    let target = opengl_target(L::layering(), D::dim());
    let mut textures = vec![0; color_formats.len() + if depth_format.is_some() { 1 } else { 0 }];
    let mut depth_texture: Option<GLuint> = None;
    let mut depth_renderbuffer: Option<GLuint> = None;

    unsafe {
      gl::GenFramebuffers(1, &mut handle);

      ctx.state().borrow_mut().bind_draw_framebuffer(handle);

      // generate all the required textures once; the textures vec will be reduced and dispatched
      // into other containers afterwards (in ColorSlot::reify_textures)
      gl::GenTextures((textures.len()) as GLint, textures.as_mut_ptr());

      // color textures
      if color_formats.is_empty() {
        gl::DrawBuffer(gl::NONE);
      } else {
        for (i, (format, texture)) in color_formats.iter().zip(&textures).enumerate() {
          ctx.state().borrow_mut().bind_texture(target, *texture);
          create_texture::<L, D>(target, size, mipmaps, *format, Default::default())
            .map_err(FramebufferError::TextureError)?;
          gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as GLenum, *texture, 0);
        }

        // specify the list of color buffers to draw to
        let color_buf_nb = color_formats.len() as GLsizei;
        let color_buffers: Vec<_> =
          (gl::COLOR_ATTACHMENT0..gl::COLOR_ATTACHMENT0 + color_buf_nb as GLenum).collect();

        gl::DrawBuffers(color_buf_nb, color_buffers.as_ptr());
      }

      // depth texture, if exists
      if let Some(format) = depth_format {
        let texture = textures.pop().unwrap();

        ctx.state().borrow_mut().bind_texture(target, texture);
        create_texture::<L, D>(target, size, mipmaps, format, Default::default())
          .map_err(FramebufferError::TextureError)?;
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, texture, 0);

        depth_texture = Some(texture);
      } else {
        let mut renderbuffer: GLuint = 0;

        gl::GenRenderbuffers(1, &mut renderbuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
        gl::RenderbufferStorage(
          gl::RENDERBUFFER,
          gl::DEPTH_COMPONENT32F,
          D::width(size) as GLsizei,
          D::height(size) as GLsizei,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0); // FIXME: see whether really needed

        gl::FramebufferRenderbuffer(
          gl::FRAMEBUFFER,
          gl::DEPTH_ATTACHMENT,
          gl::RENDERBUFFER,
          renderbuffer,
        );

        depth_renderbuffer = Some(renderbuffer);
      }

      ctx.state().borrow_mut().bind_texture(target, 0); // FIXME: see whether really needed

      let framebuffer = Framebuffer {
        handle,
        renderbuffer: depth_renderbuffer,
        w: D::width(size),
        h: D::height(size),
        color_slot: CS::reify_textures(ctx, size, mipmaps, &mut textures.into_iter()),
        depth_slot: DS::reify_texture(ctx, size, mipmaps, depth_texture),
        state: ctx.state().clone(),
        _l: PhantomData,
        _d: PhantomData,
      };

      match get_status() {
        Ok(_) => {
          ctx.state().borrow_mut().bind_draw_framebuffer(0); // FIXME: see whether really needed

          Ok(framebuffer)
        }

        Err(reason) => {
          ctx.state().borrow_mut().bind_draw_framebuffer(0); // FIXME: see whether really needed

          framebuffer.destroy();

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
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
      }

      if self.handle != 0 {
        gl::DeleteFramebuffers(1, &self.handle);
        self.state.borrow_mut().bind_vertex_array(0, Bind::Cached);
      }
    }
  }

  /// OpenGL handle of the framebuffer.
  #[inline]
  pub(crate) fn handle(&self) -> GLuint {
    self.handle
  }

  /// Width of the framebuffer.
  #[inline]
  pub fn width(&self) -> u32 {
    self.w
  }

  /// Height of the framebuffer.
  #[inline]
  pub fn height(&self) -> u32 {
    self.h
  }

  /// Access the underlying color slot.
  #[inline]
  pub fn color_slot(&self) -> &CS::ColorTextures {
    &self.color_slot
  }

  /// Access the underlying depth slot.
  #[inline]
  pub fn depth_slot(&self) -> &DS::DepthTexture {
    &self.depth_slot
  }
}

fn get_status() -> Result<(), IncompleteReason> {
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
    _ => panic!("unknown OpenGL framebuffer incomplete status! status={}", status),
  }
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub unsafe trait ColorSlot<L, D>
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy {
  /// Textures associated with this color slot.
  type ColorTextures;

  /// Turn a color slot into a list of pixel formats.
  fn color_formats() -> Vec<PixelFormat>;

  /// Reify a list of raw textures.
  fn reify_textures<C, I>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    textures: &mut I,
  ) -> Self::ColorTextures
  where
    C: GraphicsContext,
    I: Iterator<Item = GLuint>;
}

pub struct Nein;

unsafe impl<L, D> ColorSlot<L, D> for Nein
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy {
  type ColorTextures = ();

  fn color_formats() -> Vec<PixelFormat> {
    Vec::new()
  }

  fn reify_textures<C, I>(_: &mut C, _: D::Size, _: usize, _: &mut I) -> Self::ColorTextures
  where
    C: GraphicsContext,
    I: Iterator<Item = GLuint> {
  }
}

unsafe impl<L, D, P> ColorSlot<L, D> for P
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy,
      Self: ColorPixel + RenderablePixel {
  type ColorTextures = Texture<L, D, P>;

  fn color_formats() -> Vec<PixelFormat> {
    vec![P::pixel_format()]
  }

  fn reify_textures<C, I>(ctx: &mut C, size: D::Size, mipmaps: usize, textures: &mut I) -> Self::ColorTextures
  where C: GraphicsContext,
        I: Iterator<Item = GLuint> {
    let color_texture = textures.next().unwrap();

    unsafe {
      let raw = RawTexture::new(
        ctx.state().clone(),
        color_texture,
        opengl_target(L::layering(), D::dim()),
      );
      Texture::from_raw(raw, size, mipmaps)
    }
  }
}

macro_rules! impl_color_slot_tuple {
  ($($pf:ident),*) => {
    unsafe impl<L, D, $($pf),*> ColorSlot<L, D> for ($($pf),*)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          $(
            $pf: ColorPixel + RenderablePixel
          ),* {
      type ColorTextures = ($(Texture<L, D, $pf>),*);

      fn color_formats() -> Vec<PixelFormat> {
        vec![$($pf::pixel_format()),*]
      }

      fn reify_textures<C, I>(
        ctx: &mut C,
        size: D::Size,
        mipmaps: usize,
        textures: &mut I
      ) -> Self::ColorTextures
      where C: GraphicsContext,
            I: Iterator<Item = GLuint> {
        ($($pf::reify_textures(ctx, size, mipmaps, textures)),*)
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

impl_color_slot_tuples!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub unsafe trait DepthSlot<L, D>
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy {
  /// Texture associated with this color slot.
  type DepthTexture;

  /// Turn a depth slot into a pixel format.
  fn depth_format() -> Option<PixelFormat>;

  /// Reify a raw textures into a depth slot.
  fn reify_texture<C, T>(ctx: &mut C, size: D::Size, mipmaps: usize, texture: T) -> Self::DepthTexture
  where C: GraphicsContext,
        T: Into<Option<GLuint>>;
}

unsafe impl<L, D> DepthSlot<L, D> for ()
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy {
  type DepthTexture = ();

  fn depth_format() -> Option<PixelFormat> {
    None
  }

  fn reify_texture<C, T>(_: &mut C, _: D::Size, _: usize, _: T) -> Self::DepthTexture
  where C: GraphicsContext,
        T: Into<Option<GLuint>> {
  }
}

unsafe impl<L, D, P> DepthSlot<L, D> for P
where L: Layerable,
      D: Dimensionable,
      D::Size: Copy,
      P: DepthPixel {
  type DepthTexture = Texture<L, D, P>;

  fn depth_format() -> Option<PixelFormat> {
    Some(P::pixel_format())
  }

  fn reify_texture<C, T>(ctx: &mut C, size: D::Size, mipmaps: usize, texture: T) -> Self::DepthTexture
  where C: GraphicsContext,
        T: Into<Option<GLuint>> {
    unsafe {
      let raw = RawTexture::new(
        ctx.state().clone(),
        texture.into().unwrap(),
        opengl_target(L::layering(), D::dim()),
      );
      Texture::from_raw(raw, size, mipmaps)
    }
  }
}

