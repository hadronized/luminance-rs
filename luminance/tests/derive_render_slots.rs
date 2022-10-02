#![cfg(all(feature = "luminance-derive", feature = "mint"))]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

use luminance::{
  has_field::HasField,
  render_channel::RenderChannelDesc,
  render_slots::{CompatibleRenderSlots, RenderSlots},
  RenderSlots,
};

#[derive(RenderSlots)]
struct Slots {
  _diffuse: mint::Vector3<f32>,
  _normal: mint::Vector3<f32>,
  _test: f32,
}

#[derive(RenderSlots)]
struct Slots1 {
  _diffuse: mint::Vector3<f32>,
}

#[test]
fn compatible_render_slots() {
  fn has_field<const NAME: &'static str, F, S>()
  where
    S: RenderSlots + HasField<NAME, FieldType = F>,
  {
  }

  fn compatible<S1, S2>()
  where
    S1: CompatibleRenderSlots<S2>,
  {
  }

  has_field::<"_diffuse", mint::Vector3<f32>, Slots>();
  has_field::<"_normal", mint::Vector3<f32>, Slots>();
  has_field::<"_test", f32, Slots>();

  has_field::<"_diffuse", mint::Vector3<f32>, Slots1>();

  compatible::<Slots, Slots>();
  compatible::<Slots1, Slots>();
}

#[test]
fn render_channels() {
  let diffuse = RenderChannelDesc {
    name: "_diffuse",
    ty: luminance::render_channel::RenderChannelType::Floating(
      luminance::render_channel::RenderChannelDim::Dim3,
    ),
  };
  let normal = RenderChannelDesc {
    name: "_normal",
    ty: luminance::render_channel::RenderChannelType::Floating(
      luminance::render_channel::RenderChannelDim::Dim3,
    ),
  };
  let test = RenderChannelDesc {
    name: "_test",
    ty: luminance::render_channel::RenderChannelType::Floating(
      luminance::render_channel::RenderChannelDim::Dim1,
    ),
  };

  assert_eq!(Slots::color_channel_descs(), &[diffuse, normal, test]);
}
