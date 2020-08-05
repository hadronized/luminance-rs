# Changelog

This document is the changelog of [luminance](https://crates.io/crates/luminance).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolve dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.41+1](#0411)
* [0.41](#041)
  * [Migration guide from 0.40](#migration-guide-from-040)
* [0.40](#040)
  * [Migration from 0.39](#migration-from-039)
* [0.39](#039)
  * [Major changes](#major-changes)
  * [Minor changes](#minor-changes)
* [0.38](#038)
  * [Major changes](#major-changes-1)
  * [Minor changes](#minor-changes-1)
  * [Patch changes](#patch-changes)
* [0.37.1](#0371)
* [0.37](#037)
  * [Major changes](#major-changes-2)
  * [Patch changes](#patch-changes-1)
* [0.36.1](#0361)
* [0.36](#036)
  * [Major changes](#major-changes-3)
  * [Minor changes](#minor-changes-2)
  * [Patch changes](#patch-changes-2)
* [0.35](#035)
  * [Major changes](#major-changes-4)
* [0.34.1](#0341)
* [0.34](#034)
  * [Bug fixes](#bug-fixes)
  * [Major changes](#major-changes-5)
* [0.33](#033)
* [0.32](#032)
  * [Bug fixes](#bug-fixes-1)
  * [Major changes](#major-changes-6)
  * [Minor changes](#minor-changes-3)
* [0.31.1](#0311)
* [0.31](#031)
  * [Bug fixes](#bug-fixes-2)
  * [Major changes](#major-changes-7)
  * [Minor changes](#minor-changes-4)
  * [Patch & misc changes](#patch--misc-changes)
* [0.30.1](#0301)
* [0.30](#030)
* [0.29](#029)
* [0.28](#028)
* [0.27.2](#0272)
* [0.27.1](#0271)
* [0.27](#027)
* [0.26](#026)
* [0.25.7](#0257)
* [0.25.6](#0256)
* [0.25.5](#0255)
* [0.25.4](#0254)
* [0.25.3](#0253)
* [0.25.2](#0252)
* [0.25.1](#0251)
* [0.25](#025)
* [0.24.1](#0241)
* [0.24](#024)
* [0.23.1](#0231)
* [0.23](#023)
* [0.22.7](#0227)
* [0.22.6](#0226)
* [0.22.5](#0225)
* [0.22.4](#0224)
* [0.22.3](#0223)
* [0.22.2](#0222)
* [0.22.1](#0221)
* [0.22](#022)
* [0.21.3](#0213)
* [0.21.2](#0212)
* [0.21.1](#0211)
* [0.21](#021)
* [0.20](#020)
* [0.19](#019)
* [0.18.2](#0182)
* [0.18.1](#0181)
* [0.18](#018)
* [0.17.2](#0172)
* [0.17.1](#0171)
* [0.17](#017)
* [0.16](#016)
* [0.15](#015)
* [0.14](#014)
* [0.13.1](#0131)
* [0.13](#013)
* [0.12.1](#0121)
* [0.12](#012)
* [0.11](#011)
* [0.10](#010)
* [0.9.1](#091)
* [0.9](#09)
* [0.8](#08)
* [0.7](#07)
* [0.6.4](#064)
* [0.6.3](#063)
* [0.6.2](#062)
* [0.6.1](#061)
* [0.6](#06)
* [0.5.3](#053)
* [0.5.2](#052)
* [0.5.1](#051)
* [0.5](#05)
* [0.4](#04)
* [0.3.1](#031-1)
* [0.3](#03)
* [0.2.1](#021-1)
* [0.2](#02)
* [0.1](#01)

<!-- vim-markdown-toc -->

# 0.41+1

> ?

- Add support for `f64`-like shader uniforms. Those include `f64`, `[f64; 2]`, `[f64; 3]`, etc.
  Textures are not supported yet though.

# 0.41

> Jul 24th, 2020

- Add `Debug` implementor for `TextureBinding`.
- Add the _skybox_ example.
- Fix a type design flaw in `PipelineGate`, `ShadingGate`, `RenderGate` and `TessellationGate`.
  Previously, those were using the `C: GraphicsContext` type variable, which, even though is okay
  for the [luminance] crate, makes it impossible to re-export those symbols in [luminance-front].
  Also, the rest of the types in [luminance] use `B` directly and universally quantify over
  `C: GraphicsContext<Backend = B>` when needed. #414
- Change the how errors are handled in pipeline gates. Closures passed to gates now must return
  error types — i.e. `Result<(), E>`, with the  constraint that `E: From<PipelineError>`. This
  allows more flexible error handling and allow pipeline errors to _flow up_ in the pipeline gate,
  either via `PipelineError` or via whatever error type the user wants them to flow up with.
- `PipelineGate::pipeline()` doesn’t return `Result<(), PipelineError`> anymore, but `Render<E>`, a
  brand new type wrapping over `Result<(), E>`. That type provides `Deref` / `DerefMut` over
  `Result<(), E>`, so your code using `is_err()` / `is_ok()` will still work.
- Fix various documentation warnings (dead links).

## Migration guide from 0.40

- The gates used the concept of `GraphicsContext` while all the other types used `B` with
  `GraphicsContext<Backend = B>`. Even though it’s unlikely you were using those types directly,
  you will have to use directly the backend type here. It should either make things simpler for
  you, or fix compilation errors.
- The new design of error flow in the pipeline closures is a great enhancement over type safety and
  error handling. However, it will make your code break — because your code, in `luminance-0.40`,
  return `()`, which is not `Result<(), E>`. You have two easy changes to do:
  - Inside the closures, you need to remove most of the `;` you put everywhere and replace them
    with either nothing (last instruction in the closure), or with `?;`, so that eventual errors
    flow up the pipeline.
  - Outside of the `PipelineGate::pipeline()` method call, you want to chain the `Render::assume()`
    method call. That method is a simple identity function (forward no-op, `self -> self`) that is
    required to force the type system to assume the error type to be `PipelineError`. You don’t
    have to use that function and can, instead, use type ascriptions, such as
    `Render<PipelineError>`, but that requires two imports, while `Render::assume()` doesn’t even
    require you to import `Render`. Of course you can use your own error types now, so
    `Render::assume()` is only useful if you don’t care to inject your own pipeline error types
    and simply want to use `PipelineError`.

# 0.40

> Wed Jul, 15th 2020

- Remove features. Both `"std"` and `"gl"` were removed. See the [luminance-gl] crate if you want
  to use OpenGL. Right now, the demand for `no_std` being inexistent, its support got dropped
  as well; there is no replacement.
- Rework completely the [luminance] crate to make it backend-agnostic. In pre-luminance-0.40, a
  type such as `Buffer<T>` had only one type variable. From now on, types get an additional first
  type parameter, noted `B`, which represents the backend the type is compatible with. The type
  `Buffer<B, T>` a backend-agnostic buffer of type `T`. [luminance] doesn’t contain any effectful
  code: you will have to select a backend type.
- Introduce the [luminance-gl] backend type crate. This crate exposes the `GL33` backend type that
  implements the various traits exported from `luminance::backend::*` needed to be implemented to
  effectively run code on a device (typically a GPU).
- The `TessBuilder` and `Tess` types have been completely reworked to support several type-safe
  features:
  - _Type state_: `TessBuilder` now implements a form of _type state_ safety to ensure that some
    functions / methods are available for a few types only. Calling those functions will mutate
    the type of `TessBuilder`, giving access to other methods.
  - _Refinement typing_: `Tess` and `TessBuilder` are now customized via several type variables.
    Among them, the type of vertex (`V`), the type of index (`I`), the type of vertex instance
    attribute (`W`) and the type of vertex storage (`S`). The latter is described later below.
  - _Type-driven APIs_: instead of providing an index to a method to set a specific attribute set,
    you simply use the same method but with a different type: it will automatically detect where
    to put the data you provide (same for reading / slicing). This new way of doing is a huge
    improvement in the comfort, elegancy and safety when using the `Tess` APIs. More on this
    in the migration guide below.
- Introduce the `Vertices{,Mut}`, `Indices{,Mut}` and `Instances{,Mut}` types, representing
  mapped slices of respectively vertices, indices and vertex instance data.
- `Tess` now supports slicing deinterleaved memory, while it was forbidden before.
- `Tess` now embarks the concept of memory interleaving or deinterleaving in its type via the
 `S` type variable.
- Bound resources (textures, buffers) were redesigned as well to be friendlier to use. Previously,
  you would typically use a [BoundTexture](https://docs.rs/luminance/0.39.0/luminance/pipeline/struct.BoundTexture.html)
  as a uniform. Now, a bound texture gives you the method `BoundTexture::binding` that can be
  passed as uniform in shaders — same applies to bound buffers. That change was required to make
  scarce resources bind in a more flexible and backend-agnostic way.
- Completely remove any OpenGL-related symbol. Those now belong to [luminance-gl]
- Introduce the `ProgramInterface` type. This type is a new _bridge_ type between a `Program` and
  various things you can do with its interface. Right now, it supports both setting `Uniform`
  values from the uniform interface, and, as before, provides a method to make dynamic uniform
  queries (useful if you’re writing a GUI or scripting, for instance).
- Change the way `Uniform` objects work. They do not have the `Uniform::update` method anymore
  and their meaning have changed a bit. They are now used as “keys” you can pass to a
  `ProgramInterface` to update the respective shader variable. The difference yields an API that
  is more flexible and will authorize more optimization by the backends.
- Introduce the `TextureBinding` and `BufferBinding` in place of `BoundTexture` and `BoundBuffer`.
  The two later still exist and provide a method (`binding()`) to get the two former. As described
  above, the binding objects can be passed to update a `Uniform` and allow for a more flexible
  interface that can work with several backends.
- Internal refactoring of various functions and macros.
- `Texture::get_raw_texels` now works on immutable textures rather than mutable ones.
- Remove the `luminance::linear` module. It contained type aliases for matrices that don’t really
  make sense to be aliased, as they are really used only at a few places.
- Framebuffers and textures mutability schemes have been fixed to correctly lock them in place when
  needed.
- The `luminance::vertex_restart` module has been deleted as it’s now a backend-dependent feature.
- The blending (i.e. `RenderState::set_blending`) has been made more user-friendly by removing the
  weird and confusing triple and replacing it with the `Blending` type.
- Add the [luminance logo].
- Add support for [dependabot].
- Remove the `bin/viewer` project. This project is still available as part of the Chapter 3 of the
  Learn Luminance book, [here](https://github.com/rust-tutorials/learn-luminance/tree/master/examples/chapter-3).
- Add the possibility to move color slots and/or depth slots out of a `Framebuffer`.
- Make all errors `#[non_exhaustive]`. This will be a breaking change for you if you used to
  pattern-match against any error, but from now on any new error variant can be released as a
  minor bump.
- Buffer and tessellation slicing is now made safer: once you have asked to slice a buffer or a
  tessellation, you can use `Deref` and `DerefMut` to directly access the mapped memory.
- Fix a bug in shader programs that would perform double-free on the GPU when dropped in various
  tricky situations.
- Rename `GraphicsContext::pipeline_gate` into `GraphicsContext::new_pipeline_gate`.
- Fix the boolean encoding of boolean vertex attributes. It was incorrectly set and wrong type
  formats were passed to the GPU.
- Add the `ProgramBuilder` helper type. This type allows you to create shader programs without
  having to worry too much about the highly generic interface of shader programs by letting
  rustc infer the type variables for you.
- Add the [luminance-webgl] crate as a backend implementation of WebGL2 to get start with.
- Implement `std::error::Error` for various types of the crate.
- Add the [luminance-sdl2] crate as a platform implementation for the [sdl2] crate.
- Update the [CONTRIBUTING](../CONTRIBUTING.md) file.
- Rename `Buffer::from_slice` into `Buffer::from_vec`. The big difference is that the input data
  must now be owned. However, you shouldn’t have lots of things to do as the interface takes a
  `Into<Vec<T>>`, so your previous slice should work and gets cloned here.
- Add support for separate RGB/alpha blending. You can now provide per-RGB and per-alpha blending
  equations and factors.
- Enrich the possible shader errors that can happen.
- Add the [luminance-front] crate. This is a very special crate which goal is to simplify working
  with [luminance] types. See the migration guide below for further details.
- Add helper methods to create error types requiring owned data via a better API.
- Rename `TessSlice` into `TessView` and updated the subsequente method to make them simpler to
  remember (e.g. `one_whole` -> `whole`, etc.)
- The shader code now lives in `luminance::shader`.
- Make `RenderState`’s fields private so that adding new features to render states is not a
  breaking change (`#[non_exhaustive]` is not really wanted here).
- Add the possibility to enable or disable _depth writes_. If you disable it, rendering to a
  framebuffer which has a depth buffer attached will not update the depth of the rasterized
  fragments.

## Migration from 0.39

- The backend architecture has a lot of impact on the internals and, by default, on the types you
  might be using, such as `Buffer<T>` vs. `Buffer<B, T>`. You have three possibilities to migrate
  your types there:
  1. Either use a generic version by constraining `B` in `Buffer<B, T>` correctly (e.g. you need
    to constrain it with `luminance::backend::Buffer<T>` for normal operations and
    `luminance::backend::BufferSlice<T>` if you plan to slice it.
  2. Use a specific version of `B` by using a backend type. The advantage is that it’s easier but
    the drawback is that you will not be able to adapt to several backends.
  3. Use [luminance-front], that will automatically pick the right backend type for you and will
    provide type aliases without having to care about `B`. Hence,
    `luminance::buffer::Buffer<B, T>` becomes `luminance_front::buffer::Buffer<T>`. The backend
    type is selected at compile type based on your compile target and optionally the feature
    gates you enable.
- Selecting a backend type is just a better of either using [luminance-gl] and letting it do it for
  you, or depend on the backend crate you want, such as [luminance-gl] or [luminance-webgl], pick
  the type, such as `luminance_gl::GL33` or `luminance_webgl::WebGL2` and use it when creating
  a surface. Normally, if you use a platform crate, such as [luminance-glfw], [luminance-glutin] or
  [luminance-web-sys], all this selection should be done automatically for you.
- Tessellations (`TessBuilder` and `Tess`) have been considerably reworked and most of the work you
  will have to migrate will be there. On 0.39, builders’ methods were fallible, requiring you to
  either warp your `Tess` creation in a `Result<_, TessError>` function, or use the
  `Result::and_then` combinator — you could also use `Result::unwrap` / `Result::expect`, but
  _don’t_. In this release, only the last call `build` is fallible, so you can now chain the method
  in a more traditional and Rust way. The other massive difference is how you will pass data:
  - If you were using [add_vertices](https://docs.rs/luminance/0.39.0/luminance/tess/struct.TessBuilder.html#method.add_vertices),
    three possible situations:
    - You made _no_ call to it: you still don’t need to make any call.
    - You made a _single_ call to it: it means that you are using _interleaved memory_; and the
      input slice you pass represents an ordered list of vertices, with their attributes all
      interleaved. In this case, you want to change that call to `set_vertices` instead, which now
      expects owned data (i.e. `Vec<V>`) and `V: TessVertexData<Interleaved, Data = Vec<V>>` must
      be satisfied — if `V: Vertex`, then it’s always okay.
    - You made _several_ calls to it: it meant you wanted _deinterleaved memory_; every call to
      `add_vertices` adds a new set representing a new vertex attribute set. The way you migrate
      that code is actually easy: you need to replace all `add_vertices` calls with `set_attributes`.
      It expects the same owned data as input — `Vec<A>` — but the constraint is `V: Deinterleave<A>`.
      If you used [luminance-derive] on your vertex type, `Deinterleave<A>` is implemented for all
      the attributes (`A`) types you might want to use. Simply call `set_attributes` with all your
      attributes array / vectors / whatever and the type system will do the rest for you.
  - If you were using [add_instances](https://docs.rs/luminance/0.39.0/luminance/tess/struct.TessBuilder.html#method.add_instances),
    it works the same way as described above, but with the `W` type and the `set_instances` method.
  - The [set_indices](https://docs.rs/luminance/0.39.0/luminance/tess/struct.TessBuilder.html#method.set_indices)
    doesn’t change much, besides requiring owned data as well.
- Migrating to the new `BindTexture` and `BindBuffer` is easy: you still bind the resources the same
  way, but now you need to call the `binding()` method to get an object that is able to be passed down
  to shaders.
- The uniform interfaces work differently. Instead of getting your uniform interface and upload values
  to it directly, you know get both the uniform interface _and_ a `ProgramInterface`, which allows you
  to set the uniforms from the uniform interface. That might feel like a weirder API, but it’s actually
  pretty simple if think of a `Uniform<T>` as a key into a `ProgramInterface`. The
  `ProgramInterface::set` method feels pretty natural to use once you know that. Also, it enables
  backends to take smart decision regarding uniform setting.
- If you were using blending, the triple to give the equation to use, source and destinatio factors
  has been removed and replaced by `Blending`. You now have to explicitly state the field of the
  blending. You will want to replace this kind of example:
  ```rust
  render_state.set_blending((Equation::Additive, Factor::SrcAlpha, Factor::Zero));
  ```
  with:
  ```rust
  render_state.set_blending(
    Blending {
      equation: Equation::Additive,
      src: Factor::SrcAlpha,
      dst: Factor::Zero
    }
  );
  ```
  This change makes it much easier to understand what is what and even though it’s a bit more
  verbose, it’s easier to read and less dark magic.
- The new way to create GPU scarce resources, such as buffers, textures, framebuffers and shaders
  is to use methods of the `GraphicsContext` trait. You will often typically need to import it
  first — or the platform crate must implement the functions for you, which is also a good habit.
  If it’s not the case, simply add this at the top of your file:
  ```rust
  use luminance::context::GraphicsContext as _; // it’s unlikely you’ll need to refer to GraphicsContext
  ```
  Then, using the platform object (typically called a _surface_ in the platform crate), you can simply
  invoke the various methods from the trait. They usually start with `new_` and the name of the
  resource. For instance:
  ```rust
  let buffer = glfw_surface.new_buffer(10)?;
  ```
- Most types of [luminance] got replaced with a generic version (for those not already having
  type variables). The `B` type variable can now be found in types that belong to two scopes:
  - The _API_ scope.
  - The _backend_ scope.
  The API scope is what you are used to: you use types via the front-facing API and you get all
  the brain candies [luminance] has to offer — type states, refinement typing, etc. However,
  the backend scope is a new layer in the architecture that splits the responsibilities of types:
  the API part must encode all the type-level contracts, such as compile-time state tracking,
  side-effects protection, refinement typing, etc. and the backend part must implement the actual
  GPU work. That `B` type variable represents a type which is associated actual, real-world
  GPU / device types and will most of the time be selected by what is called a _platform_ crate.
  It’s up to you to decide whether you want to handle that complexity — it can be an option if you
  want to have the power to dynamically pick a different backend at runtime — or if you want to
  just get it done already. In that last case, people just writing small binaries, games or
  simply testing stuff won’t care much about writing generic code that will work for all possible
  platforms. In that case, [luminance-front] will be a great ally. The idea is that the crate
  selects the right type for `B` and export type aliases to all [luminance]’s types so that the
  `B` type variable doesn’t have to be provided. A type such as `Buffer<B, T>` becomes
  `Buffer<T>` — like it used to be pre-0.40 — with [luminance-front]. By convenience, that crate
  also exports non-polymorphic types so that you can simply `use luminance_front::` without
  having to `use luminance::`. However, keep in mind that if you use the [luminance-derive] crate,
  you will still have to have [luminance] in your `Cargo.toml`.
- Add the [luminance-web-sys] crate as a platform crate for the Web and WebGL.
- Add the [luminance-examples-web] crate that showcases some [luminance-examples] samples, but
  rewritten for the Web.
- If you were using the concept of _tessellation mapping_ (previously known as `TessSlice`), that
  has been renamed `TessView` (i.e. viewing), because texture slicing is _also_ a feature that
  allows to get a slice (`&[]` / `&mut []`) for subparts of a tessellation. You will want to update
  your code and replace call to function such as `one_whole` into `whole`, `one_slice` into
  `slice`, `one_sub` into `sub` etc. The `TessView` trait has now two renamed methods: `view` and
  `inst_view`, which works the same way they used to (with range operators).
- If you used the `Program` type directly by importing it from `luminance::shader::program`, you
  now need to import all shader-related code from `luminance::shader` directly.
- If you were reading the `RenderState`’s, because they are now private, you can access them via
  the `blending()`, `depth_test()` and `face_culling()` methods.

# 0.39

> Sat Feb, 20th 2020

## Major changes

- Remove the concept of _layering_ in textures. Textures’ layerings (i.e. either _flat_ or
  _arrayed_) is now encoded directly in the dimension of the texture.

## Minor changes

- Add support for texture arrays. They can now be passed constructed and passed as uniforms to
  shader programs.

# 0.38

> Sat Jan, 4th 2020

## Major changes

- The `tess::Mode::Patch` variant was added. It was missing, implying that no one could actually
  use tessellation shaders.
- Add the `PipelineState` type. This type is an evolution over what a `Pipeline` can be customized
  with. Before `PipelineState`, only the _clear color_ could be set. `PipelineState` encapsulates
  several other possibilities. If you would like to replace your code from `luminance-0.37` that
  was using a color before, you can simply use `PipelineState::default().set_clear_color(_)` as
  a one-liner drop-in alternative.
- The `RenderState` argument of `RenderGate::render` is now taken by reference.
- To create a `Framebuffer`, it is now required to pass another argument: a `Sampler`. This mirrors
  the way textures are created and was asked independently by several people. If you don’t care,
  just pass `Sampler::default()`.

## Minor changes

- Add support for _sRGB_ framebuffer linearization. This is part of the `PipelineState`.
- Add two _sRGB_ pixel formats:
  - `SRGB8UI`, for an 8-bit unsigned integral sRGB pixel format.
  - `SRGBA8UI`, for an 8-bit unsigned integral sRGB (with a linear alpha channel) pixel format.
- It’s now possible to decide whether _color buffers_ and _depth buffers_ will be cleared when
  running a pipeline.
- It’s now possible to override the viewport when running a pipeline. See the documentation of
  `Viewport`.

## Patch changes

- Tessellation shaders were created with the wrong internal representation. That’s fixed.
- Add displacement map example.
- README.md update.
- Internal optimization with GPU state tracking.
- Examples were removed from the `luminance` crate and put into a [luminance-examples] crate.
  This small changes has been required for a while to prevent a weird cyclic dependency apocalypse
  when updating to crates.io.
- Support of `gl-0.14`.
- Fix some typo in documentation.

# 0.37.1

> Sun Sep, 29th 2019

- Release with `[dev-dependencies]` updated.

# 0.37

> Sun Sep, 29th 2019

## Major changes

- `DepthTest` was removed from the interface and replaced by `Option<DepthComparison>`.
- The default implementation of `RenderState` for the _depth test_ is `Some(DepthComparison::Less)`.

## Patch changes

- Fix cubemap storage creation code as well as upload code.

# 0.36.1

> Fri Sep, 20th 2019

- Release with `[dev-dependencies]` updated.

# 0.36

> Fri Sep, 20th 2019

## Major changes

- Change some function signatures to take arguments by copy instead of by borrowing. Clippy found
  those to be better and will yield better performance. The public APIs are then affected. You
  should be able to quickly merge by removing some references. ;)
- Uniform type mismatch got strengtened (proper `Type` being returned as errors instead of opaque
  `String`).
- Remove pair-based `Program` construction; i.e. `(Program<S, Out, Uni>, Vec<ProgramWarning>)` now
  becomes `BuiltProgram<S, Out, Uni>`. If you don’t care about warnings, instead of
  `let (program, _) = …`, you can simply call the `ignore_warnings()` method on `BuiltProgram`.
- Remove pair-based `Program` adapt and readapt constructs. You now use the `AdaptationFailure`
  type. It has an `ignore_error` method you can use to get back the `Program` you call the
  adapt method on if it fails.
- `Semantics` types must now implement `Copy`, `Clone` and `Debug`. It was already required before
  if you were using [luminance-derive] but it’s now more explicit.

## Minor changes

- Add the `viewer` binary in [bin](bin).

## Patch changes

- Use `cargo clippy` to fix several warnings.

# 0.35

> Thur Sep, 12th 2019

## Major changes

- Implement _safe pipelines_. Those implement stricter rules regarding Rust’s borrowing so that
  you cannot accidentally invalidate the GPU’s state by trying to nest or return _gates_
  (something you are never expected to do and shouldn’t do, but now you cannot do it even if you
  want to).

# 0.34.1

> Wed Sep, 11th 2019

- Re-release with `[dev-dependencies]` updated for other crates

# 0.34

> Wed Sep, 11th 2019

## Bug fixes

- Fix a bug that would cause short-living `Tess` to prevent any other tessellation from acquiring
  the required GPU state to hold scarce resources. The bug was due to `Drop` implementors that
  were missing an interaction with the GPU. Fixing this bug implied a major change about frame
  buffers. Thanks to [@austinjones](https://github.com/austinjones) for their report of the bug.

## Major changes

- Swap the arguments in the binary closure that is passed to `ShadingGate::shade`. That is more
  logical regarding the other closures from, for instance, `Pipeline`.
- Change the `TessGate::render` function so that it now accepts `T: Into<TessSlice>`
  instead of a `TessSlice` directly. That enables you to pass `&Tess` directly instead of
  slicing it with `..` or `*_whole`.
- Because framebuffers and buffers must now have access to the GPU’s state, the
  `Framebuffer::back_buffer` function now expects an object which implements `GraphicsContext`.
- The `Surface` trait has a new method to implement: `Surface::back_buffer`. That method provides
  the `Framebuffer::back_buffer` in a much more convenient way and is automatically implemented
  by default.

# 0.33

> Fri Sep, 6th 2019

- Add support for specifying the number of instances to render with `TessSlice`. The methods to
  specify that parameter are the same as the regular, 1-instance version ones but with the prefix
  `inst_` instead of `one_`.

# 0.32

> Tue Sep, 3rd 2019

## Bug fixes

- Fix the 06-texture example (see [#189]). The problem was due to the usage of an RGB pixel format
  and non-power-of-two textures, causing un-aligned memory to be read from OpenGL.
- No bug reported but a patch is provided in advance: similarily to just above, reading texels is
  now fixed to take account packing alignment.

## Major changes

- Make uploading texels to a texture a failible operation. It can now fail with the
  `TextureError::NotEnoughPixels` error if the user provided a slice with an insufficient amount
  of bytes.

## Minor changes

- Provide more pixel formats, among _normalized signed integral_ textures.

# 0.31.1

> Fri Aug, 23th 2019

- Re-upload of `0.31.0` to fix the cyclic dependency interdiction from crates.io with
  [luminance-derive].

# 0.31

> Fri Aug, 23th 2019

## Bug fixes

- Fix and remove `panic!` and attributeless renders.
- Various internal bug fixes and performance improvements.
- Fix pixel code for `Format::R` and `Format::RG` when querying a texture’s texels.

## Major changes

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
- Revise the `Tess` type to make it easier to work with.
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
- Migrate to Rust Edition 2018.
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
- Massive documentation rewrite (among the use of `#![deny(missing_docs)]`. The situation is still
  not perfect and patch versions will be released to fix and update the documentation. Step by
  step.
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

> September 10th, 2017

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

# 0.18.2

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
[luminance-examples]: ./luminance-examples
[luminance-examples-web]: ./luminance-examples-web
[luminance-windowing]: https://crates.io/crates/luminance-windowing
[luminance-gl]: https://crates.io/crates/luminance-gl
[luminance-glfw]: https://crates.io/crates/luminance-glfw
[luminance-glutin]: https://crates.io/crates/luminance-glutin
[luminance-webgl]: https://crates.io/crates/luminance-webgl
[luminance-web-sys]: https://crates.io/crates/luminance-web-sys
[glutin]: https://crates.io/crates/glutin
[#189]: https://github.com/phaazon/luminance-rs/issues/189
[luminance logo]: ../docs/imgs/luminance.svg
[dependabot]: https://dependabot.com
[sdl2]: https://crates.io/crates/sdl2
