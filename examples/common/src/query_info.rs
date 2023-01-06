//! This program demonstrates the Query API, allowing to get information about the GPU and the backend. This example
//! outputs the information via the `log` crate via `log::info`, so don’t forget to enable information level in the
//! executor you choose.
//!
//! <https://docs.rs/luminance>

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance::{backend::Backend, context::Context};

pub struct LocalExample;

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Query Info";

  fn bootstrap(
    _: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    log::info!("Backend author: {:?}", ctx.backend_author());
    log::info!("Backend name: {:?}", ctx.backend_name());
    log::info!("Backend version: {:?}", ctx.backend_version());
    log::info!(
      "Backend shading language version: {:?}",
      ctx.backend_shading_lang_version()
    );

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
