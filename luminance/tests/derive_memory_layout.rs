#![cfg(all(feature = "luminance-derive", feature = "mint"))]
#![allow(incomplete_features)]
#![feature(const_cmp)]

use luminance::{Std140, Std430};
use std::mem::size_of;

#[test]
fn std140() {
  #[derive(Debug, Std140)]
  struct X {
    pub x: f32,
  }

  let x = XStd140::from(X { x: 1. });
  assert_eq!(x._pad_x.len(), 0);
  assert_eq!(x._pad_struct.len(), 12);

  assert_eq!(size_of::<XStd140>(), 16);

  #[derive(Debug, Std140)]
  struct XYZ {
    pub xyz: mint::Vector3<f32>,
  }

  let xyz = XYZStd140::from(XYZ {
    xyz: mint::Vector3 {
      x: 1.,
      y: 2.,
      z: 3.,
    },
  });
  assert_eq!(xyz._pad_xyz.len(), 0);
  assert_eq!(xyz._pad_struct.len(), 4);

  assert_eq!(size_of::<XYZStd140>(), 16);

  #[derive(Debug, Std140)]
  struct XYZW {
    xyz: mint::Vector3<f32>,
    w: f32,
  }

  let xyzw = XYZWStd140::from(XYZW {
    xyz: mint::Vector3 {
      x: 1.,
      y: 2.,
      z: 3.,
    },
    w: 4.,
  });
  assert_eq!(xyzw._pad_xyz.len(), 0);
  assert_eq!(xyzw._pad_w.len(), 0);
  assert_eq!(xyzw._pad_struct.len(), 0);

  assert_eq!(size_of::<XYZWStd140>(), 16);

  #[derive(Debug, Std140)]
  struct XYZWSwapped {
    w: f32,
    xyz: mint::Vector3<f32>,
  }

  let xyzw_swapped = XYZWSwappedStd140::from(XYZWSwapped {
    w: 1.,
    xyz: mint::Vector3 {
      x: 2.,
      y: 3.,
      z: 4.,
    },
  });
  assert_eq!(xyzw_swapped._pad_w.len(), 0);
  assert_eq!(xyzw_swapped._pad_xyz.len(), 12);
  assert_eq!(xyzw_swapped._pad_struct.len(), 4);

  assert_eq!(size_of::<XYZWSwappedStd140>(), 32);
}

#[test]
fn std430() {
  #[derive(Debug, Std430)]
  struct X {
    pub x: f32,
  }

  let x = XStd430::from(X { x: 1. });
  assert_eq!(x._pad_x.len(), 0);
  assert_eq!(x._pad_struct.len(), 0);

  assert_eq!(size_of::<XStd430>(), 4);

  #[derive(Debug, Std430)]
  struct XYZ {
    pub xyz: mint::Vector3<f32>,
  }

  let xyz = XYZStd430::from(XYZ {
    xyz: mint::Vector3 {
      x: 1.,
      y: 2.,
      z: 3.,
    },
  });
  assert_eq!(xyz._pad_xyz.len(), 0);
  assert_eq!(xyz._pad_struct.len(), 4);

  assert_eq!(size_of::<XYZStd430>(), 16);

  #[derive(Debug, Std430)]
  struct XYZW {
    xyz: mint::Vector3<f32>,
    w: f32,
  }

  let xyzw = XYZWStd430::from(XYZW {
    xyz: mint::Vector3 {
      x: 1.,
      y: 2.,
      z: 3.,
    },
    w: 4.,
  });
  assert_eq!(xyzw._pad_xyz.len(), 0);
  assert_eq!(xyzw._pad_w.len(), 0);
  assert_eq!(xyzw._pad_struct.len(), 0);

  assert_eq!(size_of::<XYZWStd430>(), 16);

  #[derive(Debug, Std430)]
  struct XYZWSwapped {
    w: f32,
    xyz: mint::Vector3<f32>,
  }

  let xyzw_swapped = XYZWSwappedStd430::from(XYZWSwapped {
    w: 1.,
    xyz: mint::Vector3 {
      x: 2.,
      y: 3.,
      z: 4.,
    },
  });
  assert_eq!(xyzw_swapped._pad_w.len(), 0);
  assert_eq!(xyzw_swapped._pad_xyz.len(), 12);
  assert_eq!(xyzw_swapped._pad_struct.len(), 4);

  assert_eq!(size_of::<XYZWSwappedStd430>(), 32);
}
