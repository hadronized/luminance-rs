# Changelog

This document is the changelog of [luminance-front](https://crates.io/crates/luminance-front).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.5

> Nov 24, 2021

- Update `Vertices`, `VerticesMut`, `Indices`, `IndicesMut`, `Instances` and `InstancesMut` to reflect the lifetime
  change that happened in `luminance`.
- Export the `tess::View` trait, which was missing from the public interface.
- Export the new `luminance::shader::types::*`.
- Fix architecture-based detection. The current process is that if the target architecture is not
  `wasm32-unknown-unknown`, we use `luminance-gl`. So we don’t depend on the CPU architecture anymore.

# 0.4

> Apr 25, 2021

- Support of `luminance-0.44`.
- Support of `luminance-gl-0.17`.
- Support of `luminance-webgl-0.4`.
- Re-exported `luminance::scissor`.

# 0.3.1

> Oct 28, 2020

- Support of `luminance-0.43`.
- Support of `luminance-gl-0.16`.
- Support of `luminance-webgl-0.3`.

# 0.3

> Aug 30th, 2020

- Support of `luminance-0.42`.

# 0.2.3

> Jul 28th, 2020

- Add the missing re-export `Render`.

# 0.2.2

> Jul 28th, 2020

- Add the missing re-export `UniformInterface`.

# 0.2.1

> Jul 27th, 2020

- Fix the default types of `Tess` and `TessBuilder` according to `luminance-0.41`. Those were
  missing while they shouldn’t.

# 0.2

> Jul 24th, 2020

- Re-export missing symbols, such as `RenderGate`.
- Fix the `Tessgate` -> `TessGate` symbol.
- Support of `luminance-0.41`.

# 0.1

> Wed Jul, 15th 2020

- Initial revision.
