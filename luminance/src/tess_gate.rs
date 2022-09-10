//! Tessellation gates.
//!
//! A tessellation gate is a _pipeline node_ that allows to share [`Tess`] for deeper nodes.
//!
//! [`Tess`]: crate::tess::Tess

use crate::backend::tess_gate::TessGate as TessGateBackend;
use crate::vertex_entity::{TessIndex, TessVertexData, TessView};

/// Tessellation gate.
pub struct TessGate<'a, B>
where
  B: ?Sized,
{
  pub(crate) backend: &'a mut B,
}

impl<'a, B> TessGate<'a, B>
where
  B: ?Sized,
{
  /// Enter the [`TessGate`] by sharing a [`TessView`].
  pub fn render<'b, E, T, V, I, W, S>(&'b mut self, tess_view: T) -> Result<(), E>
  where
    B: TessGateBackend<V, I, W, S>,
    T: Into<TessView<'b, B, V, I, W, S>>,
    V: TessVertexData<S> + 'b,
    I: TessIndex + 'b,
    W: TessVertexData<S> + 'b,
    S: ?Sized + 'b,
  {
    let tess_view = tess_view.into();

    unsafe {
      self.backend.render(
        &tess_view.tess.repr,
        tess_view.start_index,
        tess_view.vert_nb,
        tess_view.inst_nb,
      );

      Ok(())
    }
  }
}
