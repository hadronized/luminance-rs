use std::marker::PhantomData;

use crate::{
  dim::Dimensionable,
  render_slots::{CompatibleRenderSlots, DepthRenderSlot, RenderSlots},
};

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
  dropper: Box<dyn FnMut(usize)>,
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
    dropper: Box<dyn FnMut(usize)>,
  ) -> Self {
    Self {
      handle,
      size,
      layers,
      depth_layer,
      dropper,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }

  pub fn size(&self) -> &D::Size {
    &self.size
  }

  pub fn layers(&self) -> &RS::RenderLayers {
    &self.layers
  }

  pub fn depth_layer(&self) -> &DS::DepthRenderLayer {
    &self.depth_layer
  }
}

impl<D, RS, DS> Drop for Framebuffer<D, RS, DS>
where
  D: Dimensionable,
  RS: RenderSlots,
  DS: DepthRenderSlot,
{
  fn drop(&mut self) {
    (self.dropper)(self.handle);
  }
}

pub struct Back<S> {
  _phantom: PhantomData<*const S>,
}

impl<S> CompatibleRenderSlots<S> for Back<S> {}
