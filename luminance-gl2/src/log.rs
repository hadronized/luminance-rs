#[macro_export]
macro_rules! trace {
  ($($a:tt)*) => {
    #[cfg(feature = "log")]
    log::trace!($($a)*)
  }
}
