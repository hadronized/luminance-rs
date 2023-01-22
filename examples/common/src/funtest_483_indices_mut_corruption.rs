use luminance::{
  backend::{Backend, Error},
  context::Context,
  primitive::Point,
  vertex_entity::{VertexEntity, VertexEntityBuilder},
  vertex_storage::{Interleaved, Interleaving},
};

use crate::{shared::Vertex, Example, InputAction, LoopFeedback, PlatformServices};

pub struct LocalExample;

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "funtest-483-indices-mut-corruption";

  fn bootstrap(
    _: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let vertices = [
      Vertex {
        co: mint::Vector2 { x: 1., y: 2. },
        color: mint::Vector3 {
          x: 0.,
          y: 1.,
          z: 1.,
        },
      },
      Vertex {
        co: mint::Vector2 { x: -1., y: 2. },
        color: mint::Vector3 {
          x: 1.,
          y: 0.,
          z: 1.,
        },
      },
      Vertex {
        co: mint::Vector2 { x: 1., y: -2. },
        color: mint::Vector3 {
          x: 1.,
          y: 1.,
          z: 0.,
        },
      },
    ];

    let mut triangle: VertexEntity<Vertex, Point, Interleaving> = ctx.new_vertex_entity(
      VertexEntityBuilder::new()
        .add_vertices(Interleaved::new().set_vertices(vertices))
        .add_indices([0, 1, 2]),
    )?;

    let slice = triangle.indices();

    log::info!("slice before mutation is: {slice:?}");

    slice.copy_from_slice(&[10, 20, 30]);
    ctx.update_indices(&mut triangle)?;

    {
      let slice = triangle.indices();
      log::info!("slice after mutation is: {slice:?}");
    }

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
