//! # What is this?
//!
//! [![crates.io](https://img.shields.io/crates/v/luminance.svg)](https://crates.io/crates/luminance)
//! ![License](https://img.shields.io/badge/license-BSD3-blue.svg?style=flat)
//!
//! `luminance` is an effort to make graphics rendering simple and elegant.
//!
//! The aims of `luminance` are:
//!
//!   - Making the **unsafe** and **stateful** **OpenGL** API **safe** and **stateless**.
//!   - Providing a simple and easy interface; that is, exposing core concepts without anything extra – just
//!     the bare stuff.
//!   - Easy to read with a good documentation and set of tutorials, so that new comers don’t have
//! to learn a lot of new concepts to get their feet wet.
//!   - Need-driven: every piece of code added in the project must come from a real use case. If you feel
//!     something is missing, feel free to open an issue or even contribute!
//!
//! # What’s included?
//!
//! `luminance` is a rendering framework, not a 3D engine nor a video game framework. As so, it
//! doesn’t include specifics like lights, materials, asset management nor scene description. It
//! only provides a rendering library you can plug in whatever you want to.
//!
//!   > There are several so-called 3D-engines out there on [crates.io](https://crates.io). Feel free
//!   > to have a look around.
//!
//! ## Features set
//!
//!   - **Buffers**: buffers are way to communicate with the GPU; they represent regions of memory you can
//!     write to and read from. There’re several kinds of buffers you can create, among *vertex and index
//!     buffers*, *shader buffers*, *uniform buffers*, and so on and so forth….
//!   - **Framebuffers**: framebuffers are used to hold renders. Each time you want to perform a render, you
//!     need to perform it into a framebuffer. Framebuffers can then be combined with each other to produce
//!     effects.
//!   - **Shaders**: `luminance` supports five kinds of shader stages: + Tessellation control shaders. +
//!     Tessellation evaluation shaders. + Vertex shaders. + Geometry shaders. + Fragment shaders.
//!   - **Vertices, indices, primitives and tessellations**: those are used to define a shape you can render
//!     into a framebuffer.
//!   - **Textures**: textures represent information packed into arrays on the GPU, and can be used to
//!     customize a visual aspect or pass information around in shaders.
//!   - **Blending**: blending is the process of taking two colors from two framebuffers and mixing them
//!     between each other.
//!   - And a lot of other cool things like *GPU commands*, *pipelines*, *uniform interfaces* and so on…
//!
//! # How to dig in?
//!
//! `luminance` is written to be fairly simple. The documentation is very transparent about what the
//! library does and several articles will appear as the development goes on. Keep tuned! The
//! [online documentation](https://docs.rs/luminance) is also a good link to have around.
//!
//! # Current implementation
//!
//! Currently, **luminance is powered by OpenGL 3.3**. It might change, but it’ll always be in favor
//! on supporting more devices and technologies – a shift to Vulkan is planned, for instance.
//!
//! # Windowing
//!
//! `luminance` does not provide a way to create windows because it’s important that it not depend
//! on windowing libraries – so that end-users can use whatever they like. Furthermore, such
//! libraries typically implement windowing and events features, which have nothing to do with our
//! initial purpose.
//!
//! # User-guide and contributor-guide
//!
//! If you just plan to use `luminance`, just read the *User-guide* section.
//!
//! If you plan to contribute to `luminance` (by writing a windowing crate or hacking on `luminance`
//! directly), feel free to read the *Contributor-guide* section after having read the *User-guid*
//! section as well.
//!
//! ## User-guide
//!
//! In order to get started, you need to create an object which type implements `GraphicsContext`.
//! `luminance` ships with the trait but no implementor. You need to head over
//! [crates.io and search for luminance crates](https://crates.io/search?q=luminance) to find a
//! windowing backend first.
//!
//! Such a backend should expose a type which implements `GraphicsContext`. You can create one per
//! thread. This limitation enables `luminance` not to perform plenty of runtime branching,
//! minimizing the runtime overhead.
//!
//!   > If you really want several contexts, you will need several OS threads.
//!
//! `GraphicsContext` is the entry-point of everything `luminance` provides. Feel free to dig in its
//! documentation for further information on how to use `luminance`. Most objects you can create
//! will need a mutable reference to such a context object.
//!
//! ## Contributor-guide
//!
//! You want to hack around `luminance` or provide a windowing crate? Everything you have to know is
//! described in this section.
//!
//! ### `GraphicsContext`, `GraphicsState` and TLS
//!
//! In order to implement `GraphicsContext`, you need to know several points:
//!
//!   - You can get a `GraphicsState` with the `GraphicsState::new` function. You **have** to match the return
//!     value. Depending on whether your implementation is the first asking a `GraphicsState` on the current
//!     thread, you might get (or not) an `Ok(state)`. If not, a descriptive error is returned.
//!   - You’re advised to `map` and `map_err` over the `GraphicsState::new` returned value to implement your
//!     own `new` function for your backend type because of the restriction of having only one context per
//!     thread in `luminance`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

pub mod blending;
pub mod buffer;
pub mod context;
pub mod depth_test;
pub mod face_culling;
pub mod framebuffer;
pub mod linear;
mod metagl;
pub mod pipeline;
pub mod pixel;
pub mod render_state;
pub mod shader;
pub mod state;
pub mod tess;
pub mod texture;
pub mod vertex;
pub mod vertex_restart;
