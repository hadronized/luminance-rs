#![allow(incomplete_features)]
#![feature(adt_const_params)]

use luminance::{
  has_field::HasField,
  namespace,
  render_channel::{RenderChannel, RenderChannelDim, RenderChannelType},
  render_slots::{CompatibleRenderSlots, RenderSlots},
  RenderSlots,
};

namespace! {
  Namespace = { "_diffuse", "_normal", "_test" }
}
#[derive(RenderSlots)]
#[slot(namespace = "Namespace")]
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
fn render_slots_channels() {
  assert_eq!(
    Slots::CHANNELS,
    &[
      RenderChannel {
        index: 0,
        name: "_color",
        ty: RenderChannelType::Floating,
        dim: RenderChannelDim::Dim3
      },
      RenderChannel {
        index: 1,
        name: "_test",
        ty: RenderChannelType::Floating,
        dim: RenderChannelDim::Dim1
      }
    ]
  );
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
