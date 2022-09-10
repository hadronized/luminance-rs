use std::{error, fmt};

use crate::{
  backend::{
    color_slot::ColorSlot,
    depth_stencil_slot::DepthStencilSlot,
    framebuffer::{Framebuffer as FramebufferBackend, FramebufferBackBuffer},
  },
  context::GraphicsContext,
  texture::{Dim2, Dimensionable, Sampler, TextureError},
};

/// Typed framebuffers.
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement [backend::framebuffer::Framebuffer].
/// - `D` is the dimension type. It must implement [`Dimensionable`].
/// - `CS` is the color slot type. It must implement [`ColorSlot`].
/// - `DS` is the depth slot type. It must implement [`DepthStencilSlot`].
///
/// [backend::framebuffer::Framebuffer]: crate::backend::framebuffer::Framebuffer
pub struct Framebuffer<B, D, CS, DS>
where
  B: ?Sized + FramebufferBackend<D>,
  D: Dimensionable,
  CS: ColorSlot<B, D>,
  DS: DepthStencilSlot<B, D>,
{
  pub(crate) repr: B::FramebufferRepr,
  color_slot: CS::ColorTextures,
  depth_stencil_slot: DS::DepthStencilTexture,
}

impl<B, D, CS, DS> Framebuffer<B, D, CS, DS>
where
  B: ?Sized + FramebufferBackend<D>,
  D: Dimensionable,
  CS: ColorSlot<B, D>,
  DS: DepthStencilSlot<B, D>,
{
  /// Create a new [`Framebuffer`].
  ///
  /// The `mipmaps` argument allows to pass the number of _extra precision layers_ the texture will
  /// be created with. A precision layer contains the same image as the _base layer_ but in a lower
  /// resolution. Currently, the way the resolution is computed depends on the backend, but it is
  /// safe to assume that it’s logarithmic in base 2 — i.e. at each layer depth, the resolution
  /// is divided by 2 on each axis.
  ///
  /// # Errors
  ///
  /// It is possible that the [`Framebuffer`] cannot be created. The [`FramebufferError`] provides
  /// the reason why.
  ///
  /// # Notes
  ///
  /// You might be interested in the [`GraphicsContext::new_framebuffer`] function instead, which
  /// is the exact same function, but benefits from more type inference (based on `&mut C`).
  pub fn new<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: Sampler,
  ) -> Result<Self, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe {
      let mut repr = ctx
        .backend()
        .new_framebuffer::<CS, DS>(size, mipmaps, &sampler)?;
      let color_slot = CS::reify_color_textures(ctx, size, mipmaps, &sampler, &mut repr, 0)?;
      let depth_stencil_slot = DS::reify_depth_texture(ctx, size, mipmaps, &sampler, &mut repr)?;

      let repr = B::validate_framebuffer(repr)?;

      Ok(Framebuffer {
        repr,
        color_slot,
        depth_stencil_slot,
      })
    }
  }

  /// Get the size of the framebuffer.
  pub fn size(&self) -> D::Size {
    unsafe { B::framebuffer_size(&self.repr) }
  }

  /// Access the carried color slot's texture(s).
  pub fn color_slot(&mut self) -> &mut CS::ColorTextures {
    &mut self.color_slot
  }

  /// Access the carried depth/stencil slot's texture.
  pub fn depth_stencil_slot(&mut self) -> &mut DS::DepthStencilTexture {
    &mut self.depth_stencil_slot
  }

  /// Consume this framebuffer and return the carried slots' texture(s).
  pub fn into_slots(self) -> (CS::ColorTextures, DS::DepthStencilTexture) {
    (self.color_slot, self.depth_stencil_slot)
  }

  /// Consume this framebuffer and return the carried [`ColorSlot::ColorTextures`].
  pub fn into_color_slot(self) -> CS::ColorTextures {
    self.color_slot
  }

  /// Consume this framebuffer and return the carried [`DepthStencilSlot::DepthStencilTexture`].
  pub fn into_depth_stencil_slot(self) -> DS::DepthStencilTexture {
    self.depth_stencil_slot
  }
}

impl<B> Framebuffer<B, Dim2, (), ()>
where
  B: ?Sized + FramebufferBackend<Dim2> + FramebufferBackBuffer,
{
  /// Get the _back buffer_ from the input context and the required resolution.
  pub fn back_buffer<C>(
    ctx: &mut C,
    size: <Dim2 as Dimensionable>::Size,
  ) -> Result<Self, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe { ctx.backend().back_buffer(size) }.map(|repr| Framebuffer {
      repr,
      color_slot: (),
      depth_stencil_slot: (),
    })
  }
}

/// Framebuffer error.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FramebufferError {
  /// Cannot create the framebuffer on the GPU.
  CannotCreate,
  /// Texture error.
  ///
  /// This happen while creating / associating the color / depth slots.
  TextureError(TextureError),
  /// Incomplete error.
  ///
  /// This happens when finalizing the construction of the framebuffer.
  Incomplete(IncompleteReason),
  /// Cannot attach something to a framebuffer.
  UnsupportedAttachment,
}

impl FramebufferError {
  /// Cannot create the framebuffer on the GPU.
  pub fn cannot_create() -> Self {
    FramebufferError::CannotCreate
  }

  /// Texture error.
  pub fn texture_error(e: TextureError) -> Self {
    FramebufferError::TextureError(e)
  }

  /// Incomplete error.
  pub fn incomplete(e: IncompleteReason) -> Self {
    FramebufferError::Incomplete(e)
  }

  /// Cannot attach something to a framebuffer.
  pub fn unsupported_attachment() -> Self {
    FramebufferError::UnsupportedAttachment
  }
}

impl fmt::Display for FramebufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      FramebufferError::CannotCreate => {
        f.write_str("cannot create the framebuffer on the GPU side")
      }

      FramebufferError::TextureError(ref e) => write!(f, "framebuffer texture error: {}", e),

      FramebufferError::Incomplete(ref e) => write!(f, "incomplete framebuffer: {}", e),

      FramebufferError::UnsupportedAttachment => f.write_str("unsupported framebuffer attachment"),
    }
  }
}

impl std::error::Error for FramebufferError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      FramebufferError::CannotCreate => None,
      FramebufferError::TextureError(e) => Some(e),
      FramebufferError::Incomplete(e) => Some(e),
      FramebufferError::UnsupportedAttachment => None,
    }
  }
}

impl From<TextureError> for FramebufferError {
  fn from(e: TextureError) -> Self {
    FramebufferError::TextureError(e)
  }
}

impl From<IncompleteReason> for FramebufferError {
  fn from(e: IncompleteReason) -> Self {
    FramebufferError::Incomplete(e)
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
  /// Unsupported framebuffer.
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

impl error::Error for IncompleteReason {}
