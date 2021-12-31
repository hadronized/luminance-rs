use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pixel::RGB8UI,
  texture::{Dim2, Sampler, TexelUpload, Texture},
  Backend,
};

pub struct LocalExample;

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let _texture: Texture<Dim2, RGB8UI> = context
      .new_texture([100, 100], Sampler::default(), TexelUpload::reserve(0))
      .unwrap();

    LocalExample
  }

  fn render_frame(
    self,
    _: f32,
    _: Framebuffer<Dim2, (), ()>,
    _: impl Iterator<Item = InputAction>,
    _: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    LoopFeedback::Exit
  }
}
