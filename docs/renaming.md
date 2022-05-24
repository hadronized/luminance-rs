# Renaming

This document is a short document describing the current features and whether we should rename them.

<!-- vim-markdown-toc GFM -->

* [Context](#context)
* [Renaming](#renaming)
  * [List of features / vocabulary](#list-of-features--vocabulary)

<!-- vim-markdown-toc -->

# Context

Features in `luminance` has been highly shaped by technology (OpenGL / Vulkan). However, the opinionated API starts to
get augmented with concepts that don’t exist in low-level APIs, such as `ShaderData`, color slots, etc. It might be
necessary to rename the concepts / features to move away from the low-level API and get an API that easier to think
about, and more abstract.

# Renaming

## List of features / vocabulary

- Framebuffers.
- Color slots.
- Depth slots.
- Shader programs.
- Shader stages.
- Shader data.
- Uniforms.
- Tessellations.
- Vertex.
- Vertex index.
- Instance.
- Textures.
- Samplers.
- Mipmaps.
- Pipelines.
- Shading gates.
- Render gates.
- Tessellation gates.
- Binding points.
