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

use cgmath::{InnerSpace, One, Rotation, Rotation3};
use luminance::{
  backend::{Backend, Error},
  context::Context,
  depth_stencil::DepthWrite,
  dim::{CubeFace, Cubemap, Dim2, Off2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  pixel::{NormRGB8UI, NormUnsigned},
  primitive::TriangleStrip,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  texture::{InUseTexture, Mipmaps, Texture, TextureSampling},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
  vertex_storage::Interleaved,
  Uniforms,
};
use mint::ColumnMatrix4;
use std::{error::Error as ErrorTrait, fmt};

use crate::{
  shared::{cube, CubeVertex, FragSlot},
  Example, InputAction, LoopFeedback, PlatformServices,
};

// A bunch of shaders sources. The SKYBOX_* shaders are used to render the skybox all around your
// scene. ENV_MAP_* are used to perform environment mapping on the cube.
const SKYBOX_VS_SRC: &str = include_str!("cubemap-viewer-vs.glsl");
const SKYBOX_FS_SRC: &str = include_str!("cubemap-viewer-fs.glsl");
const ENV_MAP_VS_SRC: &str = include_str!("env-mapping-vs.glsl");
const ENV_MAP_FS_SRC: &str = include_str!("env-mapping-fs.glsl");

// In theory, you shouldn’t have to change those, but in case you need: if you increase the
// values, you get a faster movement when you move the cursor around.
const CAMERA_SENSITIVITY_YAW: f32 = 0.001;
const CAMERA_SENSITIVITY_PITCH: f32 = 0.001;
const CAMERA_SENSITIVITY_STRAFE_FORWARD: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_BACKWARD: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_LEFT: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_RIGHT: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_UP: f32 = 0.1;
const CAMERA_SENSITIVITY_STRAFE_DOWN: f32 = 0.1;
const CAMERA_SENSITIVITY_FOVY_CHANGE: f32 = 0.1;
const CAMERA_FOVY_RAD: f32 = std::f32::consts::FRAC_PI_2;

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
  CannotCreateTexture(Box<dyn ErrorTrait>),
  CannotUploadToFace(Box<dyn ErrorTrait>),
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
#[derive(Uniforms)]
struct SkyboxShaderInterface {
  #[uniform(unbound)]
  view: Uni<ColumnMatrix4<f32>>,
  #[uniform(unbound)]
  fovy: Uni<f32>,
  #[uniform(unbound)]
  aspect_ratio: Uni<f32>,
  #[uniform(unbound)]
  skybox: Uni<InUseTexture<Cubemap, NormUnsigned>>,
}

// The shader interface for the cube.
#[derive(Uniforms)]
struct EnvironmentMappingShaderInterface {
  #[uniform(unbound)]
  projection: Uni<ColumnMatrix4<f32>>,
  #[uniform(unbound)]
  view: Uni<ColumnMatrix4<f32>>,
  #[uniform(unbound)]
  aspect_ratio: Uni<f32>,
  #[uniform(unbound)]
  environment: Uni<InUseTexture<Cubemap, NormUnsigned>>,
}

pub struct LocalExample {
  skybox: Texture<Cubemap, NormRGB8UI>,
  aspect_ratio: f32,
  fovy: f32,
  projection: cgmath::Matrix4<f32>,
  cam_orient: cgmath::Quaternion<f32>,
  cam_view: cgmath::Matrix4<f32>,
  skybox_orient: cgmath::Quaternion<f32>,
  skybox_program: Program<(), (), TriangleStrip, FragSlot, SkyboxShaderInterface>,
  env_map_program:
    Program<CubeVertex, (), TriangleStrip, FragSlot, EnvironmentMappingShaderInterface>,
  fullscreen_quad: VertexEntity<(), TriangleStrip, ()>,
  cube: VertexEntity<CubeVertex, TriangleStrip, Interleaved<CubeVertex>>,
  last_cursor_pos: Option<cgmath::Vector2<f32>>,
  rotate_viewport: bool,
  x_theta: f32,
  y_theta: f32,
  eye: cgmath::Vector3<f32>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Skybox";

  fn bootstrap(
    [width, height]: [u32; 2],
    platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let skybox_img = platform.fetch_texture().expect("skybox image");
    let skybox = upload_cubemap(ctx, &skybox_img).expect("skybox cubemap");

    // Setup the camera part of the application. The projection will be used to render the cube.
    // The aspect_ratio is needed for the skybox. The rest is a simple “FPS-style” camera which
    // allows you to move around as if you were in a FPS.
    let aspect_ratio = width as f32 / height as f32;
    let fovy = clamp_fovy(CAMERA_FOVY_RAD);
    let projection = cgmath::perspective(cgmath::Rad(fovy), aspect_ratio, Z_NEAR, Z_FAR);
    let cam_orient = cgmath::Quaternion::from_angle_y(cgmath::Rad(0.));
    let cam_view = cgmath::Matrix4::one();
    let skybox_orient = cgmath::Quaternion::from_angle_y(cgmath::Rad(0.));

    // The shader program responsible in rendering the skybox.
    let skybox_program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(SKYBOX_VS_SRC)
        .no_primitive_stage()
        .add_shading_stage(SKYBOX_FS_SRC),
    )?;

    let env_map_program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(ENV_MAP_VS_SRC)
        .no_primitive_stage()
        .add_shading_stage(ENV_MAP_FS_SRC),
    )?;

    // A fullscreen quad used to render the skybox. The vertex shader will have to spawn the vertices
    // on the fly for this to work.
    let fullscreen_quad = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    // The cube that will reflect the skybox.
    let (cube_vertices, cube_indices) = cube(0.5);
    let cube = ctx.new_vertex_entity(
      VertexEntityBuilder::new()
        .add_vertices(Interleaved::new().set_vertices(cube_vertices))
        .add_indices(cube_indices),
    )?;

    // A bunch of render loop-specific variables used to track what’s happening with your keyboard and
    // mouse / trackpad.
    let last_cursor_pos = None;
    let rotate_viewport = false;
    let x_theta = 0.;
    let y_theta = 0.;
    let eye = cgmath::Vector3::new(0., 0., 3.);

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
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
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    _: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    // A special render state to use when rendering the skybox: because we render the skybox as
    // a fullscreen quad, we don’t want to write the depth (otherwise the cube won’t get displayed,
    // as there’s nothing closer than a fullscreen quad!).
    let rdr_st = RenderState::default().set_depth_write(DepthWrite::Off);

    let mut view_updated = false;
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        InputAction::Left => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            CAMERA_SENSITIVITY_STRAFE_LEFT,
            0.,
            0.,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Right => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            -CAMERA_SENSITIVITY_STRAFE_RIGHT,
            0.,
            0.,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Forward => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            0.,
            0.,
            CAMERA_SENSITIVITY_STRAFE_FORWARD,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Backward => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            0.,
            0.,
            -CAMERA_SENSITIVITY_STRAFE_BACKWARD,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Up => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            0.,
            CAMERA_SENSITIVITY_STRAFE_UP,
            0.,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Down => {
          let v = self.cam_orient.invert().rotate_vector(cgmath::Vector3::new(
            0.,
            -CAMERA_SENSITIVITY_STRAFE_DOWN,
            0.,
          ));
          self.eye -= v;
          view_updated = true;
        }

        InputAction::Resized { width, height } => {
          log::debug!("resized: {}×{}", width, height);
          self.aspect_ratio = width as f32 / height as f32;
          self.projection =
            cgmath::perspective(cgmath::Rad(self.fovy), self.aspect_ratio, Z_NEAR, Z_FAR);
          view_updated = true;
          self.back_buffer = ctx.back_buffer(Size2::new(width, height))?;
        }

        // When the cursor move, we need to update the last cursor position we know and, if needed,
        // update the Euler angles we use to orient the camera in space.
        InputAction::CursorMoved { x, y } => {
          let cursor = cgmath::Vector2::new(x, y);
          let last_cursor = self.last_cursor_pos.unwrap_or(cursor);
          let rel = cursor - last_cursor;

          self.last_cursor_pos = Some(cursor);

          if self.rotate_viewport {
            self.x_theta += CAMERA_SENSITIVITY_PITCH * rel.y as f32;
            self.y_theta += CAMERA_SENSITIVITY_YAW * rel.x as f32;

            // Stick the camera at verticals.
            self.x_theta = clamp_pitch(self.x_theta);

            view_updated = true;
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
          self.projection =
            cgmath::perspective(cgmath::Rad(self.fovy), self.aspect_ratio, Z_NEAR, Z_FAR);

          let cgmath::Deg(deg) = cgmath::Rad(self.fovy).into();
          log::info!("new fovy is {}°", deg);

          view_updated = true;
        }

        _ => (),
      }
    }

    // When the view is updated (i.e. the camera has moved or got re-oriented), we want to
    // recompute a bunch of quaternions (used to encode orientations) and matrices.
    if view_updated {
      let qy = cgmath::Quaternion::from_angle_y(cgmath::Rad(self.y_theta));
      let qx = cgmath::Quaternion::from_angle_x(cgmath::Rad(self.x_theta));

      // Orientation of the camera. Used for both the skybox (by inverting it) and the cube.
      self.cam_orient = (qx * qy).normalize();
      self.skybox_orient = self.cam_orient.invert();
      self.cam_view =
        cgmath::Matrix4::from(self.cam_orient) * cgmath::Matrix4::from_translation(-self.eye);
    }

    let skybox = &mut self.skybox;

    let projection: [[_; 4]; 4] = self.projection.into();
    let projection = projection.into();

    let view: [[_; 4]; 4] = self.cam_view.into();
    let view = view.into();

    let skybox_orient: [[_; 4]; 4] = cgmath::Matrix4::from(self.skybox_orient).into();
    let skybox_orient = skybox_orient.into();

    let skybox_program = &self.skybox_program;
    let env_map_program = &self.env_map_program;
    let fovy = self.fovy;
    let aspect_ratio = self.aspect_ratio;
    let fullscreen_quad = &self.fullscreen_quad;
    let cube = &self.cube;

    // We use two shaders in a single pipeline here: first, we render the skybox. Then, we render
    // the cube. A note here: it should be possible to change the way the skybox is rendered to
    // render it _after_ the cube. That will optimize some pixel shading when the cube is in the
    // viewport. For the sake of simplicity, we don’t do that here.
    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      let environment_map = frame.use_texture(skybox)?;

      // render the skybox
      frame.with_program(skybox_program, |mut frame| {
        frame.update(|mut update, unis| {
          update.set(&unis.view, &skybox_orient)?;
          update.set(&unis.fovy, &fovy)?;
          update.set(&unis.aspect_ratio, &aspect_ratio)?;
          update.set(&unis.skybox, &environment_map)
        })?;

        frame.with_render_state(&rdr_st, |mut frame| {
          frame.render_vertex_entity(fullscreen_quad.view(..4))
        })
      })?;

      // render the cube
      frame.with_program(env_map_program, |mut frame| {
        frame.update(|mut update, unis| {
          update.set(&unis.projection, &projection)?;
          update.set(&unis.view, &view)?;
          update.set(&unis.aspect_ratio, &aspect_ratio)?;
          update.set(&unis.environment, &environment_map)
        })?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(cube.view(..))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
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
  ctx: &mut Context<impl Backend>,
  img: &image::RgbImage,
) -> Result<Texture<Cubemap, NormRGB8UI>, AppError> {
  let width = img.width();
  let size = width / 4;

  // We discard “bad” images that don’t strictly respect the dimensions mentioned above.
  if img.height() / 3 != size {
    return Err(AppError::InvalidCubemapSize(width, img.height()));
  }

  let texels = img.as_raw();

  // Create the cubemap on the GPU; we ask for two mipmaps… because why not.
  let mut texture = ctx
    .reserve_texture(size, Mipmaps::count(4), &TextureSampling::default())
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
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
    CubeFace::PositiveX,
    width,
    size,
    [2 * 3 * size, width * 3 * size],
  )?;

  log::info!("uploading the -X face");
  face_buffer.clear();
  upload_face(
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
    CubeFace::NegativeX,
    width,
    size,
    [0, width * 3 * size],
  )?;

  log::info!("uploading the +Y face");
  face_buffer.clear();
  upload_face(
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
    CubeFace::PositiveY,
    width,
    size,
    [3 * size, 0],
  )?;

  log::info!("uploading the -Y face");
  face_buffer.clear();
  upload_face(
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
    CubeFace::NegativeY,
    width,
    size,
    [3 * size, width * 3 * size * 2],
  )?;

  log::info!("uploading the +Z face");
  face_buffer.clear();
  upload_face(
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
    CubeFace::PositiveZ,
    width,
    size,
    [3 * size, width * 3 * size],
  )?;

  log::info!("uploading the -Z face");
  face_buffer.clear();
  upload_face(
    ctx,
    &mut texture,
    &mut face_buffer,
    &texels,
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
  ctx: &mut Context<impl Backend>,
  texture: &Texture<Cubemap, NormRGB8UI>,
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

  ctx
    .set_texture_base_level(texture, (Off2::new(0, 0), face), size as u32, face_buffer)
    .map_err(|e| AppError::CannotUploadToFace(Box::new(e)))
}
