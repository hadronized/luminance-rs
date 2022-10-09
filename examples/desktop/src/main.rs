mod platform;

use crate::platform::DesktopPlatformServices;
use glfw::{
  Action, Context as _, Key, Modifiers, MouseButton, SwapInterval, WindowEvent, WindowMode,
};
use luminance_examples::{Example, InputAction, LoopFeedback};
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CLIOpts {
  /// List of textures (paths) to load from.
  #[structopt(short, long)]
  textures: Vec<String>,

  /// List available examples.
  #[structopt(short, long)]
  list_examples: bool,

  /// Example to run.
  example: Option<String>,
}

/// Macro to declaratively add examples.
macro_rules! examples {
  (examples: $($ex_name:literal, $test_ident:ident),* ,
   funtests: $($fun_name:literal $(if $fun_feature_gate:literal)?, $fun_ident:ident),* $(,)?
  ) => {
    fn show_available_examples() {
      println!("simple examples:");
      $( println!("  - {}", $ex_name); )*

      #[cfg(feature = "funtest")]
      {
        println!("\nfunctional tests:");
        $(
          print!("  - {}", $fun_name);
          $(
            #[cfg(feature = $fun_feature_gate)]
            print!(" (feature: {})", $fun_feature_gate);
          )?
          println!("");
        )*
      }
    }

    // create a function that will run an example based on its name
    fn pick_and_run_example(cli_opts: CLIOpts) {
      let example_name = cli_opts.example.as_ref().map(|n| n.as_str());
      match example_name {
        $(
          Some($ex_name) => {
            run_example::<luminance_examples::$test_ident::LocalExample>(cli_opts)
          }
        ),*

        $(
          #[cfg(all(feature = "funtest" $(, feature = $fun_feature_gate)?))]
          Some($fun_name) => {
            run_example::<luminance_examples::$fun_ident::LocalExample>(cli_opts)
          }
        ),*

        _ => {
          log::warn!("no example found");
          show_available_examples();
        }
      }
    }
  }
}

#[derive(Debug)]
pub enum PlatformError {
  CannotCreateWindow,
}

// Run an example.
fn run_example<E>(cli_opts: CLIOpts)
where
  E: Example,
{
  // Check the features so that we know what we need to load.
  let mut services = DesktopPlatformServices::new(cli_opts);

  // First thing first: we create a new surface to render to and get events from.
  let GlfwSurface {
    events_rx,
    mut window,
    mut ctx,
  } = GlfwSurface::new_gl33(|glfw| {
    let (mut window, events) = glfw
      .create_window(960, 540, E::TITLE, WindowMode::Windowed)
      .ok_or_else(|| GlfwSurfaceError::UserError(PlatformError::CannotCreateWindow))?;

    window.make_current();
    window.set_all_polling(true);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    Ok((window, events))
  })
  .expect("GLFW surface creation");

  let (fb_w, fb_h) = window.get_framebuffer_size();

  let mut example = match E::bootstrap([fb_w as _, fb_h as _], &mut services, &mut ctx) {
    Ok(example) => example,
    Err(e) => {
      log::error!("cannot bootstrap example: {}", e);
      return;
    }
  };

  let start_t = Instant::now();
  'app: loop {
    // handle events
    window.glfw.poll_events();
    let actions = glfw::flush_messages(&events_rx).flat_map(|(_, event)| adapt_events(event));

    let t = start_t.elapsed().as_secs_f32();
    let feedback = example.render_frame(t, actions, &mut ctx);

    match feedback {
      Ok(LoopFeedback::Continue(stepped)) => {
        example = stepped;
        window.swap_buffers();
      }

      Ok(LoopFeedback::Exit) => break 'app,

      Err(e) => {
        log::error!("error while rendering a frame: {}", e);
        break 'app;
      }
    }
  }
}

fn adapt_events(event: WindowEvent) -> Option<InputAction> {
  match event {
    WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
      Some(InputAction::Quit)
    }

    WindowEvent::Key(Key::Space, _, Action::Release, mods) => {
      if mods.is_empty() {
        Some(InputAction::MainToggle)
      } else if mods == Modifiers::Shift {
        Some(InputAction::AuxiliaryToggle)
      } else {
        None
      }
    }

    WindowEvent::Key(key, _, Action::Press, _) | WindowEvent::Key(key, _, Action::Repeat, _) => {
      log::debug!("key press: {:?}", key);
      match key {
        Key::A => Some(InputAction::Left),
        Key::D => Some(InputAction::Right),
        Key::W => Some(InputAction::Forward),
        Key::S => Some(InputAction::Backward),
        Key::F => Some(InputAction::Up),
        Key::R => Some(InputAction::Down),
        _ => None,
      }
    }

    WindowEvent::MouseButton(MouseButton::Button1, action, _) => match action {
      Action::Press => Some(InputAction::PrimaryPressed),
      Action::Release => Some(InputAction::PrimaryReleased),
      _ => None,
    },

    WindowEvent::CursorPos(x, y) => Some(InputAction::CursorMoved {
      x: x as _,
      y: y as _,
    }),

    WindowEvent::FramebufferSize(width, height) => Some(InputAction::Resized {
      width: width as _,
      height: height as _,
    }),

    WindowEvent::Scroll(_, amount) => Some(InputAction::VScroll {
      amount: amount as f32,
    }),

    _ => None,
  }
}

examples! {
  examples:
  "hello-world", hello_world,
  "hello-world-more", hello_world_more,
  "render-state", render_state,
  "sliced-vertex-entity", sliced_vertex_entity,
  "shader-uniforms", shader_uniforms,
  "attributeless", attributeless,
  // "texture", texture,
  // "offscreen", offscreen,
  // "shader-uniform-adapt", shader_uniform_adapt,
  // "dynamic-uniform-interface", dynamic_uniform_interface,
  // "vertex-instancing", vertex_instancing,
  // "query-texture-texels", query_texture_texels,
  // "displacement-map", displacement_map,
  // "interactive-triangle", interactive_triangle,
  // "query-info", query_info,
  // "mrt", mrt,
  // "skybox", skybox,
  // "shader-data", shader_data,
  // "stencil", stencil,

  // functional tests
  funtests:
  // "funtest-tess-no-data", funtest_tess_no_data,
  // "funtest-gl33-f64-uniform" if "funtest-gl33-f64-uniform", funtest_gl33_f64_uniform,
  // "funtest-scissor-test", funtest_scissor_test,
  // "funtest-360-manually-drop-framebuffer", funtest_360_manually_drop_framebuffer,
  // "funtest-flatten-slice", funtest_flatten_slice,
  // "funtest-pixel-array-encoding", funtest_pixel_array_encoding,
  // "funtest-483-indices-mut-corruption", funtest_483_indices_mut_corruption,
}

fn main() {
  env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();
  let cli_opts = CLIOpts::from_args();

  log::debug!("CLI options:\n{:#?}", cli_opts);

  if cli_opts.list_examples {
    show_available_examples();
  } else {
    pick_and_run_example(cli_opts);
  }
}
