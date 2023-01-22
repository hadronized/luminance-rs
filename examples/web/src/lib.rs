//! Web platform for the examples.

mod platform;

use crate::platform::WebPlatformServices;
use luminance_examples::{Example as _, InputAction, LoopFeedback};
use luminance_web_sys::WebSysWebGL2Surface;
use wasm_bindgen::prelude::*;

/// Macro to declaratively add examples.
macro_rules! examples {
  (examples: $($test_name:literal, $test_ident:ident),* , funtests: $($fun_name:literal $(if $fun_feature_gate:literal)?, $fun_ident:ident),* $(,)?) => {
    /// List of available examples.
    #[wasm_bindgen]
    pub fn examples_names() -> Box<[JsValue]> {
      let names = vec![$( $test_name.into() ),*];

      #[cfg(feature = "funtest")]
      let names = {
        let mut names = names;
        names.extend_from_slice(&[$(
          $(#[cfg(feature = $fun_feature_gate)])?
          $fun_name.into(),
        )*]);
        names
      };

      names.into_boxed_slice()
    }

    /// Main example object.
    ///
    /// This object will be passed around in JavaScript and will act as a bridge between the JavaScript code and the Rust
    /// code.
    #[wasm_bindgen]
    pub struct Showcase {
      platform: WebPlatformServices,
      surface: WebSysWebGL2Surface,
      actions: Vec<InputAction>,
      $( $test_ident: Option<luminance_examples::$test_ident::LocalExample> ),*,
      $( #[cfg(all(feature = "funtest", $(feature = $fun_feature_gate)?))] $fun_ident: Option<luminance_examples::$fun_ident::LocalExample> ),*,
    }

    #[wasm_bindgen]
    impl Showcase {
      fn new(surface: WebSysWebGL2Surface) -> Self {
        let platform = WebPlatformServices::new();
        let actions = Vec::new();
        $(
          let $test_ident = None;
        )*
        $(
          #[cfg(all(feature = "funtest", $(feature = $fun_feature_gate)?))]
          let $fun_ident = None;
        )*

        Showcase {
          platform,
          surface,
          actions,
          $( $test_ident ),*,
          $( #[cfg(all(feature = "funtest", $(feature = $fun_feature_gate)?))] $fun_ident ),*
        }
      }

      pub fn enqueue_quit_action(&mut self) {
        self.actions.push(InputAction::Quit)
      }

      pub fn enqueue_primary_pressed_action(&mut self) {
        self.actions.push(InputAction::PrimaryPressed);
      }

      pub fn enqueue_primary_released_action(&mut self) {
        self.actions.push(InputAction::PrimaryReleased);
      }

      pub fn enqueue_main_toggle_action(&mut self) {
        self.actions.push(InputAction::MainToggle);
      }

      pub fn enqueue_auxiliary_toggle_action(&mut self) {
        self.actions.push(InputAction::AuxiliaryToggle);
      }

      pub fn enqueue_resized_action(&mut self, width: u32, height: u32) {
        self.actions.push(InputAction::Resized { width, height });
      }

      pub fn enqueue_left_action(&mut self) {
        self.actions.push(InputAction::Left);
      }

      pub fn enqueue_right_action(&mut self) {
        self.actions.push(InputAction::Right);
      }

      pub fn enqueue_forward_action(&mut self) {
        self.actions.push(InputAction::Forward);
      }

      pub fn enqueue_backward_action(&mut self) {
        self.actions.push(InputAction::Backward);
      }

      pub fn enqueue_up_action(&mut self) {
        self.actions.push(InputAction::Up);
      }

      pub fn enqueue_down_action(&mut self) {
        self.actions.push(InputAction::Down);
      }

      pub fn enqueue_cursor_moved_action(&mut self, x: f32, y: f32) {
        self.actions.push(InputAction::CursorMoved { x, y });
      }

      pub fn enqueue_vscroll_action(&mut self, amount: f32) {
        self.actions.push(InputAction::VScroll { amount });
      }

      /// Cleanup all examples.
      pub fn reset(&mut self) {
        $(
          log::debug!("resetting example {}", $test_name);
          self.$test_ident = None;
        )*

        $(
          #[cfg(all(feature = "funtest", $(feature = $fun_feature_gate)?))]
          {
            log::debug!("resetting functional test {}", $fun_name);
            self.$fun_ident = None;
          }
        )*
      }

      pub fn add_texture(&mut self, blob: Vec<u8>) {
        self.platform.add_texture(blob);
      }

      pub fn render_example(&mut self, name: &str, time: f32) -> bool {
        // first, check whether the example exists
        match name {
          $(
            $test_name => {
              // check if the example is already bootstrapped; if not, bootstrap it and then render
              let platform = &mut self.platform;
              let surface = &mut self.surface;
              let actions = &mut self.actions;
              let example = self.$test_ident.take().unwrap_or_else(||
              {
                log::debug!("bootstrapping example {}", $test_name);
                let example = luminance_examples::$test_ident::LocalExample::bootstrap(platform, surface);

                // send a first input action to forward the framebuffer size, as some examples use dummy initial values
                let width = surface.window.inner_width().ok().and_then(|w| w.as_f64()).map(|w| w as u32).unwrap();
                let height = surface.window.inner_height().ok().and_then(|h| h.as_f64()).map(|h| h as u32).unwrap();

                actions.push(InputAction::Resized { width, height });
                example
              });

              let loop_feedback = example.render_frame(
                time,
                surface.back_buffer().expect("WebGL backbuffer"),
                self.actions.iter().cloned(),
                surface,
              );

              self.actions.clear();

              // deallocate the example if we exit it
              if let LoopFeedback::Continue(stepped) = loop_feedback {
                self.$test_ident = Some(stepped);
              } else {
                self.$test_ident = None;
                return false;
              }
            }
          )*

          $(
            #[cfg(all(feature = "funtest", $(feature = $fun_feature_gate)?))]
            $fun_name => {
              // check if the functional test is already bootstrapped; if not, bootstrap it and then render
              let platform = &mut self.platform;
              let surface = &mut self.surface;
              let actions = &mut self.actions;
              let example = self.$fun_ident.take().unwrap_or_else(||
              {
                log::debug!("bootstrapping functional test {}", $fun_name);
                let example = luminance_examples::$fun_ident::LocalExample::bootstrap(platform, surface);

                // send a first input action to forward the framebuffer size, as some examples use dummy initial values
                let width = surface.window.inner_width().ok().and_then(|w| w.as_f64()).map(|w| w as u32).unwrap();
                let height = surface.window.inner_height().ok().and_then(|h| h.as_f64()).map(|h| h as u32).unwrap();

                actions.push(InputAction::Resized { width, height });
                example
              });

              let loop_feedback = example.render_frame(
                time,
                surface.back_buffer().expect("WebGL backbuffer"),
                self.actions.iter().cloned(),
                surface,
              );

              self.actions.clear();

              // deallocate the example if we exit it
              if let LoopFeedback::Continue(stepped) = loop_feedback {
                self.$fun_ident = Some(stepped);
              } else {
                self.$fun_ident = None;
                return false;
              }
            }
          )*

          _ => ()
        }

        true
      }
    }
  }
}

examples! {
  examples:
  "hello-world", hello_world,
  "render-state", render_state,
  "sliced-tess", sliced_tess,
  "shader-uniforms", shader_uniforms,
  "attributeless", attributeless,
  "texture", texture,
  "offscreen", offscreen,
  "shader-uniform-adapt", shader_uniform_adapt,
  "dynamic-uniform-interface", dynamic_uniform_interface,
  "vertex-instancing", vertex_instancing,
  "displacement-map", displacement_map,
  "interactive-triangle", interactive_triangle,
  "query-info", query_info,
  "mrt", mrt,
  "skybox", skybox,
  "shader-data", shader_data,
  "stencil", stencil,

  funtests:
  "funtest-scissor-test", funtest_scissor_test,
  "funtest-360-manually-drop-framebuffer", funtest_360_manually_drop_framebuffer,
  "funtest-pixel-array-encoding", funtest_pixel_array_encoding,
  "funtest-483-indices-mut-corruption", funtest_483_indices_mut_corruption,
}

#[wasm_bindgen]
pub fn get_showcase(canvas_name: &str) -> Showcase {
  wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
  console_error_panic_hook::set_once();

  log::info!("creating the WebGL2 context…");
  let surface = WebSysWebGL2Surface::new(canvas_name).expect("WebGL2 canvas");
  Showcase::new(surface)
}
