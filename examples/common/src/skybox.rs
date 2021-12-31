//! This program shows how to use cubemaps to implement the concept of skyboxes. It expects as
//! single CLI argument the path to a texture that encodes a skybox. The supported scheme is
//! the following:
//!
//! ```text
//!           |<--- width --->|
//!         – ┌───┬───┬───────┐
//!         ^ │   │ U │       │
//!         | ├───┼───┼───┬───┤
//!  height | │ L │ F │ R │ B │
//!         | ├───┼───┼───┴───┤
//!         v │   │ D │       │
//!         – └───┴───┴───────┘
//! ```
//!
//! Where F = front, L = left, R = right, B = behind, U = up and D = down.
//!
//! <https://docs.rs/luminance>

use std::{error::Error, fmt};

// This example is heavy on linear algebra. :)
use cgmath::{
  perspective, Deg, InnerSpace as _, Matrix4, One as _, Quaternion, Rad, Rotation, Rotation3,
  Vector3,
};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  depth_stencil::Write,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, TextureBinding},
  pixel::{NormRGB8UI, NormUnsigned},
  render_state::RenderState,
  shader::{types::Mat44, Program, Uniform},
  tess::{Mode, Tess},
  texture::{CubeFace, Cubemap, Dim2, Sampler, TexelUpload, Texture},
  Backend,
};
use shared::cube;

use crate::{
  shared::{self, CubeVertex, Semantics, VertexIndex},
  Example, InputAction, LoopFeedback, PlatformServices,
};

// A bunch of shaders sources. The SKYBOX_* shader is used to render the skybox all around your
// scene. ENV_MAP_* is the shader used to perform environment mapping on the cube.
const SKYBOX_VS_SRC: &str = include_str!("cubemap-viewer-vs.glsl");
const SKYBOX_FS_SRC: &str = include_str!("cubemap-viewer-fs.glsl");
const ENV_MAP_VS_SRC: &str = include_str!("env-mapping-vs.glsl");
const ENV_MAP_FS_SRC: &str = include_str!("env-mapping-fs.glsl");

// In theory, you shouldn’t have to change those, but in case you need: if you increase the
// values, you get a faster movement when you move the cursor around.
const CAMERA_SENSITIVITY_YAW: f32 = 0.001;
const CAMERA_SENSITIVITY_PITCH: f32 = 0.001;
const CAMERA_FOVY_RAD: f32 = std::f32::consts::FRAC_PI_2;
const CAMERA_SENSITIVITY_STRAFE_FORWARD: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_BACKWARD: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_LEFT: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_RIGHT: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_UP: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_DOWN: f32 = 0.1;
const CAMERA_SENSITIVITY_FOVY_CHANGE: f32 = 0.1;

// When projecting objects from 3D to 2D, we need to encode the project with a “minimum clipping
// distance” and a “maximum” one. Those values encode such a pair of numbers. If you want to see
// objects further than Z_FAR, you need to increment Z_FAR. For the sake of this example, you
// shoudn’t need to change these.
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

// What can go wrong while running this example. We use dyn Error instead of importing the
// luminance’s error types because we don’t really care about inspecting them.
#[derive(Debug)]
enum AppError {
  InvalidCubemapSize(u32, u32),
  CannotCreateTexture(Box<dyn Error>),
  CannotUploadToFace(Box<dyn Error>),
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      AppError::InvalidCubemapSize(w, h) => {
        write!(f, "invalid cubemap size: width={}, height={}", w, h)
      }

      AppError::CannotCreateTexture(ref e) => write!(f, "cannot create texture: {}", e),

      AppError::CannotUploadToFace(ref e) => write!(f, "cannot upload to a cubemap face: {}", e),
    }
  }
}

// The shader interface for the skybox.
//
// You will notice the presence of the aspect_ratio, which is needed to correct the
// aspect ratio of your screen (we don’t have a projection matrix here).
#[derive(UniformInterface)]
struct SkyboxShaderInterface {
  #[uniform(unbound)]
  view: Uniform<Mat44<f32>>,
  #[uniform(unbound)]
  fovy: Uniform<f32>,
  #[uniform(unbound)]
  aspect_ratio: Uniform<f32>,
  #[uniform(unbound)]
  skybox: Uniform<TextureBinding<Cubemap, NormUnsigned>>,
}

// The shader interface for the cube.
#[derive(UniformInterface)]
struct EnvironmentMappingShaderInterface {
  #[uniform(unbound)]
  projection: Uniform<Mat44<f32>>,
  #[uniform(unbound)]
  view: Uniform<Mat44<f32>>,
  #[uniform(unbound)]
  aspect_ratio: Uniform<f32>,
  #[uniform(unbound)]
  environment: Uniform<TextureBinding<Cubemap, NormUnsigned>>,
}

pub struct LocalExample {
  skybox: Texture<Cubemap, NormRGB8UI>,
  aspect_ratio: f32,
  fovy: f32,
  projection: Matrix4<f32>,
  cam_orient: Quaternion<f32>,
  cam_view: Matrix4<f32>,
  skybox_orient: Quaternion<f32>,
  skybox_program: Program<(), (), SkyboxShaderInterface>,
  env_map_program: Program<Semantics, (), EnvironmentMappingShaderInterface>,
  fullscreen_quad: Tess<()>,
  cube: Tess<CubeVertex, VertexIndex>,
  last_cursor_pos: Option<[f32; 2]>,
  rotate_viewport: bool,
  x_theta: f32,
  y_theta: f32,
  eye: Vector3<f32>,
  view_updated: bool,
}

impl Example for LocalExample {
  fn bootstrap(
    platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let skybox_img = platform.fetch_texture().expect("skybox image");
    let skybox = upload_cubemap(context, &skybox_img).expect("skybox cubemap");

    let [width, height] = [800., 600.];

    // Setup the camera part of the application. The projection will be used to render the cube.
    // The aspect_ratio is needed for the skybox. The rest is a simple “FPS-style” camera which
    // allows you to move around as if you were in a FPS.
    let aspect_ratio = width as f32 / height as f32;
    let fovy = clamp_fovy(CAMERA_FOVY_RAD);
    let projection = perspective(Rad(fovy), aspect_ratio, Z_NEAR, Z_FAR);
    let cam_orient = Quaternion::from_angle_y(Rad(0.));
    let cam_view = Matrix4::one();
    let skybox_orient = Quaternion::from_angle_y(Rad(0.));

    // The shader program responsible in rendering the skybox.
    let skybox_program = context
      .new_shader_program::<(), (), SkyboxShaderInterface>()
      .from_strings(SKYBOX_VS_SRC, None, None, SKYBOX_FS_SRC)
      .expect("skybox program creation")
      .ignore_warnings();

    let env_map_program = context
      .new_shader_program::<Semantics, (), EnvironmentMappingShaderInterface>()
      .from_strings(ENV_MAP_VS_SRC, None, None, ENV_MAP_FS_SRC)
      .expect("environment mapping program creation")
      .ignore_warnings();

    // A fullscreen quad used to render the skybox. The vertex shader will have to spawn the vertices
    // on the fly for this to work.
    let fullscreen_quad = context
      .new_tess()
      .set_mode(Mode::TriangleStrip)
      .set_render_vertex_nb(4)
      .build()
      .expect("fullscreen quad tess creation");

    // The cube that will reflect the skybox.
    let (cube_vertices, cube_indices) = cube(0.5);
    let cube = context
      .new_tess()
      .set_vertices(&cube_vertices[..])
      .set_indices(&cube_indices[..])
      .set_mode(Mode::TriangleStrip)
      .set_primitive_restart_index(VertexIndex::max_value())
      .build()
      .expect("cube tess creation");

    // A bunch of renderloop-specific variables used to track what’s happening with your keyboard and
    // mouse / trackpad.
    let last_cursor_pos = None;
    let rotate_viewport = false;
    let x_theta = 0.;
    let y_theta = 0.;
    let eye = Vector3::new(0., 0., 3.);
    let view_updated = true;

    LocalExample {
      skybox,
      aspect_ratio,
      fovy,
      projection,
      cam_orient,
      cam_view,
      skybox_orient,
      skybox_program,
      env_map_program,
      fullscreen_quad,
      cube,
      last_cursor_pos,
      rotate_viewport,
      x_theta,
      y_theta,
      eye,
      view_updated,
    }
  }

  fn render_frame(
    mut self,
    _: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    // A special render state to use when rendering the skybox: because we render the skybox as
    // a fullscreen quad, we don’t want to write the depth (otherwise the cube won’t get displayed,
    // as there’s nothing closer than a fullscreen quad!).
    let rdr_st = RenderState::default().set_depth_write(Write::Off);

    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,

        InputAction::Left => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            CAMERA_SENSITIVITY_STRAFE_LEFT,
            0.,
            0.,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Right => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            -CAMERA_SENSITIVITY_STRAFE_RIGHT,
            0.,
            0.,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Forward => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            0.,
            CAMERA_SENSITIVITY_STRAFE_FORWARD,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Backward => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            0.,
            -CAMERA_SENSITIVITY_STRAFE_BACKWARD,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Up => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            CAMERA_SENSITIVITY_STRAFE_UP,
            0.,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Down => {
          let v = self.cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            -CAMERA_SENSITIVITY_STRAFE_DOWN,
            0.,
          ));
          self.eye -= v;
          self.view_updated = true;
        }

        InputAction::Resized { width, height } => {
          log::debug!("resized: {}×{}", width, height);
          self.aspect_ratio = width as f32 / height as f32;
          self.projection = perspective(Rad(self.fovy), self.aspect_ratio, Z_NEAR, Z_FAR);
        }

        // When the cursor move, we need to update the last cursor position we know and, if needed,
        // update the Euler angles we use to orient the camera in space.
        InputAction::CursorMoved { x, y } => {
          let [px, py] = self.last_cursor_pos.unwrap_or([x, y]);
          let [rx, ry] = [x - px, y - py];

          self.last_cursor_pos = Some([x, y]);

          if self.rotate_viewport {
            self.x_theta += CAMERA_SENSITIVITY_PITCH * ry as f32;
            self.y_theta += CAMERA_SENSITIVITY_YAW * rx as f32;

            // Stick the camera at verticals.
            self.x_theta = clamp_pitch(self.x_theta);

            self.view_updated = true;
          }
        }

        InputAction::PrimaryPressed => {
          self.rotate_viewport = true;
        }

        InputAction::PrimaryReleased => {
          self.rotate_viewport = false;
        }

        InputAction::VScroll { amount } => {
          self.fovy += amount * CAMERA_SENSITIVITY_FOVY_CHANGE;
          self.fovy = clamp_fovy(self.fovy);

          // Because the field-of-view has changed, we need to recompute the projection matrix.
          self.projection = perspective(Rad(self.fovy), self.aspect_ratio, Z_NEAR, Z_FAR);

          let Deg(deg) = Rad(self.fovy).into();
          log::info!("new fovy is {}°", deg);
        }

        _ => (),
      }
    }

    // When the view is updated (i.e. the camera has moved or got re-oriented), we want to
    // recompute a bunch of quaternions (used to encode orientations) and matrices.
    if self.view_updated {
      let qy = Quaternion::from_angle_y(Rad(self.y_theta));
      let qx = Quaternion::from_angle_x(Rad(self.x_theta));

      // Orientation of the camera. Used for both the skybox (by inverting it) and the cube.
      self.cam_orient = (qx * qy).normalize();
      self.skybox_orient = self.cam_orient.invert();
      self.cam_view = Matrix4::from(self.cam_orient) * Matrix4::from_translation(-self.eye);

      self.view_updated = false;
    }

    let mut pipeline_gate = context.new_pipeline_gate();
    let skybox = &mut self.skybox;
    let projection = Mat44::new(self.projection);
    let view = Mat44::new(Matrix4::from(self.cam_view));
    let skybox_program = &mut self.skybox_program;
    let env_map_program = &mut self.env_map_program;
    let skybox_orient = &self.skybox_orient;
    let fovy = self.fovy;
    let aspect_ratio = self.aspect_ratio;
    let fullscreen_quad = &self.fullscreen_quad;
    let cube = &self.cube;

    // We use two shaders in a single pipeline here: first, we render the skybox. Then, we render
    // the cube. A note here: it should be possible to change the way the skybox is rendered to
    // render it _after_ the cube. That will optimize some pixel shading when the cube is in the
    // viewport. For the sake of simplicity, we don’t do that here.
    let render = pipeline_gate
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          let environment_map = pipeline.bind_texture(skybox).unwrap();

          // render the skybox
          shd_gate.shade(skybox_program, |mut iface, unis, mut rdr_gate| {
            iface.set(&unis.view, Mat44::new(Matrix4::from(*skybox_orient)));
            iface.set(&unis.fovy, fovy);
            iface.set(&unis.aspect_ratio, aspect_ratio);
            iface.set(&unis.skybox, environment_map.binding());

            rdr_gate.render(&rdr_st, |mut tess_gate| tess_gate.render(fullscreen_quad))
          })?;

          // render the cube
          shd_gate.shade(env_map_program, |mut iface, unis, mut rdr_gate| {
            iface.set(&unis.projection, projection);
            iface.set(&unis.view, view);
            iface.set(&unis.aspect_ratio, aspect_ratio);
            iface.set(&unis.environment, environment_map.binding());

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(cube)
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}

// A helper function that prevents us from flipping the projection.
fn clamp_fovy(fovy: f32) -> f32 {
  fovy.min(std::f32::consts::PI - 0.0001).max(0.0001)
}

// A helper function that prevents moving the camera up and down in “reversed” direction. That will
// make the FPS camera “stop” at full verticals.
fn clamp_pitch(theta: f32) -> f32 {
  theta
    .max(-std::f32::consts::FRAC_PI_2)
    .min(std::f32::consts::FRAC_PI_2)
}

/// We need to extract the six faces of the cubemap from the loaded image. To do so, we divide the
/// image in 4×3 cells, and focus on the 6 cells on the following schemas:
///
///
/// ```text
///           |<--- width --->|
///         – ┌───┬───┬───────┐
///         ^ │   │ U │       │
///         | ├───┼───┼───┬───┤
///  height | │ L │ F │ R │ B │
///         | ├───┼───┼───┴───┤
///         v │   │ D │       │
///         – └───┴───┴───────┘
/// ```
///
/// Each cell has a resolution of width / 4 × width / 4, and width / 4 == height / 3(if not, then it’s not a cubemap).
fn upload_cubemap(
  context: &mut impl GraphicsContext<Backend = Backend>,
  img: &image::RgbImage,
) -> Result<Texture<Cubemap, NormRGB8UI>, AppError> {
  let width = img.width();
  let size = width / 4;

  // We discard “bad” images that don’t strictly respect the dimensions mentioned above.
  if img.height() / 3 != size {
    return Err(AppError::InvalidCubemapSize(width, img.height()));
  }

  let pixels = img.as_raw();

  // Create the cubemap on the GPU; we ask for two mipmaps… because why not.
  let mut texture = context
    .new_texture(size, Sampler::default(), TexelUpload::reserve(2))
    .map_err(|e| AppError::CannotCreateTexture(Box::new(e)))?;

  // Upload each face, starting from U, then L, F, R, B and finally D. This part of the code is
  // hideous.

  // A “face buffer” used to copy parts of the original image into a buffer that will be passed to
  // luminance to upload to a cubemap face. By the way, you might be wondering what THE FUCK are all
  // those “* 3” below -> RGB textures.
  let face_size_bytes = (size * size) as usize * 3;
  let mut face_buffer = Vec::with_capacity(face_size_bytes);

  let size = size as usize;
  let width = width as usize;

  log::info!("uploading the +X face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::PositiveX,
    width,
    size,
    [2 * 3 * size, width * 3 * size],
  )?;

  log::info!("uploading the -X face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::NegativeX,
    width,
    size,
    [0, width * 3 * size],
  )?;

  log::info!("uploading the +Y face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::PositiveY,
    width,
    size,
    [3 * size, 0],
  )?;

  log::info!("uploading the -Y face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::NegativeY,
    width,
    size,
    [3 * size, width * 3 * size * 2],
  )?;

  log::info!("uploading the +Z face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::PositiveZ,
    width,
    size,
    [3 * size, width * 3 * size],
  )?;

  log::info!("uploading the -Z face");
  face_buffer.clear();
  upload_face(
    &mut texture,
    &mut face_buffer,
    &pixels,
    CubeFace::NegativeZ,
    width as _,
    size as _,
    [3 * 3 * size, width * 3 * size],
  )?;

  Ok(texture)
}

// Upload to a single cubemap face.
//
// This is a two-step process: first, we upload to the face buffer. Then, we pass that face buffer
// to the luminance upload code.
fn upload_face(
  texture: &mut Texture<Cubemap, NormRGB8UI>,
  face_buffer: &mut Vec<u8>,
  pixels: &[u8],
  face: CubeFace,
  width: usize,
  size: usize,
  origin_offset: [usize; 2],
) -> Result<(), AppError> {
  for row in 0..size {
    let offset = origin_offset[1] + row * width as usize * 3;

    face_buffer.extend_from_slice(
      &pixels[offset + origin_offset[0]..offset + origin_offset[0] + size as usize * 3],
    );
  }

  texture
    .upload_part_raw(
      ([0, 0], face),
      size as u32,
      TexelUpload::base_level(face_buffer, 2),
    )
    .map_err(|e| AppError::CannotUploadToFace(Box::new(e)))
}
