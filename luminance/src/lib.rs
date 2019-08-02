//! # What is this?
//!
//! [![crates.io](https://img.shields.io/crates/v/luminance.svg)](https://crates.io/crates/luminance)
//! ![License](https://img.shields.io/badge/license-BSD3-blue.svg?style=flat)
//!
//! **luminance** is an effort to make graphics rendering simple and elegant.
//!
//! The aims of **luminance** are:
//!
//!   - Bringing a **safe**, **type-safe** and **stateless** API.
//!   - Providing a simple and easy interface; that is, exposing core concepts without anything
//!     extra – just the bare stuff. This is not a 3D or a video game engine. It’s a set of building
//!     blocks and graphics primitives you can use to construct more specific abstractions,
//!     libraries and applications.
//!   - To be opinionated enough to allow safety and optimizations but not to force the user into a
//!     too strict design: this is not a framework. Some constructs are restricting by design but
//!     the overall crate tries to adapt to what the user wants to do, not the over way around.
//!   - Easy to read with a good documentation and set of tutorials, so that newcomers don’t have to
//!     learn a lot of new concepts to get their feet wet. For most primitive concepts, they must
//!     be explained and detailed in their corresponding sections / modules.
//!   - Need-driven: every piece of code added in the project must come from a real use case. If you
//!     feel something is missing, feel free to open an issue or even contribute! Issue trackers
//!     exist for bug tracking but also for feature requests.
//!   - The [gfx-hal] crate is already a good crate, so **luminance** must stand out by providing an
//!     easier crate for people who just want to write some graphics code without having to cope
//!     with _too low-level_ details. The crate is low-level but not as much as [gfx-hal].
//!
//! # What’s included?
//!
//! **luminance** is a rendering crate, not a 3D engine nor a video game framework. As so, it doesn’t
//! include specific concepts, such as lights, materials, asset management nor scene description. It
//! only provides a rendering library you can plug in whatever you want to.
//!
//!   > There are several so-called 3D-engines out there on [crates.io](https://crates.io). Feel
//!   > free to have a look around.
//!
//! However, **luminance** comes in with several interesting features that might interest you.
//!
//! ## Features set
//!
//!   - **Buffers**: buffers are way to communicate with the GPU; they represent regions of memory
//!     you can write to and read from. There’re several kinds of buffers you can create, among
//!     *vertex and index buffers*, *shader buffers*, *uniform buffers*, and so on and so forth….
//!   - **Framebuffers**: framebuffers are used to hold renders. Each time you want to perform a
//!     render, you need to perform it into a framebuffer. Framebuffers can then be combined with
//!     each other to produce effects and design render layers.
//!   - **Shaders**: **luminance** supports five kinds of shader stages:
//!       - Tessellation control shaders.
//!       - Tessellation evaluation shaders.
//!       - Vertex shaders.
//!       - Geometry shaders.
//!       - Fragment shaders.
//!   - **Vertices, indices, primitives and tessellations**: those are used to define a shape you
//!     can render into a framebuffer with a shader.
//!   - **Textures**: textures represent information packed into arrays on the GPU, and can be used
//!     to customize a visual aspect or pass information around in shaders.
//!   - **Blending**: blending is the process of taking two colors from two framebuffers and mixing
//!     them between each other.
//!   - **Control on the render state**: the render state is a set of capabilities you can tweak
//!     to draw frames. It includes:
//!       - The blending equation and factors.
//!       - Whether we should have a depth test performed.
//!       - Face culling.
//!   - And a lot of other cool things like *GPU commands*, *pipelines*, *uniform interfaces* and so on…
//!
//! # How to dig in?
//!
//! **luminance** is written to be fairly simple. The documentation is very transparent about what the
//! library does and several articles will appear as the development goes on. Keep tuned! The
//! [online documentation](https://docs.rs/luminance) is also a good link to have around.
//!
//! # Current implementation
//!
//! Currently, **luminance is powered by OpenGL 3.3**: it’s the default. That version of OpenGL is
//! old enough to support a wide range of devices out there. However, it’s possible that your device
//! is older or that you target the Web or Android / iOS. In that case, you should have a look at
//! the set of feature flags, which offers the possibility to compile **luminance** on several
//! platforms.
//!
//! ## Feature flags
//!
//!   - `default = ["std"]`
//!   - `std`: Compile against the standard library. If you disable that feature, you get a tinier
//!     executable but you’re responsible for lots of stuff. **Currently, that feature is not well
//!     tested and very experimental; use with care and caution.**
//!
//! # Windowing
//!
//! **luminance** does not provide a way to create windows because it’s important that it not depend
//! on windowing libraries – so that end-users can use whatever they like. Furthermore, such
//! libraries typically implement windowing and events features, which have nothing to do with our
//! initial purpose.
//!
//! # User-guide and contributor-guide
//!
//! If you just plan to use **luminance**, just read the *User-guide* section.
//!
//! If you plan to contribute to **luminance** (by writing a windowing crate or hacking on **luminance**
//! directly), feel free to read the *Contributor-guide* section after having read the *User-guide*
//! section as well.
//!
//! ## User-guide
//!
//! ### Creating a context
//!
//! In order to get started, you need to create an object which type implements [`GraphicsContext`].
//! **luminance** ships with the trait but no implementor. You need to head over
//! [crates.io and search for luminance crates](https://crates.io/search?q=luminance) to find a
//! windowing backend first.
//!
//! Such a backend should expose a type which implements [`GraphicsContext`]. You can create one per
//! thread. That limitation enables **luminance** not to perform plenty of runtime branching,
//! minimizing the runtime overhead.
//!
//! > If you really want several contexts, you will need several OS threads.
//!
//! [`GraphicsContext`] is the entry-point of everything **luminance** provides. Feel free to dig in
//! its documentation for further information on how to use **luminance**. Most objects you can
//! create will need a mutable reference to such a context object. Even though **luminance** is
//! stateless in terms of global state, it still requires to have an object representing the GPU
//! somehow.
//!
//! ### Understanding the pipeline architecture
//!
//! **luminance** has a very particular way of doing graphics. It represents a typical _graphics
//! pipeline_ via a typed [AST] that is embedded into your code. As you might already know, when you
//! write code, you’re actually creating an [AST]: expressions, assignments, bindings, conditions,
//! function calls, etc. They all represent a typed tree that represents your program.
//!
//! **luminance** uses that property to create a dependency between resources your GPU needs to
//! have in order to perform a render. Typical engines, libraries and frameworks require you to
//! explicitly _bind_ something; instead, **luminance** requires you to go deeper in the [AST] by
//! creating a new lower node to mark the dependency.
//!
//! It might be weird at first but you’ll see how simple and easy it is. If you want to perform a
//! simple draw call of a triangle, you need several resources:
//!
//!   - A [`Tess`] that represents the triangle. It holds three vertices.
//!   - A shader [`Program`], for shading the triangle with a constant color, for short and simple.
//!   - A [`Framebuffer`], to accept and hold the actual render.
//!   - A [`RenderState`], to state how the render should be performed.
//!
//! There is a dependency _graph_ to represent how the resources must behave regarding each other:
//!
//! ```text
//! (AST1)
//!
//! Framebuffer ─> Shader ─> RenderState ─> Tess
//! ```
//!
//! The framebuffer must be _active_, _bound_, _used_ — or whatever verb you want to picture it
//! with — before the shader can start doing things. The shader must also be in use before we can
//! actually render the tessellation.
//!
//! That triple dependency relationship is already a small flat [AST]. Imagine we want to render
//! a second triangle with the same render state and a third triangle with a different render state:
//!
//! ```text
//! (AST2)
//!
//! Framebuffer ─> Shader ─> RenderState ─> Tess
//!                  │            │
//!                  │            └───────> Tess
//!                  │
//!                  └─────> RenderState ─> Tess
//! ```
//!
//! That [AST] looks more complex. Imagine now that we want to shade one other triangle with
//! another shader!
//!
//! ```text
//! (AST3)
//!
//! Framebuffer ─> Shader ─> RenderState ─> Tess
//!      │           │            │
//!      │           │            └───────> Tess
//!      │           │
//!      │           └─────> RenderState ─> Tess
//!      │
//!      └───────> Shader ─> RenderState ─> Tess
//! ```
//!
//! You can now clearly see the [AST]s and the relationships between objects. Those are encoded
//! in **luminance** within your code directly: lambdas / closures.
//!
//! ### The lambda & closure design
//!
//! A function is a perfect candidate to modelize a dependency. When you look at:
//!
//! ```rust
//! fn tronfibulate(x: Foo) -> Bar;
//! ```
//!
//! `tronfibulate` here is _covariant_ in `Bar` and _contravariant_ in `Foo`. What it implies is
//! that for the function itself, if we have a function that does `Zoo -> Foo`, then we can create
//! a new version of `tronfibulate` that will have, as input, a `Zoo`. Contravariance maps backwards
//! while covariance maps forwards (i.e. if you have `Bar -> Quux`, you can adapt `tronfibulate` to
//! create a new function that will output `Quux` value).
//!
//! All this to say that a contravariant dependency is pretty interesting in our case since we will
//! be able to adapt and create new functions just by contra-mapping the input. In terms of
//! combinational power, that is gold.
//!
//! Now, let’s try to represent `AST1` with contravariance and, hence, functions, using pseud-code
//! (this is not real **luminance** excerpt).
//!
//! ```ignore
//! // AST1
//! use_framebuffer(framebuffer, || {
//!   // here, we are passing a closure that will get called whenever the framebuffer is ready to
//!   // receive renders
//!   use_shader(shader, || {
//!     // same thing but for shader
//!     use_render_state(render_state, || {
//!       // ditto for render state
//!       triangle.render(); // render the tessellation
//!     });
//!   );
//! );
//! ```
//!
//! See how simple it is to represent `AST1` with just code and closures? Rust’s lifetimes and
//! existential quantification allows us to ensure that no resource will leave the scope of each
//! closures, hence enforcing memory and coherency safety.
//!
//! Now let’s try to tackle `AST2`.
//!
//! ```ignore
//! // AST2
//! use_framebuffer(framebuffer, || {
//!   use_shader(shader, || {
//!     use_render_state(render_state, || {
//!       first_triangle.render();
//!       second_triangle.render(); // simple and straight-forward
//!     });
//!
//!     // we can just branch a new render state here!
//!     use_render_state(other_render_state, || {
//!       third.render()
//!     });
//!   );
//! );
//! ```
//!
//! And `AST3`:
//!
//! ```ignore
//! // AST3
//! use_framebuffer(framebuffer, || {
//!   use_shader(shader, || {
//!     use_render_state(render_state, || {
//!       first_triangle.render();
//!       second_triangle.render(); // simple and straight-forward
//!     });
//!
//!     // we can just branch a new render state here!
//!     use_render_state(other_render_state, || {
//!       third.render()
//!     });
//!   );
//!
//!   use_shader(other_shader, || {
//!     use_render_state(yet_another_render_state, || {
//!       other_triangle.render();
//!     });
//!   });
//! );
//! ```
//!
//! That is a complete pipeline.
//!
//! ## Contributor-guide
//!
//! You want to hack around **luminance** or provide a windowing crate? Everything you have to know is
//! described in this section.
//!
//! ### [`GraphicsContext`], [`GraphicsState`] and TLS
//!
//! In order to implement [`GraphicsContext`], you need to know several points:
//!
//!   - You can get a [`GraphicsState`] with the [`GraphicsState::new`] function. You **have** to match the return
//!     value. Depending on whether your implementation is the first asking a [`GraphicsState`] on the current
//!     thread, you might get (or not) an `Ok(state)`. If not, a descriptive error is returned.
//!   - You’re advised to `map` and `map_err` over the [`GraphicsState::new`] returned value to implement your
//!     own `new` function for your backend type because of the restriction of having only one context per
//!     thread in **luminance**.
//!
//! [gfx-hal]: https://crates.io/crates/gfx-hal
//! [`GraphicsContext`]: crate::context::GraphicsContext
//! [`GraphicsState`]: crate::state::GraphicsState
//! [`GraphicsState::new`]: crate::state::GraphicsState::new
//! [`Tess`]: crate::tess::Tess
//! [`Program`]: crate::shader::program::Program
//! [`Framebuffer`]: crate::framebuffer::Framebuffer
//! [`RenderState`]: crate::render_state::RenderState
//! [AST]: https://en.wikipedia.org/wiki/Abstract_syntax_tree

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
