# Changelog

This document is the changelog of [luminance-glutin](https://crates.io/crates/luminance-glutin).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.13

> Nov 26, 2021

- Support of `luminance-0.45`.

# 0.12

> Jun 28, 2021

- Support of `luminance-0.44`.
- Support of `glutin-0.27`.

# 0.11.3

> Feb 16, 2021

- Support of `glutin-0.26`.

# 0.11.2

> Oct 28, 2020

- Support of `luminance-0.43`.
- Support of `luminance-gl-0.16`.

# 0.11.1

> Oct 24th, 2020

- Support of `glutin-0.25`.

# 0.11

> Aug 30th, 2020

- Support of `luminance-0.42`.
- Support of `luminance-gl-0.15`.

# 0.10.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.10

> Wed Jul, 15th 2020

- Support `glutin-0.24`.
- Implement `std::error::Error` for various types of the crate.
- Add helper methods to create error types requiring owned data via a better API.

# 0.9

> Sun Feb, 23rd 2020

- Support `glutin-0.23`.

# 0.8.2

> Wed Jan, 8th 2020

- Add `GlutinSurface::from_builders`. That function can be used to create a new window and OpenGL
  context by explicitly building those objects in closures.

# 0.8.1

> Mon Jan, 6th 2020

- Add `Display` implementation for `GlutinError`.

# 0.8

> Mon Jan, 6th 2020

## Breaking changes

- Rework the interface to make it easier for people to have access to all the underlying `glutin`
  types.
- The `luminance-windowing` interface is now just use as convenience to create a windowed context.
  The `Surface` trait is not implemented anymore as it’s subject to be deprecated very soon.

# 0.7

> Sat Jan, 4th 2020

- Support of `luminance-0.38`.
- Re-export `glutin::MouseButton`.

# 0.6.1

> Tue Nov, 5th 2017

- Expose more `glutin` symbols on the public interface.

# 0.6

> Sun Sep, 29th 2019

- Support of `luminance-0.37`.

# 0.5

> Fri Sep, 20th 2019

- `luminance-0.36` support.

# 0.4

> Thur Sep, 12th 2019

- Fix SemVer issues with ranges and duplicated dependencies.

# 0.3.1

> Thur Sep, 12th 2019

- Support of `luminance-0.35`.

# 0.3

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
