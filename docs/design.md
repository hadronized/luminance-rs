# Luminance design

This document describes the overall design of [luminance].

> Disclaimer: it’s a continuous effort to try and keep the document up-to-date with the latest version of [luminance].
> If you notice a difference, please do not hesitate to [create an
> issue](https://github.com/phaazon/luminance-rs/issues/new).

<!-- vim-markdown-toc GFM -->

* [Foreword: crate ecosystem](#foreword-crate-ecosystem)
* [Goals and main decisions](#goals-and-main-decisions)
* [Soundness and correctness](#soundness-and-correctness)
* [The backend architecture](#the-backend-architecture)
  * [Backend traits](#backend-traits)
  * [The `GraphicsContext` trait](#the-graphicscontext-trait)
* [The platform architecture](#the-platform-architecture)
* [Automatic backend type selection: `luminance-front`](#automatic-backend-type-selection-luminance-front)
* [Code generation and procedural macro](#code-generation-and-procedural-macro)
* [Detailed feature set](#detailed-feature-set)
  * [Framebuffers](#framebuffers)
    * [Dimensionality](#dimensionality)
    * [Color slot](#color-slot)
    * [Depth/stencil slot](#depthstencil-slot)
    * [Usage](#usage)
  * [Shaders](#shaders)
    * [Shader stages](#shader-stages)
    * [Shader programs](#shader-programs)
    * [Shader uniforms](#shader-uniforms)
    * [Shader data](#shader-data)
  * [Tessellation](#tessellation)
    * [Primitives](#primitives)
    * [Primitive restart](#primitive-restart)
    * [Vertex storage](#vertex-storage)
      * [Interleaved memory](#interleaved-memory)
      * [Deinterleaved memory](#deinterleaved-memory)
      * [Type-driven interface](#type-driven-interface)
    * [Vertex instancing](#vertex-instancing)
    * [Tessellation slice mapping](#tessellation-slice-mapping)
    * [Tessellation views](#tessellation-views)
  * [Textures](#textures)
    * [`Dim`, `Dimensionable` and others](#dim-dimensionable-and-others)
    * [Pixel formats](#pixel-formats)
      * [Normalized pixel formats](#normalized-pixel-formats)
    * [Samplers and sampler types](#samplers-and-sampler-types)
    * [Texture binding](#texture-binding)
  * [Pipelines and gates](#pipelines-and-gates)
    * [Obtaining a pipeline gate](#obtaining-a-pipeline-gate)
    * [Shading gates](#shading-gates)
    * [Render gates](#render-gates)
    * [Tessellation gates](#tessellation-gates)
  * [Queries](#queries)

<!-- vim-markdown-toc -->

# Foreword: crate ecosystem

[luminance] is the name of the _core_ crate but also the name of the Luminance ecosystem. The ecosystem comprises
several crates, classified in different themes:

- The core crate, [luminance], the subject of this very document.
- The proc-macro crate, [luminance-derive].
- “Backend” crates, providing technology-dependent implementation to run [luminance] code on different kind of tech,
  such as OpenGL, WebGL, Vulkan, etc.
- “Windowing” / “platform” crates, in order to run [luminance] code on specific platforms.

The goal of this document is to describe [luminance] and its core concept. For a description of the rest of the
ecosystem, you can glance through the [/docs](.).

# Goals and main decisions

[luminance] takes a different approach than other graphics crates in terms of how things should be done. The idea is
that code should drive as much as possible, from the memory perspective, bug perspective and logic perspective. Most of
the code you write should be checked as much as possible by your compiler, and the knowledge should be centralized as
much as possible. Bringing the knowledge together allows for better decisions, at the cost of flexibility. Indeed, the
goal of the ecosystem is not to be flexible in the sense to be usable with other graphics crates. The goal is to provide
a unique, safe and sound ecosystem. About this topic, see the [section about soundness](#soundness)

[luminance] was designed so that the following categories and families of _problems_ are avoided as much as possible.
Obviously, it is not possible to avoid everything, or every items in a given category, but still, the main incentive is
to minimize the number of items in this list that can bite both Luminance contributors, and Luminance end-users:

- Memory issues. Those range from memory leaks, double-free, use-after-free, (forbidden) random memory access, etc. This
  is the category that Rust defines its `unsafe` concept with, mainly.
- Panics. Panicking is a tool that can be interesting in some cases, but in a library, it does not have a place (at
  least not in the Luminance ecosystem). APIs will not be written in a way that misusing the API would result in a
  panic. For instance, accessing the _ith_ element of an array should be typed so that the returned element is
  `Option<T>`, not `T` with hidden panics.
- Typing issues. Types in Luminance are strong in the sense that they lift preconditions and invariants into the type
  system. Instead of having to check whether a value is always positive at runtime, the philosophy is to lift an
  arbitrary number into a `PositiveNumber` type once, doing the fallible conversion only once, and then assuming the
  invariant holds because we trust the type system. In that sense, types in Luminance follow closely the concept of
  _refinement typing_. That brings a lot of advantage, from less error-prone code, to better runtime performance
  (because the checks are done only once at construction, not at every use-case — if you don’t do refinement typing, you
  **must** check that kind of condition in each public functions taking such an argument, for instance; OpenGL is a good
  example of what not to do, for instance). Basically: leave the checks to the compiler, and enjoy raw runtime speed.
- Mutation and state corruption. The Luminance ecosystem is designed in a way that mutation and states are always
  decorated in a way that it is impossible (or close to) to corrupt global state or even local state. Some backend
  technologies have a huge dependency on global state invariants, and Luminance tries its hardest to keep the invariants
  safe from being violated. It goes from abstractions in the core crate, to state trackers indexed in the type system.
- Logical bugs. Lots of logical bugs can actually be avoided by webbing types and functions in a way to make it
  statically impossible to create bad runtime constructs. There are still exceptions, especially at the boundary of the
  crates (where we need to serialize / deserialize something, for instance), but Luminance does its best to ensure that
  logical expressions are sound and that users cannot express illogical statements — i.e. they won’t compile.
- Optimization problems. Some backend technologies require a very specific order of function calls, or formats, or
  arguments. All of this complexity and details are hidden behind [luminance]’s abstractions.

Because of all this, [luminance]’s API can be a bit frightening to the non-initiated. For instance, [luminance] uses a
lot all of these concepts:

- [Higher-Rank Trait Bounds (HRTBs)](https://doc.rust-lang.org/nomicon/hrtb.html).
- [Rank-2 types](https://wiki.haskell.org/Rank-N_types).
- Associated types, associated type constructors, type families.
- [Type states](https://en.wikipedia.org/wiki/Typestate_analysis).
- [Refinement types](https://en.wikipedia.org/wiki/Refinement_type).
- State trackers.
- And more…

As often as possible, complex type and function signatures will be well documented so that newcomers and people not used
to all of those concepts can understand and use the API nevertheless.

# Soundness and correctness

Rust has this `unsafe` keyword that people use to enter an unsafe section where one can do dangerous things, such as
dereferencing a pointer, type casting, calling FFI functions, etc. `unsafe` is also overloaded as soon as we start using
it to “convince people not to use someting unless they know exactly what they are doing.” In [luminance], that maps to
unsafe implementors, mostly (`unsafe trait` / `unsafe impl`). Misimplementing those traits will not make your
application crash or leak memory, but it will probably yield to misbehavior, incoherent state, etc.

For this reason, the concept of _soundness_ is a bit blended with safety in [luminance]. What we would need is an
`unsound` keyword, meaning that the code marked `unsound` can be unsound if not implemented properly. Because we do not
have such a keyword, `unsafe` is used and the definition of _safety_ in [luminance] is a superset of the one commonly
accepted in the Rust ecosystem: memory safety, plus soundness and correctness.

# The backend architecture

The main idea of [luminance] is quite simple: the core crate provides a safe API with all the exposed features as fully
and strongly typed symbols, as well as everything “shared” between possible implementations (such as blending,
depth/stencil test, etc.). The role of the [luminance] crate is to make it impossible to have incorrect constructs. The
main API lives in the `GraphicsContext` trait and all subsequent required traits. Indeed, [luminance] uses a granular
constraint system, which means that if you want to use shaders, all you need is that the backend provides a
shader implementation. It means that in theory you should be able to partially implement the interface (since it spreads
on many traits).

## Backend traits

Then you have _backend_ crates. Those provide many implementors of the various required traits to use different
[luminance] features. Here is a list of traits to implement to unlock the mapped feature — some traits have type
variables; those are constrained and will require constrained implementors as well:

- **Framebuffers**:
  - `luminance::backend::framebuffer::FramebufferBackBuffer`
  - `luminance::backend::framebuffer::Framebuffer`
  - `luminance::backend::texture::TextureBase`
- **Shaders**:
  - `luminance::backend::shader::Shader`
  - `luminance::backend::shader::ShaderData`
  - `luminance::backend::shader::Uniformable`
- **Tessellation**:
  - `luminance::backend::tess::IndexSlice`
  - `luminance::backend::tess::InstanceSlice`
  - `luminance::backend::tess::Tess`
  - `luminance::backend::tess::VertexSlice`
- **Texture**:
  - `luminance::backend::texture::Texture`
  - `luminance::backend::texture::TextureBase`
- **Pipelines and gates**:
  - `luminance::backend::framebuffer::Framebuffer`
  - `luminance::backend::pipeline::Pipeline`
  - `luminance::backend::pipeline::PipelineBase`
  - `luminance::backend::pipeline::PipelineShaderData`
  - `luminance::backend::pipeline::PipelineTexture`
  - `luminance::backend::render_gate::RenderGate`
  - `luminance::backend::shader::ShaderData`
  - `luminance::backend::shading_gate::ShadingGate`
  - `luminance::backend::tess_gate::TessGate`
  - `luminance::backend::texture::TextureBase`
  - `luminance::backend::texture::Texture`
- **Query**:
  - `luminance::backend::query::Query`

A backend crate will always expose at least one type: the _backend type_. That is the type you will have to use to
replace the various `B` type variables you will find in generic / polymorphic [luminance] code — there is an exception
to that if you use [luminance-front]; more on that later in that document.

However, a backend crate doesn’t necessarily have to expose only one backend type. Indeed, it can expose different
implementations. Most of the time, that will make sense for different versions of the API of a given backend technology
(think of OpenGL 3.3, OpenGL 4.0, OpenGL 4.5, OpenGL 4.6, WebGL1, WebGL2, etc.).

## The `GraphicsContext` trait

The `GraphicsContext` trait marks the limit of the backend zone of [luminance]. The trait doesn’t belong to the backend
module and is not implemented by backend crates. Instead, it is implemented by
[platform crates](#the-platform-architecture).

The important part to understand here is that this trait has an associated type, `GraphicsContext::Backend`. An
implementor of that trait must then pick a concrete type (or type variable correctly constrained with all backend
traits, which can get tricky) taken from a backend crate. Because of the nature of traits, it means that it is possible
to have two platform crates implementing `GraphiscContext<Backend = BackendA>` for the same `BackendA`. The `BackendA`
backend type can then have different implementations, sharing them between platforms. This is a typical case when a
technology, such as OpenGL, can be created / managed by different system crates ([glutin], [glfw], for instance).

# The platform architecture

The platform architecture is quite weak and narrow (that is not the role of the [luminance] ecosystem). The backend
interface is rich and provides a lot of details on how to design graphical code. The platform interface explains how to
run that code and tries to capture the minimal and weakest interface.

Windowing features are then completely put out of focus of the [luminance] ecosystem. The only thing a platform must
provide is:

- `GraphicsContext::Backend`: the backend type the platform is implemented for.
- Optional overrides of all of its methods. Those are automatically implemented when the backend feature is implemented,
  but platform crates can re-implement those (for logging purposes, for instance).

Creating an OpenGL / Vulkan / WebGL / whatever context is completely left out of scope of the [luminance] ecosystem, and
should be handled by the calling code. This is a wanted feature, as it allows people to write platform and technology
agnostic code, and run it everywhere.

Obviously, anything related to the following items are not covered by [luminance] and will require either an abstraction
of yours to create, or a crate that abstract this logic for you (and lets you implement the graphical part with
[luminance]):

- Windowing features, such as creating a window.
- Backend context creation.
- System and user events.
- Render buffer swapping.
- Etc.

# Automatic backend type selection: `luminance-front`

There exists a special crate in the ecosystem: [luminance-front]. As explained above, in order to be able to run
[luminance] code, you need two things:

- A backend type, that is provided by a backend crate.
- A platform type implementing `GraphicsContext` and providing the “bridge” code to run the graphical code on a given
  platform.

This association is, by default, done entirely manually. The user has to know which platform they are compiling code
for, pick the right platform type and the right backend type. For advanced use cases and power users, that can be a
wanted situation (think of switching technology on the fly or at startup). However, for people wanting a smoother and
(much) easier experience, [luminance-front] will do everything for you.

The idea of [luminance-front] is to re-export all the public symbols of [luminance] and replacing the `B` type variable
with the right backend type depending on the compilation target. If you compile for WASM, it will automatically pick the
backend type of the [luminance-webgl] crate (`WebGL2` by default; can be changed with features). For `x64`, it will pick
a type from [luminance-gl]. Etc. etc.

However, [luminance-front] will not pick the platform crate for you. You will have to pick it by yourself.

# Code generation and procedural macro

[luminance-derive] is also a special crate that allows to implement various user-facing traits to be automatically
implemented by using `#[derive(…)]` annotations on your types. Most of the time, [luminance-derive] tries to solve two
problems:

- Some traits are `unsafe` to implement. Users shouldn’t write `unsafe` code, so a procedural macro backed by the
  compiler’s static analysis is a good candidate here.
- Some traits are _boring_ to implement and the implementation can be automatically deduced by the fields of the
  `struct`, the `enum`’s variants, etc.

There are no known exception for not using [luminance-derive], unless you know what you are doing, so you should
definitely use it.

# Detailed feature set

This section describes how every [luminance] features work behind the scenes. At its core, [luminance] feature set is
pretty narrow and simple:

- Framebuffers.
- Shaders.
- “Tessellations” (name subject to change).
- Textures.
- Graphics pipelines and gates.
- Queries.

## Framebuffers

Framebuffers are objects acting as receptacle of renders. Anything that gets rendered has to be sent to a framebuffer at
some point. For this reason, framebuffers are the outermost scarce resource you will find in a graphics pipeline, since
you might want to render several objects with different techniques in the same framebuffer.

A framebuffer has a couple of properties. [luminance] differs from regular graphics API as it strongly types
framebuffers with concepts that don’t really exist outside of [luminance]:

- Its dimension. Dimensionality is an important concept for framebuffers that will greatly impact the storage and nature
  of the other properties.
- Color slots.
- Depth/stencil slots.

### Dimensionality

Framebuffers have a _dimension_, encoded via the `Dim` type and constrained via `Dimensionable`. Dimensionality is a
wide topic that is explained in the [Textures](#textures) section.

### Color slot

Color slots are an abstraction solving the problem of accessing color data in a framebuffer. Framebuffer color data can
be accessed in two different ways:

- In a write-way; i.e. a fragment shader will output _fragments_, that must end up in framebuffer color data. In this
  case case, shader outputs = color data.
- In a read-way; i.e. framebuffer color data can be accessed by different parts of the pipelines (fixed functions,
  shader code, etc.).

How the data is represented depends on the type of the color slot. In that way, [luminance] way of encoding framebuffers
is by letting the user _declares what the color data should be_ and generate the actual color slot for the user,
automatically. The following is subject to change but has been the case for years (both in the Haskell and Rust
versions of [luminance], so it might continue being that way for a while):

- If you don’t want color data in your framebuffer (e.g. a framebuffer only capturing depth and/or stencil information
  and discarding any color data), your color slot type can be set to `()`.
- If you want a single color data, for instance an RGBA 32-bit color, you can set your color slot to a pixel type
  encoding that color (here, it would be `luminance::pixel::RGBA32F`).
- If you want more than one color data, you can use tuples of pixel types.

The last point is the one that might change in the future (we would probably want to access data in a more nominal way,
so that it’s possible to share names at compile times instead of tuple indices, for instance). See
[this design doc](./strongly_typed_color_slots.md) for further details.

For each type of color slot and framebuffer dimension, the way [luminance] works is by injecting a type family,
mapping the color data type to the color slot type. The mapping is as such:

| Framebuffer dimension      | Color data type                                                                 | Color slot type                  |
| ========================== | =============================================================================== | ================================ |
| any                        | `()`                                                                            | `()`                             |
| `D where D: Dimensionable` | `P where P: ColorPixel + RenderablePixel`                                       | `Texture<D, P>`                  |
| `D where D: Dimensionable` | `(A, B) where A: ColorPixel + RenderablePixel, B: ColorPixel + RenderablePixel` | `(Texture<D, A>, Texture<D, B>)` |
| `D where D: Dimensionable` | etc. etc.                                                                       | etc. etc.                        |

So basically, if you use `()`, you will get `()` as color slot. If you use a pixel type `P`, you will get
`Texture<D, P>` where `D` is the dimension of the framebuffer. If you use a tuple of pixel types, like `(A, B, C)`, you
will get `(Texture<D, A>, Texture<D, B>, Texture<D, C>)`, etc.

> This tuple design works but is a bit uneasy to work with, especially in shader code, when it is required that the
> order of which the shader outputs declared in a _fragment shader_ matches the framebuffer the fragments should be
> rendered into.

### Depth/stencil slot

Framebuffers have a second slot: the depth/stencil slot. It works exactly the same way as color slots, but with a few
differences:

- You can use `()` to mute the slot but you cannot have tuples.
- Instead of being constrained with `ColorSlot + RenderablePixel`, if you use a depth/stencil slot, it is constrained by
  `DepthPixel` only.

### Usage

Framebuffers are used by accessing directly the color slots or depth/stencil slots (via `.color_slot()` and
`.depth_stencil_slot()` for instance) or as part of a [graphics pipeline](#pipelines-and-gates). Read on the graphics
pipeline section for further information.

## Shaders

Shaders are a group of resources gathering a couple of concepts:

- Shader stages.
- Shader programs.
- Shader uniforms.
- Shader data.

### Shader stages

Shader stages represent various steps that occur in the graphics pipeline. When rendering something on the screen,
lots of objects are going to be streamed, activated and consumed. Depending on the state of those objects, different
_shader stages_ will be executed to move to the next steps. `luminance` current supports five shader stages:

- Vertex shaders.
- Tessellation evaluation and tessellation control shaders.
- Geometry shaders.
- Fragment shaders.

> Optional compute shaders are planned but not yet implemented.

Shaders stages are currently customized with GLSL represented as opaque `String`. This is subject to change with the
shading EDSL work-in-progress.

### Shader programs

Shader programs are linked programs that run on the graphics unit (GPU most of the time). They gather shader stages and
are inserted into a graphics pipeline (more accurately, they are shared at the `Pipeline` level, yielding a
`ShadingGate` to create and nest more nodes in the graphics AST — more on that in the
[appropriate section](#pipelines-and-gates)).

There is nothing much interesting you can do with a shader program besides using it in a graphics pipeline. However,
there is one important aspect that you need to know about shader programs. Shader stages can reference _uniforms_,
customization variables coming from your application to change the behavior of the stage code. Those variables must be
set at the program level, not stage level, because their values are shared across all invocations of the stages
contained in a shader program.

Shader programs are typed with several type variables:

- `Sem`: the `VertexSemantics` the shader program is compatible with.
- `O`: the output variable. Currently, this is not used at all, and will probably go away to be replaced with color and
  depth slots. See [strongly typed color slots](./strongly_typed_color_slots.md) for further details.
- `Uni`: the _uniform interface_. Read on [shader uniforms](#shader-uniforms).

### Shader uniforms

Shader uniforms, as explained above, are customization variables users can reference in the source code of their shader
stages, and provide values for in their application code, at the Rust level.

Shader uniforms follow a very opinionated mechanism in `luminance`. In most graphics libraries / frameworks, uniforms
can be imagined as entries in a string hash map. Something akin to:

```rust
HashMap<String, UniformValue>
```

Some other frameworks would do a lookup on the graphics backend whenever you want to change the value of a uniform,
which can be typed too. For instance:

```rust
fn update_uniform(&mut self, name: &str, value: impl IntoUniform) -> Result<(), UniformError>
```

However, those situations have drawbacks:

- The `HashMap<String, UniformValue>` has two main issues:
  - We have to do a string lookup, which can be costly in a render loop, especially if we change the value of uniforms
    quite often.
  - `UniformValue` is likely to be an `enum`, which will require branching to get and update value, and also will
    require some kind of error handling (if a user tries to set a value which type doesn’t match). So this moves typing
    at the runtime, which is not ideal.
- The `update_uniform` performs a string lookup _and_ an I/O on the graphics backend, which is unacceptable (the impact
  is likely to be limited if the graphics driver implements caching, but we don’t want to be dependent on that).

Instead, the `luminance` way builds on the concept of a _uniform interface_. The idea is to recognize, in the first
place, that a shader program has a set of uniforms. That set could be structured as a type, for which the fields
represent each and every uniforms in that shader program. In a second time, it’s easy to see that uniform values should
be accessed indirectly, in a _contravariant_ way — i.e. users shouldn’t have the right to create uniforms by themselves.
Instead, users should _explain_ how the uniforms should be obtained (by using their names, for instance), and let
`luminance` retrieve the uniform variables when creating the shader programs. Finally, updating and setting uniform
values should only be done whe the appropriate shader is in-bound, so users should not have an unlimited access to
uniforms.

What all that means is that, in `luminance`, shader uniforms are types and declared by the user via the type system
only. The user can then ask `luminance` to provide the type at due time (i.e. in the graphics pipeline, when it’s needed
to update their values). An example:

```rust
#[derive(UniformInterface)]
pub struct MyUniformInterface {
  #[uniform(name = "t")]
  time: Uniform<f32>,

  #[uniform(name = "res")]
  resolution: Uniform<V2<f32>>,
}
```

Here, the user explains what the types are (two uniform types, one is a `f32` representing the time, that can be
accessed on the shader side with the `t` uniform variable, and the other is the `resolution`, a 2-component floating
point vector).

Because the shader program will be typed with `MyUniformInterface`, it will provide the user with a
`&MyUniformInterface`, allowing them to set and update the appropriate uniforms in due time.

> Note: the syntax `#[derive(UniformInterface)]` uses the procedural macro from `luminance-derive`. It is possible to
> implement `UniformInterface` without the proc-macro, but keep in mind the trait is `unsafe` to implement.

### Shader data

Shader data are a special form of shader uniforms. That abstraction doesn’t exist in any other graphics libraries /
frameworks, but is a form of generalization of graphics techniques (such as UBO, SSBO, etc.). The idea of shader data is
quite simple: instead of passing uniform values manually by setting them using a struct field, shader data represent
their own data store, backed up by GPU buffers. They allow for much bigger data storage and allow to _map_ the memory
region for fast updates. They are to be used for mass updates or quickly changing data.

They work behind `Uniform<_>`, too, but requires to be _bound_ to a graphics pipeline first, yielding a
`BoundShaderData` typed the same way as the `ShaderData`. That object can then be used to get a `ShaderDataBinding`,
which is the type of the `Uniform` to set to handle shader data of that type.

For instance, if you have a `ShaderDataBinding<ModelMatrix>`, you can bind it to a graphics pipeline to get a
`BoundShaderData<ModelMatrix>`, which in turn will provide the binding value to set your `Uniform<ShaderDataBinding<ModelMatrix>>`

## Tessellation

Tessellations (`Tess`) are the only way to pack vertices and render primitives. Their API is dense and rich but at their
core, tessellations are quite easy to understand. A couple of raw concepts must be known before diving in:

- _Vertex_: a vertex is an abstraction of a _point_. It’s an abstraction because by default, a vertex doesn’t carry any
  _attribute_, so it doesn’t really have a representation. The representation depends on the attributes the vertex is
  made of. For instance, a vertex with a _3D coordinates_ attribute is easy to imagine as a dot in 3D space. But it
  might be not enough to even represent it. You might want to add another attribute, for its _weight_, so that you can
  make the dot bigger or smaller. And you might also need the _color_ of the vertex. So verticese (plural of vertex)
  cannot be pictured / rendered without talking about their attributes.
- _Vertex index_: vertex indices represents the order in which vertex should go down the vertex stream processor to form
  _primitives_. The index refers to a vertex in the input data. By default, vertices are sent down to vertex shaders in
  the order in which they appear in the input data (in the `Tess`). You can decide to change that order. If you have
  six vertices and no indices, you can imagine that it’s like sending virtual indices `[0, 1, 2, 3, 4, 5]`. Specifying
  indices manually allows to optimize and reduce the number of vertices, and allows to implement something that is
  called [primitive restart](#primitive-restart).
- _Vertex instance data_: just like regular vertex attributes, vertex instance data are attached to vertces, but instead
  of being attached to individual vertices, their attached to instances of those vertices. See [vertex
  instancing](#vertex-instancing) for further information.
- _Primitive_: a shape formed by connecting vertices together. See [primitives](#primitives). This is often refered to
  as the _tessellation mode_, too.
- _Vertex and instance storage_: vertices have attributes (vertex data and instance data). Those attributes must follow
  a convention to know how to extract the _nth_ vertex attribute of a given type. This is managed by the
  [vertex storage](#vertex-storage) of the tessellation.
- _Slice mapping_: tessellations’ data can be mapped back to be updated. More on that in the
  [slice mapping](#tessellation-slice-mapping) section.
- Tessellation view: allow to _view_ into the whole / a subport of the tessellation. This is mainly used for rendering
  with tessellation gates.

### Primitives

Primitives are ways of connecting vertices together. By default, vertices are not connected. This corresponds to the
`Mode::Point` primitive. It is possible to connect vertices via other shapes:

- `Mode::Line`: vertices are connected two by two, by using two consecutive vertices in the vertex stream. For instance,
  if you have indices `[0, 1, 2, 3]`, indices `[0, 1]` will form a line and indices `[2, 3]` will form another, for a
  total of two lines out of four vertices.
- `Mode::LineStrip`: vertices are connected as a continuous line formed by segments. The first two points create the
  first segment and each and every additional point connects the last point of the line to the vertex. For instance, for
  indices `[0, 1, 2, 3]`, indices `[0, 1]` will form a line, `[1, 2]` will form another (that will look like attached by
  the shared vertex `1`), and `[2, 3]` will form another (attached by the shared vertex `2`.
- `Mode::Triangle`: vertices are connected three by three, by using three consecutive vertices in the vertex stream. For
  instance, if you have indices `[0, 1, 2, 3, 4, 5]`, indices `[0, 1, 2]` will form a triangle and indices `[3, 4, 5]`
  will form another.
- `Mode::TriangleStrip`: vertices are connected as a triangle strip. What it means is that the first three vertices form
  a triangle, and then every new vertex will form a triangle with the last two previous vertices, sharing an edge. For
  instance, for indices `[0, 1, 2, 3, 4, 5]`, indices `[0, 1, 2]` will form the first triangle; indices `[1, 2, 3]` will
  form the second triangle, sharing the `[1, 2]` indices; `[2, 3, 4]` will form another triangle and `[3, 4, 5]` the
  latter.
- `Mode::TriangleFan`: a useful alternative to `Mode::TriangleStrip`. It works the same way, but instead of using the
  last two previous vertices to form a triangle, it uses the very first one and the last previous one. For instances,
  for `[0, 1, 2, 3, 4, 5]`, the first triangle is still `[0, 1, 2]`. The second triangle is `[0, 2, 3]`. Then
  `[0, 3, 4]` and `[0, 4, 5]`. This shape is useful to create _fan-like_ shapes, often rotating around the first vertex.
- `Mode::Patch(nb)`: a special primitive. This doesn’t connect the vertices unless provided with the number of vertices,
  such as `Mode::Patch(3)` is a triangle. That mode is used mainly with tessellation shaders.

### Primitive restart

When using `Mode::LineStrip`, `Mode::TriangleStrip` or `Mode::TriangleFan`, it is possible to restart building a
primitive from scratch by passing a special index as vertex index. Using the maximum value of the index type will cause
the graphics pipelines to stop and end the primitev and start another one at the next index. For instance:

```rust
let indices = [0, 1, 2, 3, u32::MAX, 0, 1, 4, 5];
```

Assuming we use `Mode::TriangleStrip`, this will create four triangles: `[0, 1, 2]`, `[1, 2, 3]`, `[0, 1, 4]` and
[`1, 4, 5`].

> It is likely that the API of `Tess` allows to change the value of the primitive restart. This is deprecated and will
> soon be removed. You must use the max value of the index type.

### Vertex storage

Vertex storage refers to how vertices (both vertex attributes and vertex instance data) are laid out to memory, and
expected to be provided by users. If you think of a vertex with four attributes:

- A 3D position `V3<f32>`.
- A 3D normal `V3<f32>`.
- A 4D color `V4<f32>`.
- A random ID `i32`.

When you think about a single vertex, you can for instance encode it using this type:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MyVertex {
  pos: V3<f32>,
  nor: V3<f32>,
  col: V4<f32>,
  uid: i32,
}
```

However, now try to think how you would represent 100 vertices of type `MyVertex`. Of the possible ways to represent
this situation, `luminance` accepts two that will have a big impact on the API:

- Interleaved memory.
- Deinterleaved memory.

#### Interleaved memory

Interleaved memory will represent arrays of `MyVertex` by putting packing them all together in an array, as simple as
that:

```rust
type ArrayOfMyVertex = Vec<MyVertex>;
```

We call this encoding _interleaved_ because data is interleaved in terms of attributes:

```rust
let memory = [pos0_x, pos0_y, pos0_z, nor0_x, nor0_y, nor0_z, uid0, pos1_x, pos1_y, pos1_z, nor1_x, nor1_y, nor1_z, uid1];
```

The advantage of using this memory model is that shaders requiring all data at once will have better cache locality when
fetching vertex data. The drawbacks is when you have a piece of code that only works on positions, for instance. It will
have to also read the rest of the attributes to get the next position, and will not use the data in between.

#### Deinterleaved memory

On the opposite side, _deinterleaved_ memory extracts each attributes into its own storage, like so:

```rust
type ArrayOfMyVertexPos = Vec<V3<f32>>;
type ArrayOfMyVertexNor = Vec<V3<f32>>;
type ArrayOfMyVertexCol = Vec<V4<f32>>;
type ArrayOfMyVertexUID = Vec<f32>;
```

This is great when your shaders will only work on some attributes at the same time, which is often the case. The
drawback is that it requires more runtime checks (all arrays must have the same size) and reconstructing a full vertex
requires to tap in different buffers.

#### Type-driven interface

When building a `Tess` using a `TessBuilder`, you will see that the last type variable, `S` (which defaults to
`Interleaved`), can be set to either `Interleaved` or `Deinterleaved`. If you look closer to how a `TessBuilder` is
built, you will see that `V` (the vertex type) and `W` (the instance type) are constrained by `TessVertexData<S>`. This
to ensure that no other storage types can be used.

If you use `Interleaved`, you will get access to methods such as `set_vertices`, expecting a slice on the vertices to
upload to the `Tess` GPU storage.

If you use `Deinterleaved`, things get a bit more complicated. Not only you will need `Vertex` implemented for your
type, but will need _a bit more_. You will need the `Deinterleave<T>` trait to be implemented for all of the attributes.
If you use `luminance-derive`, this is automatically done for you. Otherwise, you need to implement it (`T` refers to
the type of the attribute, which means that you cannot have twice the same type as attribute).

When using `Deinterleaved`, you will not get access to `set_vertices`, but `set_attributes`. The power of typing here
allows nothing else required on the interface to set the right attributes: just pass the attributes with the
`set_attributes` and `luminance` will automatically know which GPU buffer to upload data to based on the type of the
attributes. Something like this will work:

```rust
let builder = TessBuilder::new()
  .set_attributes(&MY_POSITIONS)
  .set_attributes(&MY_NORMALS)
  .set_attributes(&MY_COLORS)
  .set_attributes(&MY_UIDS);
```

### Vertex instancing

By default, when building a `Tess` out of vertices, you create a _model_. The vertices usually refer to data in _object
space_: it’s like the object is centered at the origin. If you want to place your object somewhere in the world, you are
not going to change the model data, but instead create an _instance_ of your model and virtually move the vertices of
that instance (using a _vertex shader_).

The concept of _instancing_ is very overloaded in computer graphics. Several ways of doing it exist. `luminance`
supports mainly two:

- Vertex instancing.
- Geometry instancing.

> This section is about vertex instancing. You can read about geometry instancing in the
> [appropriate section](#geometry-instancing).

Vertex instancing is basically a way to extract the instance data (for instance, the position of each instance in 3D
world space) by looking up directly the data in the `Tess`. Setting it up is quite simple: the instance data is passed
at the creation of the `Tess`, along with the rest of the vertices and indices. The instances can also be retrieved via
[slice mapping](#tessellation-slice-mapping).

The number of instances to render is set by the `TessBuilder`, or can be explicitly asked when rendering the
tessellation in a graphics pipeline.

### Tessellation slice mapping

Upon the `Tess` creation, it’s possible to retrieve in an immutable or mutable way the vertices, indices or instance
data of a tessellation. That allows various kinds of drawing and update strategy, but the mechanism is the same for all
kinds:

1. Slice map the tessellation for a given set. You can use `Tess::vertices{,_mut}`, `Tess::indices{,_mut}` or
   `Tess::instances{,_mut}` methods.
2. You can use the returned slice object and `Deref / DerefMut` implementors to access and update the tessellations.
   Dropping the slices will perform the update.

It is highly recommended to prefer using slice mapping to reconstructing a full `Tess` at every frame. Indeed,
tessellation construction goes through a lot of GPU settings (buffer allocation, etc.), while slice mapping is much
faster. You also get access to function that works on slices, such as `copy_from_slice` for instance.

### Tessellation views

Tessellation views allow to render a subpart of a `Tess`. You can decide to render the whole part of the tessellation,
or a subpart, starting at a given vertex and ending at another.

Tessellation views allow for various kinds of effects: particles engine, packing objects into the same GPU region, etc.
etc.

You can create a tessellation view using `TessView::whole`, `TessView::sub`, `TessView::slice` or the instance-based
versions, which also accept the number of instances to render. Another good way to create tessellation views is to use
the `View` trait, which allows to use ranges with the `view()` and `inst_view()` methods, which are implemented on
`&Tess`. That allows calls like `tess.view(..)`, `tess.view(..12)` or `tess.inst_view(3..=10, 1000)`, for instance.

## Textures

Textures can be used in a write path (framebuffer) or read path (shader stage):

- As framebuffer color slots / depth slots (write).
- As standalone entities that can be used to customize render, by fetching their texel data in shader stages (read).

A texture is either created automatically for you (when part of a framebuffer slot) or manually. In both case, you need
to provide two important information:

- The dimension, which is a type that must implement `Dimensionable`. That type will allow the type system to track the
  dimension (2D, 3D, cubemap, etc.), size / resolution, offset, zero values etc. associated with the dimension.
- The pixel format. Pixel formats are tracked in the type system to know which kind of texels you are allowed to pass /
  retrieve, and what should be used in shaders.

Those properties are encoded in the type of a texture, as in `Texture<D, P>` where `D` is the dimension and `P` the
pixel type.

### `Dim`, `Dimensionable` and others

A _dimension_ is a property that provides various kind of information. As a runtime value, it’s most of the time the
“size” of a texture. For instance, a 2D texture will have a _width_ and _height_. A 3D texture will have a _width_,
_height_ but also _depth_. Because of the various possible flavours of textures, it was a design choice to track their
dimension type in the type system to provide more safety in the public API.

Dimensions are then encoded as simple types and used as type parameters on `Texture` and `Framebuffer`. Those are just
tags, not constant. What it means is that a texture that is tagged with `Dim2` is expected to have a dimension with a
_width_ and _height_.

The `Dimensionable` trait allows to reify the various dimension type to different kinds of other properties. The most
important one being the _size_ (width / height / depth / etc.). The way it works is by using associated types. The trait
provides several ones:

- `Dimensionable::Size`: the size type of the dimension. For instance, `<Dim2 as Dimensionable>::Size` is `[u32; 2]`.
- `Dimensionable::Offset`: the offset type of the dimension. For instance, `<Dim2 as Dimensionable>::Offset` is
  `[u32; 2]`. Offsets are mainly used to select sub-part of textures and introduce arithmetics.

Given those, `Dimensionable::ZERO_OFFSET` is a constant of type `Self::Offset` that must resolve to the _zero_ value of
the offset type. For instance, for `[u32; 2]`, it’s `[0, 0]`. This _zero_ value is to be interpreted in the sense that
it must represent the “starting point of the texture”, for instance (useful when updating a whole texture at once, for
instance).

The trait then provides a couple of methods. The important one is `Dimensionable::dim()`, which reifies the type (e.g.
`Dim2`) to the `Dim` sum type. Then, you will find methods working on associated types, such as `Dimensionable::width`
taking a `Self::Size` and returning the width. If the type doesn’t have a property (for instance, `Dim2` doesn’t have a
_depth_), the default implementation should be enough (which is `1` for `Self::Size`). Same thing with
`Dimensionable::y_offset`, returning the _y_ component of the offset (or the default if it doesn’t exist).

Finally, `Dimensionable::count` takes a `Self::Size` and returns the amount of pixels that quantity represents. For 2D
sizes, it represents the area. For 3D sizes, the volume. Etc. etc.

### Pixel formats

Pixel formats work a bit the same way as dimension types. They are empty types tracking information in the type system.
Pixel formats must implement the `Pixel` trait, which provides lots of information:

- `Pixel::Encoding`: an associated type providing the encoding of a single pixel. For instance, `RG8UI` has its
  `Encoding` type set to `[u8; 2]`.
- `Pixel::RawEncoding`: an associated type provding the raw encoding of a single pixel, ignoring channels. For instance,
  for `RG8UI`, it’s `u8`. Think of this as the type that is required to be present in a foreign crate’s `Vec` or the
  FFI.
- `Pixel::SamplerType`: the sampler type required to access this pixel format.
- `Pixel::pixel_format`: reify the type to a `PixelFormat`

`PixelFormat` is a simple data type providing two informations:

- The encoding type, among:
  - `Type::NormIntegral`.
  - `Type::NormUnsigned`.
  - `Type::Integral`.
  - `Type::Unsigned`.
  - `Type::Floating`.
- The format, among:
  - `Format::R(Size)`.
  - `Format::RG(Size, Size)`.
  - `Format::RGB(Size, Size, Size)`.
  - `Format::RGBA(Size, Size, Size, Size)`.
  - `Format::SRGB(Size, Size, Size)`.
  - `Format::SRGBA(Size, Size, Size, Size)`.
  - `Format::Depth(Size)`.
  - `Format::DepthStencil(Size, Size)`.

The `Size` type is a sum type encoding possible sizes. Instead of using a `usize`, it was decided to use a sum type to
prevent constructing unsupported sizes (and prevent checking for those). The current list is:

- `Size::Eight`: 8-bit.
- `Size::Ten`: 10-bit.
- `Size::Eleven`: 11-bit.
- `Size::Sixteen`: 16-bit.
- `Size::ThirtyTwo`: 32-bit.

#### Normalized pixel formats

Some pixel formats are _normalized_, as in `Type::NormUnsigned`. What it means is that the texture encoding is going to
be unsigned (like `u8`, `u16`, `u32`, etc.), but the data will be interpreted as normalized (floating point) when
accessed in shaders. For instance, if you use `Type::NormUnsigned` along with `Format::R(Size::Eight)`, you should have
a static type of `Pixel::Encoding` (and `Pixel::RawEncoding`) set to `u8`. The range of possible values is `0 -> 255`,
but those values will be converted to floating point values and then divided by `255.0`, yielding a `0.0 -> 1.0` range,
when accessed in shaders.

Most of the time, it’s the type people want to deal with visual effects. The non-normalized type, `Type::Unsigned`, will
not normalize the range and you will then access unsigned values in the shaders, which can be useful for various
situations, like encoding UIDs, indices, etc.

### Samplers and sampler types

Samplers are objects representing how data / texels should be _sampled_ from a shader stage. They provide various
information like:

- How sampling should wrap at borders / edge.
- How minification and magnification filters behave.
- Depth comparison for depth textures.

That is encoded in `Sampler`. This is not to be confused with `Pixel::SamplerType`, which must implement the
`SamplerType` trait. That trait allows to sample with a different type than the one used to encode the texels in the
textures, as long as the `Type` is the same.

### Texture binding

A `TextureBinding` is an opaque texture handle, obtained after binding a `Texture` in a graphics pipeline. This is the
only way to pass a texture to a shader for reading.

## Pipelines and gates

Pipelines and gates play an important role in how computations are effectively performed. The design befind pipelines
and gates is based on the Rust type system and its lifetime / borrow semantics, and how graphics scarce resources should
be shared.

The idea is that there is a tree-like hierarchy of how resources are used and shared. At the top of the tree is found a
framebuffer. Everything under it, in the tree, will then be affecting the tree (i.e. they will partake into rendering
into it). Under a framebuffer can be found shaders. Each shader is a node directly connected to the framebuffer node.
Under each shader node are render nodes, which encode various render options (those are encoded as `RenderState`),
allowing to customize how tessellations will be rendered. And under each render node is found… tessellations to render
(strictly speaking, `TessView`, because it’s possible to ask the backend to partially render tessellations).

Two main ideas were imagined to represent such a sharing and running of resources:

1. Implement a datastructure encoding that graph / tree.
2. Recognize that code itself is a tree (i.e. AST), and leverage this and Rust’s borrow and lifetime system to enforce
  it.

The second point was chosen, as it feels more natural and can benefit from using the type system to track all objects.
_“Gates”_ are the names given to those nodes mentioned earlier.

### Obtaining a pipeline gate

A pipeline gate is the top-most object in the graphics tree and is obtained via `GraphicsContext::new_pipeline_gate`.
That object can then be used to create the first framebuffer node via `PipelineGate::pipeline`. That function expects a
`FnOnce(Pipeline, ShadingGate)` (simplified for short). The `Pipeline` argument represents the running pipeline and
allows to perform various operations, such as binding a texture or a shader data. The second argument represents the
next gate down the tree, `ShadingGate`.

### Shading gates

Shading gates allow to shade with a given shader `Program`. Doing so will provide the user back with the uniform
interface attached with the shader programs and allow them to change the uniform values. Shading gates, once entered via
the `FnOnce`, will also provide a `RenderGate`, allowing to go down the tree once again.

### Render gates

Render gates create a sharing node around `RenderState`, allowing to render using the same `RenderState`. Once entered,
they give access to tessellation gates.

### Tessellation gates

Tessellation gates allow to render various `Tess`, for which the vertices types must be compatible with the shading gate
vertex semantics. Tessellations are not rendered directly, but instead require passing via a `TessView`.

## Queries

[luminance]: https://crates.io/crates/luminance
[luminance-derive]: https://crates.io/crates/luminance-derive
[luminance-front]: https://crates.io/crates/luminance-front
[luminance-webgl]: https://crates.io/crates/luminance-webgl
[luminance-gl]: https://crates.io/crates/luminance-webgl
[glutin]: https://crates.io/crates/glutin
[glfw]: https://crates.io/crates/glfw
