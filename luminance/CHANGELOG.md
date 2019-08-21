# 0.31

> ?

## Bug fixes

  - Fix and remove `panic!` and attributeless renders.
  - Various internal bug fixes and performance improvements.
  - Fix pixel code for `Format::R` and `Format::RG` when querying a texture’s texels.

## Major changes features

  - Remove the concept of `GTup`. No code was using it and it was not really elegant.
  - Remove the `uniform_interface!` macro and replace it with the `UniformInterface` procedural
    derive macro.
  - Buffer mapping is now always a `mut` operation. That is required to _lock-in_ the mapped slices
    and prevent to generate new ones, which would be an undefined behavior in most graphics backends
    such as _OpenGL_.
  - Change the framebuffer’s slots types and meanings. Those are now more natural to use (for
    instance, you don’t have to repeat the framebuffer’s associated types and dimensions nor even
    use the `Texture<..>` type anymore, as a type family is now used to ease the generation of
    color and depth slots).
  - Change the way the `Vertex` trait is implemented.
    - The `Vertex::vertex_format` method has been renamed `Vertex::vertex_desc`.
    - Instead of returning a `VertexFormat`, that method now returns a `VertexDesc`. Where a
      `VertexFormat` was a set of `VertexComponentFormat`, a `VertexDesc` is a set of
      `VertexBufferDesc`.
    - `VertexBufferDesc` is a new type that didn’t exist back then in _0.30_. It provides new data
      and information about how a vertex attribute will be spread in a GPU buffer. Especially, it has:
      - An _index_, allowing to map the vertex attribute in a shader.
      - A _name_, used by shader programs to perform mapping.
      - An _instancing_ parameter, used to determine whether we want **vertex instancing**.
      - A `VertexAttribDesc`, the new name of `VertexComponentFormat`.
    - As said above, `VertexComponentFormat` was renamed `VertexAttribDesc`.
    - Vertex attribute can now be _normalized_ if they are _signed integral_ or _unsigned integral_.
      That is encoded in the `VertexAttribType`’s integral variants.
    - A new trait has appeared: `VertexAttrib`. Such a trait is used to map a type to a
      `VertexAttribDesc`.
    - `Vertex` has zero implementor instead of several ones in _0.30_. The reason for that is that
      `VertexBufferDesc` is application-driven and depends on the _vertex semantics_ in place in the
      application or library.
    - Vertex semantics are introduced in this release and are represented via the `Semantics` trait.
      Implementing directly `Semantics` is possible, even though not recommended. Basically,
      `Semantics` provides information such as the _index_ and _name_ of a given semantics as long
      as the list of all possible semantics, encoded by `SemanticsDesc`.
    - Users are highly advised to look at the `Vertex` and `Semantics` proc-macro derive in the
      [luminance-derive] crate.
  - Revised the `Tess` type to make it easier to work with.
    - The `Tess::new` and `Tess::attributeless` functions were removed.
    - The `TessBuilder` type was added and replace both the above function.
    - That last type has a lot of methods that can be combined in different ways to build powerful
      situation of tessellations, among (but not limited to):
      - Normal and indexed tessellations.
      - Attributeless tessellations.
      - Tessellations with vertex instancing support.
      - Deinterleaved tessellations
      - Tessellations with support for _primitive restart indexing_.
    - Slicing was revised too and now has support for two new Rust operators:
      - The `a ..= b` operator, allowing to slice a `Tess` with inclusive closed bounds.
      - The `..= b` operator, allowing to slice a `Tess` with inclusive bounds open on the left
        side.
    - Previously, the `Tess::new` associated function expected indices to be a slice of `u32`. This
      new release allows to use any type that implements the `TessIndex` trait (mapping a type to a
      `TessIndexType`. Currently, you have `u8`, `u16` and `u32` available.
    - Add `Tess::{as_index_slice,as_index_slice_mut}`. Those now enable you to conditionally slice-map
    the _index buffer_ of a `Tess`, if it exists.
  - Add support for generic texture sampling.
    - This new feature is supported thanks to the `SamplerType` trait, used as constraint on the
      `Pixel::SamplerType` associated type.
    - Basically, that feature allows you to bind a `Floating` texture without caring about the
      actual type. That is especially true as you typically use `sampler2D` in a shader and not
      `sampler2DRGB32F`.
    - Such a feature reduces the number of combination needed to refactorize code.
  - Implement _vertex attrib explicit binding_. This is a huge change that is related to _vertex
    semantics_. Basically, in _0.30_, you have to ensure that the `layout (location = _)` is
    correctly set to the right value regarding what you have in your `Tess`’ vertex buffers. That
    was both _unsafe_ and terribly misleading (and not very elegant). The new situation, which
    relies on _vertex semantics_, completely gets rid of _vertex locations_ worries, which get
    overrided by [luminance] when a shader program gets linked.
  - Change boolean-like `enum`s — such as `DepthTest` — variants from `Enabled` / `Disabled` to
    `On` / `Off` or `Yes` / `No`, depending on the situation.
  - Move `swap_buffers` from `GraphicsContext` to `Surface` in [luminance-windowing].
  - Switch to `GenMipmaps` instead of `bool` to encode whether mipmaps should be generated in
    texture code. That change is a readability enhancement when facing texture creation code.
  - Make `Dimensionable::zero_offset()` a constant, `Dimensionable::ZERO_OFFSET`.
  - Change the way cursor modes are encoded from `bool` to `CursorMode`.

## Minor changes

  - Add the [luminance-glutin] crate, the windowing crate support for [glutin].
  - Add the [luminance-derive] crate.
    - That crate provides several procedural derive macros you can use to easily implement all
      required traits to work with [luminance]. Especially, some traits that are `unsafe` can be
      implemented in a safe way with that crate, so you should definitely try to use it.
    - Current available proc-macros are:
      - `#[derive(Vertex)]`: derive the `Vertex` trait for a `struct`.
      - `#[derive(Semantics)]`: derive the `Semantics` trait for an `enum`.
      - `#[derive(UniformInterface)]`: derive the `UniformInterface` trait for a `struct`.
  - Support for dynamic uniform queries. Those are used whenever you don’t know which variables will
    be used in a shader at compile-time. This might be the case if you’re writing a GUI tool or a
    video game that uses a custom scripting language / node-ish representation of shaders. That
    feature doesn’t break the already-in-place and great uniform interface but complements it. You
    can use a shader `Program<_, _, ()>` and still set uniform values by querying the uniforms
    dynamically. This feature also fully benefits from the strongly typed interface of `Uniform<_>`,
    so you will get `TypeMismatch` runtime error if you try to trick the type system.
  - Add the `std` feature gate, allowing to compile with the standard library – this is enabled by
    default. The purpose of this feature is to allow people to use `default-features = false` to
    compile without the standard library. This feature is currently very experimental and shouldn’t
    be used in any production releases so far – expect breakage / undefined behaviors as this
    feature hasn’t been quite intensively tested yet.
  - Add support for the `R11FG11FB10F` pixel format.
  - Migrate crates to Rust Edition 2018.
  - The `WindowOpt` now has support for multisampling. See the `WindowOpt::set_num_samples` for
    further details.
  - Implement dynamic edition of windowing types properties. That allows to change data on-the-fly,
    such as the cursor mode.
  - Introduce normalized texturing. That feature is encoded as pixel formats: any pixel format which
    symbol’s name starts with `Norm` is a _normalized pixel format_. Such formats state that the
    texels are encoded as integers but when fetched from a shader, they are turned into
    floating-point number by normalizing them. For instance, when fetching pixels from a texture
    encoded with `R8UI`, you get integers ranging in `[0; 255]` but when fetching pixels from a
    texture encoded with `NormR8UI`, even though texels are still stored as 8-bit unsigned integers,
    when fetched, you get floating-point numbers comprised in `[0; 1]`.

## Patch & misc changes

  - Remove `std::mem::uninitialized` references, as it is now on deprecation path. Fortunately, the
    codes that were using that function got patched with safe Rust (!) and/or simpler constructs.
    It’s a win-win.
  - Add the `#[repr(C)]` annotation on vertex types in examples. That is a bit unfortunate because
    such an annotation is very likely to be mandatory when sending data to the GPU and it should be
    done automatically instead of requiring the user to do it. That situation will be fixed in a
    next release.
  - Add more CI testing.
  - Update examples and made them available to the `cargo run --example` command. Read more
    [here](./examples/README.md).
  - Massive documentation rewrite (among the use of `#![deny(missing_docs)]` in all [luminance]
    crates. The situation is still not perfect and `0.31` patch versions will be released to fix and
    update the documentation. Step by step.
  - Add design notes and documents in the repository.
  - Massive dependencies update. Special thanks to @eijebong for his help!
  - Add the `11-query-texture-texels` example, which showcases how to query a texture’s texels and
    drop it on the filesystem.
  - Add and update _README_ files. Especially, the gitter link was removed and an IRC link was
    added. If you want to get help:
    - Server: `irc.freenode.net`
    - Channel: `#luminance`

# 0.30.1

> Monday, 10th of September, 2018

  - Minor enhancement of the documentation (about texture types and GLSL samplers).

# 0.30

> Thursday, 26th of July, 2018

  - Change the way uniform interfaces are implemented so that uniform warnings are easier to dea
    with.

# 0.29

> Thursday, 12th of July, 2018

  - Augment the `UniformInterface` trait with an *environment type variable*. This is used to pass
    more data when generating the uniform interface and enable *value-driven* implementations. The
    `UniformInterface::uniform_interface` method receives the environment as a second argument now.
  - Add `Program::*_env` creation functions to pass an environment value.
  - Add `Program::adapt`, `Program::adapt_env` to change the type of the uniform interface of a
    shader program without interacting / deallocating / reallocating anything on the GPU side.
  - Add `Program::readapt_env` to perform the same thing as `Program::adapt_env` but disallow
    changing the type of the uniform interface. This is very useful if you need to update any
    *value-driven* objects in your interface.

# 0.28

> Tuesday, 3rd of July, 2018

  - Change the `Uniformable` definition. The `dim` method disappeared and the `Dim` type doesn’t
    exist anymore has dimensions were merged into `Type`.
  - Add support for typed textures in uniform type reification for a better type mismatch error
    detection.
  - Fix the texture example so that it doesn’t use `#[unbound]` anymore. This is a simple change yet
    it will help not bringing confusion when people try to play with the example.

# 0.27.2

> Friday, 29th of June, 2018

  - Remove the `const_fn` feature gate, making the crate now completely compile on the *stable*
    channel!

# 0.27.1

> Thursday, 28th of June, 2018

  - Fix some `Cargo.toml` metadata.

# 0.27

> Thursday, 28th of June, 2018

  - Tag some unsafe traits with the `unsafe` keyword. Those traits are not supposed to be
    implemented by end-users but if they do, they might break the internal behavior of the crate, so
    the unsafety tag is necessary.
  - Use an algebraic type for encoding pixel channel size instead of `u8`.
  - Remove the `Result` type aliases used pretty much everywhere. Even though you can find them in
    `std::*`, those are considered of bad taste and lead to confusion.
  - Fix `std::error::Error` implementors. Especially, the `description` methode implementation was
    removed.
  - Disable face culling by default. This default choice was motivated by #60: people didn’t see the
    triangle they specified and thought of a bug.
  - Rename `Framebuffer::default` into `Framebuffer::back_buffer`. Again, this is motivated by a
    user incomprehension about the behavior (see #65).
  - Some internal change to boost and take advantage of the use of the GPU state tracking.
  - Clean up the interface.
  - Made `BoundBuffer` and `BoundTexture` types easier to use. Especially, you’re not supposed to
    pass `Buffer<_>` nor `Texture<_, _, _>` anymore as it’s automatically inferred.
  - Replace `TessRender` with `TessSlice` and use the `Index<_>` notation to create such objects.
    This is a simple yet great change as you don’t have to tweak around `&TessRender` anymore and
    can directly use the `.slice(_)` method.
  - Enhance the overall documentation once again.
  - Add a bunch of examples. You can find them in the directory [examples](./examples). They consist
    of a cargo workspace with one cargo project per example. Feel free to read the top documentation
    and in-code documentation of each `main.rs` for further information. Also, if you would like to
    learn something new (or if you think an example is missing to understand something), feel free
    to open a PR / issue; any kind of contribution is warmly welcomed!

# 0.26

> Sunday, 17th of June, 2018

  - Enhanced the overall documentation.
  - Fix a (GPU) memory leak in `RawBuffer` when dropping the object. The memory leak is not present
    if you never convert a `Buffer<_>` into `RawBuffer` because the GPU resource is correctly
    dropped when the `Buffer<_>` is dropped. Thus, if you don’t use `Buffer::to_raw`, you never had
    leaks.
  - Add a TLS system to create `GraphicsState`. Such objects can be generated only once per thread,
    making the creation of a `GraphicsContext` safer. This also enforces having a single context per
    thread.
  - Introduce the `GraphicsContext` trait and `GraphicsState` type.
  - Huge refactoring of the library in order to make it thread-safe.
  - Update the README to add crates comparisons.

# 0.25.7

> Tuesday, 20th of March, 2018

  - Fix `BoundBuffer` type reification in `Uniformable`.

# 0.25.6

> Sunday, 18th of March, 2018

  - Added the `TessRender::one_slice` function.

# 0.25.5

> Tuesday, 13th of February, 2018

  - Support for `gl-0.10`.

# 0.25.4

> Monday, 12th of February, 2018

  - Support visibility in `uniform_interface!`.

# 0.25.3

> Monday, 12th of February, 2018

  - Fixed some doc’s typo.

# 0.25.2

> Sunday, 11th of February, 2018

  - Added a `uniform_interface!` macro. That enables to create a new `struct` type and have inspection
    of its fields at compile-time so that a `UniformInterface impl` is automatically generated. This
    is a kind of _custom auto derive_ without depending on proc-macro. Feel free to use it as it’ll
    remove a lot of boilerplate from your code.
  - Cleanup of internal code.

# 0.25.1

  - Fixed the pixel formats support on the OpenGL side.

# 0.25

> Thursday, 14th of December, 2017

  - Replaced the `Uniformable` implementation for `pipeline::Bound*` to `&pipeline::Bound*`. This
    enables sharing of bound resources instead of dropping and re-binding the resources.

# 0.24.1

> Monday, 11th of December, 2017

  - Added more color and renderable pixel formats (all sized supported).

# 0.24

> Thursday, 9th of November, 2017

  - Added support for *face culling*.
  - Enhanced the interface of render gates with `RenderState`.

# 0.23.1

> Monday, 2nd of October, 2017

  - Implemented `Display` and `Error` for error types.

# 0.23

> September 10th 2017

  - Added `Program::from_strings`.
  - Patch: internal function that used `Option` to express an error via `Some(err)` replaced by
    `Result<(), _>` and the more appropriate `Err(err)`.

# 0.22.7

  - Use the `pub(crate)` construct to hide some `unsafe` functions and remove `unsafe` annotation on
    some of them. The safety annotation was used so that it discouraged users to use the functions.
    Now that we have a proper way to express access-control, we can remove `unsafe`.

# 0.22.6

  - Fixed MRT with more than two color slots.
  - Fixed segfault on uniform introspection on some hardware.

# 0.22.5

  - Added the `readme` attribute to `Cargo.toml` so that it gets rendered on
  [crates.io](https://crates.io).

# 0.22.4

  - Added some impl of `RenderablePixel`.

# 0.22.3

  - Enforce static type of `Tess::attributeless`. It now only works with `Tess<()>`.

# 0.22.2

  - Enforce lifetimes on `BoundTexture` and `BoundBuffer`. It’s there so that such objects cannot
    live longer than the references on the `Texture<_>` and `Buffer<_>` they’re based on.

# 0.22.1

  - Better implementation of texture and buffer binding.

# 0.22

  - Added the `Gpu` type, handling stateful operations on the GPU.
  - Rework of the `pipeline` module. It might change a lot in the near future.

# 0.21.3

  - Some `tess` and `buffer` types now implement some standard traits (`Debug`, `Eq`, `PartialEq`).

# 0.21.2

  - Updated the documentation.

# 0.21.1

  - `Tess::as_slice{,_mut}` now don’t need type annotations anymore and provide slices typed by `V`.
  - Added `as_slice{,_mut}` to `Buffer` directly, setting the phantom type to what the `Buffer<_>`
    holds – it’s safer and will remove type annotations in client code, much cool!

# 0.21

  - Renamed `Chain` into `GTup` and added some macros.
  - Made `Framebuffer::{color,depth}_slot` private and provided accessors.

# 0.20

  - Typed shader programs are re-introduced. They now accept three type variables: the input of the
    shader program, its output and its uniform interface. The output is currently unused. A lot of
    things were added, documentation is not up to date and will come in the next release.
  - Vertex-typed tessellations are now a thing – it’s implemented so that the input type used in typed
    shader programs has a sense. The `CompatibleVertex` trait must be implemented when a tessellation
    having more than the required set of vertex components is used and must adapt to the shader input.
  - `Texture::Dim` is now (mostly) an array instead of a tuple.

# 0.19

  - `Into<Option<_>>` elegancy additions.
  - Changed the whole pipeline system. It’s now a traversal-like system.

# 0.18.2

  - Fixed some internal code about pipelines.
  - Fixed some internal code about texture set binding.
  - `RenderCommand::new` is now polymorphic in its blending argument.

# 0.18.1

  - Support for the latest `gl` crate.

# 0.18

  - `TessRender` now implements `From` for basic cases.
  - All pipeline types now implement `Clone`.
  - Pipeline types have lost ownership of subcommands (shading commands, render commands, etc.). This
    is a very important change made so that we can have more sharing and then expect a performance
    boost.

# 0.17.2

  - Fixed variance of `BufferSlice*`.

# 0.17.1

  - Enhanced the interface of pipes.
  - Rewrote the documentation.

# 0.17

  - Added mipmap texture filtering to `texture::MagFilter`.
  - Splitted `texture::Filter` into `texture::MinFilter` and `texture::MagFilter` for a better static
    safety.
  - Rewrite of pipelines. Their `Pipe` don’t accept a `Fn` closure anymore – see the point just below.
    Pipes are now created without the assumptions of anything else, and you can add *uniforms*,
    *uniform buffers* or *textures* as you see fit. The interface is designed in a way that no
    breaking changes will happen if another type of resources is added in the future.
  - Introduced `AlterUniform`. This type adds purity to alter uniforms. You cannot change the value
    of uniforms as you used to – via a `T: Fn`. This is because updating a uniform shouldn’t enable
    you to execute code as freely as you’d wish. The semantic is *“update the uniform”*.
    `AlterUniform` implements such a semantic.
  - Free some structs from useless trait bounds.
  - Unsupported pixel formats are now shown in a panic. Those should never arrive in a client code.
    The panics are just for developping luminance; which is then justified.
  - `R32F` pixel format was introduced.
  - Introduced `TessRender`. This type is great as it enables you to render a sub-part of a `Tess` if
    you want to, or change the number of instances you want. It’s a clearer interface than the
    previous one.
  - Internal code clean up.
  - It’s now possible to reserve GPU memory for vertices without filling tessellations. This choice
    is made via the `TessVertices` type.
  - Fixed a panic on `Framebuffer::new()`.
  - Added `RawTexture` and `RawBuffer`. Those types are type-erased versions of `Texture<_, _, _>` and
    `Buffer<_>`, respectively, used to pass them via heterogeneous slices for instance. There’s a
    `Deref` implementation that automatically promotes them to the raw equivalent.
  - Added `UniformWarning` to the public interface.
  - Merged [luminance-gl] into [luminance]. This decision is intended to make the use of luminance
    easier and take advantage of the underlying technology (in the current case,
    [OpenGL 3.3](https://www.opengl.org). The idea is that abstracting over several low-level
    graphics API makes it almost impossible to abstract them in a unified way and still be able to
    take full advantages of them, because they also provide very specific primitives and way of
    working with them that preclude any abstraction. Keep in mind that the low-level technology used
    also has an important impact on the design of the higher level API – to, actually, take advantage
    of it and keep performance as high as possible. Thank you for your attention.

# 0.16

  - `BufferSlice{,Mut}` now implements `IntoIterator`.
  - Some internal changes.
  - Vertices (`Vertex`) are now aligned based on what decides the Rust compiler. This is very
    important, especially because of the version 0.15.0 adding non-32-bit vertex components: alignment
    and padding is now completely handled for you and you have nothing to care about.
  - Added some documentation here and there.
  - Changed the meaning of the semantic maps (uniforms). It is now required to provide a `Uniform` to
    build a new `Sem`. This is an improvement in the sense that the *unsafe* zone is restricted to the
    declaration of uniforms for a given program. This *unsafe* zone will be covered in a next release
    by a macro to make it safe.
  - `texturee::Unit` cannot be used in a uniform block context (it doesn’t have sense on the GLSL
    side).
  - Added some more types to `UniformBlock`. This trait is not very useful yet, but it’s required to
    make a `Buffer` readable from GLSL.

# 0.15

  - Texture and framebuffers have several functions that can fail with new errors.
  - Added buffer mapping. See `BufferSlice` and `BufferSliceMut` for further details.
  - `Tessellation` is now called `Tess` for simplicity (because it’s used **a lot**).
  - `VertexComponentFormat::comp_size` is now a `usize`.
  - The `Vertex` trait now accepts a lot more of new types (among changes, added support for
    non-32-bit vertex components).

# 0.14

  - `UniformWarning::TypeMismatch` now includes the name of the uniform which type mismatches with the
    requested on.
  - `Pipeline::Pipe` is now exported by the common interface.
  - `Uniform::sem` doesn’t require `&self` anymore.
  - `Uniform::new` is now a const fn.

# 0.13.1

  - Added `Uniform::sem()` function to create `Sem` out of `Uniform<C, T>` in a simpler way.

# 0.13

  - Changed the pipeline workflow by introducing `Pipe` objects.
  - Removed strong typing in shader programs (`Program<C, T>` is now `Program<C>`).
  - Removed strong typing in shader stages (`Stage<C, T>` is now `Stage<C>`).

# 0.12.1

  - Added `Binding` and `Unit` in the default export-list.

# 0.12

  - Added attribute-less tessellations.
  - Enhanced shader-related documentation.
  - Removed `Slot`.

# 0.11

  - Added the first, constraint-only, `UniformBlock` trait. Used to restrict types that can be passed
    around in uniform buffers.
  - Added *proxied* types – i.e. `TextureProxy` and `UniformBufferProxy`. Those are used to build sets
    that can passed to shaders.
  - Uniform buffers can now be sent to shaders via the `buffer::Binding` type. Put your buffer in a
    uniform buffer set and there you go.
  - Textures are not `Uniformable` anymore. You have to put the texture in a texture set and use a
    `texture::Unit`, which is `Uniformable`.
  - Added `buffer::Binding`.
  - Added `texture::Unit`.
  - `map_uniform` now takes `&str` instead of `String`.
  - Cleaned up documentation.
  - Fixed internal bugs.

# 0.10

  - Fixed the type of the uniform errors in the uniform interface builder function.

# 0.9.1

  - Documentation updates.

# 0.9

  - Several textures can now be passed as uniforms to shaders. The interface is pretty instable as it
    might change in the future, because for now, the user has to pass the texture units each textures
    should be bound to.

# 0.8

  - Documentation is now available on docs.rs.
  - Replaced references to `Vec` by slices.

# 0.7

  - Uniforms have new semantics; mapping them cannot fail anymore but it can be emitted warnings.

# 0.6.4

  - Backends now have more information to work with about uniforms. That enables using reification
    techniques to validate types, dimensions, sizes, etc…

# 0.6.3

  - Added `get_raw_texels` to `Texture`.

# 0.6.2

  - Added `upload_part_raw` and `upload_raw` to `Texture`, enabling to upload raw texels instead of
    texels directly.
  - Added `RawEncoding` to `Pixel`.

# 0.6.1

  - Added documentation field in Cargo.toml.

# 0.6

  - Removed `Default` implementation for `Framebuffer` and added a new `default()` method, taking the
    size of the `Framebuffer`.
  - Added `RenderablePixel` trait bound on `Slot` implementations for `ColorPixel`.
  - Added `RenderablePixel`.
  - Removed the need of the **core** crate.
  - Removed `UniformName`.
  - We can now have textures as uniforms.
  - New uniform system to accept value depending on the backend.
  - Using `AsRef` instead of ATexture in `update_textures`.
  - Changed the meaning of mipmaps (now it’s the number of extra mipmaps).
  - Using `usize` instead of `u32` for mipmaps.
  - Added `Dimensionable` and `Layerable` in the interface.

# 0.5.3

  - Added `update_textures` into `HasUniform`.
  - Fixed signature of `UniformUpdate::update`.
  - Fixed trait bound on `UniformUpdate::{contramap, update}`.

# 0.5.2

  - Added `UniformUpdate`.
  - Added `Uniformable` in the public interfarce shortcut.

# 0.5.1

  - Removed `run_pipeline` and added `Pipeline::run`.

# 0.5

  - Fixed uniform interfaces in `ShadingCommand` and `RenderCommand` with existential quantification.
  - Renamed `FrameCommand` into `Pipeline`.
  - Several patch fixes.
  - Added travis CI support.
  - Added documentation for `Program`.

# 0.4

  - Changed the whole `Program` API to make it safer. Now, it closely looks like the Haskell version
    of [luminance]. The idea is that the user cannot have `Uniform`s around anymore as they’re held by
    `Program`s. Furthermore, the *uniform interface* is introduced. Because Rust doesn’t have a
    **“rank-2 types”** mechanism as Haskell, `ProgramProxy` is introduced to emulate such a behavior.
  - Added a bit of documentation for shader programs.

# 0.3.1

  - Removed `rw`.

# 0.3

  - Removed A type parameter form `Buffer`. It was unnecessary safety that was never actually used.
  - Added documentation around.

# 0.2.1

  - Exposed `Vertex` directly in [luminance].

# 0.2

  - Changed `Negative*` blending factors to `*Complement`.

# 0.1

  - Initial revision.

[luminance]: https://crates.io/crates/luminance
[luminance-derive]: https://crates.io/crates/luminance-derive
[luminance-windowing]: https://crates.io/crates/luminance-windowing
[luminance-glfw]: https://crates.io/crates/luminance-glfw
[luminance-glutin]: https://crates.io/crates/luminance-glutin
[glutin]: https://crates.io/crates/glutin
