//! A collection of utilities used to perform conversion between immutable slices and JavaScript’s
//! various array types.

/// Unsafe coercion to a `js_sys::Object` for immutable slices.
///
/// This trait provides the [`into_array_buffer`] method, which is an unsafe operation, as
/// the `view()` method, defined on the various arrays in the `js-sys` crate, requires that the
/// underlying memory not be moved until the array is dropped.
///
/// [`into_array_buffer`]: crate::webgl2::array_buffer::IntoArrayBuffer::into_array_buffer
pub trait IntoArrayBuffer: Sized {
  /// Convert the input slice into a JavaScript object.
  ///
  /// # Safety
  ///
  /// The returned `Object` must not outlive the input slice, which memory must not be moved either.
  unsafe fn into_array_buffer(texels: &[Self]) -> js_sys::Object;
}

macro_rules! impl_IntoArrayBuffer {
  ($t:ty, $buffer:ty) => {
    impl IntoArrayBuffer for $t {
      unsafe fn into_array_buffer(texels: &[Self]) -> js_sys::Object {
        <$buffer>::view(texels).into()
      }
    }

    impl_tuple_IntoArrayBuffer!($t, ($t, $t), 2, $buffer);
    impl_tuple_IntoArrayBuffer!($t, ($t, $t, $t), 3, $buffer);
    impl_tuple_IntoArrayBuffer!($t, ($t, $t, $t, $t), 4, $buffer);
  };
}

macro_rules! impl_tuple_IntoArrayBuffer {
  ($t:ty, $tuple:ty, $n:literal, $buffer:ty) => {
    // statically assert that [T; 3] has the same size as (T, T, T)
    // this checks that the from_raw_parts cast has the correct value for $n and $tuple
    const _: fn() = || {
      let _ = std::mem::transmute::<[$t; $n], $tuple>;
    };

    impl IntoArrayBuffer for $tuple {
      unsafe fn into_array_buffer(texels: &[Self]) -> js_sys::Object {
        let slice: &[$t] =
          std::slice::from_raw_parts(texels.as_ptr() as *const $t, texels.len() * $n);

        <$buffer>::view(slice).into()
      }
    }
  };
}

impl_IntoArrayBuffer!(u8, js_sys::Uint8Array);
impl_IntoArrayBuffer!(i8, js_sys::Int8Array);
impl_IntoArrayBuffer!(u16, js_sys::Uint16Array);
impl_IntoArrayBuffer!(i16, js_sys::Int16Array);
impl_IntoArrayBuffer!(u32, js_sys::Uint32Array);
impl_IntoArrayBuffer!(i32, js_sys::Int32Array);

impl_IntoArrayBuffer!(f32, js_sys::Float32Array);
impl_IntoArrayBuffer!(f64, js_sys::Float64Array);
