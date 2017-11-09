//! Dynamic rendering pipelines.
//!
//! This module gives you types and functions to build *dynamic* rendering **pipelines**. A
//! pipeline represents a functional stream that consumes geometric data and rasterizes them.
//!
//! When you want to build a render, the main entry point is the `entry` function. It gives you a
//! `Gpu` object that enables you to perform stateful graphics operations.
//!
//! # Key concepts
//!
//! luminance exposes several concepts you have to be familiar with:
//!
//! - render buffers;
//! - blending;
//! - shaders;
//! - tessellations;
//! - gates;
//! - render commands;
//! - shading commands;
//! - pipelines;
//!
//! # Render buffers
//!
//! The render buffers are GPU-allocated memory regions used while rendering images into
//! framebuffers. Typically, a framebuffer has at least three buffers:
//!
//! - a *color buffer*, that will receive texels (akin to pixels, but for textures/buffers);
//! - a *depth buffer*, a special buffer mostly used to determine whether a fragment (pixel) is
//!   behind something that was previously rendered – it’s a simple solution to discard render that
//!   won’t be visible anyway;
//! - a *stencil buffer*, which often acts as a mask to create interesting effects to your renders.
//!
//! luminance gives you access to the first two – the stencil buffer will be added in a future
//! release.
//!
//! Alternatively, you can also tell your GPU that you won’t be using a depth buffer, or that you
//! need several color buffers – this is called [MRT](https://en.wikipedia.org/wiki/Multiple_Render_Targets).
//! In most frameworks, you create some textures to hold the color information, then you “bind” them
//! to your pipeline, and you’re good to go. In luminance, everything happens at the type level: you
//! say to luminance which type of framebuffer you want (for instance, a framebuffer with two color
//! outputs and a depth buffer). luminance will handle the textures for you – you can retrieve access
//! to them whenever you want to.
//!
//! If you have a depth buffer, you can ask luminance to perform a depth test that will discard any
//! fragment being “behind” the fragment already in place. You can also give luminance the *clear
//! color* it must use when you issue a new pipeline to fill the buffers.
//!
//! # Blending
//!
//! When you render a fragment A at a position P in a framebuffer, there are several configurations:
//!
//! - you have a depth buffer and the depth test is enabled: in that case, no blending will happen
//!   as either the already in-place fragment will be chosen or the new one you try to write,
//!   depending on the result of the depth test;
//! - the depth test is disabled: in that case, each time a fragment is to be written to a place in
//!   a buffer, its output will be blended with the color already present according to a *blending
//!   equation* and two *blending factors*.
//!
//! # Shaders
//!
//! Shaders in luminance are pretty simple: you have `Stage` that represents each type of shader
//! stages you can use, and `Program`, that links them into a GPU executable.
//!
//! In luminance, you’re supposed to create a `Program` with `Stage`. Some of them are mandatory and
//! others are optional. In order to customize your build, you can use a *uniform interface*. The
//! uniform interface is defined by our own type. When you create a program, you pass code to tell
//! luminance how to get such a type. Then that type will be handed back to you when needed in the
//! form of an immutable object.
//!
//! # Tessellations
//!
//! The `Tess` type represents a tessellation associated with a possible set of GPU vertices and
//! indices. Note that it’s also possible to create *attributeless* tesselations – i.e.
//! tessellations that don’t own any vertices nor indices and are used with specific shaders only.
//!
//! # Gates
//!
//! The gate concept is quite easy to understand: picture a pipeline as tree. Each node is typed,
//! hence, the structure is quite limited to what your GPU can understand. Gates are a way to
//! spread information of root nodes down. For instance, if you have a shader node, every nested
//! child of that shader node will have the possibility to use its features via a shading gate.
//!
//! # Render commands
//!
//! A set of values that tag a collection of tessellations. Typical information is whether we should
//! use a depth test, the blending factors and equation, etc.
//!
//! # Shading commands
//!
//! A set of values that tag a collection of render commands. Typical information is a shader
//! program along with its uniform interface.
//!
//! # Pipelines
//!
//! A pipeline is just an aggregation of shadings commands with a few extra information.

use gl;
use gl::types::*;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use buffer::RawBuffer;
use blending::{Equation, Factor};
use framebuffer::{ColorSlot, DepthSlot, Framebuffer};
use shader::program::{Dim, Program, Type, Uniform, Uniformable, UniformInterface};
use tess::TessRender;
use texture::{Dimensionable, Layerable, RawTexture};
use vertex::{CompatibleVertex, Vertex};

struct GpuState {
  next_texture_unit: u32,
  free_texture_units: Vec<u32>,
  next_buffer_binding: u32,
  free_buffer_bindings: Vec<u32>
}

impl GpuState {
  fn new() -> Self {
    GpuState {
      next_texture_unit: 0,
      free_texture_units: Vec::new(),
      next_buffer_binding: 0,
      free_buffer_bindings: Vec::new()
    }
  }
}

/// An opaque type representing the GPU. You can perform stateful operations on it.
pub struct Gpu {
  gpu_state: Rc<RefCell<GpuState>>
}

impl Gpu {
  fn new() -> Self {
    Gpu {
      gpu_state: Rc::new(RefCell::new(GpuState::new()))
    }
  }

  /// Bind a texture and return the bound texture.
  pub fn bind_texture<'a, T>(&self, texture: &'a T) -> BoundTexture<'a, T> where T: Deref<Target = RawTexture> {
    let mut state = self.gpu_state.borrow_mut();

    let unit = state.free_texture_units.pop().unwrap_or_else(|| {
      // no more free units; reserve one
      let unit = state.next_texture_unit;
      state.next_texture_unit += 1;
      unit
    });

    unsafe { bind_texture_at(texture.deref(), unit) };
    BoundTexture::new(self.gpu_state.clone(), unit)
  }

  /// Bind a buffer and return the bound buffer.
  pub fn bind_buffer<'a, T>(&self, buffer: &T) -> BoundBuffer<'a, T> where T: Deref<Target = RawBuffer> {
    let mut state = self.gpu_state.borrow_mut();

    let binding = state.free_buffer_bindings.pop().unwrap_or_else(|| {
      // no more free bindings; reserve one
      let binding = state.next_buffer_binding;
      state.next_buffer_binding += 1;
      binding
    });

    unsafe { bind_buffer_at(buffer.deref(), binding) };
    BoundBuffer::new(self.gpu_state.clone(), binding)
  }
}

/// An opaque type representing a bound texture in a `Gpu`. You may want to pass such an object to
/// a shader’s uniform’s update.
pub struct BoundTexture<'a, T> where T: 'a {
  unit: u32,
  gpu_state: Rc<RefCell<GpuState>>,
  _t: PhantomData<&'a T>
}

impl<'a, T> BoundTexture<'a, T> {
  fn new(gpu_state: Rc<RefCell<GpuState>>, unit: u32) -> Self {
    BoundTexture {
      unit,
      gpu_state,
      _t: PhantomData
    }
  }
}

impl<'a, T> Drop for BoundTexture<'a, T> {
  fn drop(&mut self) {
    let mut state = self.gpu_state.borrow_mut();
    // place the unit into the free list
    state.free_texture_units.push(self.unit);
  }
}

impl<'a, T> Uniformable for BoundTexture<'a, T> {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1i(u.index(), self.unit as GLint) }
  }

  fn ty() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

/// An opaque type representing a bound buffer in a `Gpu`. You may want to pass such an object to
/// a shader’s uniform’s update.
pub struct BoundBuffer<'a, T> where T: 'a {
  binding: u32,
  gpu_state: Rc<RefCell<GpuState>>,
  _t: PhantomData<&'a T>
}

impl<'a, T> BoundBuffer<'a, T> {
  fn new(gpu_state: Rc<RefCell<GpuState>>, binding: u32) -> Self {
    BoundBuffer {
      binding,
      gpu_state,
      _t: PhantomData
    }
  }
}

impl<'a, T> Drop for BoundBuffer<'a, T> {
  fn drop(&mut self) {
    let mut state = self.gpu_state.borrow_mut();
    // place the binding into the free list
    state.free_buffer_bindings.push(self.binding);
  }
}

impl<'a, T> Uniformable for BoundBuffer<'a, T> {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformBlockBinding(u.program(), u.index() as GLuint, self.binding as GLuint) }
  }

  fn ty() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

/// This is the entry point of a render.
///
/// You’re handed a `Gpu` object that allows you to perform stateful operations on the GPU. For
/// instance, you can bind textures and buffers to use them in shaders.
pub fn entry<F>(f: F) where F: FnOnce(Gpu) {
  let gpu = Gpu::new();
  f(gpu);
}

/// A dynamic rendering pipeline. A *pipeline* is responsible of rendering into a `Framebuffer`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot`(s) of the underlying
/// `Framebuffer`.
///
/// Pipelines also have several transient objects:
///
/// - a *clear color*, used to clear the framebuffer
/// - a *texture set*, used to make textures available in subsequent structures
/// - a *buffer set*, used to make uniform buffers available in subsequent structures
pub fn pipeline<L, D, CS, DS, F>(framebuffer: &Framebuffer<L, D, CS, DS>, clear_color: [f32; 4], f: F)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<L, D>,
          DS: DepthSlot<L, D>,
          F: FnOnce(&ShadingGate) {
  unsafe {
    gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.handle());
    gl::Viewport(0, 0, framebuffer.width() as GLint, framebuffer.height() as GLint);
    gl::ClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
  }

  f(&ShadingGate);
}

/// An object created only via `pipeline` and that gives you shading features.
pub struct ShadingGate;

impl ShadingGate {
  pub fn shade<In, Out, Uni, F>(&self, program: &Program<In, Out, Uni>, f: F)
      where In: Vertex, Uni: UniformInterface, F: FnOnce(&RenderGate<In>, &Uni) {
    unsafe { gl::UseProgram(program.handle()) };

    let render_gate = RenderGate {
      _v: PhantomData,
    };

    let uni_iface = program.uniform_interface();
    f(&render_gate, uni_iface);
  }
}

pub struct RenderGate<V> {
  _v: PhantomData<*const V>
}

impl<V> RenderGate<V> {
  pub fn render<B, F>(&self, rdr_st: RenderState, f: F) where F: FnOnce(&TessGate<V>) {
    unsafe {
      set_blending(rdr_st.blending);
      set_depth_test(rdr_st.depth_test);
      set_face_culling(rdr_st.face_culling);
    }

    let tess_gate = TessGate {
      _v: PhantomData,
    };

    f(&tess_gate);
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  blending: Option<(Equation, Factor, Factor)>,
  depth_test: DepthTest,
  face_culling: Option<FaceCulling>
}

impl RenderState {
  pub fn set_blending<B>(self, blending: B) -> Self where B: Into<Option<(Equation, Factor, Factor)>> {
    RenderState {
      blending: blending.into(),
      .. self
    }
  }

  pub fn blending(&self) -> Option<(Equation, Factor, Factor)> {
    self.blending
  }

  pub fn set_depth_test(self, depth_test: DepthTest) -> Self {
    RenderState {
      depth_test,
      .. self
    }
  }

  pub fn depth_test(&self) -> DepthTest {
    self.depth_test
  }

  pub fn set_face_culling<FC>(self, face_culling: FC) -> Self where FC: Into<Option<FaceCulling>> {
    RenderState {
      face_culling: face_culling.into(),
      .. self
    }
  }

  pub fn face_culling(&self) -> Option<FaceCulling> {
    self.face_culling
  }
}

impl Default for RenderState {
  fn default() -> Self {
    RenderState {
      blending: None,
      depth_test: DepthTest::Enabled,
      face_culling: Some(FaceCulling::new(FaceCullingOrder::CCW, FaceCullingMode::Back))
    }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthTest {
  Enabled,
  Disabled
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FaceCulling {
  order: FaceCullingOrder,
  mode: FaceCullingMode
}

impl FaceCulling {
  pub fn new(order: FaceCullingOrder, mode: FaceCullingMode) -> Self {
    FaceCulling { order, mode }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingOrder {
  CW,
  CCW
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingMode {
  Front,
  Back,
  Both
}

pub struct TessGate<V> {
  _v: PhantomData<*const V>
}

impl<V> TessGate<V> where V: Vertex {
  pub fn render<W>(&self, tess: TessRender<W>) where W: CompatibleVertex<V> {
    tess.render();
  }
}

#[inline]
unsafe fn bind_buffer_at(buf: &RawBuffer, binding: u32) {
  gl::BindBufferBase(gl::UNIFORM_BUFFER, binding as GLuint, buf.handle());
}

#[inline]
unsafe fn bind_texture_at(tex: &RawTexture, unit: u32) {
  gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
  gl::BindTexture(tex.target(), tex.handle());
}

#[inline]
unsafe fn set_blending(blending: Option<(Equation, Factor, Factor)>) {
  match blending {
    Some((equation, src_factor, dest_factor)) => {
      gl::Enable(gl::BLEND);
      gl::BlendEquation(blending_equation(equation));
      gl::BlendFunc(blending_factor(src_factor), blending_factor(dest_factor));
    },
    None => {
      gl::Disable(gl::BLEND);
    }
  }
}

#[inline]
unsafe fn set_depth_test(test: DepthTest) {
  match test {
    DepthTest::Enabled => gl::Enable(gl::DEPTH_TEST),
    DepthTest::Disabled => gl::Disable(gl::DEPTH_TEST)
  }
}

#[inline]
unsafe fn set_face_culling(face_culling: Option<FaceCulling>) {
  match face_culling {
    Some(face_culling) => {
      gl::Enable(gl::CULL_FACE);
      gl::FrontFace(face_culling_order(face_culling.order));
      gl::CullFace(face_culling_mode(face_culling.mode));
    },
    None => {
      gl::Disable(gl::CULL_FACE);
    }
  }
}

#[inline]
fn blending_equation(equation: Equation) -> GLenum {
  match equation {
    Equation::Additive => gl::FUNC_ADD,
    Equation::Subtract => gl::FUNC_SUBTRACT,
    Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
    Equation::Min => gl::MIN,
    Equation::Max => gl::MAX
  }
}

#[inline]
fn blending_factor(factor: Factor) -> GLenum {
  match factor {
    Factor::One => gl::ONE,
    Factor::Zero => gl::ZERO,
    Factor::SrcColor => gl::SRC_COLOR,
    Factor::SrcColorComplement => gl::ONE_MINUS_SRC_COLOR,
    Factor::DestColor => gl::DST_COLOR,
    Factor::DestColorComplement => gl::ONE_MINUS_DST_COLOR,
    Factor::SrcAlpha => gl::SRC_ALPHA,
    Factor::SrcAlphaComplement => gl::ONE_MINUS_SRC_ALPHA,
    Factor::DstAlpha => gl::DST_ALPHA,
    Factor::DstAlphaComplement => gl::ONE_MINUS_DST_ALPHA,
    Factor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE
  }
}

#[inline]
fn face_culling_order(order: FaceCullingOrder) -> GLenum {
  match order {
    FaceCullingOrder::CW => gl::CW,
    FaceCullingOrder::CCW => gl::CCW
  }
}

#[inline]
fn face_culling_mode(mode: FaceCullingMode) -> GLenum {
  match mode {
    FaceCullingMode::Front => gl::FRONT,
    FaceCullingMode::Back => gl::BACK,
    FaceCullingMode::Both => gl::FRONT_AND_BACK,
  }
}
