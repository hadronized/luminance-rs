use std::{cell::RefCell, rc::Rc};

use crate::{
  backend::{
    Backend, FramebufferError, PipelineError, ShaderError, TextureError, VertexEntityError,
  },
  dim::Dimensionable,
  framebuffer::{Back, Framebuffer},
  pipeline::{PipelineState, WithFramebuffer},
  pixel::Pixel,
  primitive::Primitive,
  render_channel::{DepthChannel, RenderChannel},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  shader::{Program, ProgramBuilder, ProgramUpdate, Uniforms},
  texture::{InUseTexture, Mipmaps, Texture, TextureSampling},
  vertex::Vertex,
  vertex_entity::VertexEntity,
  vertex_storage::AsVertexStorage,
};

#[derive(Clone, Debug)]
pub struct ContextActive(Rc<RefCell<bool>>);

impl ContextActive {
  fn new() -> Self {
    Self(Rc::new(RefCell::new(true)))
  }

  pub fn is_active(&self) -> bool {
    *self.0.borrow()
  }
}

#[derive(Debug)]
pub struct Context<B>
where
  B: Backend,
{
  backend: B,
  context_active: ContextActive,
}

impl<B> Context<B>
where
  B: Backend,
{
  pub fn new(builder: impl FnOnce(ContextActive) -> Option<B>) -> Option<Self> {
    let context_active = ContextActive::new();
    let backend = builder(context_active.clone())?;

    Some(Self {
      backend,
      context_active,
    })
  }

  pub fn new_vertex_entity<V, P, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, P, S>, VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
    I: Into<Vec<u32>>,
  {
    unsafe { self.backend.new_vertex_entity(storage, indices) }
  }

  pub fn update_vertices<V, P, S>(
    &mut self,
    entity: &mut VertexEntity<V, P, S>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
  {
    unsafe {
      self
        .backend
        .vertex_entity_update_vertices(entity.handle(), entity.storage())
    }
  }

  pub fn update_indices<V, P, S>(
    &mut self,
    entity: &mut VertexEntity<V, P, S>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    P: Primitive,
    S: AsVertexStorage<V>,
  {
    unsafe {
      self
        .backend
        .vertex_entity_update_indices(entity.handle(), entity.indices())
    }
  }

  pub fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    unsafe { self.backend.new_framebuffer(size, mipmaps) }
  }

  pub fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, Back<RS>, Back<DS>>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot,
  {
    unsafe { self.backend.back_buffer(size) }
  }

  pub fn new_program<V, W, P, Q, S, E>(
    &mut self,
    builder: ProgramBuilder<V, W, P, Q, S, E>,
  ) -> Result<Program<V, P, S, E>, ShaderError>
  where
    V: Vertex,
    W: Vertex,
    P: Primitive<Vertex = W>,
    Q: Primitive,
    S: RenderSlots,
    E: Uniforms,
  {
    unsafe {
      self.backend.new_program(
        builder.vertex_code,
        builder.primitive_code,
        builder.shading_code,
      )
    }
  }

  pub fn update_program<'a, V, P, S, E>(
    &'a mut self,
    program: &Program<V, P, S, E>,
    updater: impl FnOnce(ProgramUpdate<'a, B>, &E) -> Result<(), ShaderError>,
  ) -> Result<(), ShaderError>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
  {
    let program_update = ProgramUpdate {
      backend: &mut self.backend,
      program_handle: program.handle(),
    };

    updater(program_update, &program.uniforms)
  }

  pub fn reserve_texture<D, P>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: TextureSampling,
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe { self.backend.reserve_texture(size, mipmaps, sampling) }
  }

  pub fn new_texture<D, P>(
    &mut self,
    size: D::Size,
    mipmaps: Mipmaps,
    sampling: TextureSampling,
    texels: &[P::RawEncoding],
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe { self.backend.new_texture(size, mipmaps, sampling, texels) }
  }

  pub fn resize_texture<D, P>(
    &mut self,
    texture: &Texture<D, P>,
    size: D::Size,
    mipmaps: Mipmaps,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      self
        .backend
        .resize_texture::<D, P>(texture.handle(), size, mipmaps)
    }
  }

  pub fn new_texture_with_levels<D, P>(
    &mut self,
    size: D::Size,
    sampling: TextureSampling,
    levels: &[&[P::RawEncoding]],
  ) -> Result<Texture<D, P>, TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      let levels_count = levels.len();
      let texture = self.backend.reserve_texture(size, Mipmaps::No, sampling)?;

      for level in 0..levels_count {
        self.backend.set_texture_data::<D, P>(
          texture.handle(),
          D::ZERO_OFFSET,
          size,
          false,
          &levels[level],
          level,
        )?;
      }

      Ok(texture)
    }
  }

  pub fn set_texture_base_level<D, P>(
    &mut self,
    texture: &Texture<D, P>,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      self
        .backend
        .set_texture_data::<D, P>(texture.handle(), offset, size, true, texels, 0)
    }
  }

  pub fn set_texture_level<D, P>(
    &mut self,
    texture: &Texture<D, P>,
    offset: D::Offset,
    size: D::Size,
    texels: &[P::RawEncoding],
    level: usize,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      self
        .backend
        .set_texture_data::<D, P>(texture.handle(), offset, size, false, texels, level)
    }
  }

  pub fn clear_texture_data<D, P>(
    &mut self,
    texture: &Texture<D, P>,
    offset: D::Offset,
    size: D::Size,
    clear_value: P::RawEncoding,
  ) -> Result<(), TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      self
        .backend
        .clear_texture_data::<D, P>(texture.handle(), offset, size, true, clear_value)
    }
  }

  pub fn get_texels<D, P>(
    &mut self,
    texture: &Texture<D, P>,
  ) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe { self.backend.get_texels::<D, P>(texture.handle()) }
  }

  pub fn with_framebuffer<D, CS, DS, Err>(
    &mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    state: &PipelineState,
    f: impl for<'a> FnOnce(WithFramebuffer<'a, B, CS>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<PipelineError>,
  {
    unsafe { self.backend.with_framebuffer(framebuffer, state, f) }
  }

  pub fn use_texture<D, P>(
    &mut self,
    texture: &Texture<D, P>,
  ) -> Result<InUseTexture<D, P::Type>, TextureError>
  where
    D: Dimensionable,
    P: Pixel,
  {
    unsafe { self.backend.use_texture(texture.handle()) }
  }

  pub fn use_render_layer<D, RC>(
    &mut self,
    render_layer: &RenderLayer<D, RC>,
  ) -> Result<InUseTexture<D, RC::Type>, FramebufferError>
  where
    D: Dimensionable,
    RC: RenderChannel,
  {
    unsafe { self.backend.use_render_layer(render_layer.handle()) }
  }

  pub fn use_depth_render_layer<D, DC>(
    &mut self,
    render_layer: &RenderLayer<D, DC>,
  ) -> Result<InUseTexture<D, DC::Type>, FramebufferError>
  where
    D: Dimensionable,
    DC: DepthChannel,
  {
    unsafe { self.backend.use_depth_render_layer(render_layer.handle()) }
  }
}

impl<B> Drop for Context<B>
where
  B: Backend,
{
  fn drop(&mut self) {
    unsafe {
      self.backend.unload();
    }

    *self.context_active.0.borrow_mut() = false;
  }
}
