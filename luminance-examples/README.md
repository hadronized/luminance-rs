# luminance examples

This crate holds several examples you can use to learn how to use luminance.

Each example comes in with a few explanations and how to use them at the top of the `.rs` file.
A [common](./src/common/mod.rs) module is present so that the code can be shared and referenced from
all examples.

Also, some examples might be target-dependent (such as web examples). You will need specific
instructions to compile them, which are located at each example’s entry below.

If you think a specific feature is missing, feel free to open a PR and add new examples! The more
we have, the better! Also, keep in mind that this example repository is _not the proper way to
learn [luminance] as a whole!_ If you would like to learn from scratch, it is highly recommended to
have a look at [the book] first.

**Have fun!**

<!-- vim-markdown-toc GFM -->

* [01 – Hello World](#01--hello-world)
* [02 – Render State](#02--render-state)
* [03 – Sliced Tessellation](#03--sliced-tessellation)
* [04 – Shader Uniforms](#04--shader-uniforms)
* [05 – Attributeless Render](#05--attributeless-render)
* [06 – Texture](#06--texture)
* [07 – Offscreen](#07--offscreen)
* [08 – Shader Uniforms Adapt](#08--shader-uniforms-adapt)
* [09 – Dynamic Uniform Interface](#09--dynamic-uniform-interface)
* [10 – Vertex Instancing](#10--vertex-instancing)
* [11 – Query texture texels](#11--query-texture-texels)
* [12 – Displacement Map](#12--displacement-map)
* [13 – Interactive triangle](#13--interactive-triangle)
* [14 – Skybox and environment mapping](#14--skybox-and-environment-mapping)

<!-- vim-markdown-toc -->

## [01 – Hello World](./src/hello-world.rs)

Learn how to draw two colored triangles by using vertex colors (comes in *direct* and *indexed*
geometry versions). This first example is really important as it shows off lots of features that
are necessary to wrap your fingers around, especially _vertex semantics_, _buffer formats_, _derive
procedural macros_, _graphics pipelines_, etc. etc.

![](../docs/imgs/01-screenshot.png)

> A version using [glutin] is available [here](./src/hello-world-glutin.rs).

## [02 – Render State](./src/render-state.rs)

Learn how to change the render state to tweak the way primitives are rendered or how fragment
blending happens.

![](../docs/imgs/02-screenshot.png)
![](../docs/imgs/02-screenshot-alt.png)
![](../docs/imgs/02-screenshot-alt2.png)

## [03 – Sliced Tessellation](./src/sliced-tess.rs)

Learn how to slice a single GPU geometry to dynamically select contiguous regions of it to render!

![](../docs/imgs/03-screenshot.png)
![](../docs/imgs/03-screenshot-alt.png)
![](../docs/imgs/03-screenshot-alt2.png)

## [04 – Shader Uniforms](./src/shader-uniforms.rs)

Send colors and position information to the GPU to add interaction with a simple yet colorful
triangle!

![](../docs/imgs/04-screenshot.png)
![](../docs/imgs/04-screenshot-alt.png)

## [05 – Attributeless Render](./src/attributeless.rs)

Render a triangle without sending any vertex data to the GPU!

![](../docs/imgs/05-screenshot.png)

## [06 – Texture](./src/texture.rs)

Learn how to use a loaded image as a luminance texture on the GPU!

## [07 – Offscreen](./src/offscreen.rs)

Get introduced to *offscreen rendering*, a powerful technique used to render frames into memory
without directly displaying them on your screen. Offscreen framebuffers can be seen as a
generalization of your screen.

## [08 – Shader Uniforms Adapt](./src/shader-uniforms-adapt.rs)

Learn how to change the type of a shader program’s uniform interface on the fly.

![](../docs/imgs/08-screenshot.png)
![](../docs/imgs/08-screenshot-alt.png)

## [09 – Dynamic Uniform Interface](./src/dynamic-uniform-interface.rs)

Implement a dynamic lookup for shader program the easy way by using program interfaces to query
uniforms on the fly!

## [10 – Vertex Instancing](./src/vertex-instancing.rs)

Learn how to implement a famous technique known as _vertex instancing_, allowing to render multiple
instances of the same object, each instances having their own properties.

![](../docs/imgs/10-screenshot.png)

## [11 – Query texture texels](./src/query-texture-texels.rs)

Query texture texels from a framebuffer and output them as a rendered image on your file system.

## [12 – Displacement Map](./src/displacement-map.rs)

Use a grayscale texture to implement a _displacement map_ effect on a color map.

![](../docs/imgs/displacement_map.gif)

## [13 – Interactive triangle](./src/interactive-triangle.rs)

Learn how to move the triangle from the hello world with your mouse or cursor!

## [14 – Skybox and environment mapping](./src/skybox.rs)

Load a skybox from a file, display it and render a cube reflecting the sky!

[luminance]: https://crates.io/crates/luminance
[glutin]: https://crates.io/crates/glutin
[the book]: https://rust-tutorials.github.io/learn-luminance
[wasm]: https://webassembly.org
[cargo-web]: https://crates.io/crates/cargo-web
