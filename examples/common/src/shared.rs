use luminance::{namespace, Vertex};

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
  pub co: mint::Vector2<f32>,
  pub color: mint::Vector3<f32>,
}

// definition of a single instance
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "Namespace")]
pub struct Instance {
  pub position: mint::Vector2<f32>,
  pub weight: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "Namespace")]
pub struct CubeVertex {
  pub co3: mint::Vector3<f32>,
  pub nor: mint::Vector3<f32>,
}

impl CubeVertex {
  pub fn new(co3: mint::Vector3<f32>, nor: mint::Vector3<f32>) -> Self {
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

// /// RGB texture.
// pub type RGBTexture = Texture<Dim2, NormRGB8UI>;
//
// pub fn load_texture(
//   context: &mut impl GraphicsContext<Backend = Backend>,
//   platform: &mut impl PlatformServices,
// ) -> Option<RGBTexture> {
//   let img = platform
//     .fetch_texture()
//     .map_err(|e| log::error!("error while loading image: {}", e))
//     .ok()?;
//   let (width, height) = img.dimensions();
//   let texels = img.as_raw();
//
//   // create the luminance texture; the third argument is the number of mipmaps we want (leave it
//   // to 0 for now) and the latest is the sampler to use when sampling the texels in the
//   // shader (we’ll just use the default one)
//   //
//   // the GenMipmaps argument disables mipmap generation (we don’t care so far)
//   context
//     .new_texture_raw(
//       [width, height],
//       Sampler::default(),
//       TexelUpload::base_level(texels, 0),
//     )
//     .map_err(|e| log::error!("error while creating texture: {}", e))
//     .ok()
// }
