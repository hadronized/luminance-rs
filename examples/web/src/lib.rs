//! Web platform for the examples.

mod platform;

use crate::platform::WebPlatformServices;
use luminance_examples::{Example as _, Features, InputAction, LoopFeedback};
use luminance_web_sys::WebSysWebGL2Surface;
use wasm_bindgen::prelude::*;

/// Web features.
#[wasm_bindgen]
pub struct WebFeatures(Features);

#[wasm_bindgen]
impl WebFeatures {
  pub fn textures(&self) -> Box<[JsValue]> {
    let v: Vec<_> = self
      .0
      .textures()
      .iter()
      .map(|n| JsValue::from_str(n))
      .collect();
    v.into_boxed_slice()
  }
}

/// Macro to declaratively add examples.
macro_rules! examples {
  ($($test_name:literal, $test_ident:ident),*) => {
    /// List of available examples.
    #[wasm_bindgen]
    pub fn examples_names() -> Box<[JsValue]> {
      vec![$( $test_name.into() ),*].into_boxed_slice()
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
      $( $test_ident: Option<luminance_examples::$test_ident::LocalExample> ),*
    }

    #[wasm_bindgen]
    impl Showcase {
      fn new(surface: WebSysWebGL2Surface) -> Self {
        let platform = WebPlatformServices::new();
        let actions = Vec::new();
        $(
          let $test_ident = None;
        )*

        Showcase {
          platform,
          surface,
          actions,
          $( $test_ident ),*
        }
      }

      pub fn enqueue_quit_action(&mut self) {
        log::debug!("pushing input action: quit");
        self.actions.push(InputAction::Quit)
      }

      pub fn enqueue_main_toggle_action(&mut self) {
        log::debug!("pushing input action: main toggle");
        self.actions.push(InputAction::MainToggle);
      }

      pub fn enqueue_auxiliary_toggle_action(&mut self) {
        log::debug!("pushing input action: auxiliary toggle");
        self.actions.push(InputAction::AuxiliaryToggle);
      }

      pub fn enqueue_resized_action(&mut self, width: u32, height: u32) {
        log::debug!("pushing input action: resized {}×{}", width, height);
        self.actions.push(InputAction::Resized { width, height });
      }

      pub fn enqueue_left_action(&mut self) {
        log::debug!("pushing input action: left");
        self.actions.push(InputAction::Left);
      }

      pub fn enqueue_right_action(&mut self) {
        log::debug!("pushing input action: right");
        self.actions.push(InputAction::Right);
      }

      pub fn enqueue_up_action(&mut self) {
        log::debug!("pushing input action: up");
        self.actions.push(InputAction::Up);
      }

      pub fn enqueue_down_action(&mut self) {
        log::debug!("pushing input action: down");
        self.actions.push(InputAction::Down);
      }

      /// Cleanup all examples.
      pub fn reset(&mut self) {
        $(
          log::debug!("resetting {}", $test_name);
          self.$test_ident = None;
        )*
      }

      pub fn get_features(&mut self, name: &str) -> Option<WebFeatures> {
        match name {
          $(
            $test_name => Some(WebFeatures(luminance_examples::$test_ident::LocalExample::features())),
          )*
          _ => None
        }
      }

      pub fn add_texture(&mut self, name: &str, blob: Vec<u8>) {
        self.platform.add_texture(name, blob);
      }

      pub fn render_example(&mut self, name: &str, time: f32) -> bool {
        // first, check whether the example exists
        match name {
          $(
            $test_name => {
              // check if the example is already bootstrapped; if not, bootstrap it and then render
              let platform = &mut self.platform;
              let surface = &mut self.surface;
              let example = self.$test_ident.take().unwrap_or_else(||
              {
                log::debug!("bootstrapping {}", $test_name);
                luminance_examples::$test_ident::LocalExample::bootstrap(platform, surface)
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

          _ => ()
        }

        true
      }
    }
  }
}

examples! {
  "hello-world", hello_world,
  "render-state", render_state,
  "sliced-tess", sliced_tess,
  "shader-uniforms", shader_uniforms,
  "attributeless", attributeless,
  "texture", texture,
  "offscreen", offscreen,
  "shader-uniform-adapt", shader_uniform_adapt,
  "dynamic-uniform-interface", dynamic_uniform_interface,
  "vertex-instancing", vertex_instancing
}

#[wasm_bindgen]
pub fn get_showcase(canvas_name: &str) -> Showcase {
  wasm_logger::init(wasm_logger::Config::default());
  console_error_panic_hook::set_once();

  log::info!("creating the WebGL2 context…");
  let surface = WebSysWebGL2Surface::new(canvas_name).expect("WebGL2 canvas");
  Showcase::new(surface)
}