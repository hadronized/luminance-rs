# Changelog

This document is the changelog of [luminance-web-sys](https://crates.io/crates/luminance-web-sys).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.5

> Dec 31, 2021

- Support of `luminance-0.46`.

# 0.4

> Nov 26, 2021

- Support of `luminance-0.45`.
- Remove useless dependency (`luminance-windowing`).

# 0.3

> Apr 25, 2021

- Support of `luminance-0.44`.
- Document `WebSysWebGL2Surface::from_canvas`.
- Support opnengi WebGL context with parameters.

# 0.2.3

> Apr 20, 2021

- Add `WebSysWebGL2Surface::from_canvas`.

# 0.2.2

> Oct 28th, 2020

- Remove some warnings.

# 0.2.1

> Oct 28, 2020

## Patch

- Support of `luminance-0.43`.
- Support of `luminance-webgl-0.3`.

## Breaking changes

- Remove the `WindowOpt` argument from `WebSysWebGL2Surface::new`. It was confusing people because most of its
  properties are held by the JavaScript object passed through wasm (typically, the canvas directly). If you were passing
  width and height via a `WindowOpt`, you can simply set those on the canvas JS-side directly.

# 0.2

> Aug 30th, 2020

- Support of `luminance-0.42`.

# 0.1.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.1

> Wed Jul, 15th 2020

- Initial revision.
