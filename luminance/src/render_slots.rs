use crate::render_channel::RenderChannel;

/// Render slots.
///
/// Render slots are used to represent the “structure” of render layer. For instance, a render layer might have a color
/// channel and a depth channel. For more complex examples, it could have a diffuse, specular, normal and shininess
/// channel.
pub trait RenderSlots {
  const CHANNELS: &'static [RenderChannel];

  fn channels_count() -> usize {
    Self::CHANNELS.len()
  }
}
