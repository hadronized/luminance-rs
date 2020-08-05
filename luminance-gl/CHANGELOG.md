# Changelog

This document is the changelog of [luminance-gl](https://crates.io/crates/luminance-gl).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolve dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.14.1+1](#01411)
* [0.14.1](#0141)
* [0.14](#014)
* [Pre 0.14](#pre-014)

<!-- vim-markdown-toc -->

# 0.14.1+1

> ?

- Add the `GL_ARB_gpu_shader_fp64` feature gate, allowing to use `f64`-like shader uniforms.
  Textures are not currently supported.

# 0.14.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.14

> Wed Jul 15th, 2020

- Replace mipmap creation’s square calls with bitwise left shifts to speed up computing the sizes
  of mipmaps.
- Implement `std::error::Error` for various types of the crate.
- It is now possible to _reset_ (i.e. _invalidate_) the OpenGL GPU state via an `unsafe` method.
  Few to almost no user should have a need for this — if you find yourself using that feature, then
  it’s either you’re doing something wrong, or something is missing to luminance, or finally you
  have to interop with a foreign system like imgui. That last case is the reason why that feature
  was developed. You should not need it.
- Texture unit state tracking has been enhanced to minimize the number of GPU texture units when a
  bind would occur. This small optimization might often mean that one to several textures will get
  bound once and no texture binding will occur unless a dynamic change occur that requires another
  texture unit.
- Fix a potential double-free when a `Program` doesn’t link.

# Pre 0.14

- The crate was available on https://crates.io with a different scope. If you were using it, please update to
  the latest [luminance](https://crates.io/crates/luminance) architecture.
