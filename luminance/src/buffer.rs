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

use crate::context::GraphicsContext;

/// Buffer support in backends.
pub trait BufferBackend<'sliced, C, T> where C: GraphicsContext {
  type Slice: 'sliced;

  type SliceMut: 'sliced;

  type Err;

  /// Create a new [`Buffer`] with a given number of elements.
  ///
  /// That function leaves the buffer _uninitialized_, which is `unsafe`. If you prefer not to use
  /// any `unsafe` function, feel free to use [`Buffer::from_slice`] or [`Buffer::repeat`] instead.
  unsafe fn new(ctx: &mut C, len: usize) -> Self;

  /// Create a buffer out of a slice.
  fn from_slice<S>(
    ctx: &mut C,
    slice: S
  ) -> Self
  where S: AsRef<[T]>;

  /// Create a new [`Buffer`] with a given number of elements and ininitialize all the elements to
  /// the same value.
  fn repeat(ctx: &mut C, len: usize, value: T) -> Self where T: Copy;

  /// Retrieve an element from the [`Buffer`].
  ///
  /// This version checks boundaries.
  fn at(&self, i: usize) -> Option<T> where T: Copy;

  /// Retrieve the whole content of the [`Buffer`].
  fn whole(&self) -> Vec<T> where T: Copy;

  /// Set a value at a given index in the [`Buffer`].
  ///
  /// This version checks boundaries.
  fn set(&mut self, i: usize, x: T) -> Result<(), Self::Err> where T: Copy;

  /// Write a whole slice into a buffer.
  ///
  /// If the slice you pass in has less items than the length of the buffer, you’ll get a
  /// [`BufferError::TooFewValues`] error. If it has more, you’ll get
  /// [`BufferError::TooManyValues`].
  ///
  /// This function won’t write anything on any error.
  fn write_whole(&mut self, values: &[T]) -> Result<(), Self::Err>;

  /// Fill the [`Buffer`] with a single value.
  fn clear(&mut self, x: T) -> Result<(), Self::Err> where T: Copy;

  /// Fill the whole buffer with an array.
  fn fill<V>(&mut self, values: V) -> Result<(), Self::Err> where V: AsRef<[T]>;

  /// Obtain an immutable slice view into the buffer.
  fn as_slice(&'sliced mut self) -> Result<Self::Slice, Self::Err>;

  /// Obtain a mutable slice view into the buffer.
  fn as_slice_mut(&'sliced mut self) -> Result<Self::SliceMut, Self::Err>;
}
