//! Tessellation backend interface.
//!
//! This interface defines the low-level API tessellations must implement to be usable.

use std::ops::{Deref, DerefMut};

use crate::tess::{Mode, TessError, TessIndex, TessMapError, TessVertexData};

pub unsafe trait Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type TessRepr;

  unsafe fn build(
    &mut self,
    vert: (Option<V::Data>, usize),
    inst: (Option<W::Data>, usize),
    index_data: Vec<I>,
    restart_index: Option<I>,
    mode: Mode,
  ) -> Result<Self::TessRepr, TessError>;

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError>;
}

pub unsafe trait VertexSlice<V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type VertexSliceRepr: Deref<Target = [T]>;
  type VertexSliceMutRepr: DerefMut<Target = [T]>;

  unsafe fn vertices(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError>;

  unsafe fn vertices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError>;
}

pub unsafe trait IndexSlice<V, I, W, S>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type IndexSliceRepr: Deref<Target = [I]>;
  type IndexSliceMutRepr: DerefMut<Target = [I]>;

  unsafe fn indices(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError>;

  unsafe fn indices_mut(tess: &mut Self::TessRepr)
    -> Result<Self::IndexSliceMutRepr, TessMapError>;
}

pub unsafe trait InstanceSlice<V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type InstanceSliceRepr: Deref<Target = [T]>;
  type InstanceSliceMutRepr: DerefMut<Target = [T]>;

  unsafe fn instances(tess: &mut Self::TessRepr) -> Result<Self::InstanceSliceRepr, TessMapError>;

  unsafe fn instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError>;
}
