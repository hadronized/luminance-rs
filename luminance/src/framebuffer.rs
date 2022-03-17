//! Framebuffers and render targets.
//!
//! A framebuffer is a GPU object which responsibility is to hold renders — i.e. a rasterized
//! scene. Currently, a framebuffer is the only support of rendering: you cannot render directly
//! into a texture. Instead, you have to use a [`Framebuffer`], which automatically handles for
//! you the texture creation, mapping and handling to receive renders.
//!
//! # Framebuffer creation
//!
//! Framebuffers are created via the [`Framebuffer::new`] method. Creating framebuffers require a
//! bit of explanation as it highly depends on _refinement types_. When you create a new
//! framebuffer, you are required to provide types to drive the creation and subsequent possible
//! operations you will be able to perform. Framebuffers have three important type variables:
//!
//! - `D`, a type representing a dimension and that must implement [`Dimensionable`]. That types
//!   gives information on what kind of sizes and offsets a framebuffer will operate on/with.
//! - `CS`, a _color slot_. Color slots are described in the [backend::color_slot] module.
//! - `DS`, a _depth/stencil slot_. Depth/stencil slots are described in the
//!   [backend::depth_stencil_slot] module.
//!
//! You are limited in which types you can choose — the list is visible as implementors of traits
//! in [backend::color_slot] and [backend::depth_stencil_slot].
//!
//! Once a [`Framebuffer`] is created, you can do basically two main operations on it:
//!
//! - Render things to it.
//! - Retreive color and depth slots to perform further operations.
//!
//! # Rendering to a framebuffer
//!
//! Rendering is pretty straightforward: you have to use a [`PipelineGate`] to initiate a render by
//! passing a reference on your [`Framebuffer`]. Once the pipeline is done, the [`Framebuffer`]
//! contains the result of the render.
//!
//! # Manipulating slots
//!
//! Slots’ types depend entirely on the types you choose in [`Framebuffer`]. The rule is that any
//! type that implements [`ColorSlot`] will be associated another type: that other type (in this
//! case, [`ColorSlot::ColorTextures`]) will be the type you can use to manipulate textures. The
//! same applies to [`DepthStencilSlot`] with [`DepthStencilSlot::DepthStencilTexture`].
//!
//! In the case you don’t want a given slot, you can mute it with the unit type: [`()`], which will
//! effectively completely disable generating textures for that slot.
//!
//! You can access the color slot via [`Framebuffer::color_slot`]. You can access the depth/stencil
//! slot via [`Framebuffer::depth_stencil_slot`]. Once you get textures from the color slots, you
//! can use them as regular textures as input of next renders, for instance.
//!
//! ## Note on type generation
//!
//! Because framebuffers are highly subject to refinement typing, types are transformed at
//! compile-time by using the type-system to provide you with a good and comfortable experience.
//! If you use a single pixel format as color slot, for instance, you will get a single texture
//! (whose pixel format will be the same as the type variable you set). The dimension of the
//! texture will be set to the same as the framebuffer, too.
//!
//! Now if you use a tuple of pixel formats, you will get a tuple of textures, each having the
//! correct pixel format. That feature allows to generate complex types by using a _pretty simple_
//! input type. This is what we call _type constructors_ — type families in functional languages.
//! All this look a bit magical but the type-system ensures it’s total and not as magic as you
//! might think.
//!
//! [backend::color_slot]: crate::backend::color_slot
//! [backend::depth_stencil_slot]: crate::backend::depth_stencil_slot
//! [`PipelineGate`]: crate::pipeline::PipelineGate

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
