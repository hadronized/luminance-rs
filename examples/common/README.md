# Luminance examples

This crate holds several examples you can use to learn how to use luminance.

Each example comes in with a few explanations and how to use them at the top of the `.rs` file.
A [shared](./src/shared.rs) module is present so that the code can be shared and referenced from
all examples. Don’t forget to go visit that file to understand more about how things work.

This crate is also used to implement functional tests. By default, they are not compiled, you need
to set the `"funtest"` feature.

If you think a specific feature is missing, feel free to open a PR and add new examples! The more
we have, the better! Also, keep in mind that this example repository is _not the proper way to
learn [luminance] as a whole!_ If you would like to learn from scratch, it is highly recommended to
have a look at [the book] first.

**Have fun!**

* [Prologue: architecture](#prologue-architecture)
* [Hello World](#01--hello-world)
* [Hello World, more](#01a--polymorphic-hello-world)
* [Render State](#02--render-state)
* [Sliced Tessellation](#03--sliced-tessellation)
* [Shader Uniforms](#04--shader-uniforms)
* [Attributeless Render](#05--attributeless-render)
* [Texture](#06--texture)
* [Offscreen](#07--offscreen)
* [Dynamic Uniform Interface](#09--dynamic-uniform-interface)
* [Vertex Instancing](#10--vertex-instancing)
* [Query texture texels](#11--query-texture-texels)
* [Displacement Map](#12--displacement-map)
* [Interactive triangle](#13--interactive-triangle)
* [Skybox and environment mapping](#14--skybox-and-environment-mapping)
* [Texture resize](#15--texture-resize)
* [Query information](#16--query-information)
* [MRT (Multi Render Target)](#17--mrt-multi-render-target)
* [Uniform buffer](#18--uniform-buffer)

## Prologue: architecture

Because the examples use a _backend-agnostic_ approach, it is important to explain how they work. The general idea is
defined in [lib.rs](./src/lib.rs) file via several concepts:

- `Example`: most important trait, this interface defines what an example can actually do. The interface is
  purposefully general enough to allow the implementations  to execute them the way they want.
- `Features`: a type allowing examples to instruct executors about some features they require and that must be loaded,
  enabled or checked before running the example.
- `InputAction`: an abstraction of user interaction. Because examples don’t assume anything about the running
  executor, we cannot assume a _mouse_ or a _keyboard_ will be used. Those are devices that implement an interface
  yielding a stream of `InputAction`.
- `LoopFeedback`: when a frame has finished rendering or must abort, it returns to the executor a value of type
  `LoopFeedback`, allowing to know whether execution should be aborted, and if not, what is the next step of the
  example.
- `PlatformServices`: various services provided by the platform executor to the examples, such as fetching textures.

## [01 – Hello World](./src/hello_world.rs)

Learn how to draw two colored triangles by using vertex colors. This first example is really important as it shows off
lots of features that are necessary to wrap your fingers around, especially _namespaces, _vertex type declaration,
_derive procedural macros_, _graphics pipelines_, etc. etc. All in all, everything is easy to grasp, yet very important.

![](../../docs/imgs/01-screenshot.png)

## [Polymorphic Hello World](./src/hello_world_more.rs)

A variant of [01 – Hello World](#01--hello-world) that shows more options for the vertex entities. It will teach you how
to use interleaved, deinterleaved and indexed vertex entities (and mix of those!).

## [Render State](./src/render_state.rs)

Learn how to change the render state to tweak the way primitives are rendered or how fragment
blending happens.

![](../../docs/imgs/02-screenshot.png)
![](../../docs/imgs/02-screenshot-alt.png)
![](../../docs/imgs/02-screenshot-alt2.png)

## [Sliced Tessellation](./src/sliced_tess.rs)

Learn how to slice a single GPU geometry to dynamically select contiguous regions of it to render!

![](../../docs/imgs/03-screenshot.png)
![](../../docs/imgs/03-screenshot-alt.png)
![](../../docs/imgs/03-screenshot-alt2.png)

## [Shader Uniforms](./src/shader_uniforms.rs)

Send colors and position information to the GPU to add interaction with a simple yet colorful
triangle!

![](../../docs/imgs/04-screenshot.png)
![](../../docs/imgs/04-screenshot-alt.png)

## [Attributeless Render](./src/attributeless.rs)

Render a triangle without sending any vertex data to the GPU!

![](../../docs/imgs/05-screenshot.png)

## [Texture](./src/texture.rs)

Learn how to use a loaded image as a luminance texture on the GPU!

## [Offscreen](./src/offscreen.rs)

Get introduced to *offscreen rendering*, a powerful technique used to render frames into memory
without directly displaying them on your screen. Offscreen framebuffers can be seen as a
generalization of your screen.

## [Dynamic Uniform Interface](./src/dynamic_uniform_interface.rs)

Implement a dynamic lookup for shader program the easy way by using program interfaces to query
uniforms on the fly!

## [Vertex Instancing](./src/vertex_instancing.rs)

Learn how to implement a famous technique known as _vertex instancing_, allowing to render multiple
instances of the same object, each instances having their own properties.

![](../../docs/imgs/10-screenshot.png)

## [Query texture texels](./src/query_texture_texels.rs)

Query texture texels from a framebuffer and output them as a rendered image on your file system.

## [Displacement Map](./src/displacement_map.rs)

Use a grayscale texture to implement a _displacement map_ effect on a color map.

![](../../docs/imgs/displacement_map.gif)

## [Interactive triangle](./src/interactive_triangle.rs)

Learn how to move the triangle from the hello world with your mouse or cursor!

## [Skybox and environment mapping](./src/skybox.rs)

Load a skybox from a file, display it and render a cube reflecting the sky!

## [Texture resize](./src/texture_resize.rs)

This program is a showcase to demonstrate how you can use texture from an image loaded from the disk and re-use it to
load another image with a different size.

## [Query information](./src/query_info.rs)

Shows how to get some information about the backend and luminance.

## [MRT (Multi Render Target)](./src/mrt.rs)

This program shows how to render a single triangle into an offscreen framebuffer with two target textures, and how to
render the contents of these textures into the back buffer (i.e. the screen), combining data from both.

## [Uniform buffer](./src/uni_buffer.rs)

This example shows how to use _uniform buffers_ to implement _geometry instancing_, a technique allowing to quickly 
render a large number of instances of the same model (here, a simple square). This example shows how to update all the 
squares’ positions at once in the render loop.

[luminance]: https://crates.io/crates/luminance
[luminance-front]: https://crates.io/crates/luminance-front
[glutin]: https://crates.io/crates/glutin
[the book]: https://phaazon.github.io/learn-luminance
[wasm]: https://webassembly.org
[cargo-web]: https://crates.io/crates/cargo-web
