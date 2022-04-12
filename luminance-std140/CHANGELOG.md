# Changelog

This document is the changelog of [luminance-std140](https://crates.io/crates/luminance-std140).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

# 0.2.1

> Apr 12, 2022

- Support of `luminance-0.47`.

# 0.2

- ?

# 0.1

> Nov 26, 2021

- Initial revision.
