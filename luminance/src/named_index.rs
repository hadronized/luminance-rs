/// Map string names to indexes.
pub trait NamedIndex<const NAME: &'static str> {
  /// Unique index for `NAME`.
  const INDEX: usize;
}

#[macro_export]
macro_rules! namespace {
  ($namespace:ident = { $first:literal, $($rest:tt)* }) => {
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct $namespace;

    impl $crate::named_index::NamedIndex<$first> for $namespace {
      const INDEX: usize = 0;
    }

    namespace!($namespace 1, $($rest)*);
  };

  ($namespace:ident $index:expr, $name:literal) => {
    impl $crate::named_index::NamedIndex<$name> for $namespace {
      const INDEX: usize = $index;
    }
  };

  ($namespace:ident $index:expr, $name:literal, $($rest:tt)*) => {
    impl $crate::named_index::NamedIndex<$name> for $namespace {
      const INDEX: usize = $index;
    }

    namespace!($namespace ($index + 1), $($rest)*);
  }
}
