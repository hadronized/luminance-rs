//! WebGL2 buffer implementation.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::rc::Rc;
use std::slice;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::webgl2::state::{Bind, WebGL2State};
use crate::webgl2::WebGL2;
use luminance::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
use luminance::buffer::BufferError;

/// WebGL buffer.
#[derive(Clone)]
pub struct Buffer<T> {
  pub(crate) gl_buf: WebGlBuffer,
  /// A cached version of the GPU buffer; emulate persistent mapping.
  pub(crate) buf: Vec<T>,
  state: Rc<RefCell<WebGL2State>>,
}

impl<T> Drop for Buffer<T> {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();

    state.unbind_buffer(&self.gl_buf);
    state.ctx.delete_buffer(Some(&self.gl_buf));
  }
}

impl<T> Buffer<T> {
  /// Create a new buffer from a length and a type. This is needed to implement repeat without Default.
  fn new(webgl2: &mut WebGL2, len: usize, clear_value: T) -> Result<Self, BufferError>
  where
    T: Copy,
  {
    let mut state = webgl2.state.borrow_mut();

    let mut buf = Vec::new();
    buf.resize_with(len, || clear_value);

    // generate a buffer and force binding the handle; this prevent side-effects from previous bound
    // resources to prevent binding the buffer
    let gl_buf = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&gl_buf), Bind::Forced);

    let bytes = mem::size_of::<T>() * len;
    state.ctx.buffer_data_with_i32(
      WebGl2RenderingContext::ARRAY_BUFFER,
      bytes as i32,
      WebGl2RenderingContext::STREAM_DRAW,
    );

    Ok(Buffer {
      gl_buf,
      buf,
      state: webgl2.state.clone(),
    })
  }

  /// Update the WebGL buffer by copying the cached vec.
  fn update_gl_buffer(
    state: &mut WebGL2State,
    gl_buf: &WebGlBuffer,
    data: *const T,
    offset: usize,
    size: usize,
  ) {
    state.bind_array_buffer(Some(gl_buf), Bind::Cached);

    let bytes = size * mem::size_of::<T>();
    let data = unsafe { slice::from_raw_parts(data as _, bytes) };
    state.ctx.buffer_sub_data_with_i32_and_u8_array(
      WebGl2RenderingContext::ARRAY_BUFFER,
      bytes as _,
      data,
    );
  }
}

unsafe impl<T> BufferBackend<T> for WebGL2
where
  T: Copy,
{
  type BufferRepr = Buffer<T>;

  unsafe fn new_buffer(&mut self, len: usize) -> Result<Self::BufferRepr, BufferError>
  where
    T: Default,
  {
    Buffer::<T>::new(self, len, T::default())
  }

  unsafe fn len(buffer: &Self::BufferRepr) -> usize {
    buffer.buf.len()
  }

  unsafe fn from_slice<S>(&mut self, slice: S) -> Result<Self::BufferRepr, BufferError>
  where
    S: AsRef<[T]>,
  {
    let mut state = self.state.borrow_mut();
    let slice = slice.as_ref();
    let len = slice.len();

    let gl_buf = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&gl_buf), Bind::Forced);

    let bytes = mem::size_of::<T>() * len;
    let data = slice::from_raw_parts(slice.as_ptr() as *const _, bytes);
    state.ctx.buffer_data_with_u8_array(
      WebGl2RenderingContext::ARRAY_BUFFER,
      data,
      WebGl2RenderingContext::STREAM_DRAW,
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
    Buffer::<T>::new(self, len, value)
  }

  unsafe fn at(buffer: &Self::BufferRepr, i: usize) -> Option<T> {
    buffer.buf.get(i).copied()
  }

  unsafe fn whole(buffer: &Self::BufferRepr) -> Vec<T> {
    buffer.buf.iter().copied().collect()
  }

  unsafe fn set(buffer: &mut Self::BufferRepr, i: usize, x: T) -> Result<(), BufferError> {
    let buffer_len = buffer.buf.len();

    if i >= buffer_len {
      Err(BufferError::Overflow {
        index: i,
        buffer_len,
      })
    } else {
      // update the cache first
      buffer.buf[i] = x;

      // then update the WebGL buffer
      let mut state = buffer.state.borrow_mut();
      Buffer::<T>::update_gl_buffer(&mut state, &buffer.gl_buf, buffer.buf.as_ptr(), i, 1);

      Ok(())
    }
  }

  unsafe fn write_whole(buffer: &mut Self::BufferRepr, values: &[T]) -> Result<(), BufferError> {
    let len = values.len();
    let buffer_len = buffer.buf.len();

    // error if we don’t pass the right number of items
    match len.cmp(&buffer_len) {
      Ordering::Less => {
        return Err(BufferError::TooFewValues {
          provided_len: len,
          buffer_len,
        })
      }

      Ordering::Greater => {
        return Err(BufferError::TooManyValues {
          provided_len: len,
          buffer_len,
        })
      }

      _ => (),
    }

    // update the internal representation of the vector; we clear it first then we extend with
    // the input slice to re-use the allocated region
    buffer.buf.clear();
    buffer.buf.extend_from_slice(values);

    // update the data on GPU
    let mut state = buffer.state.borrow_mut();
    Buffer::<T>::update_gl_buffer(
      &mut state,
      &buffer.gl_buf,
      buffer.buf.as_ptr(),
      0,
      values.len(),
    );

    Ok(())
  }

  unsafe fn clear(buffer: &mut Self::BufferRepr, x: T) -> Result<(), BufferError> {
    // copy the value everywhere in the buffer, then simply update the WebGL buffer
    for item in &mut buffer.buf {
      *item = x;
    }

    let mut state = buffer.state.borrow_mut();
    Buffer::<T>::update_gl_buffer(
      &mut state,
      &buffer.gl_buf,
      buffer.buf.as_ptr(),
      0,
      buffer.buf.len(),
    );

    Ok(())
  }
}

// Here, for buffer slices, we are going to use the property that when a buffer is mapped (immutably
// or mutably), we are the only owner of it. We can then _only_ write to the mapped buffer, and then
// update the GPU buffer on the Drop implementation.

pub struct BufferSlice<T> {
  gl_buf: WebGlBuffer,
  ptr: *const T,
  len: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl<T> Drop for BufferSlice<T> {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    Buffer::<T>::update_gl_buffer(&mut state, &self.gl_buf, self.ptr, 0, self.len);
  }
}

pub struct BufferSliceMut<T> {
  gl_buf: WebGlBuffer,
  ptr: *mut T,
  len: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl<T> Drop for BufferSliceMut<T> {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    Buffer::<T>::update_gl_buffer(&mut state, &self.gl_buf, self.ptr, 0, self.len);
  }
}

unsafe impl<T> BufferSliceBackend<T> for WebGL2
where
  T: Copy,
{
  type SliceRepr = BufferSlice<T>;

  type SliceMutRepr = BufferSliceMut<T>;

  unsafe fn slice_buffer(buffer: &Self::BufferRepr) -> Result<Self::SliceRepr, BufferError> {
    let slice = BufferSlice {
      gl_buf: buffer.gl_buf.clone(),
      ptr: buffer.buf.as_ptr(),
      len: buffer.buf.len(),
      state: buffer.state.clone(),
    };

    Ok(slice)
  }

  unsafe fn slice_buffer_mut(
    buffer: &mut Self::BufferRepr,
  ) -> Result<Self::SliceMutRepr, BufferError> {
    let slice = BufferSliceMut {
      gl_buf: buffer.gl_buf.clone(),
      ptr: buffer.buf.as_mut_ptr(),
      len: buffer.buf.len(),
      state: buffer.state.clone(),
    };

    Ok(slice)
  }

  unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError> {
    Ok(slice::from_raw_parts(slice.ptr, slice.len))
  }

  unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], BufferError> {
    Ok(slice::from_raw_parts_mut(slice.ptr, slice.len))
  }
}
