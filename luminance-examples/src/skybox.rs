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
//! Move the cursor around by keeping the left click pressed to move the camera.
//! Use WASD keys to move around.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

// This example is heavy on linear algebra. :)
use cgmath::{
  perspective, Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation, Rotation3, Transform, Vector3,
};
use glfw::{Action, Context as _, Key, MouseButton, WindowEvent};
use log::{error, info};
use luminance_derive::UniformInterface;
use luminance_front::context::GraphicsContext;
use luminance_front::depth_test::DepthWrite;
use luminance_front::pipeline::{PipelineState, TextureBinding};
use luminance_front::pixel::{NormRGB8UI, NormUnsigned};
use luminance_front::render_state::RenderState;
use luminance_front::shader::Uniform;
use luminance_front::tess::Mode;
use luminance_front::texture::{CubeFace, Cubemap, GenMipmaps, Sampler, Texture};
use luminance_front::Backend;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

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
// luminance’s error types because we don’t really care into inspecting them.
#[derive(Debug)]
enum AppError {
  MissingCLIArgument,
  CannotCreateSurface(Box<dyn Error>),
  UnknownFile(PathBuf),
  InvalidCubemapSize(u32, u32),
  CannotCreateTexture(Box<dyn Error>),
  CannotUploadToFace(Box<dyn Error>),
  CannotGrabBackBuffer(Box<dyn Error>),
  ShaderCompilationFailed(Box<dyn Error>),
  CannotBuildFullscreenQuad(Box<dyn Error>),
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      AppError::InvalidCubemapSize(w, h) => {
        write!(f, "invalid cubemap size: width={}, height={}", w, h)
      }

      AppError::CannotCreateSurface(ref e) => write!(f, "cannot create rendering surface: {}", e),
      AppError::UnknownFile(ref path) => write!(f, "cannot find cubemap file: {}", path.display()),

      AppError::MissingCLIArgument => f.write_str("missing path to cubemap file"),

      AppError::CannotCreateTexture(ref e) => write!(f, "cannot create texture: {}", e),

      AppError::CannotUploadToFace(ref e) => write!(f, "cannot upload to a cubemap face: {}", e),

      AppError::CannotGrabBackBuffer(ref e) => write!(f, "cannot grab the back buffer: {}", e),

      AppError::ShaderCompilationFailed(ref e) => write!(f, "cannot compile shader: {}", e),

      AppError::CannotBuildFullscreenQuad(ref e) => {
        write!(f, "cannot build fullscreen quad: {}", e)
      }
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
  view: Uniform<[[f32; 4]; 4]>,
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
  projection: Uniform<[[f32; 4]; 4]>,
  #[uniform(unbound)]
  view: Uniform<[[f32; 4]; 4]>,
  #[uniform(unbound)]
  aspect_ratio: Uniform<f32>,
  #[uniform(unbound)]
  environment: Uniform<TextureBinding<Cubemap, NormUnsigned>>,
}

fn main() {
  if let Err(e) = run() {
    error!("an error occurred: {}", e);
  }
}

fn run() -> Result<(), AppError> {
  // We use this simply because the log crate is cool and you should use it!
  env_logger::init();

  // Mandatory path to the skybox texture. We load it with the image crate and perform a set of
  // “cut operations” to extract each face. See the documentation of the upload_cubemap function.
  let img_path = std::env::args()
    .into_iter()
    .nth(1)
    .ok_or_else(|| AppError::MissingCLIArgument)?;
  let img = load_img(img_path)?;

  // Regular luminance surface creation.
  let dim = WindowDim::Windowed {
    width: 540,
    height: 960,
  };
  let mut surface = GlfwSurface::new_gl33("skybox example", WindowOpt::default().set_dim(dim))
    .map_err(|e| AppError::CannotCreateSurface(Box::new(e)))?;

  // Upload the loaded image into a luminance cubemap.
  let mut skybox = upload_cubemap(&mut surface, &img)?;

  let mut back_buffer = surface
    .back_buffer()
    .map_err(|e| AppError::CannotGrabBackBuffer(Box::new(e)))?;
  let [width, height] = back_buffer.size();

  // Setup the camera part of the application. The projection will be used to render the cube.
  // The aspect_ratio is needed for the skybox. The rest is a simple “FPS-style” camera which
  // allows you to move around as if you were in a FPS.
  let mut aspect_ratio = width as f32 / height as f32;
  let mut fovy = clamp_fovy(CAMERA_FOVY_RAD);
  let mut projection = perspective(Rad(fovy), aspect_ratio, Z_NEAR, Z_FAR);
  let mut cam_orient = Quaternion::from_angle_y(Rad(0.));
  let mut cam_view = Matrix4::one();
  let mut skybox_orient = Quaternion::from_angle_y(Rad(0.));

  // The shader program responsible in rendering the skybox.
  let mut skybox_program = surface
    .new_shader_program::<(), (), SkyboxShaderInterface>()
    .from_strings(SKYBOX_VS_SRC, None, None, SKYBOX_FS_SRC)
    .map_err(|e| AppError::ShaderCompilationFailed(Box::new(e)))?
    .ignore_warnings();

  let mut environment_mapping_program = surface
    .new_shader_program::<common::Semantics, (), EnvironmentMappingShaderInterface>()
    .from_strings(ENV_MAP_VS_SRC, None, None, ENV_MAP_FS_SRC)
    .map_err(|e| AppError::ShaderCompilationFailed(Box::new(e)))?
    .ignore_warnings();

  // A fullscreen quad used to render the skybox. The vertex shader will have to spawn the vertices
  // on the fly for this to work.
  let fullscreen_quad = surface
    .new_tess()
    .set_mode(Mode::TriangleStrip)
    .set_vertex_nb(4)
    .build()
    .map_err(|e| AppError::CannotBuildFullscreenQuad(Box::new(e)))?;

  // The cube that will reflect the skybox.
  let (cube_vertices, cube_indices) = common::cube(0.5);
  let cube = surface
    .new_tess()
    .set_vertices(&cube_vertices[..])
    .set_indices(&cube_indices[..])
    .set_mode(Mode::TriangleStrip)
    .set_primitive_restart_index(common::VertexIndex::max_value())
    .build()
    .unwrap();

  // A bunch of renderloop-specific variables used to track what’s happening with your keyboard and
  // mouse / trackpad.
  let mut resize = false;
  let mut last_cursor_pos = None;
  let mut rotate_viewport = false;
  let mut x_theta = 0.;
  let mut y_theta = 0.;
  let mut eye = Vector3::new(0., 0., 3.);
  let mut view_updated = true;

  // A special render state to use when rendering the skybox: because we render the skybox as
  // a fullscreen quad, we don’t want to write the depth (otherwise the cube won’t get displayed,
  // as there’s nothing closer than a fullscreen quad!).
  let rdr_st = RenderState::default().set_depth_write(DepthWrite::Off);

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Key(Key::Escape, _, Action::Release, _) | WindowEvent::Close => break 'app,

        // Move the camera left.
        WindowEvent::Key(Key::A, _, Action::Release, _)
        | WindowEvent::Key(Key::A, _, Action::Repeat, _) => {
          let v =
            cam_orient
              .invert()
              .rotate_vector(Vector3::new(CAMERA_SENSITIVITY_STRAFE_LEFT, 0., 0.));
          eye -= v;
          view_updated = true;
        }

        // Move the camera right.
        WindowEvent::Key(Key::D, _, Action::Release, _)
        | WindowEvent::Key(Key::D, _, Action::Repeat, _) => {
          let v = cam_orient.invert().rotate_vector(Vector3::new(
            -CAMERA_SENSITIVITY_STRAFE_RIGHT,
            0.,
            0.,
          ));
          eye -= v;
          view_updated = true;
        }

        // Move the camera forward.
        WindowEvent::Key(Key::W, _, Action::Release, _)
        | WindowEvent::Key(Key::W, _, Action::Repeat, _) => {
          let v = cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            0.,
            CAMERA_SENSITIVITY_STRAFE_FORWARD,
          ));
          eye -= v;
          view_updated = true;
        }

        // Move the camera backward.
        WindowEvent::Key(Key::S, _, Action::Release, _)
        | WindowEvent::Key(Key::S, _, Action::Repeat, _) => {
          let v = cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            0.,
            -CAMERA_SENSITIVITY_STRAFE_BACKWARD,
          ));
          eye -= v;
          view_updated = true;
        }

        // Move the camera up.
        WindowEvent::Key(Key::Q, _, Action::Release, _)
        | WindowEvent::Key(Key::Q, _, Action::Repeat, _) => {
          let v =
            cam_orient
              .invert()
              .rotate_vector(Vector3::new(0., CAMERA_SENSITIVITY_STRAFE_UP, 0.));
          eye -= v;
          view_updated = true;
        }

        // move the camera down.
        WindowEvent::Key(Key::E, _, Action::Release, _)
        | WindowEvent::Key(Key::E, _, Action::Repeat, _) => {
          let v = cam_orient.invert().rotate_vector(Vector3::new(
            0.,
            -CAMERA_SENSITIVITY_STRAFE_DOWN,
            0.,
          ));
          eye -= v;
          view_updated = true;
        }

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        // When the cursor move, we need to update the last cursor position we know and, if needed,
        // update the Euler angles we use to orient the camera in space.
        WindowEvent::CursorPos(x, y) => {
          let [px, py] = last_cursor_pos.unwrap_or([x, y]);
          let [rx, ry] = [x - px, y - py];

          last_cursor_pos = Some([x, y]);

          if rotate_viewport {
            x_theta += CAMERA_SENSITIVITY_PITCH * ry as f32;
            y_theta += CAMERA_SENSITIVITY_YAW * rx as f32;

            // Stick the camera at verticals.
            x_theta = clamp_pitch(x_theta);

            view_updated = true;
          }
        }

        // When the “left” button is pressed, we want to rotate the viewport.
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
          rotate_viewport = true;
        }

        // When the “left” button is released, we want to stop rotating the view.
        WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
          rotate_viewport = false;
        }

        // Scrolling allows us to change the field-of-view.
        WindowEvent::Scroll(_, scroll) => {
          fovy += scroll as f32 * CAMERA_SENSITIVITY_FOVY_CHANGE;
          fovy = clamp_fovy(fovy);

          // Because the field-of-view has changed, we need to recompute the projection matrix.
          projection = perspective(Rad(fovy), aspect_ratio, Z_NEAR, Z_FAR);

          let Deg(deg) = Rad(fovy).into();
          info!("new fovy is {}°", deg);
        }

        _ => (),
      }
    }

    // When the viewport gets resized, we want to recompute the aspect ratio (and then recreate the
    // projection matrix).
    if resize {
      back_buffer = surface
        .back_buffer()
        .map_err(|e| AppError::CannotGrabBackBuffer(Box::new(e)))?;
      let [width, height] = back_buffer.size();

      aspect_ratio = width as f32 / height as f32;

      projection = perspective(Rad(fovy), aspect_ratio, Z_NEAR, Z_FAR);

      resize = false;
    }

    // When the view is updated (i.e. the camera has moved or got re-oriented), we want to
    // recompute a bunch of quaternions (used to encode orientations) and matrices.
    if view_updated {
      let qy = Quaternion::from_angle_y(Rad(y_theta));
      let qx = Quaternion::from_angle_x(Rad(x_theta));

      // Orientation of the camera. Used for both the skybox (by inverting it) and the cube.
      cam_orient = (qx * qy).normalize();
      skybox_orient = cam_orient.invert();
      cam_view = Matrix4::from(cam_orient) * Matrix4::from_translation(-eye);

      view_updated = false;
    }

    let mut pipeline_gate = surface.new_pipeline_gate();
    let projection = projection.into();
    let view = cam_view.into();

    // We use two shaders in a single pipeline here: first, we render the skybox. Then, we render
    // the cube. A note here: it should be possible to change the way the skybox is rendered to
    // render it _after_ the cube. That will optimize some pixel shading when the cube is in the
    // viewport. For the sake of simplicity, we don’t do that here.
    let render = pipeline_gate
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          let environment_map = pipeline.bind_texture(&mut skybox).unwrap();

          // render the skybox
          shd_gate.shade(&mut skybox_program, |mut iface, unis, mut rdr_gate| {
            iface.set(&unis.view, Matrix4::from(skybox_orient).into());
            iface.set(&unis.fovy, fovy);
            iface.set(&unis.aspect_ratio, aspect_ratio);
            iface.set(&unis.skybox, environment_map.binding());

            rdr_gate.render(&rdr_st, |mut tess_gate| tess_gate.render(&fullscreen_quad))
          })?;

          // render the cube
          shd_gate.shade(
            &mut environment_mapping_program,
            |mut iface, unis, mut rdr_gate| {
              iface.set(&unis.projection, projection);
              iface.set(&unis.view, view);
              iface.set(&unis.aspect_ratio, aspect_ratio);
              iface.set(&unis.environment, environment_map.binding());

              rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                tess_gate.render(&cube)
              })
            },
          )
        },
      )
      .assume();

    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      error!("dropped a frame");
      break 'app;
    }
  }

  Ok(())
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

/// Load an image.
fn load_img(path: impl AsRef<Path>) -> Result<image::DynamicImage, AppError> {
  let path = path.as_ref();

  info!("loading {}", path.display());
  let image = image::open(path).map_err(|_| AppError::UnknownFile(path.to_owned()))?;
  info!("cubemap image {} loaded", path.display());

  Ok(image)
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
  surface: &mut impl GraphicsContext<Backend = Backend>,
  img: &image::DynamicImage,
) -> Result<Texture<Cubemap, NormRGB8UI>, AppError> {
  let img = img.to_rgb();

  let width = img.width();
  let size = width / 4;

  // We discard “bad” images that don’t strictly respect the dimensions mentioned above.
  if img.height() / 3 != size {
    return Err(AppError::InvalidCubemapSize(width, img.height()));
  }

  let pixels = img.into_raw();

  // Create the cubemap on the GPU; we ask for two mipmaps… because why not.
  let mut texture = surface
    .new_texture(size, 2, Sampler::default())
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

  info!("uploading the +X face");
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

  info!("uploading the -X face");
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

  info!("uploading the +Y face");
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

  info!("uploading the -Y face");
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

  info!("uploading the +Z face");
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

  info!("uploading the -Z face");
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
    .upload_part_raw(GenMipmaps::Yes, ([0, 0], face), size as u32, &face_buffer)
    .map_err(|e| AppError::CannotUploadToFace(Box::new(e)))
}
