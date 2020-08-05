use gl::types::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::gl33::state::{Bind, GLState};
use crate::gl33::GL33;
use luminance::backend::color_slot::ColorSlot;
use luminance::backend::depth_slot::DepthSlot;
use luminance::backend::framebuffer::{Framebuffer as FramebufferBackend, FramebufferBackBuffer};
use luminance::framebuffer::{FramebufferError, IncompleteReason};
use luminance::texture::{Dim2, Dimensionable, Sampler};

pub struct Framebuffer<D>
where
  D: Dimensionable,
{
  pub(crate) handle: GLuint,
  renderbuffer: Option<GLuint>,
  pub(crate) size: D::Size,
  state: Rc<RefCell<GLState>>,
}

impl<D> Drop for Framebuffer<D>
where
  D: Dimensionable,
{
  fn drop(&mut self) {
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
}

unsafe impl<D> FramebufferBackend<D> for GL33
where
  D: Dimensionable,
{
  type FramebufferRepr = Framebuffer<D>;

  unsafe fn new_framebuffer<CS, DS>(
    &mut self,
    size: D::Size,
    _: usize,
    _: &Sampler,
  ) -> Result<Self::FramebufferRepr, FramebufferError>
  where
    CS: ColorSlot<Self, D>,
    DS: DepthSlot<Self, D>,
  {
    let mut handle: GLuint = 0;
    let color_formats = CS::color_formats();
    let depth_format = DS::depth_format();
    let mut depth_renderbuffer: Option<GLuint> = None;

    gl::GenFramebuffers(1, &mut handle);

    {
      let mut state = self.state.borrow_mut();

      state.bind_draw_framebuffer(handle);

      // reserve textures to speed slots creation
      let textures_needed = color_formats.len() + depth_format.map_or(0, |_| 1);
      state.reserve_textures(textures_needed);
    }

    // color textures
    if color_formats.is_empty() {
      gl::DrawBuffer(gl::NONE);
    } else {
      // specify the list of color buffers to draw to
      let color_buf_nb = color_formats.len() as GLsizei;
      let color_buffers: Vec<_> =
        (gl::COLOR_ATTACHMENT0..gl::COLOR_ATTACHMENT0 + color_buf_nb as GLenum).collect();

      gl::DrawBuffers(color_buf_nb, color_buffers.as_ptr());
    }

    // depth texture
    if depth_format.is_none() {
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

    let framebuffer = Framebuffer {
      handle,
      renderbuffer: depth_renderbuffer,
      size,
      state: self.state.clone(),
    };

    Ok(framebuffer)
  }

  unsafe fn attach_color_texture(
    _: &mut Self::FramebufferRepr,
    texture: &Self::TextureRepr,
    attachment_index: usize,
  ) -> Result<(), FramebufferError> {
    gl::FramebufferTexture(
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0 + attachment_index as GLenum,
      texture.handle,
      0,
    );

    Ok(())
  }

  unsafe fn attach_depth_texture(
    _: &mut Self::FramebufferRepr,
    texture: &Self::TextureRepr,
  ) -> Result<(), FramebufferError> {
    gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, texture.handle, 0);

    Ok(())
  }

  unsafe fn validate_framebuffer(
    framebuffer: Self::FramebufferRepr,
  ) -> Result<Self::FramebufferRepr, FramebufferError> {
    get_framebuffer_status()
      .map(move |_| framebuffer)
      .map_err(FramebufferError::from)
  }

  unsafe fn framebuffer_size(framebuffer: &Self::FramebufferRepr) -> D::Size {
    framebuffer.size
  }
}

fn get_framebuffer_status() -> Result<(), IncompleteReason> {
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
    _ => panic!(
      "unknown OpenGL framebuffer incomplete status! status={}",
      status
    ),
  }
}

unsafe impl FramebufferBackBuffer for GL33 {
  unsafe fn back_buffer(
    &mut self,
    size: <Dim2 as Dimensionable>::Size,
  ) -> Result<Self::FramebufferRepr, FramebufferError> {
    Ok(Framebuffer {
      handle: 0,
      renderbuffer: None,
      size,
      state: self.state.clone(),
    })
  }
}
