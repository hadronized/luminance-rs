use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  pixel::RGB8UI,
  texture::{Mipmaps, Texture, TextureSampling},
};

use crate::{Example, InputAction, LoopFeedback, PlatformServices};

pub struct LocalExample;

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "funtest-pixel-array-encoding";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let _texture: Texture<Dim2, RGB8UI> = ctx.reserve_texture(
      Size2::new(width, height),
      Mipmaps::No,
      &TextureSampling::default(),
    )?;

    Ok(LocalExample)
  }

  fn render_frame(
    self,
    _: f32,
    _: impl Iterator<Item = InputAction>,
    _: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    Ok(LoopFeedback::Exit)
  }
}
