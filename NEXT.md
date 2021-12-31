# Next changes to be available

> This document lists all the last changes that occurred lately and that are about to be publicly available soon. These
> are items that must be formatted accordingly and ready to be moved to the [CHANGELOG](./CHANGELOG.md).

# `luminance`

- Add `TexelUpload::Reserve` to be able to reserve texels. The previous way of doing (by passing an empty slice `&[]`)
  was unsound and clashing with internal size checks.
- Remove the concept of optional mimaps (`Option<usize>`). Indeed, that encoding was error-prone and not a normal form
  (e.g. `None` vs. `Some(0)`).
- Methods `base_texels_with_mipmaps` and `base_texels_without_mipmaps` got merged into `base_texels`, using a `usize`
  that must be set to `0` for “no mipmap.”
- Rename the `base_texels` method into `get_base_texels` (for backends).

# `luminance-derive`

# `luminance-front`

# `luminance-gl`

- Adapt to `TexelUpload::Reserve`.

# `luminance-glfw`

- Dependency bump: `glfw-0.43`.

# `luminance-glutin`

- Dependency bump: `glutin-0.28`.

# `luminance-sdl2`

# `luminance-std140`

# `luminance-web-sys`

# `luminance-webgl`

- Adapt to `TexelUpload::Reserve`.
