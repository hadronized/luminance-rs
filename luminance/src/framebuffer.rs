use crate::{
  dim::Dimensionable,
  render_slots::{DepthRenderSlot, RenderSlots},
};

#[derive(Debug)]
pub struct Framebuffer<D, RS, DS>
where
  D: Dimensionable,
  RS: RenderSlots,
  DS: DepthRenderSlot,
{
  handle: usize,
  size: D::Size,
  layers: RS::RenderLayers,
  depth_layer: DS::DepthRenderLayer,
}

impl<D, RS, DS> Framebuffer<D, RS, DS>
where
  D: Dimensionable,
  RS: RenderSlots,
  DS: DepthRenderSlot,
{
  pub unsafe fn new(
    handle: usize,
    size: D::Size,
    layers: RS::RenderLayers,
    depth_layer: DS::DepthRenderLayer,
  ) -> Self {
    Self {
      handle,
      size,
      layers,
      depth_layer,
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

  pub fn depth_layer(&self) -> &DS::DepthRenderLayer {
    &self.depth_layer
  }
}
