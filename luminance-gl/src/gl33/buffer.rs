//! OpenGL buffer implementation.

use gl;
use gl::types::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::slice;

use crate::gl33::state::{Bind, GLState};
use crate::gl33::GL33;
use luminance::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
use luminance::buffer::BufferError;

/// OpenGL buffer.
#[derive(Clone)]
pub struct Buffer<T> {
  pub(crate) gl_buf: GLuint,
  /// A cached version of the GPU buffer; emulate persistent mapping.
  pub(crate) buf: Vec<T>,
  state: Rc<RefCell<GLState>>,
}

impl<T> Drop for Buffer<T> {
  fn drop(&mut self) {
    unsafe {
      self.state.borrow_mut().unbind_buffer(self.gl_buf);
      gl::DeleteBuffers(1, &self.gl_buf);
    }
  }
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
    let mut gl_buf: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut gl_buf);
      gl33
        .state
        .borrow_mut()
        .bind_array_buffer(gl_buf, Bind::Forced);

      let bytes = mem::size_of::<T>() * len;
      gl::BufferData(
        gl::ARRAY_BUFFER,
        bytes as isize,
        buf.as_ptr() as _,
        gl::STREAM_DRAW,
      );
    }

    Ok(Buffer {
      gl_buf,
      buf,
      state: gl33.state.clone(),
    })
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
    Buffer::new::<T>(self, len, T::default())
  }

  unsafe fn len(buffer: &Self::BufferRepr) -> usize {
    buffer.buf.len()
  }

  unsafe fn from_slice<S>(&mut self, slice: S) -> Result<Self::BufferRepr, BufferError>
  where
    S: AsRef<[T]>,
  {
    let mut gl_buf: GLuint = 0;

    gl::GenBuffers(1, &mut gl_buf);
    self
      .state
      .borrow_mut()
      .bind_array_buffer(gl_buf, Bind::Forced);

    let slice = slice.as_ref();
    let len = slice.len();
    let bytes = mem::size_of::<T>() * len;
    gl::BufferData(
      gl::ARRAY_BUFFER,
      bytes as isize,
      slice.as_ptr() as _,
      gl::STREAM_DRAW,
    );

    // clone what the slice points to so that we can box and persist it
    let buf = slice.iter().copied().collect();

    Ok(Buffer {
      gl_buf,
      buf,
      state: self.state.clone(),
    })
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
    if i >= buffer.len() {
      Err(BufferError::Overflow {
        index: i,
        buffer_len: buffer.len(),
      })
    } else {
      // update cache first
      buffer.buf[i] = x;

      // then update the OpenGL buffer
      buffer
        .state
        .borrow_mut()
        .bind_array_buffer(buffer.gl_buf, Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;
      *ptr.add(i) = x;
      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      Ok(())
    }
  }

  unsafe fn write_whole(buffer: &mut Self::BufferRepr, values: &[T]) -> Result<(), BufferError> {
    let provided_len = values.len();
    let buffer_len = buffer.len();

    // error if we don’t pass the right number of items
    match provided_len.cmp(&buffer_len) {
      Ordering::Less => {
        return Err(BufferError::TooFewValues {
          provided_len,
          buffer_len,
        })
      }

      Ordering::Greater => {
        return Err(BufferError::TooManyValues {
          provided_len,
          buffer_len,
        })
      }

      _ => (),
    }

    // first update the OpenGL buffer; if it’s okay, then we can update the cache buffer
    buffer
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.gl_buf, Bind::Cached);

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

    unsafe {
      buffer
        .state
        .borrow_mut()
        .bind_array_buffer(buffer.gl_buf, Bind::Cached);

      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
      ptr::copy_nonoverlapping(buffer.buf.as_ptr(), ptr as *mut T, buffer.len());
      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }

    Ok(())
  }
}

pub struct BufferSlice<T> {
  handle: GLuint,
  len: usize,
  ptr: *const T,
  state: Rc<RefCell<GLState>>,
}

impl<T> Drop for BufferSlice<T> {
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

pub struct BufferSliceMut<T> {
  handle: GLuint,
  len: usize,
  ptr: *mut T,
  state: Rc<RefCell<GLState>>,
}

impl<T> Drop for BufferSliceMut<T> {
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

unsafe impl<T> BufferSliceBackend<T> for GL33
where
  T: Copy,
{
  type SliceRepr = BufferSlice<T>;

  type SliceMutRepr = BufferSliceMut<T>;

  unsafe fn slice_buffer(buffer: &Self::BufferRepr) -> Result<Self::SliceRepr, BufferError> {
    buffer
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle, Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;
    let handle = buffer.handle;
    let len = buffer.len;
    let state = buffer.state.clone();

    if ptr.is_null() {
      Err(BufferError::MapFailed)
    } else {
      Ok(BufferSlice {
        handle,
        len,
        ptr,
        state,
      })
    }
  }

  unsafe fn slice_buffer_mut(
    buffer: &mut Self::BufferRepr,
  ) -> Result<Self::SliceMutRepr, BufferError> {
    buffer
      .state
      .borrow_mut()
      .bind_array_buffer(buffer.handle, Bind::Cached);

    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T;
    let handle = buffer.handle;
    let len = buffer.len;
    let state = buffer.state.clone();

    if ptr.is_null() {
      Err(BufferError::MapFailed)
    } else {
      Ok(BufferSliceMut {
        handle,
        len,
        ptr,
        state,
      })
    }
  }

  unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError> {
    Ok(slice::from_raw_parts(slice.ptr, slice.len))
  }

  unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], BufferError> {
    Ok(slice::from_raw_parts_mut(slice.ptr, slice.len))
  }
}
