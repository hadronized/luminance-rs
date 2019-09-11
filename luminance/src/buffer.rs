//! Static GPU typed arrays.
//!
//! A GPU buffer is a typed continuous region of data. It has a size and can hold several elements.
//!
//! Once the buffer is created, you can perform several operations on it:
//!
//! - Writing to it.
//! - Reading from it.
//! - Passing it around as uniforms.
//! - Etc.
//!
//! Buffers are created with the [`Buffer::new`] associated function, which is `unsafe`. You pass in
//! the number of elements you want in the buffer along with the [`GraphicsContext`] to create the
//! buffer in.
//!
//! ```ignore
//! let buffer: Buffer<f32> = Buffer::new(&mut ctx, 5);
//! ```
//!
//! Another important point is the fact that creating a buffer with [`Buffer::new`] allocates the
//! array on the GPU but leaves it _uninitialized_. You will have to fill its memory by hand. Or
//! you can use the [`Buffer::from_slice`] method, which both allocates and initializes:
//!
//! ```ignore
//! let buffer = Buffer::from_slice(&mut ctx, [1, 2, 3]);
//! ```
//!
//! If you would like to allocate a buffer and initialize it with the same value everywhere, you
//! can use [`Buffer::repeat`].
//!
//! ```ignore
//! let buffer = Buffer::repeat(&mut ctx, 3, 0); // same as Buffer::from_slice(&mut ctx, [0, 0, 0])
//! ```
//!
//! # Writing to a buffer
//!
//! [`Buffer`]s support several write methods. The simple one is _clearing_. That is, replacing the
//! whole content of the buffer with a single value. Use the [`Buffer::clear`] function to do so.
//!
//! ```ignore
//! buffer.clear(0.);
//! ```
//!
//! If you want to clear the buffer by providing a value for each elements, you want _filling_
//! instead. Use the [`Buffer::fill`] function:
//!
//! ```ignore
//! buffer.fill([1, 2, 3]);
//! ```
//!
//! You want to change a value at a given index? Easy, you can use the [`Buffer::set`] function:
//!
//! ```ignore
//! buffer.set(2, 42);
//! ```
//!
//! # Reading from the buffer
//!
//! You can either retrieve the _whole_ content of the [`Buffer`] or _get_ a value with an index.
//!
//! ```ignore
//! // get the whole content
//! let all_elems = buffer.whole();
//! assert_eq!(all_elems, vec![1, 2, 42]);
//!
//! // get the element at index 2
//! assert_eq!(buffer.at(2), Some(42));
//! ```
//!
//! # Uniform buffer
//!
//! It’s possible to use buffers as *uniform buffers*. That is, buffers that will be in bound at
//! rendering time and which content will be available for a shader to read (no write).
//!
//! In order to use your buffers in a uniform context, the inner type has to implement
//! [`UniformBlock`]. Keep in mind alignment must be respected and is a bit peculiar. TODO: explain
//! std140 here.
//!
//! [`Buffer`]: crate::buffer::Buffer
//! [`Buffer::new`]: crate::buffer::Buffer::new
//! [`Buffer::from_slice`]: crate::buffer::Buffer::from_slice
//! [`Buffer::repeat`]: crate::buffer::Buffer::repeat
//! [`Buffer::clear`]: crate::buffer::Buffer::clear
//! [`Buffer::fill`]: crate::buffer::Buffer::fill
//! [`Buffer::set`]: crate::buffer::Buffer::set
//! [`GraphicsContext`]: crate::context::GraphicsContext
//! [`UniformBlock`]: crate::buffer::UniformBlock

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
use crate::state::{Bind, GraphicsState};

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

      BufferError::TooFewValues(nb, size) => {
        write!(
          f,
          "too few values passed to the buffer (nb = {}, size = {})",
          nb, size
        )
      }

      BufferError::TooManyValues(nb, size) => {
        write!(
          f,
          "too many values passed to the buffer (nb = {}, size = {})",
          nb, size
        )
      }

      BufferError::MapFailed => write!(f, "buffer mapping failed"),
    }
  }
}

/// A [`Buffer`] is a GPU region you can picture as an array.
///
/// You’re strongly advised to use either [`Buffer::from_slice`] or [`Buffer::repeat`] to create a
/// [`Buffer`]. The [`Buffer::new`] should only be used if you know what you’re doing.
pub struct Buffer<T> {
  raw: RawBuffer,
  _t: PhantomData<T>,
}

impl<T> Buffer<T> {
  /// Create a new [`Buffer`] with a given number of elements.
  ///
  /// That function leaves the buffer _uninitialized_, which is `unsafe`. If you prefer not to use
  /// any `unsafe` function, feel free to use [`Buffer::from_slice`] or [`Buffer::repeat`] instead.
  pub unsafe fn new<C>(ctx: &mut C, len: usize) -> Buffer<T> where C: GraphicsContext {
    let mut buffer: GLuint = 0;
    let bytes = mem::size_of::<T>() * len;

    // generate a buffer and force binding the handle; this prevent side-effects from previous bound
    // resources to prevent binding the buffer
    gl::GenBuffers(1, &mut buffer);
    ctx.state().borrow_mut().bind_array_buffer(buffer, Bind::Forced);
    gl::BufferData(gl::ARRAY_BUFFER, bytes as isize, ptr::null(), gl::STREAM_DRAW);

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
  pub fn from_slice<C, S>(
    ctx: &mut C,
    slice: S
  ) -> Buffer<T>
  where C: GraphicsContext,
        S: AsRef<[T]> {
    let mut buffer: GLuint = 0;
    let slice = slice.as_ref();
    let len = slice.len();
    let bytes = mem::size_of::<T>() * len;

    unsafe {
      gl::GenBuffers(1, &mut buffer);
      ctx.state().borrow_mut().bind_array_buffer(buffer, Bind::Cached);
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

  /// Create a new [`Buffer`] with a given number of elements and ininitialize all the elements to
  /// the same value.
  pub fn repeat<C>(ctx: &mut C, len: usize, value: T) -> Self where C: GraphicsContext, T: Copy {
    let mut buf = unsafe { Self::new(ctx, len) };
    buf.clear(value).unwrap();
    buf
  }

  /// Retrieve an element from the [`Buffer`].
  ///
  /// This version checks boundaries.
  pub fn at(&self, i: usize) -> Option<T> where T: Copy {
    if i >= self.len {
      return None;
    }

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      let x = *ptr.offset(i as isize);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      Some(x)
    }
  }

  /// Retrieve the whole content of the [`Buffer`].
  pub fn whole(&self) -> Vec<T> where T: Copy {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *mut T;

      let values = Vec::from_raw_parts(ptr, self.len, self.len);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      values
    }
  }

  /// Set a value at a given index in the [`Buffer`].
  ///
  /// This version checks boundaries.
  pub fn set(&mut self, i: usize, x: T) -> Result<(), BufferError> where T: Copy {
    if i >= self.len {
      return Err(BufferError::Overflow(i, self.len));
    }

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;

      *ptr.offset(i as isize) = x;

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }

    Ok(())
  }

  /// Write a whole slice into a buffer.
  ///
  /// If the slice you pass in has less items than the length of the buffer, you’ll get a
  /// [`BufferError::TooFewValues`] error. If it has more, you’ll get
  /// [`BufferError::TooManyValues`].
  ///
  /// This function won’t write anything on any error.
  pub fn write_whole(&mut self, values: &[T]) -> Result<(), BufferError> {
    let len = values.len();
    let in_bytes = len * mem::size_of::<T>();

    // generate warning and recompute the proper number of bytes to copy
    let real_bytes = match in_bytes.cmp(&self.bytes) {
      Ordering::Less => return Err(BufferError::TooFewValues(len, self.len)),
      Ordering::Greater => return Err(BufferError::TooManyValues(len, self.len)),
      _ => in_bytes,
    };

    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);

      ptr::copy_nonoverlapping(values.as_ptr() as *const c_void, ptr, real_bytes);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }

    Ok(())
  }

  /// Fill the [`Buffer`] with a single value.
  pub fn clear(&mut self, x: T) -> Result<(), BufferError> where T: Copy {
    self.write_whole(&vec![x; self.len])
  }

  /// Fill the whole buffer with an array.
  pub fn fill<V>(&mut self, values: V) -> Result<(), BufferError> where V: AsRef<[T]> {
    self.write_whole(values.as_ref())
  }

  /// Convert a buffer to its raw representation.
  ///
  /// Becareful: once you have called this function, it is not possible to go back to a [`Buffer`].
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
  pub fn as_slice(&mut self) -> Result<BufferSlice<T>, BufferError> {
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
  pub(crate) fn as_slice<T>(&mut self) -> Result<BufferSlice<T>, BufferError> {
    unsafe {
      self.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);

      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      if ptr.is_null() {
        return Err(BufferError::MapFailed);
      }

      Ok(BufferSlice { raw: self, ptr })
    }
  }

  /// Obtain a mutable slice view into the buffer.
  pub(crate) fn as_slice_mut<T>(&mut self) -> Result<BufferSliceMut<T>, BufferError> {
    unsafe {
      self.state.borrow_mut().bind_array_buffer(self.handle, Bind::Cached);

      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T;

      if ptr.is_null() {
        return Err(BufferError::MapFailed);
      }

      Ok(BufferSliceMut { raw: self, ptr })
    }
  }

  // Get the underlying GPU handle.
  #[inline(always)]
  pub(crate) fn handle(&self) -> GLuint {
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
    unsafe {
      self.state.borrow_mut().unbind_buffer(self.handle);
      gl::DeleteBuffers(1, &self.handle);
    }
  }
}

impl<T> From<Buffer<T>> for RawBuffer {
  fn from(buffer: Buffer<T>) -> Self {
    buffer.to_raw()
  }
}

/// A buffer slice mapped into GPU memory.
///
/// Such a slice allows you to read the data contained in a buffer directly via a Rust slice.
/// While a buffer is mapped, the buffer is _sealed_ and no one can use it. You need to drop the
/// [`BufferSlice`] in order to unlock the buffer.
pub struct BufferSlice<'a, T> where T: 'a {
  // Borrowed raw buffer.
  raw: &'a RawBuffer,
  // Raw pointer into the GPU memory.
  ptr: *const T,
}

impl<'a, T> Drop for BufferSlice<'a, T> where T: 'a {
  fn drop(&mut self) {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.raw.handle, Bind::Cached);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, T> Deref for BufferSlice<'a, T> where T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.raw.len) }
  }
}

impl<'a, 'b, T> IntoIterator for &'b BufferSlice<'a, T> where T: 'a {
  type IntoIter = slice::Iter<'b, T>;
  type Item = &'b T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

/// A mutable buffer slice mapped into GPU memory.
///
/// Such a slice allows you to read the data contained in a buffer directly via a Rust slice.
/// While a buffer is mapped, the buffer is _sealed_ and no one can use it. You need to drop the
/// [`BufferSliceMut`] in order to unlock the buffer.
pub struct BufferSliceMut<'a, T> where T: 'a {
  // Borrowed buffer.
  raw: &'a RawBuffer,
  // Raw pointer into the GPU memory.
  ptr: *mut T,
}

impl<'a, T> Drop for BufferSliceMut<'a, T> where T: 'a {
  fn drop(&mut self) {
    unsafe {
      self.raw.state.borrow_mut().bind_array_buffer(self.raw.handle, Bind::Cached);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, 'b, T> IntoIterator for &'b BufferSliceMut<'a, T> where T: 'a {
  type IntoIter = slice::Iter<'b, T>;
  type Item = &'b T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

impl<'a, 'b, T> IntoIterator for &'b mut BufferSliceMut<'a, T> where T: 'a {
  type IntoIter = slice::IterMut<'b, T>;
  type Item = &'b mut T;

  fn into_iter(self) -> Self::IntoIter {
    self.deref_mut().into_iter()
  }
}

impl<'a, T> Deref for BufferSliceMut<'a, T> where T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.raw.len) }
  }
}

impl<'a, T> DerefMut for BufferSliceMut<'a, T> where T: 'a {
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
