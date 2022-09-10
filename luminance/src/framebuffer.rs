use crate::{dim::Dimensionable, render_slots::RenderSlots};

#[derive(Debug)]
pub struct Framebuffer<D, RS>
where
  D: Dimensionable,
  RS: RenderSlots,
{
  handle: usize,
  size: D::Size,
  layers: RS::RenderLayers,
}

impl<D, RS> Framebuffer<D, RS>
where
  D: Dimensionable,
  RS: RenderSlots,
{
  pub unsafe fn new(handle: usize, size: D::Size, layers: RS::RenderLayers) -> Self {
    Self {
      handle,
      size,
      layers,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }

  pub fn size(&self) -> D::Size {
    self.size
  }

  pub fn layers(&self) -> &RS::RenderLayers {
    &self.layers
  }
}
