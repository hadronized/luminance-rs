//! OpenGL buffer implementation.

use gl::types::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::rc::Rc;
use std::slice;

use crate::gl33::state::{Bind, GLState};
use crate::gl33::GL33;
use luminance::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
use luminance::buffer::BufferError;

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
  /// Build a buffer with a number of elements for a given type.
  ///
  /// That function is required to implement repeat without Default.
  fn new(gl33: &mut GL33, len: usize, clear_value: T) -> Result<Self, BufferError>
  where
    T: Copy,
  {
    let mut buf = Vec::new();
    buf.resize_with(len, || clear_value);

    // generate a buffer and force binding the handle; this prevent side-effects from previous bound
    // resources to prevent binding the buffer
    let mut handle: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut handle);
      gl33
        .state
        .borrow_mut()
        .bind_array_buffer(handle, Bind::Forced);

      let bytes = mem::size_of::<T>() * len;
      gl::BufferData(
        gl::ARRAY_BUFFER,
        bytes as isize,
        buf.as_ptr() as _,
        gl::STREAM_DRAW,
      );
    }
    let state = gl33.state.clone();
    let gl_buf = BufferWrapper { handle, state };

    Ok(Buffer { gl_buf, buf })
  }

  pub(crate) fn handle(&self) -> GLuint {
    self.gl_buf.handle
  }

  pub(crate) fn into_raw(self) -> Buffer<u8> {
    let boxed = self.buf.into_boxed_slice();
    let len = boxed.len();
    let ptr = Box::into_raw(boxed) as _;
    let buf = unsafe { Vec::from_raw_parts(ptr, len, len) };

    Buffer {
      buf,
      gl_buf: self.gl_buf,
    }
  }
}

unsafe impl<T> BufferBackend<T> for GL33
where
  T: Copy,
{
  type BufferRepr = Buffer<T>;

  unsafe fn new_buffer(&mut self, len: usize) -> Result<Self::BufferRepr, BufferError>
  where
    T: Default,
  {
    Buffer::new(self, len, T::default())
  }

  unsafe fn len(buffer: &Self::BufferRepr) -> usize {
    buffer.buf.len()
  }

  unsafe fn from_vec(&mut self, vec: Vec<T>) -> Result<Self::BufferRepr, BufferError> {
    let mut handle: GLuint = 0;

    gl::GenBuffers(1, &mut handle);
    self
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
    let state = self.state.clone();
    let gl_buf = BufferWrapper { handle, state };

    Ok(Buffer { gl_buf, buf: vec })
  }

  unsafe fn repeat(&mut self, len: usize, value: T) -> Result<Self::BufferRepr, BufferError> {
    Buffer::new(self, len, value)
  }

  unsafe fn at(buffer: &Self::BufferRepr, i: usize) -> Option<T> {
    buffer.buf.get(i).copied()
  }

  unsafe fn whole(buffer: &Self::BufferRepr) -> Vec<T> {
    buffer.buf.iter().copied().collect()
  }

  unsafe fn set(buffer: &mut Self::BufferRepr, i: usize, x: T) -> Result<(), BufferError> {
    if i >= buffer.buf.len() {
      Err(BufferError::overflow(i, buffer.buf.len()))
    } else {
      // update cache first
      buffer.buf[i] = x;

      // then update the OpenGL buffer
      buffer
        .gl_buf
        .state
        .borrow_mut()
        .bind_array_buffer(buffer.handle(), Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;
      *ptr.add(i) = x;
      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      Ok(())
    }
  }

  unsafe fn write_whole(buffer: &mut Self::BufferRepr, values: &[T]) -> Result<(), BufferError> {
    let provided_len = values.len();
    let buffer_len = buffer.buf.len();

    // error if we don’t pass the right number of items
    match provided_len.cmp(&buffer_len) {
      Ordering::Less => return Err(BufferError::too_few_values(provided_len, buffer_len)),

      Ordering::Greater => return Err(BufferError::too_many_values(provided_len, buffer_len)),

      _ => (),
    }

    // first update the OpenGL buffer; if it’s okay, then we can update the cache buffer
    buffer
      .gl_buf
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle(), Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
    ptr::copy_nonoverlapping(values.as_ptr(), ptr as *mut T, buffer_len);
    let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

    buffer.buf.copy_from_slice(values);

    Ok(())
  }

  unsafe fn clear(buffer: &mut Self::BufferRepr, x: T) -> Result<(), BufferError> {
    for item in &mut buffer.buf {
      *item = x;
    }

    buffer
      .gl_buf
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle(), Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
    ptr::copy_nonoverlapping(buffer.buf.as_ptr(), ptr as *mut T, buffer.buf.len());
    let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

    Ok(())
  }
}

/// Wrapper to drop buffer slices.
struct BufferSliceWrapper {
  handle: GLuint,
  state: Rc<RefCell<GLState>>,
}

impl Drop for BufferSliceWrapper {
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

pub struct BufferSlice<T> {
  raw: BufferSliceWrapper,
  len: usize,
  ptr: *const T,
}

impl BufferSlice<u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSlice<T> {
    let len = self.len / mem::size_of::<T>();
    let ptr = self.ptr as _;

    BufferSlice {
      raw: self.raw,
      len,
      ptr,
    }
  }
}

impl<T> Deref for BufferSlice<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

pub struct BufferSliceMut<T> {
  raw: BufferSliceWrapper,
  len: usize,
  ptr: *mut T,
}

impl BufferSliceMut<u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSliceMut<T> {
    let len = self.len / mem::size_of::<T>();
    let ptr = self.ptr as _;

    BufferSliceMut {
      raw: self.raw,
      len,
      ptr,
    }
  }
}

impl<T> Deref for BufferSliceMut<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr as *const _, self.len) }
  }
}

impl<T> DerefMut for BufferSliceMut<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
  }
}

unsafe impl<T> BufferSliceBackend<T> for GL33
where
  T: Copy,
{
  type SliceRepr = BufferSlice<T>;

  type SliceMutRepr = BufferSliceMut<T>;

  unsafe fn slice_buffer(buffer: &Self::BufferRepr) -> Result<Self::SliceRepr, BufferError> {
    buffer
      .gl_buf
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle(), Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

    if ptr.is_null() {
      Err(BufferError::map_failed())
    } else {
      let handle = buffer.handle();
      let state = buffer.gl_buf.state.clone();
      let raw = BufferSliceWrapper { handle, state };
      let len = buffer.buf.len();

      Ok(BufferSlice { raw, len, ptr })
    }
  }

  unsafe fn slice_buffer_mut(
    buffer: &mut Self::BufferRepr,
  ) -> Result<Self::SliceMutRepr, BufferError> {
    buffer
      .gl_buf
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle(), Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T;

    if ptr.is_null() {
      Err(BufferError::map_failed())
    } else {
      let handle = buffer.handle();
      let state = buffer.gl_buf.state.clone();
      let raw = BufferSliceWrapper { handle, state };
      let len = buffer.buf.len();

      Ok(BufferSliceMut { raw, len, ptr })
    }
  }

  unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError> {
    Ok(slice::from_raw_parts(slice.ptr, slice.len))
  }

  unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], BufferError> {
    Ok(slice::from_raw_parts_mut(slice.ptr, slice.len))
  }
}
