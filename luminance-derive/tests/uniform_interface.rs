use luminance::shader::program::Uniform;
use luminance_derive::{UniformInterface};

#[test]
fn derive_uniform_interface() {
  #[derive(UniformInterface)]
  struct SimpleUniformInterface {
    _t: Uniform<f32>
  }
}

#[test]
fn derive_unbound_uniform_interface() {
  #[derive(UniformInterface)]
  struct SimpleUniformInterface {
    #[uniform(unbound)]
    _t: Uniform<f32>
  }
}

#[test]
fn derive_renamed_uniform_interface() {
  #[derive(UniformInterface)]
  struct SimpleUniformInterface {
    #[uniform(name = "time")]
    _t: Uniform<f32>
  }
}

#[test]
fn derive_unbound_renamed_uniform_interface() {
  #[derive(UniformInterface)]
  struct SimpleUniformInterface {
    #[uniform(name = "time", unbound)]
    _t: Uniform<f32>,
    #[uniform(unbound, name = "time")]
    _t2: Uniform<f32>
  }
}
