//! <https://github.com/phaazon/luminance-rs/issues/360>
//!
//! When a framebuffer is created, dropped and a new one is created, the new framebuffer doesn’t
//! seem to behave correctly. At the time of #360, it was detected on Windows.
//!
//! Because this example simply creates a framebuffer, drops it and creates another one, nothing is
//! displayed and the window flashes on the screen. Run the application in a tool such as apitrace
//! or renderdoc to analyze it.

use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::Framebuffer,
  pixel::Depth32F,
  texture::{Mipmaps, TextureSampling},
};

use crate::{shared::FragSlot, Example, InputAction, LoopFeedback, PlatformServices};

pub struct LocalExample;

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "funtest-#360-manually-drop-framebuffer";

  fn bootstrap(
    _: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let framebuffer: Framebuffer<Dim2, FragSlot, Depth32F> = ctx.new_framebuffer(
      Size2::new(1024, 1024),
      Mipmaps::No,
      &TextureSampling::default(),
    )?;

    std::mem::drop(framebuffer);

    // #360 occurs here after the drop
    let _: Framebuffer<Dim2, FragSlot, Depth32F> = ctx.new_framebuffer(
      Size2::new(1024, 1024),
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
