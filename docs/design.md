# Luminance design

This document describes the overall design of [luminance] starting from its current version, 0.46.

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
  * [Shaders](#shaders)
  * [Tessellation](#tessellation)
  * [Textures](#textures)
  * [Pipelines and gates](#pipelines-and-gates)
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
ecosystem, you can glance through the [/docs].

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

## Shaders

## Tessellation

## Textures

## Pipelines and gates

## Queries

[luminance]: https://crates.io/crates/luminance
[luminance-derive]: https://crates.io/crates/luminance-derive
[luminance-front]: https://crates.io/crates/luminance-front
[luminance-webgl]: https://crates.io/crates/luminance-webgl
[luminance-gl]: https://crates.io/crates/luminance-webgl
[glutin]: https://crates.io/crates/glutin
[glfw]: https://crates.io/crates/glfw
