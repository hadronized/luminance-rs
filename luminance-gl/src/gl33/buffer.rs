//! OpenGL buffer implementation.

use crate::gl33::{
  state::{Bind, GLState},
  GL33,
};
use gl;
use gl::types::*;
use luminance::vertex_entity::TessMapError;
use std::{
  cell::RefCell,
  error, fmt, mem,
  ops::{Deref, DerefMut},
  rc::Rc,
  slice,
};

#[derive(Debug, Eq, PartialEq)]
pub enum SliceBufferError {
  /// Buffer mapping failed.
  MapFailed,
}

impl fmt::Display for SliceBufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      SliceBufferError::MapFailed => f.write_str("buffer mapping failed"),
    }
  }
}

impl error::Error for SliceBufferError {}

impl From<SliceBufferError> for TessMapError {
  fn from(_: SliceBufferError) -> Self {
    TessMapError::CannotMap
  }
}

/// Wrapped OpenGL buffer.
///
/// Used to drop the buffer.
#[derive(Debug)]
struct BufferWrapper {
  handle: GLuint,
  state: Rc<RefCell<GLState>>,
}

impl Drop for BufferWrapper {
  fn drop(&mut self) {
    unsafe {
      self.state.borrow_mut().unbind_buffer(self.handle);
      gl::DeleteBuffers(1, &self.handle);
    }
  }
}

/// OpenGL buffer.
#[derive(Debug)]
pub struct Buffer<T> {
  /// A cached version of the GPU buffer; emulate persistent mapping.
  pub(crate) buf: Vec<T>,
  gl_buf: BufferWrapper,
}

impl<T> Buffer<T> {
  pub(crate) unsafe fn from_vec(gl33: &mut GL33, vec: Vec<T>) -> Self {
    let mut handle: GLuint = 0;

    gl::GenBuffers(1, &mut handle);
    gl33
      .state
      .borrow_mut()
      .bind_array_buffer(handle, Bind::Forced);

    let len = vec.len();
    let bytes = mem::size_of::<T>() * len;
    gl::BufferData(
      gl::ARRAY_BUFFER,
      bytes as isize,
      vec.as_ptr() as _,
      gl::STREAM_DRAW,
    );
    let state = gl33.state.clone();
    let gl_buf = BufferWrapper { handle, state };

    Buffer { gl_buf, buf: vec }
  }

  pub(crate) fn handle(&self) -> GLuint {
    self.gl_buf.handle
  }

  /// Length of the buffer (number of elements).
  #[inline]
  pub fn len(&self) -> usize {
    self.buf.len()
  }

  pub(crate) fn slice_buffer(&self) -> Result<BufferSlice<T>, SliceBufferError> {
    unsafe {
      self
        .gl_buf
        .state
        .borrow_mut()
        .bind_array_buffer(self.handle(), Bind::Cached);
    }

    mapping_buffer(gl::ARRAY_BUFFER, gl::READ_ONLY, |ptr| {
      let handle = self.handle();
      let state = &self.gl_buf.state;
      let raw = BufferSliceWrapper { handle, state };
      let len = self.buf.len();

      BufferSlice { raw, len, ptr }
    })
  }

  pub(crate) fn slice_buffer_mut(&mut self) -> Result<BufferSliceMut<T>, SliceBufferError> {
    unsafe {
      self
        .gl_buf
        .state
        .borrow_mut()
        .bind_array_buffer(self.handle(), Bind::Cached);
    }

    mapping_buffer(gl::ARRAY_BUFFER, gl::READ_WRITE, move |ptr| {
      let handle = self.handle();
      let state = &self.gl_buf.state;
      let raw = BufferSliceWrapper { handle, state };
      let len = self.buf.len();

      BufferSliceMut { raw, len, ptr }
    })
  }
}

/// Wrapper to drop buffer slices.
struct BufferSliceWrapper<'a> {
  handle: GLuint,
  // we use a &'a to the state to prevent cloning it when creating a buffer slice and keep the lifetime around
  state: &'a Rc<RefCell<GLState>>,
}

impl Drop for BufferSliceWrapper<'_> {
  fn drop(&mut self) {
    unsafe {
      self
        .state
        .borrow_mut()
        .bind_array_buffer(self.handle, Bind::Cached);

      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

pub struct BufferSlice<'a, T> {
  raw: BufferSliceWrapper<'a>,
  len: usize,
  ptr: *const T,
}

impl<T> Deref for BufferSlice<'_, T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

impl<'a> BufferSlice<'a, u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSlice<'a, T> {
    let len = self.len / mem::size_of::<T>();
    let ptr = self.ptr as _;

    BufferSlice {
      raw: self.raw,
      len,
      ptr,
    }
  }
}

pub struct BufferSliceMut<'a, T> {
  raw: BufferSliceWrapper<'a>,
  len: usize,
  ptr: *mut T,
}

impl<T> Deref for BufferSliceMut<'_, T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr as *const _, self.len) }
  }
}

impl<T> DerefMut for BufferSliceMut<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
  }
}

impl<'a> BufferSliceMut<'a, u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSliceMut<'a, T> {
    let len = self.len / mem::size_of::<T>();
    let ptr = self.ptr as _;

    BufferSliceMut {
      raw: self.raw,
      len,
      ptr,
    }
  }
}

/// Map a buffer and execute an action if correctly mapped; otherwise, return an error.
fn mapping_buffer<A, T>(
  target: GLenum,
  access: GLenum,
  f: impl FnOnce(*mut T) -> A,
) -> Result<A, SliceBufferError> {
  let ptr = unsafe { gl::MapBuffer(target, access) } as *mut T;

  if ptr.is_null() {
    Err(SliceBufferError::MapFailed)
  } else {
    Ok(f(ptr))
  }
}
