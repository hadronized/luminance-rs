use std::marker::PhantomData;

use crate::{backend::Backend, dim::Dimensionable, render_channel::RenderChannel};

/// Render slots.
///
/// Render slots are used to represent the “structure” of render layer. For instance, a render layer might have a color
/// channel and a depth channel. For more complex examples, it could have a diffuse, specular, normal and shininess
/// channel.
pub trait RenderSlots {
  type RenderLayers;

  const CHANNELS: &'static [RenderChannel];

  fn channels_count() -> usize {
    Self::CHANNELS.len()
  }

  unsafe fn new_render_layers<B, D>(
    backend: &mut B,
    size: D::Size,
  ) -> Result<Self::RenderLayers, B::Err>
  where
    B: Backend,
    D: Dimensionable;
}

impl RenderSlots for () {
  type RenderLayers = ();

  const CHANNELS: &'static [RenderChannel] = &[];

  fn channels_count() -> usize {
    0
  }

  unsafe fn new_render_layers<B, D>(_: &mut B, _: D::Size) -> Result<Self::RenderLayers, B::Err>
  where
    B: Backend,
    D: Dimensionable,
  {
    Ok(())
  }
}

pub trait CompatibleRenderSlots<S> {}

#[derive(Debug)]
pub struct RenderLayer<RC> {
  handle: usize,
  _phantom: PhantomData<*const RC>,
}

impl<RC> RenderLayer<RC> {
  pub unsafe fn new(handle: usize) -> Self {
    Self {
      handle,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }
}
