# luminance-1.0: Design Draft

This document is an exhaustive description of `luminance-1.0` along the axis of its design. It
provides information about the internal architecture and the public API yielded by such a design.


<!-- vim-markdown-toc GFM -->

* [Context & Problems](#context--problems)
* [Existing solutions](#existing-solutions)
* [Design](#design)
  * [Ecosystem](#ecosystem)
  * [Vertex](#vertex)
    * [Vertex semantics](#vertex-semantics)
  * [Tessellations](#tessellations)
    * [Attributeless tessellations](#attributeless-tessellations)
    * [Vertex instancing](#vertex-instancing)
    * [Geometry instancing](#geometry-instancing)
    * [Slicing](#slicing)
    * [Buffer mapping](#buffer-mapping)
  * [Shaders](#shaders)
    * [Customizing shaders with uniform interfaces](#customizing-shaders-with-uniform-interfaces)
    * [Adapting and re-adapting shader programs](#adapting-and-re-adapting-shader-programs)
  * [Framebuffers](#framebuffers)
  * [Textures](#textures)
    * [Pixels](#pixels)
  * [Graphics pipelines](#graphics-pipelines)
  * [The backend architecture](#the-backend-architecture)
    * [On distributing implementations](#on-distributing-implementations)
* [Rationale](#rationale)
  * [Release candidates](#release-candidates)
* [Unresolved questions](#unresolved-questions)

<!-- vim-markdown-toc -->

## Context & Problems

[luminance] is a graphics library that tackles the problem of common graphics APIs and more
especially [OpenGL]. Those APIs suffer from a set of problems that are not trivially removed:

  - Most build upon the C programming language, which is an unsafe, low-level systems programming
    language. OpenGL, for instance, is often distributed as a set of C header files and relocatable
    dynamic libraries (`.dll` / `.so` / `.dylib` / etc.). Because C lacks a good type system, it is
    pretty easy to mess things up.
  - It’s possible to generate bindings (often unsafe) in other languages by using generators (for
    OpenGL, Khronos gives a `gl.xml` file that provides everything needed to generate bindings).
    However, since OpenGL is loosely typed (i.e. all objects are hidden behind types like
    `GLuint` or `GLenum`, which makes it very hard not to confuse two objects’ types, for instance).
    it leads to runtime problems with unneeded checks (i.e. check that a value has the right type
    type, for instance) — if you don’t check types, you might end up with really weird behaviors.
  - The global state is something that makes an **OpenGL** function context-sensitive. It means that
    you always need to be sure that the right objects are bound and that you’re calling the function
    at the right moment. This is a very bad situation for pretty much any developer who would like
    to provide a more explicit and easy experience to their end-users.

[luminance] tackles these problems by providing a type-safe, memory-safe and stateless crate. All
the global state handling is done internally and a public interface is exposed in a way the user
doesn’t know they actually use a stateful API.

Another situation arose: how do we compile our Rust code to run on WASM, or on Android?

Finally, we would like [luminance] to be able to target several versions of OpenGL and, if possible,
[Vulkan] too, as it becomes (has become? :)) a standard in the graphics industry.

In order to solve all those problems, `luminance-1.0` is to be released. The current version of
[luminance] (pre-1.0) already solves lots of problems above, but some are missing, such as the
multi-platform support.

## Existing solutions

Some solutions already exist that solve all those problems. The most famous one being [gfx]. [gfx]
is, currently, a set of crates:

  - [gfx-hal]: the *hardware abstraction layer* of [gfx].
  - `gfx-backend-*`: several backends for the [gfx-hal] crate.
  - [gfx-warden]: data-driven reference test framework.

This crate was designed with a specific need for performances, low-level primitives and
cross-platform concerns. The [gfx] ecosystem is quite bigger than [luminance] in both its scope and
its community. [gfx] is, from what it looks like from the samples, lower level than [luminance] and
is a thiner layer over the different graphics APIs than [luminance]. [luminance] is more opinionated
in its design.

The important part is that [luminance] originated from the [Haskell version] in 2015, when I decided
to extract it from [quaazar](https://github.com/phaazon/quaazar), a demoscene engine, long
abandoned since. The implication is that [luminance]’s designs are way different than e.g. [gfx].
[luminance] tries to solve simple problems for lightweight applications (i.e. demoscene
productions), even though currently, [luminance] has only been used with *demos* and not *intros*
(typically 64k).

## Design

The core concepts of [luminance] are described in this section. Most of the concepts have been around
for several months or even years (most of them come from the [Haskell version], rethought to work
efficiently in Rust).

> Disclaimer: the following content describes the `1.0` version of [luminance].

### Ecosystem

[luminance] is a set of crates, each having a specific scope:

- [luminance]: this crate is the most abstract and serves as interface. It contains all the design
  code that makes luminance what it is. You will find several common and concrete symbols too, like
  pixel types and formats, texture dimensions, traits, vertex definitions, etc.
- [luminance-derive]: an optional yet important companion crate. This crate helps people implement
  several traits from [luminance] by exposing _derive proc-macros_.
- [luminance-windowing]: an interoperability library for windowing purposes. It exports several
  simple and naive type to open windows and handle events. It covers the most basics needs so it
  might not always be suited for everything you do.
- [luminance-glfw]: GLFW support for [luminance] and [luminance-windowing].
- [luminance-glutin]: Glutin support for [luminance] and [luminance-windowing].
- [luminance-gl]: OpenGL implementation of [luminance].
- [luminance-gles]: OpenGL ES implementation of [luminance].
- [luminance-webgl]: WebGL implementation of [luminance].

The current design allows a new crate to implement traits exposed in [luminance] to support a new
graphics backend. That’s what [luminance-gl] and [luminance-webgl] do, for instance. As another
example, we could write a [luminance-vk] to support Vulkan at some point, too.

“Using luminance” really just means choosing a backend, for instance [luminance-gles], and using it
along with [luminance] to build graphics code. To actually run the application, one needs to create
a rendering surface to handle both renders and system events. Either this is done manually or via
[luminance-windowing] and a crate implementing it, for instance [luminance-glfw].

> It is important to understand that [luminance] is just about graphics code, not windowing code.
> This clear separation allows to “execute” luminance code whatever the windowing setup. A contract
> exists between what is expected by luminance and the windowing setup, but this will be detailed
> later in this document. The important point is that you can implement that contract on your own
> (and at your own risks) if you want to.

### Vertex

[luminance] allows you to create objects made out of geometrical points. Those points have *a
structure* that is not enforced by [luminance]: it is on the user of to define such a structure. A
trait — the `Vertex` trait — is responsible for ensuring that the structure is correct. Because
implementing this trait is `unsafe` and not as trivial as you might think, you’re highly
advised to use the `Vertex` _derive proc-macro_ available via the [luminance-derive] crate.

Basically, `struct` types are typically the ones you will be using if you use the [luminance-derive]
crate. Each field must use a special type that implements the `Semantics` trait (more on this in the
next section). Your `struct` must be tagged with the appropriate `Vertex` annotation to bind it to a
semantics. This is needed so that [luminance] can perform compile-time check between your vertex
type and the vertex semantics it refers to, but also to map your structure in shader stages, when you
send geometrical data down a graphics pipeline.

```rust
#[derive(Vertex)]
#[vertex(sem = "SomeSemantics")] // assumption made that we have a semantics called SomeSemantics in scope
pub struct Vertex {
  pos: VertexPosition, // VertexPosition must be in scope
  rgb: VertexColor // same for VertexColor
}
```

> Note: `SomeSemantics`, `VertexPosition` and `VertexColor` are not provided by [luminance]. How to
> create such types is described in the next section.

When using a value like `a_vertex: Vertex`, [luminance] has lots of information about how it should:

  - Store that vertex / several vertices into a GPU memory region called *a buffer*.
  - Thanks to the semantics mapping, it also knows how to read vertices’ attributes from GPU
    buffers.
  - Still thanks to semantics, it also knows how to fetch the data from shader stages without any
    special interaction or knowledge about memory from the perspective of the user writing the
    shader. That allows for a way more composable experience with writing shaders.

The [luminance-derive] proc-macro `Vertex` will also create several methods to help you build
vertices. Feel free to have a look at the documentation of the `Vertex` proc-macro.

Instancing is also supported via a special property set on the semantics mapping. See the
[Vertex instancing](#vertex-instancing) section for further details.

#### Vertex semantics

[luminance] revolves around the concept of *vertex semantics*. Semantics are pretty central to the
whole design of `luminance-1.0` because, as seen in the previous section, they provide enough
information and coherency for client to GPU, GPU memory region and shaders to work altogether in
harmony without sacrificing on composition.

Vertex semantics are introduced via the `Semantics` trait. This trait defines the interface of
vertex semantics. Again, you don’t have to worry too much about implementing it if you are using the
[luminance-derive] crate, but if you really want to implement it by hand, you will have to ensure
you implement it correctly. Especially, each semantics should be mapped to a _unique index_
(`usize`), have a unique name (`&'static str`) and you must be able to provide a set of every
possible values semantics can have (`Vec<SemanticDesc>`). Violating one of the uniqueness rules
described above or not providing all the possible values in the semantics set will result in
incorrect setup of either GPU memory buffers or shader variables fetches.

The idea is the following: you create an `enum` that is to be used everywhere in your application /
library. You don’t have to follow this rule but if you do, it will make everything way easier. That
`enum`’s variants are semantics. Use the `Semantics` proc-macro to automatically implement the
`Semantics` trait for your `enum`.

Each variant is tagged with several important properties:

  - `name`: the name of the semantics. This is the name that will represent the semantics in, e.g.,
    shader stage sources. It must be unique.
  - `repr`: the type that this semantics represents. In [luminance], everything is strongly-typed,
    even semantics. This type must implement `Vertex`.
  - `wrapper`: the name to use for the wrapper type for `repr`. You will not directly use values of
    type `repr` but `wrapper` instead. Those wrapper types are thin wrappers, implement `Deref` for
    `repr` and comes with several easy methods to build and work with them.

Here is an example of a perfectly fine and usable semantics.

```rust
#[derive(Semantics)]
pub enum SomeSemantics {
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
  // reference vertex colors with the color variable in vertex shaders
  #[sem(name = "color", repr = "[f32; 3]", wrapper = "VertexColor")]
  Color
}
```

Semantics types are used to *phantom tag* other objects, such as *framebuffers* and *shaders* in
order to remove the user from implementing and explictly defining mappings.

The `HasSemantics` trait is implemented for all the wrapper types and is required so that a vertex
attribute type is accepted by [luminance]. This is done for you automatically if you use
[luminance-derive]. If you don’t, you simply have to provide a single constant value for your type,
mapping your type to a single value that gives the semantics of that attribute type.

### Tessellations

Tessellations have been supported since the beginning of [luminance] because they’re currently the
single way to initiate a render. A tessellation is an object that gathers several concepts required
for rendering:

- Some vertices, or none. Vertices are stored in *buffers*. If you decide not to use any buffer
  and then are running without vertices, you can still use a tessellation to perform a render.
  See the [Attributeless tessellations](#attributeless-tessellations) section for further details.
- For indexed rendering only, some indices. Indices are used to construct primitives by indexing
  the buffers of vertices. If you don’t use any, it is assumed you want a non-indexed, standard
  rendering.
- A *primitive mode*, which is simply a mode of connecting vertices to each other. You can find
  *points* (no connection), *lines* (two by two), *line strips* (continuous lines), *triangles*,
  etc.
- Optionally some vertex instancing attributes. See the [Vertex instancing](#vertex-instancing)
  chapter for further details.
- The default number of vertices to render. That allows [luminance] to be able to render a *whole*
  tessellation without requiring you to keep track of the number of vertices inside the
  tessellation.
- The default number of indices to render. Same thing as above.
- The primitive restart index. In case of indexed rendering, this special value is used to notify
  the graphics pipeline when to break a *strip* or *fan* primitive to start a new primitive.

Tessellations are built via the [builder pattern] and heavily depend on the vertex definition of
the previous section. It is not possible, by default, to pass every type you want as geometrical.
You must use a vertex-compatible type.

#### Attributeless tessellations

Some tessellations are special as they have no vertices and no indices. It might feel strange at
first but this is a perfectly fine and desirable situation. We call those tessellations
*attributeless* tessellations, meaning that they don’t have attributes for the points you want to
render. However, they still have points. Imagine an attributeless tessellation made of 32 points.
Those points are akin to 32 units (`()`). However, since you can easily guess which vertex is being
rendered when you are in your vertex shader — via its index, provided by the graphics pipeline when
running a shader — you can “spawn” attributes out of the void by hardcoding them in shader stages.

This feature is very frequent in [demoscene] when programming *intros* or *demos*. They allow you
to render simple or implicit geometry without having to fill your GPU memory.

You typically use attributeless tessellations by not providing any vertex and index and
setting both the primitive mode and the number of vertices to render.

#### Vertex instancing

Tessellations accept additional and special buffers: *vertex instancing buffers*. Those are buffers
of values which types must be vertex-compatible too, but they are quite different as they are used as
properties for instances, not vertices. Keep in mind that the vertex type you use must be mapped to
the same semantics as the one used for regular vertices as they share the same indexing space.

#### Geometry instancing

Geometry instancing is performed by allowing you to render several instances of your tessellation at
render time. It is then your responsibility to customize your shader to take instancing into
account. As with _attributelless tessellations_ where the graphics pipeline gives you the index of
each vertex being treated, geometry instancing has the graphics pipeline gives you the index of the
instancing being rendered. You can use that index to lookup indexed property to customize each
instances.

#### Slicing

Tessellations by default are not very useful for the graphics pipeline. In order to “render” a
tessellation, a graphics pipeline needs to know which part of it you want to render. You can render
the whole tessellation, only a few vertices at the beginning, at the end, etc.

That concept is called a `TessSlice` and obtaining one is a cheap and easy operation. It’s highly
encouraged to use the standard operators for representing ranges (`..`, `a ..`, `.. b`, `a .. b`,
`a ..= b` and `..= b`).

> For convenience purposes, `&Tess` can be converted to `TessSlice`.

That feature is useful to pack several objects into a single GPU memory buffer and render them by
slicing the whole buffer.

#### Buffer mapping

It is possible to retrieve immutable and mutable slices of buffers (both vertices and instances’
data as well as index buffers). However, it is only possible to do it if you use interleaved memory
and you don’t use attributeless rendering.

Buffer mapping is often needed when you want to update a tessellation on the fly with user inputs,
for instance. Instead of recreating a tessellation on the fly, it’s faster to map buffers and change
their contents.

By combining several features, such as buffer mapping and slicing, you can implement a tessellation
which the number of points is dynamic.

### Shaders

Shaders in [luminance] are not really special: they gather *shader stages* by compiling them from
sources (typically strings) and link them into *shader programs*. They are strongly typed with
three type variables:

- The _uniform interface_ type, explain below.
- The _vertex semantics_. As explained in the vertex chapter, that type allows to know how to map
  vertex attributes (e.g. in GLSL, `in` declared variables) to actual GPU memory buffers without
  having to explicitly state it in the shader source (e.g. `layout (location = X)` is not needed!).
- The _render targets_ type. That type allows to state exactly what the shader program will output
  to. It’s akin to _uniform interface_ but serves as connectivity with _framebuffers_.

Shaders cannot be used directly and require the use of [Graphics pipelines](#graphics-pipelines).

#### Customizing shaders with uniform interfaces

Currently, shaders are parametered by several types. One of them is called the *uniform interface*
as it defines an interface between the GPU shader object and your code. This interface is
*contravariant*: it will be held for you until you actually run a *graphics pipeline* with the
given shader program. At this point, you will have access to the interface and will be in position
to update GPU’s shader memory variables (e.g. `uniform` in GLSL).

Uniform interfaces are built at shader program creation. This allows for several optimizations to be
done, such as caching graphics calls to prevent fetching data from the GPU every time you want to
update a uniform value or for ease purposes, such as remapping indexes (i.e. `layout (location = X)`
in GLSL is not needed because [luminance] takes care of that for you thanks to _vertex semantics_).

It is also important to note that a special exception exists to update uniforms on the GPU:
*dynamic uniform lookups*. Those allow you to lookup a uniform variable without passing through the
interface. That has obviously a runtime cost and the current API is very explicit about it. It’s
also a fallible operation as type mismatch is checked. However, you might want to use that feature
when dealing with unknown (at compile-time) uniform interfaces, such as material editors or GUIs,
for instance.

Uniform interface types must implement the `UniformInterface` trait but, again, you’re strongly
advised to use [luminance-derive] and the _derive proc-macro_ `UniformInterface` to do it with
`struct` automatically. Each `Uniform` variables can be name-remapped but also be _unbound_,
meaning that if the uniform interface fails to map them (because they don’t exist on the shader
program side or are inactive), they will be silently ignored and set to a no-op state.

#### Adapting and re-adapting shader programs

Once a shader program is built, you cannot mutate it anymore — i.e. add shader stages, link again,
perform uniform interface regeneration, etc. That might be limiting in some specific and niche
cases, so there exists two methods coming in different flavours to avoid having to recreate a shader
program: adaptation.

Adaptation is what to seek when you want to recreate a uniform interface without having to recompile
and link the shader program. Because of the nature of uniform interfaces, it’s possible to switch
from one type to another by using adaptation and re-adaptation feature.

### Framebuffers

Framebuffers play an import role in [luminance]. They are rendered images into by graphics
pipelines. Framebuffers, in [luminance], have several important properties:

  - Their dimensions and layer property. The most common kind of framebuffer is a 2D, flat
    framebuffer, which is basically an object holding a 2D surface of pixels. Layering is an
    advanced feature allowing to create layers inside a framebuffer.
  - A *color slot*. A color slot is typically a texture or a set of several textures that represent
    render targets. It’s also possible to find framebuffers with the `()` color slot: those are
    special framebuffers that won’t get anything explicitly output to from shaders. They can be
    useful to implement some special techniques that require depths information only, for instance.
  - A *depth slot*. A depth slot can either be nothing or a texture and plays the same role as a
    *color slot*, but for depth information flowing through the graphics pipeline.

As for with shaders, you cannot use a framebuffer directly and are required to use it inside a
graphics pipeline.

The typical usage of framebuffers is to render into your scene and retrieve rendered pixels via
color or depth slots.

### Textures

Textures are a way to store images, composed of pixels. Several types of images are available,
defined by:

  - The pixel format (RGB, RGBA, 32-bit, 8-bit, more exotic, etc.).
  - The layout (a flat image or a layered texture).
  - The dimension (1D, 2D, 3D, cubemap, etc.).

Textures play an important role in customizing the visual aspect of your application. You can use
them in _uniform interfaces_ to customize renders or you can ask access to them via _framebuffers_.

#### Pixels

Pixels have received a special attention in [luminance]. They are represented with several types and
traits, giving enough information about them to unlock several features in shaders and textures.

First, pixel types must implement the `Pixel` trait, reifying format data. The format of a pixel is
a gathering of several information:

- Its encoding format. That gives information about the number of channels are required to
  represent such a pixel as well as, for each channel, the number of bits of information.
- Its type. The type serves as how the bits should be interpreted. For instance, an 8-bit RGB pixel
  format is not enough to know how we should interpret those three channels, because it could be
  8-bit signed integers, 8-bit unsigned integers, etc. The type is also used as a tag to allow
  special pixel formats to exist, such as _normalized_ pixel formats, which store data as integers
  but expose them as floating point value when sampled from a shader stage.
- Its raw encoding, which is an associated Rust type allowing for interoperability when uploading
  data to a texture efficiently or retrieving from external sources (e.g. the
  [image](https://crates.io/crates/image) crate).
- Its sampler type, which allows to create uniform interfaces that will work with several flavours
  of pixels without having to create one interface for each.

Pixel types are used in several places: framebuffers _color slots_ and _depth slots_, textures,
shader uniform interfaces, etc.

### Graphics pipelines

The most opinionated part of [luminance] might be the way *graphics pipelines* are handled. The idea
showed up in the [Haskell version] first: a graphics pipeline is like a dependency graph. In that
dependency graph, some resources, in order to be used or rendered to or whatever, need other
resources to be available, done or in use too.

When [luminance] was imagined, it was using *OpenGL 3.3* in mind (even older and younger versions in
the [Haskell version], ranging from 2.0 to 4.5). Within such a graphics API, resources need to be
available (bound) while some other resources are performing some work. For instance, in order to
render, you need a shader and a framebuffer to be bound. But you might also imagine that you will
want to use several shaders (to perform several kind of renders) into the same framebuffer. The other
way around is also possible: you might want to render with the same shader into several, different
framebuffers.

In [luminance], the control-flow of your code is used as a dependency graph. Because a typical code
AST has scopes, scopes are used to naturally implement scoped resource handling. What it means is
that when you start using a framebuffer, you are given an object that will live until you don’t actually
need it anymore, creating a virtual scope for you to play with one or several shaders.

You typically have this hierarchy of scopes:

```
create pipeline {
  use framebuffer {
    use shader {
      use render state {
        render tessellations
      }
    }
  }
}
```

Going from one scope to a nested one requires some mechanisms and concepts to understand. First,
the pipeline creation is a pretty *free* operation: it just prepares [luminance]’s internal state to
observe the graphics pipeline. Creating a pipeline binds it to a framebuffer, so the framebuffer is
the highest item in our dependency graph. If you want to make a render that juggles with several
framebuffers, it is currently not possible to do so with a graphics pipeline: you need to create
at least two and play around with the *color slots* and/or *depth slots* of the framebuffers.

> That last situation might change in the future.

Introducing a render with a shader first allows to share shaders for tessellations. A shader will
shade future renders into the framebuffer you used to create the pipeline object. Using a pipeline
creates a scoped object called a *shading gate*. Shading gates are the only way to create subsequent
renders. Shading gates also provide you with your *uniform interface* via a *query* object. That query
object allows you to access the uniform interface to customize your shader’s behavior or dynamically
change uniform values if you prefer.

Entering a graphics render state via shading gates allows to batch tessellations by render state,
limiting the number of changes the GPU will have to do. Shading will give you a *render gate*,
allowing to create scoped render state renders.

Render gates, when used, provide you with a *tessellation gate*, which is the latest object in the
dependency graph you will use. A tessellation gate is used to render tessellations by slicing them
via the use of *tessellation slices*.

All the gate objects are passed to subsequent part of the dependency graph via arguments to lambdas.
That allows lifetimes to be captured and prevents you to leak objects out and break the unsafe
graphics API.

At any point, you can use special extern objects in your pipeline. Tessellations are handled at the
very bottom with a *tessellation gate*, but you can customize every other part of the pipeline by
placing some calls at any place you want. Those objects are currently:

  - Textures: you can *bind* textures, yielding *bound textures*, that are also scoped.
  - Buffers: you can *bind* buffers, yielding *bound buffers*, scoped too.

Those resources can then be used to customize renders from anywhere in the dependency graph.

### The backend architecture

The backend architecture is a new feature of [luminance] that is the most important change in the
1.0 version. Drivers are a way to abstract everything that was discussed so far behind a simpler,
type-unsafe and type-erased interface, in order to implement all the features discussed so far.

The way drivers work is the following: every namespace of feature is assigned a trait, that must be
implemented to use the feature. For instance, tessellations are set behind the `TessDriver` trait.
Buffers are behind the `BufferDriver`. Some driver traits have other driver traits as super traits.
Etc., etc.

The design and choice of the public API for those traits is highly inspired by *OpenGL* and thus, it
is very likely that it will clash with some graphics API (such as *Vulkan*, which has different
assumptions than *OpenGL*, especially about memory safety and synchronization). However, it is very
probable that it be sufficient to implement [luminance] for several versions of *OpenGL* (among the
latest and *WebGL* / *OpenGL ES*, which are required and wanted supports).

Drivers are typically summed to yield a big `Driver` trait that allows to use the whole [luminance]
API. Because drivers need to implement several traits, a type must be available (and unique) for
each implementation. For instance, [luminance] over *OpenGL 3.3* uses the `GL33` opaque type, which
holds an internal state of the *OpenGL* global context for you.

All those traits and methods are `unsafe` because they rely on lots of assumptions and implementing
them is not trivial.

#### On distributing implementations

If you know [gfx] a bit, you will have noticed that they have chosen to have a core library (a.k.a.
[gfx-hal]) and several `gfx-backend-*` crates that provide [gfx-hal] support for technologies such
as *OpenGL*, *Metal*, *Vulkan*, etc. [luminance] doesn’t work like that. Instead, graphics APIs are
implemented directly in the [luminance] crate and hidden behind feature-gates. That is done this
way for a simple reason: those implementations might use shared code / common primitives that must
not appear in the public interface.

> Important note: see the [Unresolved questions](#unresolved-questions) section part as that chapter
> is not clear enough.

## Rationale

It was imagined not to go this way and stay around *OpenGL 3.3*. However, technology evolves and
use cases as well. Most people today expect to be able to run their code on several platforms, such
as smartphones and web. Making [luminance] able to go this way removes people the need to care about
using other graphics crates for other platforms other than Linux / Mac OSX / Windows.

The main change to [luminance] is the *driver architecture*, that will help with explicitly making
[luminance] polymorphic and thus adapt to any technology that can implement [luminance]. It is
currently a huge assumption. Some people, on IRC and/or Reddit, commented on the possibility that
[luminance] gets a *Vulkan* backend “a line of reasoning to be rather naive” and similar lines. The
purpose of this very document is not to discuss whether that comment was well-founded but to remove
the blurred lines. [luminance]’s aims are to be an easy library to use. The public API was formed
years ago in the [Haskell version] and has changed quite a bit since then. However, the API is
pleasant to use and, despite some parts of the crate that must be changed a bit, will not change
because a technology is intended to be used differently. If using *Vulkan* is not compatible with
the current design of [luminance] *for various reasons*, the priorities are as follow:

  1. Tear the internal driver interface to make implementing *Vulkan* a possible reality.
  2. If still struggling, add more layered code. What it means is that [luminance] will never have
     the same API as *Vulkan*. As a comparison, [gfx-hal] has an API that was based on *Vulkan*, so
     it is *possible* to implement *Vulkan* for [gfx-hal]. However, that yields a complicated and
     not-so-easy API (the [wgpu-native] crate exists to abstract over [gfx-hal]). [luminance]’s
     goals are to implement the current [luminance] API for any λ graphics technology. It has the
     implication that not every part of each API will be available: you will have **only have
     access** to the common denominator.
  3. If it’s impossible to have a *Vulkan* backend, break the public API to make it more
     *Vulkan*-friendly.

### Release candidates

Before releasing the `luminance-1.0` crate, a beta version of the form of a *release candidate* will
be available for some weeks. I plan to use it to make a demo (and why not an *intro* if I can
migrate an experimental `no_std` driver). The release candidates will have the following form:

```
luminance-1.0-rc.N
```

Where `N` lies in `[1..]`. Anything can happen between two release candidates but the idea is to
release the first one as soon as everything is stabilized enough. Incrementing the release candidate
will occur only:

  - In presence of bugs.
  - If the public API has a flaw or must be broken for important enhancement purposes.

## Unresolved questions

  - The [On distributing implementations](#on-distributing-implementations) section is not
    completely certain and needs refinement. Maybe it’s acceptable to have a public `unsafe` API for
    other crates to implement `luminance`. Plus, `luminance-gl` already exists! — it is currently
    deprecated but I still have the rights to push to it.

[luminance]: https://crates.io/crates/luminance
[luminance-derive]: https://crates.io/crates/luminance-derive
[luminance-gl]: https://crates.io/crates/luminance-gl
[luminance-gles]: https://crates.io/crates/luminance-gl
[luminance-glfw]: https://crates.io/crates/luminance-glfw
[luminance-glutin]: https://crates.io/crates/luminance-glutin
[luminance-windowing]: https://crates.io/crates/luminance-windowing
[luminance-webgl]: https://crates.io/crates/luminance-windowing
[OpenGL]: https://www.opengl.org
[Vulkan]: https://www.khronos.org/vulkan
[gfx]: https://crates.io/crates/gfx
[gfx-hal]: https://crates.io/crates/gfx-hal
[gfx-warden]: https://crates.io/crates/gfx-warden
[luminance-0.30.1]: https://crates.io/crates/luminance/0.30.1
[builder pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
[demoscene]: https://en.wikipedia.org/wiki/Demoscene
[Haskell version]: https://hackage.haskell.org/package/luminance
[wgpu-native]: https://crates.io/crates/wgpu-native
