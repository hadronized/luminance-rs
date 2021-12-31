# Changelog

This document is the changelog of [luminance-sdl2](https://crates.io/crates/luminance-sdl2).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.5

> Dec 31, 2021

- Support `luminance-0.46`.

# 0.4

> Nov 26, 2021

- Add support for `luminance-0.45`.
- Bump support of `sdl2-0.35.1`.

# 0.3

> Jun 28, 2021

- Add support for `luminance-0.44`.
- Add support for `luminance-gl-0.17`.

# 0.2.2

> Feb 14, 2021

- Add `GL33Surface::window_mut`.

# 0.2.1

> Oct 28, 2020

- Support of `luminance-0.43`.
- Support of `luminance-gl-0.16`.

# 0.2

> Aug 30th, 2020

- Support of `luminance-0.42`.
- Support of `luminance-gl-0.15`.

# 0.1.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.1

> Wed Jul, 15th 2020

- Initial revision.
