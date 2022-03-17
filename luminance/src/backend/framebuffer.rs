//! Framebuffer backend interface.
//!
//! This interface defines the low-level API framebuffers must implement to be usable.

use crate::{
  backend::{color_slot::ColorSlot, depth_stencil_slot::DepthStencilSlot, texture::TextureBase},
  framebuffer::FramebufferError,
  texture::{Dim2, Dimensionable, Sampler},
};

/// Framebuffer backend.
///
/// A type implementing [`Framebuffer`] must implement [`TextureBase`] in the first place, because framebuffers and
/// textures have a strong relationship.
///
/// Framebuffers implement a strong type contract with the safe interface and are associated with
/// [`ColorSlot`] and [`DepthStencilSlot`]. Those types provide associated types to adapt the kind
/// of data that will be allocated and provided to the user. For instance, if you want to have a
/// framebuffer that will not have any depth information, you can use `()` as a
/// [`DepthStencilSlot`]. The backends will then not allocate anything and no depth data will be
/// present at runtime.
///
/// The whole process of implementing this trait revolves around three main aspects:
///
/// 1. How to create the framebuffer on the backend. This is the [`Framebuffer::new_framebuffer`] method.
/// 2. How to attach color and depth data, which are [`Framebuffer::attach_color_texture`] and
///   [`Framebuffer::attach_depth_texture`].
/// 3. Valide the resulting framebuffer.
///
/// `D` is the dimension of the framebuffer, which must implement [`Dimensionable`], reifying the dimension at runtime.
/// This is a useful type to the backends, as it will provide methods to get the size, offsets, etc. to correctly create textures.
pub unsafe trait Framebuffer<D>: TextureBase
where
  D: Dimensionable,
{
  /// Backend representation of the framebuffer.
  type FramebufferRepr;

  /// Create a new framebuffer on the backend.
  ///
  /// `CS` is the [`ColorSlot`] and `DS` is the [`DepthStencilSlot`]. This function must create the part that is _only_
  /// relevant to the framebuffer, not to the color slots directly. It can still allocate enough storage for the slots
  /// but it doesn’t have the handles / representations of the slots yet.
  unsafe fn new_framebuffer<CS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
  ) -> Result<Self::FramebufferRepr, FramebufferError>
  where
    CS: ColorSlot<Self, D>,
    DS: DepthStencilSlot<Self, D>;

  /// Attach a single color data to the framebuffer.
  ///
  /// The `attachment_index` gives the rank of the texture in the case of MRT. This method will never be called if the
  /// color slot is `()`.
  unsafe fn attach_color_texture(
    framebuffer: &mut Self::FramebufferRepr,
    texture: &Self::TextureRepr,
    attachment_index: usize,
  ) -> Result<(), FramebufferError>;

  /// Attach a single depth data to the framebuffer.
  ///
  /// This method will never be called if the depth slot is `()`.
  unsafe fn attach_depth_texture(
    framebuffer: &mut Self::FramebufferRepr,
    texture: &Self::TextureRepr,
  ) -> Result<(), FramebufferError>;

  /// Validate the status of the framebuffer.
  ///
  /// This function is required because of the multi-step process required to create a full framebuffer. Once the
  /// framebuffer is created and its color and depth slots added, that method is called to ensure the state of the
  /// framebuffer is correct.
  unsafe fn validate_framebuffer(
    framebuffer: Self::FramebufferRepr,
  ) -> Result<Self::FramebufferRepr, FramebufferError>;

  /// Get the size of the framebuffer.
  ///
  /// The size is currently stored on the backend side, so this function extracts it from the backend.
  unsafe fn framebuffer_size(framebuffer: &Self::FramebufferRepr) -> D::Size;
}

/// Back buffer.
///
/// A back buffer is a special kind of [`Framebuffer`]. It’s a 2D (c.f. [`Dim2`]) framebuffer that is provided
/// exclusively by the backend. Even though it should be cached by the application, its method is — most of the
/// time — cheap to call, so it can be called in a render loop.
pub unsafe trait FramebufferBackBuffer: Framebuffer<Dim2> {
  /// Get the back buffer from the backend.
  ///
  /// The `size` argument should match the current size of the actual system framebuffer. Because this value depends on
  /// the system platform, it is not possible to compute it directly.
  unsafe fn back_buffer(
    &mut self,
    size: <Dim2 as Dimensionable>::Size,
  ) -> Result<Self::FramebufferRepr, FramebufferError>;
}
