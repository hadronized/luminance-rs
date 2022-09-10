#![doc(
  html_logo_url = "https://raw.githubusercontent.com/phaazon/luminance-rs/master/docs/imgs/luminance_alt.svg"
)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

#[cfg(feature = "derive")]
pub use luminance_derive::*;

pub mod backend;
pub mod blending;
pub mod context;
pub mod depth_stencil;
pub mod face_culling;
pub mod framebuffer;
pub mod has_field;
pub mod named_index;
pub mod primitive;
pub mod render_channel;
pub mod render_slots;
// pub mod pipeline;
pub mod pixel;
// pub mod query;
// pub mod render_gate;
pub mod dim;
pub mod render_state;
pub mod scissor;
// pub mod shader;
// pub mod shading_gate;
// pub mod tess_gate;
// pub mod texture;
pub mod vertex;
pub mod vertex_entity;
pub mod vertex_storage;
