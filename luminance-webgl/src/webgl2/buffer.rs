//! WebGL2 buffer implementation.

use crate::webgl2::{
  state::{Bind, WebGL2State},
  WebGL2,
};
use core::fmt;
use luminance::vertex_entity::TessError;
use std::{
  cell::RefCell,
  error,
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
  rc::Rc,
  slice,
};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

/// Errors that can occur when dealing with buffers.
#[derive(Clone, Debug)]
pub enum BufferError {
  /// Cannot create the buffer on the backend.
  CannotCreate,
}

impl fmt::Display for BufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      BufferError::CannotCreate => f.write_str("cannot create buffer on the backend"),
    }
  }
}

impl error::Error for BufferError {}

impl From<BufferError> for TessError {
  fn from(e: BufferError) -> Self {
    TessError::cannot_create(e.to_string())
  }
}

/// Wrapped WebGL buffer.
///
/// Used to drop the buffer.
#[derive(Clone, Debug)]
struct BufferWrapper<const TARGET: u32> {
  handle: WebGlBuffer,
  state: Rc<RefCell<WebGL2State>>,
}

impl<const TARGET: u32> Drop for BufferWrapper<TARGET> {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();

    state.unbind_buffer(&self.handle);
    state.ctx.delete_buffer(Some(&self.handle));
  }
}

/// WebGL buffer.
#[derive(Clone, Debug)]
pub struct Buffer<T, const TARGET: u32> {
  /// A cached version of the GPU buffer; emulate persistent mapping.
  pub(crate) buf: Vec<T>,
  gl_buf: BufferWrapper<TARGET>,
}

impl<T, const TARGET: u32> Buffer<T, TARGET>
where
  WebGL2State: BindBuffer<TARGET>,
{
  pub(crate) fn from_vec(webgl2: &mut WebGL2, vec: Vec<T>) -> Result<Self, BufferError> {
    let mut state = webgl2.state.borrow_mut();
    let len = vec.len();

    let handle = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;

    state.bind_buffer(&handle, Bind::Forced);

    let bytes = mem::size_of::<T>() * len;
    let data = unsafe { slice::from_raw_parts(vec.as_ptr() as *const _, bytes) };
    state
      .ctx
      .buffer_data_with_u8_array(TARGET, data, WebGl2RenderingContext::STREAM_DRAW);

    let gl_buf = BufferWrapper {
      handle,
      state: webgl2.state.clone(),
    };

    Ok(Buffer { gl_buf, buf: vec })
  }

  pub(crate) fn handle(&self) -> &WebGlBuffer {
    &self.gl_buf.handle
  }

  pub(crate) fn slice_buffer(&self) -> BufferSlice<T> {
    BufferSlice {
      handle: &self.gl_buf.handle,
      ptr: self.buf.as_ptr(),
      len: self.buf.len(),
      state: self.gl_buf.state.clone(),
    }
  }

  pub(crate) fn slice_buffer_mut(&mut self) -> BufferSliceMut<T, TARGET> {
    let raw = BufferSliceMutWrapper {
      handle: &self.gl_buf.handle,
      ptr: self.buf.as_mut_ptr() as *mut u8,
      bytes: self.buf.len() * mem::size_of::<T>(),
      state: self.gl_buf.state.clone(),
    };

    BufferSliceMut {
      raw,
      _phantom: PhantomData,
    }
  }
}

pub struct BufferSlice<'a, T> {
  handle: &'a WebGlBuffer,
  ptr: *const T,
  len: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl<'a> BufferSlice<'a, u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSlice<'a, T> {
    let handle = self.handle;
    let ptr = self.ptr as *const T;
    let len = self.len / mem::size_of::<T>();
    let state = self.state;

    BufferSlice {
      handle,
      ptr,
      len,
      state,
    }
  }
}

impl<T> Deref for BufferSlice<'_, T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

/// Buffer mutable slice wrapper.
///
/// When a buffer is mapped, we are the only owner of it. We can then read or write from/to the
/// mapped buffer, and then update the GPU buffer on the [`Drop`] implementation.
pub struct BufferSliceMutWrapper<'a, const TARGET: u32>
where
  WebGL2State: BindBuffer<TARGET>,
{
  handle: &'a WebGlBuffer,
  ptr: *mut u8,
  bytes: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl<const TARGET: u32> Drop for BufferSliceMutWrapper<'_, TARGET>
where
  WebGL2State: BindBuffer<TARGET>,
{
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    let _ = update_webgl_buffer::<TARGET>(&mut state, &self.handle, self.ptr, self.bytes, 0);
  }
}

pub struct BufferSliceMut<'a, T, const TARGET: u32>
where
  WebGL2State: BindBuffer<TARGET>,
{
  raw: BufferSliceMutWrapper<'a, TARGET>,
  _phantom: PhantomData<T>,
}

impl<'a, const TARGET: u32> BufferSliceMut<'a, u8, TARGET>
where
  WebGL2State: BindBuffer<TARGET>,
{
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSliceMut<'a, T, TARGET> {
    BufferSliceMut {
      raw: self.raw,
      _phantom: PhantomData,
    }
  }
}

impl<T, const TARGET: u32> Deref for BufferSliceMut<'_, T, TARGET>
where
  WebGL2State: BindBuffer<TARGET>,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe {
      slice::from_raw_parts(
        self.raw.ptr as *const T,
        self.raw.bytes / mem::size_of::<T>(),
      )
    }
  }
}

impl<T, const TARGET: u32> DerefMut for BufferSliceMut<'_, T, TARGET>
where
  WebGL2State: BindBuffer<TARGET>,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe {
      slice::from_raw_parts_mut(self.raw.ptr as *mut T, self.raw.bytes / mem::size_of::<T>())
    }
  }
}

pub trait BindBuffer<const TARGET: u32> {
  fn bind_buffer(&mut self, handle: &WebGlBuffer, bind_mode: Bind);
}

impl BindBuffer<{ WebGl2RenderingContext::ARRAY_BUFFER }> for WebGL2State {
  fn bind_buffer(&mut self, handle: &WebGlBuffer, bind_mode: Bind) {
    self.bind_array_buffer(Some(handle), bind_mode);
  }
}

impl BindBuffer<{ WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER }> for WebGL2State {
  fn bind_buffer(&mut self, handle: &WebGlBuffer, bind_mode: Bind) {
    self.bind_element_array_buffer(Some(handle), bind_mode);
  }
}

impl BindBuffer<{ WebGl2RenderingContext::UNIFORM_BUFFER }> for WebGL2State {
  fn bind_buffer(&mut self, handle: &WebGlBuffer, bind: Bind) {
    self.bind_uniform_buffer(Some(handle), bind);
  }
}

/// Update a WebGL buffer by copying an input slice.
fn update_webgl_buffer<const TARGET: u32>(
  state: &mut WebGL2State,
  handle: &WebGlBuffer,
  data: *const u8,
  bytes: usize,
  offset: usize,
) -> Result<(), BufferError>
where
  WebGL2State: BindBuffer<TARGET>,
{
  state.bind_buffer(handle, Bind::Cached);

  let data = unsafe { slice::from_raw_parts(data as _, bytes) };
  state
    .ctx
    .buffer_sub_data_with_i32_and_u8_array_and_src_offset(TARGET, offset as _, data, 0);

  Ok(())
}
