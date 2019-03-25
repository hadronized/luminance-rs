//! Static GPU typed arrays.
//!
//! A GPU buffer is a typed continuous region of data. It has a size and can hold several elements.
//!
//! Buffers are created with the `new` associated function. You pass in the number of elements you
//! want in the buffer along with the `GraphicsContext` to create the buffer in.
//!
//! ```ignore
//! let buffer: Buffer<f32> = Buffer::new(&mut ctx, 5);
//! ```
//!
//! Once the buffer is created, you can perform several operations on them:
//!
//! - Writing to them.
//! - Reading from them.
//! - Passing them around as uniforms.
//! - Etc.
//!
//! However, you cannot change their size at runtime.
//!
//! # Writing to a buffer
//!
//! `Buffer`s support several write methods. The simple one is *clearing*. That is, replacing the
//! whole content of the buffer with a single value. Use the `clear` function to do so.
//!
//! ```ignore
//! buffer.clear(0.);
//! ```
//!
//! If you want to clear the buffer by providing a value for each elements, you want *filling*. Use
//! the `fill` function:
//!
//! ```ignore
//! buffer.fill([1., 2., 3., 4., 5.]);
//! ```
//!
//! You want to change a value at a given index? Easy, you can use the `set` function:
//!
//! ```ignore
//! buffer.set(3, 3.14);
//! ```
//!
//! # Reading from the buffer
//!
//! You can either retrieve the `whole` content of the `Buffer` or `get` a value with an index.
//!
//! ```ignore
//! // get the whole content
//! let all_elems = buffer.whole();
//! assert_eq!(all_elems, vec![1., 2., 3., 3.14, 5.]); // admit floating equalities
//!
//! // get the element at index 3
//! assert_eq!(buffer.at(3), Some(3.14));
//! ```
//!
//! # Uniform buffer
//!
//! It’s possible to use buffers as *uniform buffers*. That is, buffers that will be in bound at
//! rendering time and which content will be available for a shader to read (no write).
//!
//! In order to use your buffers in a uniform context, the inner type has to implement
//! `UniformBlock`. Keep in mind alignment must be respected and is a bit peculiar. TODO: explain
//! std140 here.

#[cfg(feature = "std")]
use std::cell::RefCell;
#[cfg(feature = "std")]
use std::cmp::Ordering;
#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::marker::PhantomData;
#[cfg(feature = "std")]
use std::mem;
#[cfg(feature = "std")]
use std::ops::{Deref, DerefMut};
#[cfg(feature = "std")]
use std::os::raw::c_void;
#[cfg(feature = "std")]
use std::ptr;
#[cfg(feature = "std")]
use std::rc::Rc;
#[cfg(feature = "std")]
use std::slice;

#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::cell::RefCell;
#[cfg(not(feature = "std"))]
use core::cmp::Ordering;
#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(not(feature = "std"))]
use core::marker::PhantomData;
#[cfg(not(feature = "std"))]
use core::mem;
#[cfg(not(feature = "std"))]
use core::ops::{Deref, DerefMut};
#[cfg(not(feature = "std"))]
use core::ptr;
#[cfg(not(feature = "std"))]
use core::slice;

use crate::context::GraphicsContext;
use crate::linear::{M22, M33, M44};
use crate::metagl::*;
use crate::state::GraphicsState;

/// Buffer errors.
#[derive(Debug, Eq, PartialEq)]
pub enum BufferError {
  /// Overflow when setting a value with a specific index.
  ///
  /// Contains the index and the size of the buffer.
  Overflow(usize, usize),
  /// Too few values were passed to fill a buffer.
  ///
  /// Contains the number of passed value and the size of the buffer.
  TooFewValues(usize, usize),
  /// Too many values were passed to fill a buffer.
  ///
  /// Contains the number of passed value and the size of the buffer.
  TooManyValues(usize, usize),
  /// Mapping the buffer failed.
  MapFailed,
}

impl fmt::Display for BufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      BufferError::Overflow(i, size) => write!(f, "buffer overflow (index = {}, size = {})", i, size),

      BufferError::TooFewValues(nb, size) => write!(
        f,
        "too few values passed to the buffer (nb = {}, size = {})",
        nb, size
      ),

      BufferError::TooManyValues(nb, size) => write!(
        f,
        "too many values passed to the buffer (nb = {}, size = {})",
        nb, size
      ),

      BufferError::MapFailed => write!(f, "buffer mapping failed"),
    }
  }
}

/// A `Buffer` is a GPU region you can picture as an array. It has a static size and cannot be
/// resized. The size is expressed in number of elements lying in the buffer – not in bytes.
pub struct Buffer<T> {
  raw: RawBuffer,
  _t: PhantomData<T>,
}

impl<T> Buffer<T> {
  /// Create a new `Buffer` with a given number of elements.
  pub fn new<C>(ctx: &mut C, len: usize) -> Buffer<T>
  where
    C: GraphicsContext,
  {
    let mut buffer: GLuint = 0;
    let bytes = mem::size_of::<T>() * len;

    unsafe {
      gl::GenBuffers(1, &mut buffer);
      ctx.state().borrow_mut().bind_array_buffer(buffer);
      gl::BufferData(gl::ARRAY_BUFFER, bytes as isize, ptr::null(), gl::STREAM_DRAW);
    }

    Buffer {
      raw: RawBuffer {
        handle: buffer,
        bytes,
        len,
        state: ctx.state().clone(),
      },
      _t: PhantomData,
    }
  }

  /// Create a buffer out of a slice.
  pub fn from_slice<C>(ctx: &mut C, slice: &[T]) -> Buffer<T>
  where
    C: GraphicsContext,
  {
    let mut buffer: GLuint = 0;
    let len = slice.len();
    let bytes = mem::size_of::<T>() * len;

    unsafe {
      gl::GenBuffers(1, &mut buffer);
      ctx.state().borrow_mut().bind_array_buffer(buffer);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        bytes as isize,
        slice.as_ptr() as *const c_void,
        gl::STREAM_DRAW,
      );
    }

    Buffer {
      raw: RawBuffer {
        handle: buffer,
        bytes,
        len,
        state: ctx.state().clone(),
      },
      _t: PhantomData,
    }
  }

  /// Retrieve an element from the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn at(&self, i: usize) -> Option<T>
  where
    T: Copy,
  {
    if i >= self.len {
      return None;
    }

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      let x = *ptr.offset(i as isize);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      Some(x)
    }
  }

  /// Retrieve the whole content of the `Buffer`.
  pub fn whole(&self) -> Vec<T>
  where
    T: Copy,
  {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *mut T;

      let values = Vec::from_raw_parts(ptr, self.len, self.len);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      values
    }
  }

  /// Set a value at a given index in the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn set(&mut self, i: usize, x: T) -> Result<(), BufferError>
  where
    T: Copy,
  {
    if i >= self.len {
      return Err(BufferError::Overflow(i, self.len));
    }

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;

      *ptr.offset(i as isize) = x;

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }

    Ok(())
  }

  /// Write a whole slice into a buffer.
  ///
  /// If the slice you pass in has less items than the length of the buffer, you’ll get a
  /// `BufferError::TooFewValues` error. If it has more, you’ll get `BufferError::TooManyValues`.
  ///
  /// This function won’t write anything on any error.
  pub fn write_whole(&self, values: &[T]) -> Result<(), BufferError> {
    let len = values.len();
    let in_bytes = len * mem::size_of::<T>();

    // generate warning and recompute the proper number of bytes to copy
    let real_bytes = match in_bytes.cmp(&self.bytes) {
      Ordering::Less => return Err(BufferError::TooFewValues(len, self.len)),
      Ordering::Greater => return Err(BufferError::TooManyValues(len, self.len)),
      _ => in_bytes,
    };

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);

      ptr::copy_nonoverlapping(values.as_ptr() as *const c_void, ptr, real_bytes);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }

    Ok(())
  }

  /// Fill the `Buffer` with a single value.
  pub fn clear(&self, x: T) -> Result<(), BufferError>
  where
    T: Copy,
  {
    self.write_whole(&vec![x; self.len])
  }

  /// Fill the whole buffer with an array.
  pub fn fill(&self, values: &[T]) -> Result<(), BufferError> {
    self.write_whole(values)
  }

  /// Convert a buffer to its raw representation.
  ///
  /// Becareful: once you have called this function, it is not possible to go back to a `Buffer<_>`.
  pub fn to_raw(self) -> RawBuffer {
    let raw = RawBuffer {
      handle: self.raw.handle,
      bytes: self.raw.bytes,
      len: self.raw.len,
      state: self.raw.state.clone(),
    };

    // forget self so that we don’t call drop on it after the function has returned
    mem::forget(self);
    raw
  }

  /// Obtain an immutable slice view into the buffer.
  pub fn as_slice(&self) -> Result<BufferSlice<T>, BufferError> {
    self.raw.as_slice()
  }

  /// Obtain a mutable slice view into the buffer.
  pub fn as_slice_mut(&mut self) -> Result<BufferSliceMut<T>, BufferError> {
    self.raw.as_slice_mut()
  }
}

impl<T> Deref for Buffer<T> {
  type Target = RawBuffer;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

impl<T> DerefMut for Buffer<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.raw
  }
}

/// Raw buffer. Any buffer can be converted to that type. However, keep in mind that even though
/// type erasure is safe, creating a buffer from a raw buffer is not.
pub struct RawBuffer {
  handle: GLuint,
  bytes: usize,
  len: usize,
  state: Rc<RefCell<GraphicsState>>,
}

impl RawBuffer {
  /// Obtain an immutable slice view into the buffer.
  pub fn as_slice<T>(&self) -> Result<BufferSlice<T>, BufferError> {
    unsafe {
      self.state.borrow_mut().bind_array_buffer(self.handle);

      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      if ptr.is_null() {
        return Err(BufferError::MapFailed);
      }

      Ok(BufferSlice { raw: self, ptr })
    }
  }

  /// Obtain a mutable slice view into the buffer.
  pub fn as_slice_mut<T>(&mut self) -> Result<BufferSliceMut<T>, BufferError> {
    unsafe {
      self.state.borrow_mut().bind_array_buffer(self.handle);

      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T;

      if ptr.is_null() {
        return Err(BufferError::MapFailed);
      }

      Ok(BufferSliceMut { raw: self, ptr })
    }
  }

  // Get the underlying GPU handle.
  pub fn handle(&self) -> GLuint {
    self.handle
  }

  /// Get the length of the buffer.
  #[inline(always)]
  pub fn len(&self) -> usize {
    self.len
  }
}

impl Drop for RawBuffer {
  fn drop(&mut self) {
    unsafe { gl::DeleteBuffers(1, &self.handle) }
  }
}

impl<T> From<Buffer<T>> for RawBuffer {
  fn from(buffer: Buffer<T>) -> Self {
    buffer.to_raw()
  }
}

/// A buffer slice mapped into GPU memory.
pub struct BufferSlice<'a, T>
where
  T: 'a,
{
  // Borrowed raw buffer.
  raw: &'a RawBuffer,
  // Raw pointer into the GPU memory.
  ptr: *const T,
}

impl<'a, T> Drop for BufferSlice<'a, T>
where
  T: 'a,
{
  fn drop(&mut self) {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.raw.handle);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, T> Deref for BufferSlice<'a, T>
where
  T: 'a,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.raw.len) }
  }
}

impl<'a, 'b, T> IntoIterator for &'b BufferSlice<'a, T>
where
  T: 'a,
{
  type IntoIter = slice::Iter<'b, T>;
  type Item = &'b T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

/// A buffer mutable slice into GPU memory.
pub struct BufferSliceMut<'a, T>
where
  T: 'a,
{
  // Borrowed buffer.
  raw: &'a RawBuffer,
  // Raw pointer into the GPU memory.
  ptr: *mut T,
}

impl<'a, T> Drop for BufferSliceMut<'a, T>
where
  T: 'a,
{
  fn drop(&mut self) {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.raw.handle);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, 'b, T> IntoIterator for &'b BufferSliceMut<'a, T>
where
  T: 'a,
{
  type IntoIter = slice::Iter<'b, T>;
  type Item = &'b T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

impl<'a, 'b, T> IntoIterator for &'b mut BufferSliceMut<'a, T>
where
  T: 'a,
{
  type IntoIter = slice::IterMut<'b, T>;
  type Item = &'b mut T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref_mut().into_iter()
  }
}

impl<'a, T> Deref for BufferSliceMut<'a, T>
where
  T: 'a,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.raw.len) }
  }
}

impl<'a, T> DerefMut for BufferSliceMut<'a, T>
where
  T: 'a,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.raw.len) }
  }
}

/// Typeclass of types that can be used inside a uniform block. You have to be extra careful when
/// using uniform blocks and ensure you respect the OpenGL *std140* alignment / size rules. This
/// will be fixed in a future release.
pub unsafe trait UniformBlock {}

unsafe impl UniformBlock for u8 {}
unsafe impl UniformBlock for u16 {}
unsafe impl UniformBlock for u32 {}

unsafe impl UniformBlock for i8 {}
unsafe impl UniformBlock for i16 {}
unsafe impl UniformBlock for i32 {}

unsafe impl UniformBlock for f32 {}
unsafe impl UniformBlock for f64 {}

unsafe impl UniformBlock for bool {}

unsafe impl UniformBlock for M22 {}
unsafe impl UniformBlock for M33 {}
unsafe impl UniformBlock for M44 {}

unsafe impl UniformBlock for [u8; 2] {}
unsafe impl UniformBlock for [u16; 2] {}
unsafe impl UniformBlock for [u32; 2] {}

unsafe impl UniformBlock for [i8; 2] {}
unsafe impl UniformBlock for [i16; 2] {}
unsafe impl UniformBlock for [i32; 2] {}

unsafe impl UniformBlock for [f32; 2] {}
unsafe impl UniformBlock for [f64; 2] {}

unsafe impl UniformBlock for [bool; 2] {}

unsafe impl UniformBlock for [u8; 3] {}
unsafe impl UniformBlock for [u16; 3] {}
unsafe impl UniformBlock for [u32; 3] {}

unsafe impl UniformBlock for [i8; 3] {}
unsafe impl UniformBlock for [i16; 3] {}
unsafe impl UniformBlock for [i32; 3] {}

unsafe impl UniformBlock for [f32; 3] {}
unsafe impl UniformBlock for [f64; 3] {}

unsafe impl UniformBlock for [bool; 3] {}

unsafe impl UniformBlock for [u8; 4] {}
unsafe impl UniformBlock for [u16; 4] {}
unsafe impl UniformBlock for [u32; 4] {}

unsafe impl UniformBlock for [i8; 4] {}
unsafe impl UniformBlock for [i16; 4] {}
unsafe impl UniformBlock for [i32; 4] {}

unsafe impl UniformBlock for [f32; 4] {}
unsafe impl UniformBlock for [f64; 4] {}

unsafe impl UniformBlock for [bool; 4] {}

unsafe impl<T> UniformBlock for [T] where T: UniformBlock {}

macro_rules! impl_uniform_block_tuple {
  ($( $t:ident ),*) => {
    unsafe impl<$($t),*> UniformBlock for ($($t),*) where $($t: UniformBlock),* {}
  }
}

impl_uniform_block_tuple!(A, B);
impl_uniform_block_tuple!(A, B, C);
impl_uniform_block_tuple!(A, B, C, D);
impl_uniform_block_tuple!(A, B, C, D, E);
impl_uniform_block_tuple!(A, B, C, D, E, F);
impl_uniform_block_tuple!(A, B, C, D, E, F, G);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H, I);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H, I, J);
