# Changelog

This document is the changelog of [luminance-derive](https://crates.io/crates/luminance-derive).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.8

> Nov 26, 2021

- Support of `luminance-0.45`.

# 0.7

> Apr 25, 2021

- Support of `luminance-0.44`.
- Add a check when a `Vertex` type has fields of the same type and raise a compile-time error if that’s the case.
- Remove the `dev-dependencies` entry on `luminance`. This is was needed for tests only, which were moved to `luminance`
  directly, and the cycle dependency was preventing from releasing major bumps of crates.

# 0.6.3

> Oct 28, 2020

- Support of `luminance-0.43`.

# 0.6.2

> Aug 30th, 2020

- Support of `luminance-0.42`.

# 0.6.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.6

> Wed Jul, 15th 2020

- The `UniformInterface` proc-macro got patched to adapt to the new backend architecture.
- Implement `std::error::Error` for various types of the crate.
- Add helper methods to create error types requiring owned data via a better API.

# 0.5.2

> Tue Jan, 7th 2020

- Add `Deref` and `DerefMut` implementors for semantics’ generated variant types. You can now
  access the underlying (wrapped) repr type.
- In the case of `Deref` and `DerefMut` not being enough, the underlying field can also be
  directly accessed (it’s now `pub`).

# 0.5.1

> Sat Jan, 4th 2020

- Support of `luminance-0.38`.

# 0.5

> Sun Sep, 29th 2019

- Support of `luminance-0.37`.

# 0.4

> Fri Sep, 20th 2019

## Major changes

- Add `new` methods for types annotated with `Vertex`. This is considered a breaking change as
  it would break your code if you already have a `new` method, which is very likely.

## Minor changes

- Add support for struct-tuple when deriving `Vertex`.

## Patch changes

- Empty `Semantics` types are forbidden and now reported correctly as errors.

# 0.3

> Thur Sep, 12th 2019

- Fix SemVer issues with ranges and duplicated dependencies.

# 0.2.2

> Thur Sep, 12th 2019

- Support of `luminance-0.35`.

# 0.2.1

> Wed Sep, 11th 2019

- Support of `luminance-0.34`.

# 0.2

> Fri Sep, 6th 2019

- Support of `luminance-0.33`.

# 0.1.1

> Tue Sep, 3rd 2019

- Support of `luminance-0.32`.

# 0.1

> Fri Aug, 23th 2019

- Initial revision.
