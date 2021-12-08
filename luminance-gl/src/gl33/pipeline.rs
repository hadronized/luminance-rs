use super::buffer::Buffer;
use crate::gl33::{
  state::{BlendingState, DepthTest, FaceCullingState, GLState, ScissorState},
  GL33,
};
use gl::types::*;
use luminance::{
  backend::{
    pipeline::{Pipeline as PipelineBackend, PipelineBase, PipelineShaderData, PipelineTexture},
    render_gate::RenderGate,
    shader::ShaderData,
    shading_gate::ShadingGate,
    tess::Tess,
    tess_gate::TessGate,
  },
  blending::BlendingMode,
  pipeline::{PipelineError, PipelineState, Viewport},
  pixel::Pixel,
  render_state::RenderState,
  tess::{Deinterleaved, DeinterleavedData, Interleaved, TessIndex, TessVertexData},
  texture::Dimensionable,
};
use luminance_std140::{ArrElem, Std140};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub struct Pipeline {
  state: Rc<RefCell<GLState>>,
}

pub struct BoundTexture<D, P>
where
  D: Dimensionable,
  P: Pixel,
{
  pub(crate) unit: u32,
  state: Rc<RefCell<GLState>>,
  _phantom: PhantomData<*const (D, P)>,
}

impl<D, P> Drop for BoundTexture<D, P>
where
  D: Dimensionable,
  P: Pixel,
{
  fn drop(&mut self) {
    // place the binding into the free list
    let mut state = self.state.borrow_mut();
    state.binding_stack_mut().free_texture_units.push(self.unit);
  }
}

pub struct BoundShaderData<T> {
  pub(crate) binding: u32,
  state: Rc<RefCell<GLState>>,
  _phantom: PhantomData<*const T>,
}

impl<T> Drop for BoundShaderData<T> {
  fn drop(&mut self) {
    // place the binding into the free list
    self
      .state
      .borrow_mut()
      .binding_stack_mut()
      .free_shader_data
      .push(self.binding);
  }
}

unsafe impl PipelineBase for GL33 {
  type PipelineRepr = Pipeline;

  unsafe fn new_pipeline(&mut self) -> Result<Self::PipelineRepr, PipelineError> {
    let pipeline = Pipeline {
      state: self.state.clone(),
    };

    Ok(pipeline)
  }
}

unsafe impl<D> PipelineBackend<D> for GL33
where
  D: Dimensionable,
{
  unsafe fn start_pipeline(
    &mut self,
    framebuffer: &Self::FramebufferRepr,
    pipeline_state: &PipelineState,
  ) {
    let mut state = self.state.borrow_mut();

    state.bind_draw_framebuffer(framebuffer.handle);

    let size = framebuffer.size;

    match pipeline_state.viewport {
      Viewport::Whole => {
        state.set_viewport([0, 0, D::width(size) as GLint, D::height(size) as GLint]);
      }

      Viewport::Specific {
        x,
        y,
        width,
        height,
      } => {
        state.set_viewport([x as GLint, y as GLint, width as GLint, height as GLint]);
      }
    }

    let mut clear_buffer_bits = 0;
    if let Some(clear_color) = pipeline_state.clear_color {
      state.set_clear_color([
        clear_color[0] as _,
        clear_color[1] as _,
        clear_color[2] as _,
        clear_color[3] as _,
      ]);

      clear_buffer_bits |= gl::COLOR_BUFFER_BIT;
    }

    if let Some(clear_depth) = pipeline_state.clear_depth {
      state.set_clear_depth(clear_depth);
      state.set_depth_write(luminance::depth_stencil::Write::On);
      clear_buffer_bits |= gl::DEPTH_BUFFER_BIT;
    }

    if let Some(clear_stencil) = pipeline_state.clear_stencil {
      state.set_clear_stencil(clear_stencil);
      clear_buffer_bits |= gl::STENCIL_BUFFER_BIT;
    }

    match pipeline_state.scissor().as_ref() {
      Some(region) => {
        state.set_scissor_state(ScissorState::On);
        state.set_scissor_region(region);
      }

      None => state.set_scissor_state(ScissorState::Off),
    }

    if clear_buffer_bits != 0 {
      gl::Clear(clear_buffer_bits);
    }

    state.enable_srgb_framebuffer(pipeline_state.srgb_enabled);
  }
}

unsafe impl<D, P> PipelineTexture<D, P> for GL33
where
  D: Dimensionable,
  P: Pixel,
{
  type BoundTextureRepr = BoundTexture<D, P>;

  unsafe fn bind_texture(
    pipeline: &Self::PipelineRepr,
    texture: &Self::TextureRepr,
  ) -> Result<Self::BoundTextureRepr, PipelineError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    let mut state = pipeline.state.borrow_mut();
    let bstack = state.binding_stack_mut();

    let unit = bstack.free_texture_units.pop().unwrap_or_else(|| {
      // no more free units; reserve one
      let unit = bstack.next_texture_unit;
      bstack.next_texture_unit += 1;
      unit
    });

    state.bind_texture_at(texture.target, texture.handle, unit);

    Ok(BoundTexture {
      unit,
      state: pipeline.state.clone(),
      _phantom: PhantomData,
    })
  }

  unsafe fn texture_binding(bound: &Self::BoundTextureRepr) -> u32 {
    bound.unit
  }
}

unsafe impl<T> PipelineShaderData<T> for GL33
where
  Self: ShaderData<T, ShaderDataRepr = Buffer<<ArrElem<T> as Std140>::Encoded>>,
  T: Std140,
{
  type BoundShaderDataRepr = BoundShaderData<T>;

  unsafe fn bind_shader_data(
    pipeline: &Self::PipelineRepr,
    shader_data: &Self::ShaderDataRepr,
  ) -> Result<Self::BoundShaderDataRepr, PipelineError> {
    let mut state = pipeline.state.borrow_mut();
    let bstack = state.binding_stack_mut();

    let binding = bstack.free_shader_data.pop().unwrap_or_else(|| {
      // no more free bindings; reserve one
      let binding = bstack.next_shader_data;
      bstack.next_shader_data += 1;
      binding
    });

    state.bind_uniform_buffer(shader_data.handle(), binding);

    Ok(BoundShaderData {
      binding,
      state: pipeline.state.clone(),
      _phantom: PhantomData,
    })
  }

  unsafe fn shader_data_binding(bound: &Self::BoundShaderDataRepr) -> u32 {
    bound.binding
  }
}

unsafe impl<V, I, W> TessGate<V, I, W, Interleaved> for GL33
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) {
    let _ = <Self as Tess<V, I, W, Interleaved>>::render(tess, start_index, vert_nb, inst_nb);
  }
}

unsafe impl<V, I, W> TessGate<V, I, W, Deinterleaved> for GL33
where
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
{
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) {
    let _ = <Self as Tess<V, I, W, Deinterleaved>>::render(tess, start_index, vert_nb, inst_nb);
  }
}

unsafe impl RenderGate for GL33 {
  unsafe fn enter_render_state(&mut self, rdr_st: &RenderState) {
    let mut gfx_state = self.state.borrow_mut();

    // blending state
    match rdr_st.blending() {
      Some(blending) => {
        gfx_state.set_blending_state(BlendingState::On);
        match blending {
          BlendingMode::Combined(b) => {
            gfx_state.set_blending_equation(b.equation);
            gfx_state.set_blending_func(b.src, b.dst);
          }
          BlendingMode::Separate { rgb, alpha } => {
            gfx_state.set_blending_equation_separate(rgb.equation, alpha.equation);
            gfx_state.set_blending_func_separate(rgb.src, rgb.dst, alpha.src, alpha.dst);
          }
        }
      }
      None => {
        gfx_state.set_blending_state(BlendingState::Off);
      }
    }

    // depth-related state
    if let Some(depth_comparison) = rdr_st.depth_test() {
      gfx_state.set_depth_test(DepthTest::On);
      gfx_state.set_depth_test_comparison(depth_comparison);
    } else {
      gfx_state.set_depth_test(DepthTest::Off);
    }

    gfx_state.set_depth_write(rdr_st.depth_write());

    // stencil-related state
    if let Some(stencil_test) = rdr_st.stencil_test() {
      gfx_state.enable_stencil_test(true);
      gfx_state.set_stencil_test(*stencil_test);
    } else {
      gfx_state.enable_stencil_test(false);
    }

    // stencil operations are always active
    gfx_state.set_stencil_operations(*rdr_st.stencil_operations());

    // face-culling state
    match rdr_st.face_culling() {
      Some(face_culling) => {
        gfx_state.set_face_culling_state(FaceCullingState::On);
        gfx_state.set_face_culling_order(face_culling.order);
        gfx_state.set_face_culling_mode(face_culling.mode);
      }
      None => {
        gfx_state.set_face_culling_state(FaceCullingState::Off);
      }
    }

    // scissor related state
    match rdr_st.scissor().as_ref() {
      Some(region) => {
        gfx_state.set_scissor_state(ScissorState::On);
        gfx_state.set_scissor_region(region);
      }

      None => {
        gfx_state.set_scissor_state(ScissorState::Off);
      }
    }
  }
}

unsafe impl ShadingGate for GL33 {
  unsafe fn apply_shader_program(&mut self, shader_program: &Self::ProgramRepr) {
    self.state.borrow_mut().use_program(shader_program.handle);
  }
}
