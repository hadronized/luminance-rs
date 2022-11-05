use std::fmt;

pub trait Dimensionable {
  /// Size type of a dimension (used to caracterize dimensions’ areas).
  type Size: Copy;

  /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
  type Offset: Copy;

  /// Zero offset.
  ///
  /// Any size added with `ZERO_OFFSET` must remain the size itself.
  const ZERO_OFFSET: Self::Offset;

  /// Reified [`Dim`].
  ///
  /// Implementors must ensure they map to the right variant of [`Dim`].
  fn dim() -> Dim;

  /// Width of the associated [`Dimensionable::Size`].
  fn width(size: &Self::Size) -> u32;

  /// Height of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
  fn height(_: &Self::Size) -> u32 {
    1
  }

  /// Depth of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
  fn depth(_: &Self::Size) -> u32 {
    1
  }

  /// X offset.
  fn x_offset(offset: &Self::Offset) -> u32;

  /// Y offset. If it doesn’t have one, set it to 0.
  fn y_offset(_: &Self::Offset) -> u32 {
    1
  }

  /// Z offset. If it doesn’t have one, set it to 0.
  fn z_offset(_: &Self::Offset) -> u32 {
    1
  }

  /// Amount of pixels this size represents.
  ///
  /// For 2D sizes, it represents the area; for 3D sizes, the volume; etc.
  /// For cubemaps, it represents the side length of the cube.
  fn count(size: &Self::Size) -> usize;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Size2 {
  pub width: u32,
  pub height: u32,
}

impl Size2 {
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Size3 {
  pub width: u32,
  pub height: u32,
  pub depth: u32,
}

impl Size3 {
  pub const fn new(width: u32, height: u32, depth: u32) -> Self {
    Self {
      width,
      height,
      depth,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Off2 {
  pub x: u32,
  pub y: u32,
}

impl Off2 {
  pub const fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Off3 {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}

impl Off3 {
  pub const fn new(x: u32, y: u32, z: u32) -> Self {
    Self { x, y, z }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Cubemap,
  Dim1Array,
  Dim2Array,
}

impl fmt::Display for Dim {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Dim::Dim1 => f.write_str("1D"),
      Dim::Dim2 => f.write_str("2D"),
      Dim::Dim3 => f.write_str("3D"),
      Dim::Cubemap => f.write_str("cubemap"),
      Dim::Dim1Array => f.write_str("1D array"),
      Dim::Dim2Array => f.write_str("2D array"),
    }
  }
}

/// 1D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1;

impl Dimensionable for Dim1 {
  type Offset = u32;
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = 0;

  fn dim() -> Dim {
    Dim::Dim1
  }

  fn width(w: &Self::Size) -> u32 {
    *w
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    *off
  }

  fn count(size: &Self::Size) -> usize {
    *size as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Offset = Off2;
  type Size = Size2;

  const ZERO_OFFSET: Self::Offset = Off2::new(0, 0);

  fn dim() -> Dim {
    Dim::Dim2
  }

  fn width(size: &Self::Size) -> u32 {
    size.width
  }

  fn height(size: &Self::Size) -> u32 {
    size.height
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    off.x
  }

  fn y_offset(off: &Self::Offset) -> u32 {
    off.y
  }

  fn count(size: &Self::Size) -> usize {
    size.width as usize * size.height as usize
  }
}

/// 3D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Offset = Off3;
  type Size = Size3;

  const ZERO_OFFSET: Self::Offset = Off3::new(0, 0, 0);

  fn dim() -> Dim {
    Dim::Dim3
  }

  fn width(size: &Self::Size) -> u32 {
    size.width
  }

  fn height(size: &Self::Size) -> u32 {
    size.height
  }

  fn depth(size: &Self::Size) -> u32 {
    size.depth
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    off.x
  }

  fn y_offset(off: &Self::Offset) -> u32 {
    off.y
  }

  fn z_offset(off: &Self::Offset) -> u32 {
    off.z
  }

  fn count(size: &Self::Size) -> usize {
    size.width as usize * size.height as usize * size.depth as usize
  }
}

/// Cubemap dimension.
#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Offset = (Off2, CubeFace);
  type Size = u32;

  const ZERO_OFFSET: Self::Offset = (Off2::new(0, 0), CubeFace::PositiveX);

  fn dim() -> Dim {
    Dim::Cubemap
  }

  fn width(s: &Self::Size) -> u32 {
    *s
  }

  fn height(s: &Self::Size) -> u32 {
    *s
  }

  fn depth(_: &Self::Size) -> u32 {
    6
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    off.0.x
  }

  fn y_offset(off: &Self::Offset) -> u32 {
    off.0.y
  }

  fn z_offset(off: &Self::Offset) -> u32 {
    match off.1 {
      CubeFace::PositiveX => 0,
      CubeFace::NegativeX => 1,
      CubeFace::PositiveY => 2,
      CubeFace::NegativeY => 3,
      CubeFace::PositiveZ => 4,
      CubeFace::NegativeZ => 5,
    }
  }

  fn count(size: &Self::Size) -> usize {
    let size = *size as usize;
    size * size
  }
}

/// Faces of a cubemap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CubeFace {
  /// The +X face of the cube.
  PositiveX,
  /// The -X face of the cube.
  NegativeX,
  /// The +Y face of the cube.
  PositiveY,
  /// The -Y face of the cube.
  NegativeY,
  /// The +Z face of the cube.
  PositiveZ,
  /// The -Z face of the cube.
  NegativeZ,
}

/// 1D array dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1Array;

impl Dimensionable for Dim1Array {
  type Offset = (u32, u32);
  type Size = (u32, u32);

  const ZERO_OFFSET: Self::Offset = (0, 0);

  fn dim() -> Dim {
    Dim::Dim1Array
  }

  fn width(size: &Self::Size) -> u32 {
    size.0
  }

  fn height(size: &Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    off.0
  }

  fn y_offset(off: &Self::Offset) -> u32 {
    off.1
  }

  fn count(&(width, layer): &Self::Size) -> usize {
    width as usize * layer as usize
  }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2Array;

impl Dimensionable for Dim2Array {
  type Offset = (Off2, u32);
  type Size = (Size2, u32);

  const ZERO_OFFSET: Self::Offset = (Off2::new(0, 0), 0);

  fn dim() -> Dim {
    Dim::Dim2Array
  }

  fn width(size: &Self::Size) -> u32 {
    size.0.width
  }

  fn height(size: &Self::Size) -> u32 {
    size.0.height
  }

  fn depth(size: &Self::Size) -> u32 {
    size.1
  }

  fn x_offset(off: &Self::Offset) -> u32 {
    off.0.x
  }

  fn y_offset(off: &Self::Offset) -> u32 {
    off.0.y
  }

  fn z_offset(off: &Self::Offset) -> u32 {
    off.1
  }

  fn count(&(Size2 { width, height }, layer): &Self::Size) -> usize {
    width as usize * height as usize * layer as usize
  }
}
