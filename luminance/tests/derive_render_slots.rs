#![cfg(all(feature = "luminance-derive", feature = "mint"))]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

use luminance::{
  has_field::HasField,
  pixel::{Pixel, R32F, RGB32F},
  render_slots::RenderChannelDesc,
  render_slots::{CompatibleRenderSlots, RenderSlots},
  RenderSlots,
};

#[derive(RenderSlots)]
struct Slots {
  _diffuse: RGB32F,
  _normal: RGB32F,
  _test: R32F,
}

#[derive(RenderSlots)]
struct Slots1 {
  _diffuse: RGB32F,
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

  has_field::<"_diffuse", RGB32F, Slots>();
  has_field::<"_normal", RGB32F, Slots>();
  has_field::<"_test", R32F, Slots>();

  has_field::<"_diffuse", RGB32F, Slots1>();

  compatible::<Slots, Slots>();
  compatible::<Slots1, Slots>();
}

#[test]
fn render_slots() {
  let diffuse = RenderChannelDesc {
    name: "_diffuse",
    fmt: RGB32F::PIXEL_FMT,
  };
  let normal = RenderChannelDesc {
    name: "_normal",
    fmt: RGB32F::PIXEL_FMT,
  };
  let test = RenderChannelDesc {
    name: "_test",
    fmt: R32F::PIXEL_FMT,
  };

  assert_eq!(Slots::color_channel_descs(), &[diffuse, normal, test]);
}
