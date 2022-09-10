use std::ops::{Deref, DerefMut};

use crate::vertex_entity::{Mode, TessError, TessIndex, TessMapError, TessVertexData};

pub unsafe trait Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Backend representation of the tessellation.
  type TessRepr;

  /// Build a tessellation from vertex, index, instance and mode data.
  ///
  /// This method is used after a builder has enough information to build a [`Tess`]. The data is highly polymorphic so
  /// you will have to provide the types for the data containers when implementing both [`TessVertexData`]  and
  /// [`TessIndex`]. By convention, the idea is that:
  ///
  /// - If `S` is [`Interleaved`], you will want to take a single [`Vec`] for vertices and either a [`Vec`] or [`()`]
  ///   for indices and instance data.
  /// - If `S` is [`Deinterleaved`], you will want to take a [`Vec`] of [`DeinterleavedData`] for vertices and instance
  ///   data (it doesn’t make any difference for indices, so stick to [`Vec`] and [`()`]). [`DeinterleavedData`]
  ///   contains its own [`Vec`], so you basically end up with a [`Vec`] of [`Vec`], allowing to provide separate
  ///   attributes for all the vertices in their own containers.
  ///
  /// [`Interleaved`]: crate::tess::Interleaved
  /// [`Deinterleaved`]: crate::tess::Deinterleaved
  /// [`DeinterleavedData`]: crate::tess::DeinterleavedData
  unsafe fn build(
    &mut self,
    vertex_data: Option<V::Data>,
    index_data: Vec<I>,
    instance_data: Option<W::Data>,
    mode: Mode,
    restart_index: Option<I>,
  ) -> Result<Self::TessRepr, TessError>;

  /// Number of vertices available in the [`Tess`].
  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;

  /// Number of indices available in the [`Tess`].
  unsafe fn tess_indices_nb(tess: &Self::TessRepr) -> usize;

  /// Number of instance data available in the [`Tess`].
  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;

  /// Render the tessellation, starting at `start_index`, rendering `vert_nb` vertices, instantiating `inst_nb` times.
  ///
  /// If `inst_nb` is `0`, you should perform a render as if you were asking for `1`.
  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError>;
}

/// Slice vertex data on CPU.
///
/// This trait must be implemented by the backend so that it’s possible to _slice_ the vertex data. The idea is that the
/// vertex storage is backend-dependent; the backend can decide to cache the data, or not, and we should assume the data
/// to live in a memory that is costly to access. For this reason, slicing the vertex data requires to get an object on
/// which one can use [`Deref`] (and possibly [`DerefMut`]). The [`VertexSlice::vertices`] and
/// [`VertexSlice::vertices_mut`] methods must get such objects. Implementations will typically map memory regions and
/// retain the mapped data until the [`VertexSlice::VertexSliceRepr`] and [`VertexSlice::VertexSliceMutRepr`] objects
/// are dropped (c.f. [`Drop`]).
pub unsafe trait VertexSlice<'a, V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Backend representation of an immutable vertex slice.
  type VertexSliceRepr: 'a + Deref<Target = [T]>;

  /// Backend representation of a mutable vertex slice.
  type VertexSliceMutRepr: 'a + DerefMut<Target = [T]>;

  /// Obtain an immutable vertex slice.
  ///
  /// Even though this method returns an immutable slice, it has to mutably borrow the tessellation to prevent having
  /// two immutable slices living at the same time. This is a limitation that some backends might need.
  unsafe fn vertices(tess: &'a mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError>;

  /// Obtain a mutable vertex slice.
  unsafe fn vertices_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError>;
}

/// Slice index data on CPU.
///
/// This trait must be implemented by the backend so that it’s possible to _slice_ the index data. The idea is that the
/// index storage is backend-dependent; the backend can decide to cache the data, or not, and we should assume the data
/// to live in a memory that is costly to access. For this reason, slicing the index data requires to get an object on
/// which one can use [`Deref`] (and possibly [`DerefMut`]). The [`IndexSlice::indices`] and
/// [`IndexSlice::indices_mut`] methods must get such objects. Implementations will typically map memory regions and
/// retain the mapped data until the [`IndexSlice::IndexSliceRepr`] and [`IndexSlice::IndexSliceMutRepr`] objects
/// are dropped (c.f. [`Drop`]).
pub unsafe trait IndexSlice<'a, V, I, W, S>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Backend representation of an immutable index slice.
  type IndexSliceRepr: 'a + Deref<Target = [I]>;

  /// Backend representation of a mutable index slice.
  type IndexSliceMutRepr: 'a + DerefMut<Target = [I]>;

  /// Obtain an immutable index slice.
  ///
  /// Even though this method returns an immutable slice, it has to mutably borrow the tessellation to prevent having
  /// two immutable slices living at the same time. This is a limitation that some backends might need.
  unsafe fn indices(tess: &'a mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError>;

  /// Obtain a mutable index slice.
  unsafe fn indices_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::IndexSliceMutRepr, TessMapError>;
}

/// Slice instance data on CPU.
///
/// This trait must be implemented by the backend so that it’s possible to _slice_ the instance data. The idea is that the
/// instance storage is backend-dependent; the backend can decide to cache the data, or not, and we should assume the data
/// to live in a memory that is costly to access. For this reason, slicing the instance data requires to get an object on
/// which one can use [`Deref`] (and possibly [`DerefMut`]). The [`InstanceSlice::instances`] and
/// [`InstanceSlice::instances_mut`] methods must get such objects. Implementations will typically map memory regions and
/// retain the mapped data until the [`InstanceSlice::InstanceSliceRepr`] and [`InstanceSlice::InstanceSliceMutRepr`] objects
/// are dropped (c.f. [`Drop`]).
pub unsafe trait InstanceSlice<'a, V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Backend representation of an immutable instance slice.
  type InstanceSliceRepr: 'a + Deref<Target = [T]>;

  /// Backend representation of a mutable instance slice.
  type InstanceSliceMutRepr: 'a + DerefMut<Target = [T]>;

  /// Obtain an immutable instance slice.
  ///
  /// Even though this method returns an immutable slice, it has to mutably borrow the tessellation to prevent having
  /// two immutable slices living at the same time. This is a limitation that some backends might need.
  unsafe fn instances(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceRepr, TessMapError>;

  /// Obtain a mutable instance slice.
  unsafe fn instances_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError>;
}
