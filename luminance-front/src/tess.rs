use crate::Backend;

pub use luminance::vertex_entity::{
  Deinterleaved, DeinterleavedData, Interleaved, Mode, TessError, TessIndexType, TessMapError,
  TessViewError, View,
};

pub type TessBuilder<'a, V, I = (), W = (), S = Interleaved> =
  luminance::vertex_entity::TessBuilder<'a, Backend, V, I, W, S>;
pub type Tess<V, I = (), W = (), S = Interleaved> =
  luminance::vertex_entity::Tess<Backend, V, I, W, S>;
pub type Vertices<'a, V, I, W, S, T> =
  luminance::vertex_entity::Vertices<'a, Backend, V, I, W, S, T>;
pub type VerticesMut<'a, V, I, W, S, T> =
  luminance::vertex_entity::VerticesMut<'a, Backend, V, I, W, S, T>;
pub type Indices<'a, V, I, W, S> = luminance::vertex_entity::Indices<'a, Backend, V, I, W, S>;
pub type IndicesMut<'a, V, I, W, S> = luminance::vertex_entity::IndicesMut<'a, Backend, V, I, W, S>;
pub type Instances<'a, V, I, W, S, T> =
  luminance::vertex_entity::Instances<'a, Backend, V, I, W, S, T>;
pub type InstancesMut<'a, V, I, W, S, T> =
  luminance::vertex_entity::InstancesMut<'a, Backend, V, I, W, S, T>;
pub type TessView<'a, V, I, W, S> = luminance::vertex_entity::TessView<'a, Backend, V, I, W, S>;
