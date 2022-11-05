use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  namespace,
  pixel::NormRGB8UI,
  texture::{TextureSampling, Texture},
  RenderSlots, Vertex,
};
use mint::{Vector2, Vector3};

use crate::PlatformServices;

// Render slots.
//
// A render slot represents the channels the end stage of a shader program is going to end up writing to. In our case,
// since we are only interested in rendering the color of each pixel, we will just have one single channel for the
// color.
#[derive(Clone, Copy, Debug, PartialEq, RenderSlots)]
pub struct FragSlot {
  frag: Vector3<f32>,
}

namespace! {
  Namespace = {
    "co",
    "co3",
    "color",
    "nor",
    "position",
    "weight"
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "Namespace")]
pub struct Vertex {
  pub co: Vector2<f32>,
  pub color: Vector3<f32>,
}

impl Vertex {
  pub const fn new(co: Vector2<f32>, color: Vector3<f32>) -> Self {
    Self { co, color }
  }
}

// definition of a single instance
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "Namespace")]
pub struct Instance {
  pub position: Vector2<f32>,
  pub weight: f32,
}

impl Instance {
  pub const fn new(position: Vector2<f32>, weight: f32) -> Self {
    Self { position, weight }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "Namespace")]
pub struct CubeVertex {
  pub co3: Vector3<f32>,
  pub nor: Vector3<f32>,
}

impl CubeVertex {
  pub const fn new(co3: Vector3<f32>, nor: Vector3<f32>) -> Self {
    Self { co3, nor }
  }
}

// Simple interleaved cube of given size.
#[rustfmt::skip]
pub fn cube(size: f32) -> ([CubeVertex; 24], [u32; 30]) {
  let s = size * 0.5;

  let vertices = [
    // first face
    CubeVertex::new([-s, -s,  s].into(), [ 0.,  0.,  1.].into()),
    CubeVertex::new([ s, -s,  s].into(), [ 0.,  0.,  1.].into()),
    CubeVertex::new([-s,  s,  s].into(), [ 0.,  0.,  1.].into()),
    CubeVertex::new([ s,  s,  s].into(), [ 0.,  0.,  1.].into()),
    // second face
    CubeVertex::new([ s, -s, -s].into(), [ 0.,  0., -1.].into()),
    CubeVertex::new([-s, -s, -s].into(), [ 0.,  0., -1.].into()),
    CubeVertex::new([ s,  s, -s].into(), [ 0.,  0., -1.].into()),
    CubeVertex::new([-s,  s, -s].into(), [ 0.,  0., -1.].into()),
    // third face
    CubeVertex::new([ s, -s,  s].into(), [ 1.,  0.,  0.].into()),
    CubeVertex::new([ s, -s, -s].into(), [ 1.,  0.,  0.].into()),
    CubeVertex::new([ s,  s,  s].into(), [ 1.,  0.,  0.].into()),
    CubeVertex::new([ s,  s, -s].into(), [ 1.,  0.,  0.].into()),
    // forth face
    CubeVertex::new([-s, -s, -s].into(), [-1.,  0.,  0.].into()),
    CubeVertex::new([-s, -s,  s].into(), [-1.,  0.,  0.].into()),
    CubeVertex::new([-s,  s, -s].into(), [-1.,  0.,  0.].into()),
    CubeVertex::new([-s,  s,  s].into(), [-1.,  0.,  0.].into()),
    // fifth face
    CubeVertex::new([-s,  s,  s].into(), [ 0.,  1.,  0.].into()),
    CubeVertex::new([ s,  s,  s].into(), [ 0.,  1.,  0.].into()),
    CubeVertex::new([-s,  s, -s].into(), [ 0.,  1.,  0.].into()),
    CubeVertex::new([ s,  s, -s].into(), [ 0.,  1.,  0.].into()),
    // sixth face
    CubeVertex::new([-s, -s, -s].into(), [ 0., -1.,  0.].into()),
    CubeVertex::new([ s, -s, -s].into(), [ 0., -1.,  0.].into()),
    CubeVertex::new([-s, -s,  s].into(), [ 0., -1.,  0.].into()),
    CubeVertex::new([ s, -s,  s].into(), [ 0., -1.,  0.].into()),
  ];

  let indices = [
    0, 1, 2, 3, u32::max_value(),
    4, 5, 6, 7, u32::max_value(),
    8, 9, 10,  11, u32::max_value(),
    12, 13, 14, 15, u32::max_value(),
    16, 17, 18, 19, u32::max_value(),
    20, 21, 22, 23, u32::max_value(),
  ];

  (vertices, indices)
}

/// RGB texture.
pub type RGBTexture = Texture<Dim2, NormRGB8UI>;

pub fn load_texture(
  context: &mut Context<impl Backend>,
  platform: &mut impl PlatformServices,
  mipmaps: usize,
) -> Option<RGBTexture> {
  let img = platform
    .fetch_texture()
    .map_err(|e| log::error!("error while loading image: {}", e))
    .ok()?;
  let (width, height) = img.dimensions();
  let texels = img.as_raw();

  // create the luminance texture; the third argument is the number of mipmaps we want (leave it
  // to 0 for now) and the latest is the sampler to use when sampling the texels in the
  // shader (we’ll just use the default one)
  //
  // the GenMipmaps argument disables mipmap generation (we don’t care so far)
  context
    .new_texture(
      Size2::new(width, height),
      mipmaps,
      TextureSampling::default(),
      texels,
    )
    .map_err(|e| log::error!("error while creating texture: {}", e))
    .ok()
}
