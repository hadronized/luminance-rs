//! # A simple, type-safe and opinionated graphics crate
//!
//! luminance is an effort to make graphics rendering simple and elegant. It is a _low-level_
//! and opinionated graphics API, highly typed (type-level computations, refined types, etc.)
//! which aims to be simple and performant. Instead of providing users with as many low-level
//! features as possible, luminance provides you with _some ways_ to do rendering. That has
//! both advantages and drawbacks:
//!
//! - On one side, because the API is opinionated, some dynamic branching and decisions are
//!   completely removed / optimized. Some operations breaking state mutations or invariant
//!   violation are not statically constructible, ensuring safety. Because strong typing is
//!   used, lots of runtime checks are also not needed, helping with performance.
//! - On the other side, if you want to do something very specific and very low-level, you
//!   will find luminance not to be friendly as it doesn’t like, most of the time, exposing
//!   its internal design to the outer world — mostly for runtime safety reason.
//!
//! > A note on _safety_: here, _safety_ is not used as with the Rust definiton, but most in
//! > terms of undefined behavior and unwanted behavior. If something can lead to a weird
//! > behavior, a crash, a panic or a black screen, it’s considered `unsafe`. That definition
//! > obviously includes the Rust definiton of safety — memory safety.
//!
//! # What’s included?
//!
//! luminance is a rendering crate, not a 3D engine nor a video game framework. As so, it doesn’t
//! include specific concepts, such as lights, materials, asset management nor scene description. It
//! only provides a rendering library you can plug in whatever you want to.
//!
//! > There are several so-called 3D-engines out there on [crates.io](https://crates.io). Feel free
//! > to have a look around.
//!
//! However, luminance comes with several interesting features:
//!
//! - **Framebuffers**: framebuffers are used to hold renders. Each time you want to perform a
//!   render, you need to perform it into a framebuffer. Framebuffers can then be combined with
//!   each other to produce effects and design render layers — this is called compositing.
//! - **Shaders**: luminance supports five kinds of shader stages:
//!     - Vertex shaders.
//!     - Tessellation control shaders.
//!     - Tessellation evaluation shaders.
//!     - Geometry shaders.
//!     - Fragment shaders.
//! - **Vertices, indices, primitives and tessellations**: those are used to define a shape you
//!   can render into a framebuffer with a shader. They are mandatory when it comes to rendering.
//!   Even if you don’t need vertex data, you still need tessellations to issue draw calls.
//! - **Textures**: textures represent information packed into arrays on the GPU, and can be used
//!   to customize a visual aspect or pass information around in shaders. They come in several
//!   flavours — e.g. 1D, 2D, cube maps, etc.
//! - **Control on the render state**: the render state is a set of capabilities you can tweak
//!   to draw frames. It includes:
//!     - The blending equation and factors. Blending is the process of taking two colors from two
//!       framebuffers and mixing them.
//!     - Whether we should have a depth test performed.
//!     - Face culling.
//!     - Etc.
//! - And a lot of other cool things like *GPU commands*, *pipelines*, *uniform interfaces* and so
//!   on…
//!
//! # How to dig in?
//!
//! luminance is written to be fairly simple. There are several ways to learn how to use luminance:
//!
//! - The [online documentation](https://docs.rs/luminance) is a mandatory start for newcomers.
//! - The [“Learn luminance” book](https://phaazon.github.io/learn-luminance). Ideal for
//!   newcomers as well as people already used to luminance, as it’s always updated to the latest
//!   version — you might learn new things!
//! - The [luminance-examples](https://github.com/phaazon/luminance-rs/tree/master/examples)
//!   project. It contains lots of examples describing how to do specifics things. Not adapted for
//!   newcomers, you will likely want to consult those examples if you’re already familiar with
//!   graphics programing and to look for how to do a specific thing. Still, for newcomers, the
//!   [hello-world](../../examples/common/src/hello_world.rs) example might be a good read.
//!
//! # Implementation and architecture
//!
//! **luminance** has been originally designed around the OpenGL 3.3 and OpenGL 4.5 APIs. However,
//! it has mutated _a lot_ to adapt to new technologies and modern graphics programming. Even though its API
//! is _not_ meant to converge towards something like Vulkan, it’s changing over time to meet
//! better design decisions and performance implications.
//!
//! The current state of luminance comprises several crates:
//!
//! - A “core” crate, [luminance], which is about all the abstract, common and interface code.
//! - A proc-macro crate, [luminance-derive], which is exported by [luminance] if you use the `"derive"`
//!   feature flag. That crate allows to implement various important traits of the core crate.
//! - A set of _backend implementation_ crates, implementing the [luminance] crate backend interfaces.
//! - A set of _windowing_ crates, executing your code written with the core and backend crate on native
//!   systems (most of the time, _windowing platforms_, but not limited to).
//! - A special crate, [luminance-front], a special _backend_ crate that allows to combine
//!   several “official” crates to provide a cross-platform experience without having to pick
//!   several backend crates — the crate does it for you. This crate is mainly designed for end-user
//!   crates and should be a good fit for most users.
//!
//! ## The core crate
//!
//! The [luminance] crate gathers all the logic and rendering abstractions necessary to write code
//! over various graphics technologies. It contains parametric types and functions that abstract over
//! the actual _implementation type_ — as a convention, the type variable `B` (for backend) is
//! used.
//!
//! Backend types — i.e. `B` — are not provided by [luminance] directly. They are typically
//! provided by crates containing the name of the technology as suffix, such as [luminance-gl],
//! [luminance-webgl], luminance-vk, etc. The interface between those backend crates and
//! luminance is specified in [luminance::backend].
//!
//! On a general note, `Something<ConcreteType, u8>` is a monomorphic type that will be usable
//! **only** with code working over the `ConcreteType` backend. If you want to write a function
//! that accepts an 8-bit integer something without specifying a concrete type, you will have to
//! write something along the lines of:
//!
//! ```ignore
//! use luminance::backend::something::Something as SomethingBackend;
//! use luminance::something::Something;
//!
//! fn work<B>(b: &Something<B, u8>) where B: SomethingBackend<u8> {
//!   todo!();
//! }
//! ```
//!
//! This kind of code is intended for people writing libraries with luminance. For the more usual case
//! of using the [luminance-front] crate, you will end up writing something like:
//!
//! ```ignore
//! use luminance_front::something::Something;
//!
//! fn work(b: &Something<u8>) {
//!   todo()!;
//! }
//! ```
//!
//! > In [luminance-front], the backend type is selected at compile and link time. This is often
//! > what people want, but keep in mind that [luminance-front] doesn’t allow to have several
//! > backend types at the same time, which might be something you would like to use, too.
//!
//! ## Backend implementations
//!
//! Backends implement the [luminance::backend] traits and provide, mostly, a single type for each
//! implementation. It’s important to understand that a backend crate can provide several backends
//! (for instance, [luminance-gl] can provide one backend — so one type — for each supported OpenGL
//! version). That backend type will be used throughout the rest of the ecosystem to deduce subsequent
//! implementors and associated types.
//!
//! If you want to implement a backend, you don’t have to push any code to any `luminance` crate.
//! `luminance-*` crates are _official_ ones, but you can write your own backend as well. The
//! interface is highly `unsafe`, though, and based mostly on `unsafe impl` on `unsafe trait`. For
//! more information, feel free to read the documentation of the [luminance::backend] module.
//!
//! ## Windowing
//!
//! luminance doesn’t know anything about the context it executes in. That means that it doesn’t
//! know whether it’s used within SDL, GLFW, glutin, Qt, a web canvas or an embedded specific hardware such as
//! the Nintendo Switch. That is actually powerful, because it allows luminance to be
//! completely agnostic of the execution platform it’s running on: one problem less. However, there
//! is an important point to take into account: a single backend type can be used with several windowing
//! crates / implementations. That allows to re-use a backend with several windowing
//! implementations. The backend will typically explain what are the conditions to create it (like,
//! in OpenGL, the windowing crate must set some specific flags when creating the OpenGL context).
//!
//! luminance does not provide a way to create windows because it’s important that it not depend
//! on windowing libraries – so that end-users can use whatever they like. Furthermore, such
//! libraries typically implement windowing and event features, which have nothing to do with our
//! initial purpose.
//!
//! A platform crate supporting luminance will typically provide native types by re-exporting
//! symbols (types, functions, etc.) from a windowing crate and the necessary code to make it
//! compatible with luminance. That means providing a way to access a backend type, which
//! implements the [luminance::backend] interface. However, platform crates are not supposed to
//! be a replacement for the underlying platform system; you will typically still have to depend
//! it as well.
//!
//! ## luminance-derive
//!
//! If you are compiling against the `"derive"` feature, you get access to [`luminance-derive`] automatically, which
//! provides a set of _procedural macros_.
//!
//! ### `Vertex`
//!
//! The [`Vertex`] derive proc-macro.
//!
//! That proc-macro allows you to create custom vertex types easily without having to care about
//! implementing the required traits.
//!
//! The [`Vertex`] trait must be implemented if you want to use a type as vertex (consumed by [`Tess`]).
//! Either you can decide to implement it on your own, or you could just let this crate do the job for you.
//!
//! > Important: the [`Vertex`] trait is `unsafe`, which means that all of its implementors must be
//! > as well. This is due to the fact that vertex formats include information about the structure of the
//! > data that will be sent to the backend, and a bad implementation can lead to undefined behaviors.
//!
//! You can derive the [`Vertex`] trait if your type follows these conditions:
//!
//! - It must be a `struct` with named fields. This is just a temporary limitation that will get
//!   dropped as soon as the crate is stable enough.
//! - Its fields must have a type that implements [`VertexAttrib`]. This is mandatory so that the
//!   backend knows enough about the types used in the structure to correctly align memory, pick
//!   the right types, etc.
//! - Its fields must have a type that implements [`HasSemantics`] as well. This trait is just a
//!   type family that associates a single constant (i.e. the semantics) that the vertex attribute
//!   uses.
//! - Each field's type must be different.
//!
//! Once all those requirements are met, you can derive [`Vertex`] pretty easily.
//!
//! > Note: feel free to look at the [`Semantics`] proc-macro as well, that provides a way
//! > to generate semantics types in order to completely both implement [`Semantics`] for an
//! > `enum` of your choice, but also generate *field* types you can use when defining your vertex
//! > type.
//!
//! The syntax is the following:
//!
//! ```rust
//! use luminance::{Vertex, Semantics};
//!
//! // visit the Semantics proc-macro documentation for further details
//! #[derive(Clone, Copy, Debug, PartialEq, Semantics)]
//! pub enum Semantics {
//!   #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
//!   Position,
//!   #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
//!   Color,
//! }
//!
//! #[derive(Clone, Copy, Debug, PartialEq, Vertex)]
//! #[vertex(sem = "Semantics")] // specify the semantics to use for this type
//! struct MyVertex {
//!   position: VertexPosition,
//!   color: VertexColor,
//! }
//! ```
//!
//! > Note: the `Semantics` enum must be public because of the implementation of [`HasSemantics`]
//! > trait.
//!
//! Besides the [`Semantics`]-related code, this will:
//!
//! - Create a type called `MyVertex`, a struct that will hold a single vertex.
//! - Implement `Vertex for MyVertex`.
//!
//! The proc-macro also supports an optional `#[vertex(instanced = "<bool>")]` struct attribute.
//! This attribute allows you to specify whether the fields are to be instanced or not. For more
//! about that, have a look at [`VertexInstancing`].
//!
//! ### `Semantics`
//!
//! The [`Semantics`] derive proc-macro.
//!
//! ### `UniformInterface`
//!
//! The [`UniformInterface`] derive proc-macro.
//!
//! The procedural macro is very simple to use. You declare a struct as you would normally do:
//!
//! ```
//! use luminance::{shader::{types::Vec4, Uniform}, UniformInterface};
//!
//! #[derive(Debug, UniformInterface)]
//! struct MyIface {
//!   time: Uniform<f32>,
//!   resolution: Uniform<Vec4<f32>>,
//! }
//! ```
//!
//! The effect of this declaration is declaring the `MyIface` struct along with an effective
//! implementation of `UniformInterface` that will try to get the `"time"` and `"resolution"`
//! uniforms in the corresponding shader program. If any of the two uniforms fails to map (inactive
//! uniform, for instance), the whole struct cannot be generated, and an error is arisen (see
//! `UniformInterface::uniform_interface`’s documentation for further details).
//!
//! If you don’t use a parameter in your shader, you might not want the whole interface to fail
//! building if that parameter cannot be mapped. You can do that via the `#[unbound]` field
//! attribute:
//!
//! ```
//! # use luminance::{shader::{types::Vec4, Uniform}, UniformInterface};
//! #[derive(Debug, UniformInterface)]
//! struct MyIface {
//!   #[uniform(unbound)]
//!   time: Uniform<f32>, // if this field cannot be mapped, it’ll be ignored
//!   resolution: Uniform<Vec4<f32>>,
//! }
//! ```
//!
//! You can also change the default mapping with the `#[uniform(name = "string_mapping")]`
//! attribute. This changes the name that must be queried from the shader program for the mapping
//! to be complete:
//!
//! ```
//! # use luminance::{shader::{types::Vec4, Uniform}, UniformInterface};
//! #[derive(Debug, UniformInterface)]
//! struct MyIface {
//!   time: Uniform<f32>,
//!   #[uniform(name = "res")]
//!   resolution: Uniform<Vec4<f32>>, // maps "res" from the shader program
//! }
//! ```
//!
//! Finally, you can mix both attributes if you want to change the mapping and have an unbound
//! uniform if it cannot be mapped:
//!
//! ```
//! # use luminance::{shader::{types::Vec4, Uniform}, UniformInterface};
//! #[derive(Debug, UniformInterface)]
//! struct MyIface {
//!   time: Uniform<f32>,
//!   #[uniform(name = "res", unbound)]
//!   resolution: Uniform<Vec4<f32>>,
//! }
//! ```
//!
//! [luminance]: https://crates.io/crates/luminance
//! [luminance-gl]: https://crates.io/crates/luminance-gl
//! [luminance-front]: https://crates.io/crates/luminance-front
//! [luminance::backend]: crate::backend
//! [`Semantics`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Semantics.html
//! [`HasSemantics`]: https://docs.rs/luminance/latest/luminance/vertex/trait.HasSemantics.html
//! [`Tess`]: https://docs.rs/luminance/latest/luminance/tess/struct.Tess.html
//! [`Vertex`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Vertex.html
//! [`VertexAttrib`]: https://docs.rs/luminance/latest/luminance/vertex/trait.VertexAttrib.html
//! [`VertexInstancing`]: https://docs.rs/luminance/latest/luminance/vertex/enum.VertexInstancing.html
//! [`UniformInterface`]: https://docs.rs/luminance/latest/luminance/shader/program/trait.UniformInterface.html

#![doc(
  html_logo_url = "https://raw.githubusercontent.com/phaazon/luminance-rs/master/docs/imgs/luminance_alt.svg"
)]
#![deny(missing_docs)]

#[cfg(feature = "derive")]
pub use luminance_derive::*;

pub mod backend;
pub mod blending;
pub mod context;
pub mod depth_stencil;
pub mod face_culling;
pub mod framebuffer;
pub mod pipeline;
pub mod pixel;
pub mod query;
pub mod render_gate;
pub mod render_state;
pub mod scissor;
pub mod shader;
pub mod shading_gate;
pub mod tess;
pub mod tess_gate;
pub mod texture;
pub mod vertex;
