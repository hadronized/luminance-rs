//! Tessellation gate backend interface.
//!
//! This interface defines the low-level API tessellation gates must implement to be usable.
//!
//! A tessellation gate allows to render [`Tess`] objects.

use crate::backend::vertex_entity::Tess;
use crate::vertex_entity::{TessIndex, TessVertexData};

/// Trait to implement to be able to render [`Tess`] objects.
///
/// Obviously, this trait requires [`Tess`] with its regular type variables (see its documentation for a better
/// understanding of the various type variables).
pub unsafe trait TessGate<V, I, W, S>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Render the [`Tess`] starting at `start_index`, for `vert_nb` vertices with `inst_nb` instances.
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  );
}
