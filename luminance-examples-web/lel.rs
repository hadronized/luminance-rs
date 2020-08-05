#![feature(prelude_import)]
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
//! # Feature flags
//!
//! None so far.
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
//! - **Buffers**: buffers are ways to communicate with the GPU; they represent regions of memory
//!   you can write to and read from. There’re several kinds of buffers you can create, among
//!   *vertex and index buffers*, *uniform buffers*, and so on and so forth…. They look like
//!   regular array but have some differences you might be aware of. Most of the time, you will
//!   use them to customize behaviors in shader stages.
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
//! - The [“Learn luminance” book](https://rust-tutorials.github.io/learn-luminance). Ideal for
//!   newcomers as well as people already used to luminance, as it’s always updated to the latest
//!   version — you might learn new things!
//! - The [luminance-examples](https://github.com/phaazon/luminance-rs/tree/master/luminance-examples)
//!   project. It contains lots of examples describing how to do specifics things. Not adapted for
//!   newcomers, you will likely want to consult those examples if you’re already familiar with
//!   graphics programing and to look for how to do a specific thing.
//!
//! # Implementation and architecture
//!
//! **luminance** has been originally designed around the OpenGL 3.3 and OpenGL 4.5 APIs. However,
//! it has mutated to adapt to new technologies and modern graphics programming. Even though its API
//! is _not_ meant to converge towards something like Vulkan, it’s changing over time to meet
//! better design decisions and performance implications.
//!
//! The current state of luminance comprises several crates:
//!
//! - A “core” crate, [luminance], which is about all the
//!   abstract, common and interface code.
//! - A set of _backend implementation_ crates, implementing the [luminance] crate.
//! - A set of _windowing_ crates, executing your code written with the core and backend crate.
//! - A special crate, [luminance-front], a special _backend_ crate that allows to combine
//!   several “official” crates to provide a cross-platform experience without having to pick
//!   several backend crates — the crate does it for you. This crate is mainly designed for end-user
//!   crates.
//!
//! ## The core crate
//!
//! The luminance crate gathers all the logic and rendering abstractions necessary to write code
//! over various graphics technologies. It contains parametric types and functions that depend on
//! the actual _implementation type_ — as a convention, the type variable `B` (for backend) is
//! used. For instance, the type `Buffer<B, u8>` is an 8-bit unsigned integer buffer for which the
//! implementation is provided via the `B` type.
//!
//! Backend types — i.e. `B` — are not provided by [luminance] directly. They are typically
//! provided by crates containing the name of the technology as suffix, such as luminance-gl,
//! luminance-webgl, luminance-vk, etc. The interface between those backend crates and
//! luminance is specified in [luminance::backend].
//!
//! On a general note, `Buffer<ConcreteType, u8>` is a monomorphic type that will be usable
//! **only** with code working over the `ConcreteType` backend. If you want to write a function
//! that accepts an 8-bit integer buffer without specifying a concrete type, you will have to
//! write something along the lines of:
//!
//! ```
//! use luminance::backend::buffer::Buffer as BufferBackend;
//! use luminance::buffer::Buffer;
//!
//! fn work<B>(b: &Buffer<B, u8>) where B: BufferBackend<u8> {
//!   todo!();
//! }
//! ```
//!
//! This kind of code is intented for people writing libraries with luminance. For the special case
//! of using the [luminance-front] crate, you will end up writing something like:
//!
//! ```ignore
//! use luminance_front::buffer::Buffer;
//!
//! fn work(b: &Buffer<u8>) {
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
//! libraries typically implement windowing and events features, which have nothing to do with our
//! initial purpose.
//!
//! A windowing crate supporting luminance will typically provide native types by re-exporting
//! symbols (types, functions, etc.) from a windowing crate and the necessary code to make it
//! compatible with luminance. That means providing a way to access a backend type, which
//! implements the [luminance::backend] interface.
//!
//! [luminance]: https://crates.io/crates/luminance
//! [luminance-gl]: https://crates.io/crates/luminance-gl
//! [luminance-front]: https://crates.io/crates/luminance-front
//! [luminance::backend]: crate::backend
#![doc(
    html_logo_url = "https://github.com/phaazon/luminance-rs/blob/master/docs/imgs/luminance_alt.svg"
)]
#![deny(missing_docs)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
#[allow(clippy::missing_safety_doc)]
pub mod backend {
    //! Backend interfacing.
    //!
    //! Almost everything declared in this module and its submodules is `unsafe`. An end-user **is not
    //! supposed to implement any of this.** Library authors might use some traits from here, required
    //! by generic code, but no one but backend authors should implement any symbols from here.
    //!
    //! # Conventions
    //!
    //! ## Constrained types
    //!
    //! When a public symbol, like [`Buffer`], has type variables and the first one is `B`, it’s likely
    //! to be the _backend type_. Very often, that type will be constrained by a trait. For instance,
    //! for [`Buffer`], it is constrained by [`BufferBackend`]. That trait will provide the interface
    //! that backends need to implement to support the API type — in our case, [`Buffer`]. Implementing
    //! such traits is `unsafe` and using their methods is `unsafe` too.
    //!
    //! ## Associated types and representation objects
    //!
    //! You will notice, if you have a look at backend traits, that most of them have associated types.
    //! Most of them end with the `Repr` suffix. Those types are the concrete _representation_ of the
    //! general concept. For [`Buffer`], [`BufferBackend::BufferRepr`] must be provided by a backend
    //! and used with the rest of the methods of the [`BufferBackend`] trait. As with `unsafe` methods,
    //! accessing a `Repr` object is not possible from the public interface — if you have found a way
    //! to do without `unsafe`, this is a bug: please consider filing an issue.
    //!
    //! ## Relationship to GraphicsContext
    //!
    //! The [`GraphicsContext`] is an important trait as it is the parent trait of _platform backends_ —
    //! i.e. windowing. It makes it possible to use luminance with a wide variety of technologies,
    //! systems and platforms.
    //!
    //! On the other side, traits defined in this module — and its submodules — are the traits
    //! representing all concrete implementations of the code people will write with luminance. They
    //! contain the concrete implementation of what it means to create a new [`Buffer`] or set
    //! a value at a given index in it, for instance.
    //!
    //! [`GraphicsContext`] has an associated type, [`GraphicsContext::Backend`], that maps the
    //! type implementing [`GraphicsContext`] to a backend type. The graphics context doesn’t have to
    //! ship its backend, as they’re loosely coupled: it’s possible to write a backend implementation
    //! and use / share it as [`GraphicsContext::Backend`] in several system backends.
    //!
    //! [`GraphicsContext::Backend`] is surjective — i.e. all backends have a [`GraphicsContext`]
    //! mapped, which means that some backends are available via different graphics contexts. The
    //! implication is that:
    //!
    //! - Given a [`GraphicsContext`], you immediately know its associated backend.
    //! - Give a backend, there is no unique [`GraphicsContext`] you can map backwards, because
    //!   several [`GraphicsContext`] might use that backend.
    //!
    //! That property allows to write a backend type and use it in several graphics contexts.
    //!
    //! ## What should a backend crate expose?
    //!
    //! If you would like to implement your own backend, you must implement all the traits defined in
    //! this module — and its submodules. Your crate should then only expose a type — the backend
    //! type — and make it available to pick by end-users. The [`GraphicsContext::Backend`] associated
    //! type makes a strong contract to find all the other types you will be using in your crate, so you
    //! don’t have to worry too much about them.
    //!
    //! > Note: however, when implementing a public trait, all associated types must be `pub` too. So
    //! > it’s likely you will have to declare them `pub`, too.
    //!
    //! [`Buffer`]: crate::buffer::Buffer
    //! [`BufferBackend`]: crate::backend::buffer::Buffer
    //! [`BufferBackend::BufferRepr`]: crate::backend::buffer::Buffer::BufferRepr
    //! [`GraphicsContext`]: crate::context::GraphicsContext
    //! [`GraphicsContext::Backend`]: crate::context::GraphicsContext::Backend
    #![allow(missing_docs)]
    pub mod buffer {
        //! Buffer backend interface.
        //!
        //! This interface defines the low-level API buffers must implement to be usable.
        use crate::buffer::BufferError;
        pub unsafe trait Buffer<T>
        where
            T: Copy,
        {
            /// The inner representation of the buffer for this backend.
            type BufferRepr;
            /// Create a new buffer with a given number of uninitialized elements.
            unsafe fn new_buffer(&mut self, len: usize) -> Result<Self::BufferRepr, BufferError>
            where
                T: Default;
            unsafe fn len(buffer: &Self::BufferRepr) -> usize;
            unsafe fn from_vec(&mut self, vec: Vec<T>) -> Result<Self::BufferRepr, BufferError>;
            unsafe fn repeat(
                &mut self,
                len: usize,
                value: T,
            ) -> Result<Self::BufferRepr, BufferError>;
            unsafe fn at(buffer: &Self::BufferRepr, i: usize) -> Option<T>;
            unsafe fn whole(buffer: &Self::BufferRepr) -> Vec<T>;
            unsafe fn set(buffer: &mut Self::BufferRepr, i: usize, x: T)
                -> Result<(), BufferError>;
            unsafe fn write_whole(
                buffer: &mut Self::BufferRepr,
                values: &[T],
            ) -> Result<(), BufferError>;
            unsafe fn clear(buffer: &mut Self::BufferRepr, x: T) -> Result<(), BufferError>;
        }
        pub unsafe trait BufferSlice<T>: Buffer<T>
        where
            T: Copy,
        {
            type SliceRepr;
            type SliceMutRepr;
            unsafe fn slice_buffer(
                buffer: &Self::BufferRepr,
            ) -> Result<Self::SliceRepr, BufferError>;
            unsafe fn slice_buffer_mut(
                buffer: &mut Self::BufferRepr,
            ) -> Result<Self::SliceMutRepr, BufferError>;
            unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError>;
            unsafe fn obtain_slice_mut(
                slice: &mut Self::SliceMutRepr,
            ) -> Result<&mut [T], BufferError>;
        }
        pub unsafe trait UniformBlock {}
        unsafe impl UniformBlock for u8 {}
        unsafe impl UniformBlock for u16 {}
        unsafe impl UniformBlock for u32 {}
        unsafe impl UniformBlock for i8 {}
        unsafe impl UniformBlock for i16 {}
        unsafe impl UniformBlock for i32 {}
        unsafe impl UniformBlock for f32 {}
        unsafe impl UniformBlock for f64 {}
        unsafe impl UniformBlock for bool {}
        unsafe impl UniformBlock for [[f32; 2]; 2] {}
        unsafe impl UniformBlock for [[f32; 3]; 3] {}
        unsafe impl UniformBlock for [[f32; 4]; 4] {}
        unsafe impl UniformBlock for [u8; 2] {}
        unsafe impl UniformBlock for [u16; 2] {}
        unsafe impl UniformBlock for [u32; 2] {}
        unsafe impl UniformBlock for [i8; 2] {}
        unsafe impl UniformBlock for [i16; 2] {}
        unsafe impl UniformBlock for [i32; 2] {}
        unsafe impl UniformBlock for [f32; 2] {}
        unsafe impl UniformBlock for [f64; 2] {}
        unsafe impl UniformBlock for [bool; 2] {}
        unsafe impl UniformBlock for [u8; 3] {}
        unsafe impl UniformBlock for [u16; 3] {}
        unsafe impl UniformBlock for [u32; 3] {}
        unsafe impl UniformBlock for [i8; 3] {}
        unsafe impl UniformBlock for [i16; 3] {}
        unsafe impl UniformBlock for [i32; 3] {}
        unsafe impl UniformBlock for [f32; 3] {}
        unsafe impl UniformBlock for [f64; 3] {}
        unsafe impl UniformBlock for [bool; 3] {}
        unsafe impl UniformBlock for [u8; 4] {}
        unsafe impl UniformBlock for [u16; 4] {}
        unsafe impl UniformBlock for [u32; 4] {}
        unsafe impl UniformBlock for [i8; 4] {}
        unsafe impl UniformBlock for [i16; 4] {}
        unsafe impl UniformBlock for [i32; 4] {}
        unsafe impl UniformBlock for [f32; 4] {}
        unsafe impl UniformBlock for [f64; 4] {}
        unsafe impl UniformBlock for [bool; 4] {}
        unsafe impl<T> UniformBlock for [T] where T: UniformBlock {}
        unsafe impl<A, B> UniformBlock for (A, B)
        where
            A: UniformBlock,
            B: UniformBlock,
        {
        }
        unsafe impl<A, B, C> UniformBlock for (A, B, C)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D> UniformBlock for (A, B, C, D)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E> UniformBlock for (A, B, C, D, E)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E, F> UniformBlock for (A, B, C, D, E, F)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
            F: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E, F, G> UniformBlock for (A, B, C, D, E, F, G)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
            F: UniformBlock,
            G: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E, F, G, H> UniformBlock for (A, B, C, D, E, F, G, H)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
            F: UniformBlock,
            G: UniformBlock,
            H: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E, F, G, H, I> UniformBlock for (A, B, C, D, E, F, G, H, I)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
            F: UniformBlock,
            G: UniformBlock,
            H: UniformBlock,
            I: UniformBlock,
        {
        }
        unsafe impl<A, B, C, D, E, F, G, H, I, J> UniformBlock for (A, B, C, D, E, F, G, H, I, J)
        where
            A: UniformBlock,
            B: UniformBlock,
            C: UniformBlock,
            D: UniformBlock,
            E: UniformBlock,
            F: UniformBlock,
            G: UniformBlock,
            H: UniformBlock,
            I: UniformBlock,
            J: UniformBlock,
        {
        }
    }
    pub mod color_slot {
        //! Color slot backend interface.
        //!
        //! This interface defines the low-level API color slots must implement to be usable.
        use crate::backend::framebuffer::Framebuffer;
        use crate::backend::texture::Texture as TextureBackend;
        use crate::context::GraphicsContext;
        use crate::framebuffer::FramebufferError;
        use crate::pixel::{ColorPixel, PixelFormat, RenderablePixel};
        use crate::texture::{Dimensionable, Sampler};
        use crate::texture::Texture;
        pub trait ColorSlot<B, D>
        where
            B: ?Sized + Framebuffer<D>,
            D: Dimensionable,
            D::Size: Copy,
        {
            type ColorTextures;
            fn color_formats() -> Vec<PixelFormat>;
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>;
        }
        impl<B, D> ColorSlot<B, D> for ()
        where
            B: ?Sized + Framebuffer<D>,
            D: Dimensionable,
            D::Size: Copy,
        {
            type ColorTextures = ();
            fn color_formats() -> Vec<PixelFormat> {
                Vec::new()
            }
            fn reify_color_textures<C>(
                _: &mut C,
                _: D::Size,
                _: usize,
                _: &Sampler,
                _: &mut B::FramebufferRepr,
                _: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                Ok(())
            }
        }
        impl<B, D, P> ColorSlot<B, D> for P
        where
            B: ?Sized + Framebuffer<D> + TextureBackend<D, P>,
            D: Dimensionable,
            D::Size: Copy,
            Self: ColorPixel + RenderablePixel,
        {
            type ColorTextures = Texture<B, D, P>;
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [P::pixel_format()])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let texture = Texture::new(ctx, size, mipmaps, sampler.clone())?;
                unsafe { B::attach_color_texture(framebuffer, &texture.repr, attachment_index)? };
                Ok(texture)
            }
        }
        impl<B, D, P10, P11> ColorSlot<B, D> for (P10, P11)
        where
            B: ?Sized + Framebuffer<D> + TextureBackend<D, P10> + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (Texture<B, D, P10>, Texture<B, D, P11>);
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [P10::pixel_format(), P11::pixel_format()])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P10 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P9, P10, P11> ColorSlot<B, D> for (P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (Texture<B, D, P9>, Texture<B, D, P10>, Texture<B, D, P11>);
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [P9::pixel_format(), P10::pixel_format(), P11::pixel_format()])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P9 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P8, P9, P10, P11> ColorSlot<B, D> for (P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P8 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P7, P8, P9, P10, P11> ColorSlot<B, D> for (P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P7 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P6, P7, P8, P9, P10, P11> ColorSlot<B, D> for (P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P6 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D> for (P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P5 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P4, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D> for (P4, P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P4>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P4: ColorPixel + RenderablePixel,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P4>,
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P4::pixel_format(),
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P4 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P5 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P3, P4, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D>
            for (P3, P4, P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P3>
                + TextureBackend<D, P4>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P3: ColorPixel + RenderablePixel,
            P4: ColorPixel + RenderablePixel,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P3>,
                Texture<B, D, P4>,
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P3::pixel_format(),
                    P4::pixel_format(),
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P3 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P4 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P5 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D>
            for (P2, P3, P4, P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P2>
                + TextureBackend<D, P3>
                + TextureBackend<D, P4>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P2: ColorPixel + RenderablePixel,
            P3: ColorPixel + RenderablePixel,
            P4: ColorPixel + RenderablePixel,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P2>,
                Texture<B, D, P3>,
                Texture<B, D, P4>,
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P2::pixel_format(),
                    P3::pixel_format(),
                    P4::pixel_format(),
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P2 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P3 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P4 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P5 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D>
            for (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P1>
                + TextureBackend<D, P2>
                + TextureBackend<D, P3>
                + TextureBackend<D, P4>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P1: ColorPixel + RenderablePixel,
            P2: ColorPixel + RenderablePixel,
            P3: ColorPixel + RenderablePixel,
            P4: ColorPixel + RenderablePixel,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P1>,
                Texture<B, D, P2>,
                Texture<B, D, P3>,
                Texture<B, D, P4>,
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P1::pixel_format(),
                    P2::pixel_format(),
                    P3::pixel_format(),
                    P4::pixel_format(),
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P1 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P2 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P3 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P4 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P5 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
        impl<B, D, P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11> ColorSlot<B, D>
            for (P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11)
        where
            B: ?Sized
                + Framebuffer<D>
                + TextureBackend<D, P0>
                + TextureBackend<D, P1>
                + TextureBackend<D, P2>
                + TextureBackend<D, P3>
                + TextureBackend<D, P4>
                + TextureBackend<D, P5>
                + TextureBackend<D, P6>
                + TextureBackend<D, P7>
                + TextureBackend<D, P8>
                + TextureBackend<D, P9>
                + TextureBackend<D, P10>
                + TextureBackend<D, P11>,
            D: Dimensionable,
            D::Size: Copy,
            P0: ColorPixel + RenderablePixel,
            P1: ColorPixel + RenderablePixel,
            P2: ColorPixel + RenderablePixel,
            P3: ColorPixel + RenderablePixel,
            P4: ColorPixel + RenderablePixel,
            P5: ColorPixel + RenderablePixel,
            P6: ColorPixel + RenderablePixel,
            P7: ColorPixel + RenderablePixel,
            P8: ColorPixel + RenderablePixel,
            P9: ColorPixel + RenderablePixel,
            P10: ColorPixel + RenderablePixel,
            P11: ColorPixel + RenderablePixel,
        {
            type ColorTextures = (
                Texture<B, D, P0>,
                Texture<B, D, P1>,
                Texture<B, D, P2>,
                Texture<B, D, P3>,
                Texture<B, D, P4>,
                Texture<B, D, P5>,
                Texture<B, D, P6>,
                Texture<B, D, P7>,
                Texture<B, D, P8>,
                Texture<B, D, P9>,
                Texture<B, D, P10>,
                Texture<B, D, P11>,
            );
            fn color_formats() -> Vec<PixelFormat> {
                <[_]>::into_vec(box [
                    P0::pixel_format(),
                    P1::pixel_format(),
                    P2::pixel_format(),
                    P3::pixel_format(),
                    P4::pixel_format(),
                    P5::pixel_format(),
                    P6::pixel_format(),
                    P7::pixel_format(),
                    P8::pixel_format(),
                    P9::pixel_format(),
                    P10::pixel_format(),
                    P11::pixel_format(),
                ])
            }
            fn reify_color_textures<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
                mut attachment_index: usize,
            ) -> Result<Self::ColorTextures, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let textures = (
                    <P0 as ColorSlot<B, D>>::reify_color_textures(
                        ctx,
                        size,
                        mipmaps,
                        sampler,
                        framebuffer,
                        attachment_index,
                    )?,
                    {
                        attachment_index += 1;
                        let texture = <P1 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P2 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P3 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P4 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P5 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P6 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P7 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P8 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P9 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P10 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                    {
                        attachment_index += 1;
                        let texture = <P11 as ColorSlot<B, D>>::reify_color_textures(
                            ctx,
                            size,
                            mipmaps,
                            sampler,
                            framebuffer,
                            attachment_index,
                        )?;
                        texture
                    },
                );
                Ok(textures)
            }
        }
    }
    pub mod depth_slot {
        //! Depth slot backend interface.
        //!
        //! This interface defines the low-level API depth slots must implement to be usable.
        use crate::backend::framebuffer::Framebuffer;
        use crate::backend::texture::Texture as TextureBackend;
        use crate::context::GraphicsContext;
        use crate::framebuffer::FramebufferError;
        use crate::pixel::{DepthPixel, PixelFormat};
        use crate::texture::{Dimensionable, Sampler};
        use crate::texture::Texture;
        pub trait DepthSlot<B, D>
        where
            B: ?Sized + Framebuffer<D>,
            D: Dimensionable,
            D::Size: Copy,
        {
            /// Texture associated with this color slot.
            type DepthTexture;
            /// Turn a depth slot into a pixel format.
            fn depth_format() -> Option<PixelFormat>;
            /// Reify a raw textures into a depth slot.
            fn reify_depth_texture<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
            ) -> Result<Self::DepthTexture, FramebufferError>
            where
                C: GraphicsContext<Backend = B>;
        }
        impl<B, D> DepthSlot<B, D> for ()
        where
            B: ?Sized + Framebuffer<D>,
            D::Size: Copy,
            D: Dimensionable,
        {
            type DepthTexture = ();
            fn depth_format() -> Option<PixelFormat> {
                None
            }
            fn reify_depth_texture<C>(
                _: &mut C,
                _: D::Size,
                _: usize,
                _: &Sampler,
                _: &mut B::FramebufferRepr,
            ) -> Result<Self::DepthTexture, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                Ok(())
            }
        }
        impl<B, D, P> DepthSlot<B, D> for P
        where
            B: ?Sized + Framebuffer<D> + TextureBackend<D, P>,
            D: Dimensionable,
            D::Size: Copy,
            P: DepthPixel,
        {
            type DepthTexture = Texture<B, D, P>;
            fn depth_format() -> Option<PixelFormat> {
                Some(P::pixel_format())
            }
            fn reify_depth_texture<C>(
                ctx: &mut C,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
                framebuffer: &mut B::FramebufferRepr,
            ) -> Result<Self::DepthTexture, FramebufferError>
            where
                C: GraphicsContext<Backend = B>,
            {
                let texture = Texture::new(ctx, size, mipmaps, *sampler)?;
                unsafe { B::attach_depth_texture(framebuffer, &texture.repr)? };
                Ok(texture)
            }
        }
    }
    pub mod framebuffer {
        //! Framebuffer backend interface.
        //!
        //! This interface defines the low-level API framebuffers must implement to be usable.
        use crate::backend::color_slot::ColorSlot;
        use crate::backend::depth_slot::DepthSlot;
        use crate::backend::texture::TextureBase;
        use crate::framebuffer::FramebufferError;
        use crate::texture::{Dim2, Dimensionable, Sampler};
        pub unsafe trait Framebuffer<D>: TextureBase
        where
            D: Dimensionable,
        {
            type FramebufferRepr;
            unsafe fn new_framebuffer<CS, DS>(
                &mut self,
                size: D::Size,
                mipmaps: usize,
                sampler: &Sampler,
            ) -> Result<Self::FramebufferRepr, FramebufferError>
            where
                CS: ColorSlot<Self, D>,
                DS: DepthSlot<Self, D>;
            unsafe fn attach_color_texture(
                framebuffer: &mut Self::FramebufferRepr,
                texture: &Self::TextureRepr,
                attachment_index: usize,
            ) -> Result<(), FramebufferError>;
            unsafe fn attach_depth_texture(
                framebuffer: &mut Self::FramebufferRepr,
                texture: &Self::TextureRepr,
            ) -> Result<(), FramebufferError>;
            unsafe fn validate_framebuffer(
                framebuffer: Self::FramebufferRepr,
            ) -> Result<Self::FramebufferRepr, FramebufferError>;
            unsafe fn framebuffer_size(framebuffer: &Self::FramebufferRepr) -> D::Size;
        }
        pub unsafe trait FramebufferBackBuffer: Framebuffer<Dim2> {
            unsafe fn back_buffer(
                &mut self,
                size: <Dim2 as Dimensionable>::Size,
            ) -> Result<Self::FramebufferRepr, FramebufferError>;
        }
    }
    pub mod pipeline {
        //! Pipeline backend interface.
        //!
        //! This interface defines the low-level API pipelines must implement to be usable.
        use crate::backend::buffer::Buffer;
        use crate::backend::framebuffer::Framebuffer as FramebufferBackend;
        use crate::backend::shading_gate::ShadingGate as ShadingGateBackend;
        use crate::backend::texture::{Texture, TextureBase};
        use crate::pipeline::{PipelineError, PipelineState};
        use crate::pixel::Pixel;
        use crate::texture::Dimensionable;
        pub unsafe trait PipelineBase: ShadingGateBackend + TextureBase {
            type PipelineRepr;
            unsafe fn new_pipeline(&mut self) -> Result<Self::PipelineRepr, PipelineError>;
        }
        pub unsafe trait Pipeline<D>: PipelineBase + FramebufferBackend<D>
        where
            D: Dimensionable,
        {
            unsafe fn start_pipeline(
                &mut self,
                framebuffer: &Self::FramebufferRepr,
                pipeline_state: &PipelineState,
            );
        }
        pub unsafe trait PipelineBuffer<T>: PipelineBase + Buffer<T>
        where
            T: Copy,
        {
            type BoundBufferRepr;
            unsafe fn bind_buffer(
                pipeline: &Self::PipelineRepr,
                buffer: &Self::BufferRepr,
            ) -> Result<Self::BoundBufferRepr, PipelineError>;
            unsafe fn buffer_binding(bound: &Self::BoundBufferRepr) -> u32;
        }
        pub unsafe trait PipelineTexture<D, P>: PipelineBase + Texture<D, P>
        where
            D: Dimensionable,
            P: Pixel,
        {
            type BoundTextureRepr;
            unsafe fn bind_texture(
                pipeline: &Self::PipelineRepr,
                texture: &Self::TextureRepr,
            ) -> Result<Self::BoundTextureRepr, PipelineError>
            where
                D: Dimensionable,
                P: Pixel;
            unsafe fn texture_binding(bound: &Self::BoundTextureRepr) -> u32;
        }
    }
    pub mod render_gate {
        //! Render gate backend interface.
        //!
        //! This interface defines the low-level API render gates must implement to be usable.
        use crate::render_state::RenderState;
        pub unsafe trait RenderGate {
            unsafe fn enter_render_state(&mut self, rdr_st: &RenderState);
        }
    }
    pub mod shader {
        //! Shader backend interface.
        //!
        //! This interface defines the low-level API shaders must implement to be usable.
        use crate::shader::{
            ProgramError, StageError, StageType, TessellationStages, Uniform, UniformType,
            UniformWarning, VertexAttribWarning,
        };
        use crate::vertex::Semantics;
        pub unsafe trait Uniformable<S>
        where
            S: ?Sized + Shader,
        {
            unsafe fn ty() -> UniformType;
            unsafe fn update(self, program: &mut S::ProgramRepr, uniform: &Uniform<Self>);
        }
        pub unsafe trait Shader {
            type StageRepr;
            type ProgramRepr;
            type UniformBuilderRepr;
            unsafe fn new_stage(
                &mut self,
                ty: StageType,
                src: &str,
            ) -> Result<Self::StageRepr, StageError>;
            unsafe fn new_program(
                &mut self,
                vertex: &Self::StageRepr,
                tess: Option<TessellationStages<Self::StageRepr>>,
                geometry: Option<&Self::StageRepr>,
                fragment: &Self::StageRepr,
            ) -> Result<Self::ProgramRepr, ProgramError>;
            unsafe fn apply_semantics<Sem>(
                program: &mut Self::ProgramRepr,
            ) -> Result<Vec<VertexAttribWarning>, ProgramError>
            where
                Sem: Semantics;
            unsafe fn new_uniform_builder(
                program: &mut Self::ProgramRepr,
            ) -> Result<Self::UniformBuilderRepr, ProgramError>;
            unsafe fn ask_uniform<T>(
                uniform_builder: &mut Self::UniformBuilderRepr,
                name: &str,
            ) -> Result<Uniform<T>, UniformWarning>
            where
                T: Uniformable<Self>;
            unsafe fn unbound<T>(uniform_builder: &mut Self::UniformBuilderRepr) -> Uniform<T>
            where
                T: Uniformable<Self>;
        }
    }
    pub mod shading_gate {
        //! Shading gates backend interface.
        //!
        //! This interface defines the low-level API shading gates must implement to be usable.
        use crate::backend::shader::Shader as ShaderBackend;
        pub unsafe trait ShadingGate: ShaderBackend {
            unsafe fn apply_shader_program(&mut self, shader_program: &Self::ProgramRepr);
        }
    }
    pub mod tess {
        //! Tessellation backend interface.
        //!
        //! This interface defines the low-level API tessellations must implement to be usable.
        use std::ops::{Deref, DerefMut};
        use crate::tess::{Mode, TessError, TessIndex, TessMapError, TessVertexData};
        pub unsafe trait Tess<V, I, W, S>
        where
            V: TessVertexData<S>,
            I: TessIndex,
            W: TessVertexData<S>,
            S: ?Sized,
        {
            type TessRepr;
            unsafe fn build(
                &mut self,
                vert: (Option<V::Data>, usize),
                inst: (Option<W::Data>, usize),
                index_data: Vec<I>,
                restart_index: Option<I>,
                mode: Mode,
            ) -> Result<Self::TessRepr, TessError>;
            unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;
            unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;
            unsafe fn render(
                tess: &Self::TessRepr,
                start_index: usize,
                vert_nb: usize,
                inst_nb: usize,
            ) -> Result<(), TessError>;
        }
        pub unsafe trait VertexSlice<V, I, W, S, T>: Tess<V, I, W, S>
        where
            V: TessVertexData<S>,
            I: TessIndex,
            W: TessVertexData<S>,
            S: ?Sized,
        {
            type VertexSliceRepr: Deref<Target = [T]>;
            type VertexSliceMutRepr: DerefMut<Target = [T]>;
            unsafe fn vertices(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::VertexSliceRepr, TessMapError>;
            unsafe fn vertices_mut(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::VertexSliceMutRepr, TessMapError>;
        }
        pub unsafe trait IndexSlice<V, I, W, S>: Tess<V, I, W, S>
        where
            V: TessVertexData<S>,
            I: TessIndex,
            W: TessVertexData<S>,
            S: ?Sized,
        {
            type IndexSliceRepr: Deref<Target = [I]>;
            type IndexSliceMutRepr: DerefMut<Target = [I]>;
            unsafe fn indices(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::IndexSliceRepr, TessMapError>;
            unsafe fn indices_mut(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::IndexSliceMutRepr, TessMapError>;
        }
        pub unsafe trait InstanceSlice<V, I, W, S, T>: Tess<V, I, W, S>
        where
            V: TessVertexData<S>,
            I: TessIndex,
            W: TessVertexData<S>,
            S: ?Sized,
        {
            type InstanceSliceRepr: Deref<Target = [T]>;
            type InstanceSliceMutRepr: DerefMut<Target = [T]>;
            unsafe fn instances(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::InstanceSliceRepr, TessMapError>;
            unsafe fn instances_mut(
                tess: &mut Self::TessRepr,
            ) -> Result<Self::InstanceSliceMutRepr, TessMapError>;
        }
    }
    pub mod tess_gate {
        //! Tessellation gate backend interface.
        //!
        //! This interface defines the low-level API tessellation gates must implement to be usable.
        use crate::backend::tess::Tess;
        use crate::tess::{TessIndex, TessVertexData};
        pub unsafe trait TessGate<V, I, W, S>: Tess<V, I, W, S>
        where
            V: TessVertexData<S>,
            I: TessIndex,
            W: TessVertexData<S>,
            S: ?Sized,
        {
            unsafe fn render(
                &mut self,
                tess: &Self::TessRepr,
                start_index: usize,
                vert_nb: usize,
                inst_nb: usize,
            );
        }
    }
    pub mod texture {
        //! Texture backend interface.
        //!
        //! This interface defines the low-level API textures must implement to be usable.
        use crate::pixel::Pixel;
        use crate::texture::{Dimensionable, GenMipmaps, Sampler, TextureError};
        /// The base texture trait.
        pub unsafe trait TextureBase {
            type TextureRepr;
        }
        pub unsafe trait Texture<D, P>: TextureBase
        where
            D: Dimensionable,
            P: Pixel,
        {
            unsafe fn new_texture(
                &mut self,
                size: D::Size,
                mipmaps: usize,
                sampler: Sampler,
            ) -> Result<Self::TextureRepr, TextureError>;
            unsafe fn mipmaps(texture: &Self::TextureRepr) -> usize;
            unsafe fn clear_part(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                offset: D::Offset,
                size: D::Size,
                pixel: P::Encoding,
            ) -> Result<(), TextureError>;
            unsafe fn clear(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                size: D::Size,
                pixel: P::Encoding,
            ) -> Result<(), TextureError>;
            unsafe fn upload_part(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                offset: D::Offset,
                size: D::Size,
                texels: &[P::Encoding],
            ) -> Result<(), TextureError>;
            unsafe fn upload(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                size: D::Size,
                texels: &[P::Encoding],
            ) -> Result<(), TextureError>;
            unsafe fn upload_part_raw(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                offset: D::Offset,
                size: D::Size,
                texels: &[P::RawEncoding],
            ) -> Result<(), TextureError>;
            unsafe fn upload_raw(
                texture: &mut Self::TextureRepr,
                gen_mipmaps: GenMipmaps,
                size: D::Size,
                texels: &[P::RawEncoding],
            ) -> Result<(), TextureError>;
            unsafe fn get_raw_texels(
                texture: &Self::TextureRepr,
                size: D::Size,
            ) -> Result<Vec<P::RawEncoding>, TextureError>
            where
                P::RawEncoding: Copy + Default;
        }
    }
}
pub mod blending {
    //! That module exports blending-related types and functions.
    //!
    //! Given two pixels *src* and *dst* – source and destination, we associate each pixel a blending
    //! factor – respectively, *srcK* and *dstK*. *src* is the pixel being computed, and *dst* is the
    //! pixel that is already stored in the framebuffer.
    //!
    //! The pixels can be blended in several ways. See the documentation of [`Equation`] for further
    //! details.
    //!
    //! The factors are encoded with [`Factor`].
    //!
    //! [`Equation`]: crate::blending::Equation
    //! [`Factor`]: crate::blending::Factor
    /// Blending equation. Used to state how blending factors and pixel data should be blended.
    pub enum Equation {
        /// `Additive` represents the following blending equation:
        ///
        /// > `blended = src * srcK + dst * dstK`
        Additive,
        /// `Subtract` represents the following blending equation:
        ///
        /// > `blended = src * srcK - dst * dstK`
        Subtract,
        /// Because subtracting is not commutative, `ReverseSubtract` represents the following additional
        /// blending equation:
        ///
        /// > `blended = dst * dstK - src * srcK`
        ReverseSubtract,
        /// `Min` represents the following blending equation:
        ///
        /// > `blended = min(src, dst)`
        Min,
        /// `Max` represents the following blending equation:
        ///
        /// > `blended = max(src, dst)`
        Max,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Equation {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Equation {
        #[inline]
        fn clone(&self) -> Equation {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Equation {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Equation::Additive,) => {
                    let mut debug_trait_builder = f.debug_tuple("Additive");
                    debug_trait_builder.finish()
                }
                (&Equation::Subtract,) => {
                    let mut debug_trait_builder = f.debug_tuple("Subtract");
                    debug_trait_builder.finish()
                }
                (&Equation::ReverseSubtract,) => {
                    let mut debug_trait_builder = f.debug_tuple("ReverseSubtract");
                    debug_trait_builder.finish()
                }
                (&Equation::Min,) => {
                    let mut debug_trait_builder = f.debug_tuple("Min");
                    debug_trait_builder.finish()
                }
                (&Equation::Max,) => {
                    let mut debug_trait_builder = f.debug_tuple("Max");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Equation {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Equation {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for Equation {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Equation {
        #[inline]
        fn eq(&self, other: &Equation) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Blending factors. Pixel data are multiplied by these factors to achieve several effects driven
    /// by *blending equations*.
    pub enum Factor {
        /// `1 * color = color`
        One,
        /// `0 * color = 0`
        Zero,
        /// `src * color`
        SrcColor,
        /// `(1 - src) * color`
        SrcColorComplement,
        /// `dst * color`
        DestColor,
        /// `(1 - dst) * color`
        DestColorComplement,
        /// `srcA * color`
        SrcAlpha,
        /// `(1 - src) * color`
        SrcAlphaComplement,
        /// `dstA * color`
        DstAlpha,
        /// `(1 - dstA) * color`
        DstAlphaComplement,
        /// For colors, `min(srcA, 1 - dstA)`, for alpha, `1`
        SrcAlphaSaturate,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Factor {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Factor {
        #[inline]
        fn clone(&self) -> Factor {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Factor {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Factor::One,) => {
                    let mut debug_trait_builder = f.debug_tuple("One");
                    debug_trait_builder.finish()
                }
                (&Factor::Zero,) => {
                    let mut debug_trait_builder = f.debug_tuple("Zero");
                    debug_trait_builder.finish()
                }
                (&Factor::SrcColor,) => {
                    let mut debug_trait_builder = f.debug_tuple("SrcColor");
                    debug_trait_builder.finish()
                }
                (&Factor::SrcColorComplement,) => {
                    let mut debug_trait_builder = f.debug_tuple("SrcColorComplement");
                    debug_trait_builder.finish()
                }
                (&Factor::DestColor,) => {
                    let mut debug_trait_builder = f.debug_tuple("DestColor");
                    debug_trait_builder.finish()
                }
                (&Factor::DestColorComplement,) => {
                    let mut debug_trait_builder = f.debug_tuple("DestColorComplement");
                    debug_trait_builder.finish()
                }
                (&Factor::SrcAlpha,) => {
                    let mut debug_trait_builder = f.debug_tuple("SrcAlpha");
                    debug_trait_builder.finish()
                }
                (&Factor::SrcAlphaComplement,) => {
                    let mut debug_trait_builder = f.debug_tuple("SrcAlphaComplement");
                    debug_trait_builder.finish()
                }
                (&Factor::DstAlpha,) => {
                    let mut debug_trait_builder = f.debug_tuple("DstAlpha");
                    debug_trait_builder.finish()
                }
                (&Factor::DstAlphaComplement,) => {
                    let mut debug_trait_builder = f.debug_tuple("DstAlphaComplement");
                    debug_trait_builder.finish()
                }
                (&Factor::SrcAlphaSaturate,) => {
                    let mut debug_trait_builder = f.debug_tuple("SrcAlphaSaturate");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Factor {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Factor {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for Factor {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Factor {
        #[inline]
        fn eq(&self, other: &Factor) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Basic blending configuration.
    pub struct Blending {
        /// Blending equation to use.
        pub equation: Equation,
        /// Source factor.
        pub src: Factor,
        /// Destination factor.
        pub dst: Factor,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Blending {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Blending {
        #[inline]
        fn clone(&self) -> Blending {
            {
                let _: ::core::clone::AssertParamIsClone<Equation>;
                let _: ::core::clone::AssertParamIsClone<Factor>;
                let _: ::core::clone::AssertParamIsClone<Factor>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Blending {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Blending {
                    equation: ref __self_0_0,
                    src: ref __self_0_1,
                    dst: ref __self_0_2,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Blending");
                    let _ = debug_trait_builder.field("equation", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("src", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("dst", &&(*__self_0_2));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Blending {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Blending {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Equation>;
                let _: ::core::cmp::AssertParamIsEq<Factor>;
                let _: ::core::cmp::AssertParamIsEq<Factor>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Blending {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Blending {
        #[inline]
        fn eq(&self, other: &Blending) -> bool {
            match *other {
                Blending {
                    equation: ref __self_1_0,
                    src: ref __self_1_1,
                    dst: ref __self_1_2,
                } => match *self {
                    Blending {
                        equation: ref __self_0_0,
                        src: ref __self_0_1,
                        dst: ref __self_0_2,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &Blending) -> bool {
            match *other {
                Blending {
                    equation: ref __self_1_0,
                    src: ref __self_1_1,
                    dst: ref __self_1_2,
                } => match *self {
                    Blending {
                        equation: ref __self_0_0,
                        src: ref __self_0_1,
                        dst: ref __self_0_2,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                    }
                },
            }
        }
    }
    /// Blending configuration to represent combined or separate options.
    pub enum BlendingMode {
        /// Blending with combined RGBA.
        Combined(Blending),
        /// Blending with RGB and alpha separately.
        Separate {
            /// Blending configuration for RGB components.
            rgb: Blending,
            /// Blending configuration for alpha component.
            alpha: Blending,
        },
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for BlendingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for BlendingMode {
        #[inline]
        fn clone(&self) -> BlendingMode {
            {
                let _: ::core::clone::AssertParamIsClone<Blending>;
                let _: ::core::clone::AssertParamIsClone<Blending>;
                let _: ::core::clone::AssertParamIsClone<Blending>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for BlendingMode {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&BlendingMode::Combined(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Combined");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&BlendingMode::Separate {
                    rgb: ref __self_0,
                    alpha: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("Separate");
                    let _ = debug_trait_builder.field("rgb", &&(*__self_0));
                    let _ = debug_trait_builder.field("alpha", &&(*__self_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for BlendingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for BlendingMode {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Blending>;
                let _: ::core::cmp::AssertParamIsEq<Blending>;
                let _: ::core::cmp::AssertParamIsEq<Blending>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for BlendingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for BlendingMode {
        #[inline]
        fn eq(&self, other: &BlendingMode) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &BlendingMode::Combined(ref __self_0),
                            &BlendingMode::Combined(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &BlendingMode::Separate {
                                rgb: ref __self_0,
                                alpha: ref __self_1,
                            },
                            &BlendingMode::Separate {
                                rgb: ref __arg_1_0,
                                alpha: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &BlendingMode) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &BlendingMode::Combined(ref __self_0),
                            &BlendingMode::Combined(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &BlendingMode::Separate {
                                rgb: ref __self_0,
                                alpha: ref __self_1,
                            },
                            &BlendingMode::Separate {
                                rgb: ref __arg_1_0,
                                alpha: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    true
                }
            }
        }
    }
    impl From<Blending> for BlendingMode {
        fn from(blending: Blending) -> Self {
            BlendingMode::Combined(blending)
        }
    }
}
pub mod buffer {
    //! Graphics buffers.
    //!
    //! A GPU buffer is a typed continuous region of data. It has a size and can hold several elements.
    //!
    //! Once the buffer is created, you can perform several operations on it:
    //!
    //! - Writing to it.
    //! - Reading from it.
    //! - Passing it around as uniforms.
    //! - Etc.
    //!
    //! # Parametricity
    //!
    //! The [`Buffer`] type is parametric over the backend type and item type. For the backend type,
    //! the [`backend::buffer::Buffer`] trait must be implemented to be able to use it with the buffe.
    //!
    //! # Buffer creation, reading, writing and getting information
    //!
    //! Buffers are created with the [`Buffer::new`], [`Buffer::from_vec`] and [`Buffer::repeat`]
    //! methods. All these methods are fallible — they might fail with [`BufferError`].
    //!
    //! Once you have a [`Buffer`], you can read from it and write to it.
    //! Writing is done with [`Buffer::set`] — which allows to set a value at a given index in the
    //! buffer, [`Buffer::write_whole`] — which writes a whole slice to the buffer — and
    //! [`Buffer::clear`] — which sets the same value to all items in a buffer. Reading is performed
    //! with [`Buffer::at`] — which retrieves the item at a given index and [`Buffer::whole`] which
    //! retrieves the whole buffer by copying it to a `Vec<T>`.
    //!
    //! It’s possible to get data via several methods, such as [`Buffer::len`] to get the number of
    //! items in the buffer.
    //!
    //! # Buffer slicing
    //!
    //! Buffer slicing is the act of getting a [`BufferSlice`] out of a [`Buffer`]. That operation
    //! allows to _map_ a GPU region onto a CPU one and access the underlying data as a regular slice.
    //! Two methods exist to slice a buffer
    //!
    //! - Read-only: [`Buffer::slice`].
    //! - Mutably: [`Buffer::slice_mut`].
    //!
    //! Both methods take a mutable reference on a buffer because even in the read-only case, exclusive
    //! borrowing must be enforced.
    //!
    //! [`backend::buffer::Buffer`]: crate::backend::buffer::Buffer
    use crate::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
    use crate::context::GraphicsContext;
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;
    /// A GPU buffer.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [`backend::buffer::Buffer`].
    /// -`T` is the type of stored items. No restriction are currently enforced on that type, besides
    ///   the fact it must be [`Sized`].
    ///
    /// [`backend::buffer::Buffer`]: crate::backend::buffer::Buffer
    pub struct Buffer<B, T>
    where
        B: ?Sized + BufferBackend<T>,
        T: Copy,
    {
        pub(crate) repr: B::BufferRepr,
        _t: PhantomData<T>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<B: ::core::fmt::Debug, T: ::core::fmt::Debug> ::core::fmt::Debug for Buffer<B, T>
    where
        B: ?Sized + BufferBackend<T>,
        T: Copy,
        B::BufferRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Buffer {
                    repr: ref __self_0_0,
                    _t: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Buffer");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_t", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, T> Buffer<B, T>
    where
        B: ?Sized + BufferBackend<T>,
        T: Copy,
    {
        /// Create a new buffer with a given length
        ///
        /// The buffer will be created on the GPU with a contiguous region large enough to fit `len`
        /// items.
        ///
        /// The stored item must be [`Default`], as this function will initialize the whole buffer
        /// with the default value.
        ///
        /// # Errors
        ///
        /// That function can fail creating the buffer for various reasons, in which case it returns
        /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
        /// information.
        ///
        /// # Notes
        ///
        /// You might be interested in the [`GraphicsContext::new_buffer`] function instead, which
        /// is the exact same function, but benefits from more type inference (based on `&mut C`).
        pub fn new<C>(ctx: &mut C, len: usize) -> Result<Self, BufferError>
        where
            C: GraphicsContext<Backend = B>,
            T: Default,
        {
            let repr = unsafe { ctx.backend().new_buffer(len)? };
            Ok(Buffer {
                repr,
                _t: PhantomData,
            })
        }
        /// Create a new buffer from a slice of items.
        ///
        /// The buffer will be created with a length equal to the length of the input size, and items
        /// will be copied from the slice inside the contiguous GPU region.
        ///
        /// # Errors
        ///
        /// That function can fail creating the buffer for various reasons, in which case it returns
        /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
        /// information.
        ///
        /// # Notes
        ///
        /// You might be interested in the [`GraphicsContext::new_buffer_from_vec`] function instead,
        /// which is the exact same function, but benefits from more type inference (based on `&mut C`).
        pub fn from_vec<C, X>(ctx: &mut C, vec: X) -> Result<Self, BufferError>
        where
            C: GraphicsContext<Backend = B>,
            X: Into<Vec<T>>,
        {
            let repr = unsafe { ctx.backend().from_vec(vec.into())? };
            Ok(Buffer {
                repr,
                _t: PhantomData,
            })
        }
        /// Create a new buffer by repeating `len` times a `value`.
        ///
        /// The buffer will be comprised of `len` items, all equal to `value`.
        ///
        /// # Errors
        ///
        /// That function can fail creating the buffer for various reasons, in which case it returns
        /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
        /// information.
        ///
        /// # Notes
        ///
        /// You might be interested in the [`GraphicsContext::new_buffer_repeating`] function instead,
        /// which is the exact same function, but benefits from more type inference (based on `&mut C`).
        pub fn repeat<C>(ctx: &mut C, len: usize, value: T) -> Result<Self, BufferError>
        where
            C: GraphicsContext<Backend = B>,
        {
            let repr = unsafe { ctx.backend().repeat(len, value)? };
            Ok(Buffer {
                repr,
                _t: PhantomData,
            })
        }
        /// Get the item at the given index.
        pub fn at(&self, i: usize) -> Option<T> {
            unsafe { B::at(&self.repr, i) }
        }
        /// Get the whole content of the buffer and store it inside a [`Vec`].
        pub fn whole(&self) -> Vec<T> {
            unsafe { B::whole(&self.repr) }
        }
        /// Set a value `x` at index `i` in the buffer.
        ///
        /// # Errors
        ///
        /// That function returns [`BufferError::Overflow`] if `i` is bigger than the length of the
        /// buffer. Other errors are possible; please consider reading the documentation of
        /// [`BufferError`] for further information.
        pub fn set(&mut self, i: usize, x: T) -> Result<(), BufferError> {
            unsafe { B::set(&mut self.repr, i, x) }
        }
        /// Set the content of the buffer by using a slice that will be copied at the buffer’s memory
        /// location.
        ///
        /// # Errors
        ///
        /// [`BufferError::TooFewValues`] is returned if the input slice has less items than the buffer.
        ///
        /// [`BufferError::TooManyValues`] is returned if the input slice has more items than the buffer.
        pub fn write_whole(&mut self, values: &[T]) -> Result<(), BufferError> {
            unsafe { B::write_whole(&mut self.repr, values) }
        }
        /// Clear the content of the buffer by copying the same value everywhere.
        pub fn clear(&mut self, x: T) -> Result<(), BufferError> {
            unsafe { B::clear(&mut self.repr, x) }
        }
        /// Return the length of the buffer (i.e. the number of elements).
        #[inline(always)]
        pub fn len(&self) -> usize {
            unsafe { B::len(&self.repr) }
        }
        /// Check whether the buffer is empty (i.e. it has no elements).
        ///
        /// # Note
        ///
        /// Since right now, it is not possible to grow vectors, it is highly recommended not to create
        /// empty buffers. That function is there only for convenience and demonstration; you shouldn’t
        /// really have to use it.
        #[inline(always)]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }
    impl<B, T> Buffer<B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
    {
        /// Create a new [`BufferSlice`] from a buffer, allowing to get `&[T]` out of it.
        ///
        /// # Errors
        ///
        /// That function might fail and return a [`BufferError::MapFailed`].
        pub fn slice(&mut self) -> Result<BufferSlice<B, T>, BufferError> {
            unsafe {
                B::slice_buffer(&self.repr).map(|slice| BufferSlice {
                    slice,
                    _a: PhantomData,
                })
            }
        }
        /// Create a new [`BufferSliceMut`] from a buffer, allowing to get `&mut [T]` out of it.
        ///
        /// # Errors
        ///
        /// That function might fail and return a [`BufferError::MapFailed`].
        pub fn slice_mut(&mut self) -> Result<BufferSliceMut<B, T>, BufferError> {
            unsafe {
                B::slice_buffer_mut(&mut self.repr).map(|slice| BufferSliceMut {
                    slice,
                    _a: PhantomData,
                })
            }
        }
    }
    /// Buffer errors.
    ///
    /// Please keep in mind that this `enum` is _non exhaustive_; you will not be able to exhaustively
    /// pattern-match against it.
    #[non_exhaustive]
    pub enum BufferError {
        /// Cannot create buffer.
        CannotCreate,
        /// Overflow when setting a value with a specific index.
        ///
        /// Contains the index and the size of the buffer.
        Overflow {
            /// Tried index.
            index: usize,
            /// Actuall buffer length.
            buffer_len: usize,
        },
        /// Too few values were passed to fill a buffer.
        ///
        /// Contains the number of passed value and the size of the buffer.
        TooFewValues {
            /// Length of the provided data.
            provided_len: usize,
            /// Actual buffer length.
            buffer_len: usize,
        },
        /// Too many values were passed to fill a buffer.
        ///
        /// Contains the number of passed value and the size of the buffer.
        TooManyValues {
            /// Length of the provided data.
            provided_len: usize,
            /// Actual buffer length.
            buffer_len: usize,
        },
        /// Buffer mapping failed.
        MapFailed,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for BufferError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&BufferError::CannotCreate,) => {
                    let mut debug_trait_builder = f.debug_tuple("CannotCreate");
                    debug_trait_builder.finish()
                }
                (&BufferError::Overflow {
                    index: ref __self_0,
                    buffer_len: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("Overflow");
                    let _ = debug_trait_builder.field("index", &&(*__self_0));
                    let _ = debug_trait_builder.field("buffer_len", &&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&BufferError::TooFewValues {
                    provided_len: ref __self_0,
                    buffer_len: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("TooFewValues");
                    let _ = debug_trait_builder.field("provided_len", &&(*__self_0));
                    let _ = debug_trait_builder.field("buffer_len", &&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&BufferError::TooManyValues {
                    provided_len: ref __self_0,
                    buffer_len: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("TooManyValues");
                    let _ = debug_trait_builder.field("provided_len", &&(*__self_0));
                    let _ = debug_trait_builder.field("buffer_len", &&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&BufferError::MapFailed,) => {
                    let mut debug_trait_builder = f.debug_tuple("MapFailed");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for BufferError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for BufferError {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for BufferError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for BufferError {
        #[inline]
        fn eq(&self, other: &BufferError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &BufferError::Overflow {
                                index: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::Overflow {
                                index: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        (
                            &BufferError::TooFewValues {
                                provided_len: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::TooFewValues {
                                provided_len: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        (
                            &BufferError::TooManyValues {
                                provided_len: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::TooManyValues {
                                provided_len: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &BufferError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &BufferError::Overflow {
                                index: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::Overflow {
                                index: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        (
                            &BufferError::TooFewValues {
                                provided_len: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::TooFewValues {
                                provided_len: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        (
                            &BufferError::TooManyValues {
                                provided_len: ref __self_0,
                                buffer_len: ref __self_1,
                            },
                            &BufferError::TooManyValues {
                                provided_len: ref __arg_1_0,
                                buffer_len: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl BufferError {
        /// Cannot create [`Buffer`].
        pub fn cannot_create() -> Self {
            BufferError::CannotCreate
        }
        /// Overflow when setting a value with a specific index.
        pub fn overflow(index: usize, buffer_len: usize) -> Self {
            BufferError::Overflow { index, buffer_len }
        }
        /// Too few values were passed to fill a buffer.
        pub fn too_few_values(provided_len: usize, buffer_len: usize) -> Self {
            BufferError::TooFewValues {
                provided_len,
                buffer_len,
            }
        }
        /// Too many values were passed to fill a buffer.
        pub fn too_many_values(provided_len: usize, buffer_len: usize) -> Self {
            BufferError::TooManyValues {
                provided_len,
                buffer_len,
            }
        }
        /// Buffer mapping failed.
        pub fn map_failed() -> Self {
            BufferError::MapFailed
        }
    }
    impl fmt::Display for BufferError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                BufferError::CannotCreate => f.write_str("cannot create buffer"),
                BufferError::Overflow { index, buffer_len } => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["buffer overflow (index = ", ", size = ", ")"],
                        &match (&index, &buffer_len) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))
                }
                BufferError::TooFewValues {
                    provided_len,
                    buffer_len,
                } => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[
                        "too few values passed to the buffer (nb = ",
                        ", size = ",
                        ")",
                    ],
                    &match (&provided_len, &buffer_len) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                )),
                BufferError::TooManyValues {
                    provided_len,
                    buffer_len,
                } => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[
                        "too many values passed to the buffer (nb = ",
                        ", size = ",
                        ")",
                    ],
                    &match (&provided_len, &buffer_len) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                )),
                BufferError::MapFailed => f.write_str("buffer mapping failed"),
            }
        }
    }
    impl error::Error for BufferError {}
    /// A buffer slice, allowing to get `&[T]`.
    pub struct BufferSlice<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
    {
        slice: B::SliceRepr,
        _a: PhantomData<&'a mut ()>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, B: ::core::fmt::Debug, T: ::core::fmt::Debug> ::core::fmt::Debug for BufferSlice<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
        B::SliceRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                BufferSlice {
                    slice: ref __self_0_0,
                    _a: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("BufferSlice");
                    let _ = debug_trait_builder.field("slice", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_a", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<'a, B, T> BufferSlice<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
    {
        /// Obtain a `&[T]`.
        ///
        /// # Errors
        ///
        /// It is possible that obtaining a slice is not possible. In that case,
        /// [`BufferError::MapFailed`] is returned.
        pub fn as_slice(&self) -> Result<&[T], BufferError> {
            unsafe { B::obtain_slice(&self.slice) }
        }
    }
    /// A buffer mutable slice, allowing to get `&mut [T]`.
    pub struct BufferSliceMut<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
    {
        slice: B::SliceMutRepr,
        _a: PhantomData<&'a mut ()>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, B: ::core::fmt::Debug, T: ::core::fmt::Debug> ::core::fmt::Debug
        for BufferSliceMut<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
        B::SliceMutRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                BufferSliceMut {
                    slice: ref __self_0_0,
                    _a: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("BufferSliceMut");
                    let _ = debug_trait_builder.field("slice", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_a", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<'a, B, T> BufferSliceMut<'a, B, T>
    where
        B: ?Sized + BufferSliceBackend<T>,
        T: Copy,
    {
        /// Obtain a `&mut [T]`.
        pub fn as_slice_mut(&mut self) -> Result<&mut [T], BufferError> {
            unsafe { B::obtain_slice_mut(&mut self.slice) }
        }
    }
}
pub mod context {
    //! Graphics context.
    //!
    //! # Graphics context and backends
    //!
    //! A graphics context is an external type typically implemented by other crates and which provides
    //! support for backends. Its main scope is to unify all possible implementations of backends
    //! behind a single trait: [`GraphicsContext`]. A [`GraphicsContext`] really only requires two items
    //! to be implemented:
    //!
    //! - The type of the backend to use — [`GraphicsContext::Backend`]. That type will often be used
    //!   to access the GPU, cache costly operations, etc.
    //! - A method to get a mutable access to the underlying backend — [`GraphicsContext::backend`].
    //!
    //! Most of the time, if you want to work with _any_ windowing implementation, you will want to
    //! use a type variable such as `C: GraphicsContext`. If you want to work with any context
    //! supporting a specific backend, use `C: GraphicsContext<Backend = YourBackendType`. Etc.
    //!
    //! This crate doesn’t provide you with creating such contexts. Instead, you must do it yourself
    //! or rely on crates doing it for you.
    //!
    //! # Default implementation of helper functions
    //!
    //! By default, graphics contexts automatically get several methods implemented on them. Those
    //! methods are helper functions available to write code in a more elegant and compact way than
    //! passing around mutable references on the context. Often, it will help you not having to
    //! use type ascription, too, since the [`GraphicsContext::Backend`] type is known when calling
    //! those functions.
    //!
    //! Instead of:
    //!
    //! ```ignore
    //! use luminance::context::GraphicsContext as _;
    //! use luminance::buffer::Buffer;
    //!
    //! let buffer: Buffer<SomeBackendType, u8> = Buffer::from_slice(&mut context, slice).unwrap();
    //! ```
    //!
    //! You can simply do:
    //!
    //! ```ignore
    //! use luminance::context::GraphicsContext as _;
    //!
    //! let buffer = context.new_buffer_from_slice(slice).unwrap();
    //! ```
    use crate::backend::buffer::Buffer as BufferBackend;
    use crate::backend::color_slot::ColorSlot;
    use crate::backend::depth_slot::DepthSlot;
    use crate::backend::framebuffer::Framebuffer as FramebufferBackend;
    use crate::backend::shader::Shader;
    use crate::backend::tess::Tess as TessBackend;
    use crate::backend::texture::Texture as TextureBackend;
    use crate::buffer::{Buffer, BufferError};
    use crate::framebuffer::{Framebuffer, FramebufferError};
    use crate::pipeline::PipelineGate;
    use crate::pixel::Pixel;
    use crate::shader::{ProgramBuilder, Stage, StageError, StageType};
    use crate::tess::{Deinterleaved, Interleaved, TessBuilder, TessVertexData};
    use crate::texture::{Dimensionable, Sampler, Texture, TextureError};
    use crate::vertex::Semantics;
    /// Class of graphics context.
    ///
    /// Graphics context must implement this trait to be able to be used throughout the rest of the
    /// crate.
    pub unsafe trait GraphicsContext: Sized {
        /// Internal type used by the backend to cache, optimize and store data. This roughly represents
        /// the GPU data / context a backend implementation needs to work correctly.
        type Backend: ?Sized;
        /// Access the underlying backend.
        fn backend(&mut self) -> &mut Self::Backend;
        /// Create a new pipeline gate
        fn new_pipeline_gate(&mut self) -> PipelineGate<Self::Backend> {
            PipelineGate::new(self)
        }
        /// Create a new buffer.
        ///
        /// See the documentation of [`Buffer::new`] for further details.
        fn new_buffer<T>(&mut self, len: usize) -> Result<Buffer<Self::Backend, T>, BufferError>
        where
            Self::Backend: BufferBackend<T>,
            T: Copy + Default,
        {
            Buffer::new(self, len)
        }
        /// Create a new buffer from a slice.
        ///
        /// See the documentation of [`Buffer::from_vec`] for further details.
        fn new_buffer_from_vec<T, X>(
            &mut self,
            vec: X,
        ) -> Result<Buffer<Self::Backend, T>, BufferError>
        where
            Self::Backend: BufferBackend<T>,
            X: Into<Vec<T>>,
            T: Copy,
        {
            Buffer::from_vec(self, vec)
        }
        /// Create a new buffer by repeating a value.
        ///
        /// See the documentation of [`Buffer::repeat`] for further details.
        fn new_buffer_repeating<T>(
            &mut self,
            len: usize,
            value: T,
        ) -> Result<Buffer<Self::Backend, T>, BufferError>
        where
            Self::Backend: BufferBackend<T>,
            T: Copy,
        {
            Buffer::repeat(self, len, value)
        }
        /// Create a new framebuffer.
        ///
        /// See the documentation of [`Framebuffer::new`] for further details.
        fn new_framebuffer<D, CS, DS>(
            &mut self,
            size: D::Size,
            mipmaps: usize,
            sampler: Sampler,
        ) -> Result<Framebuffer<Self::Backend, D, CS, DS>, FramebufferError>
        where
            Self::Backend: FramebufferBackend<D>,
            D: Dimensionable,
            CS: ColorSlot<Self::Backend, D>,
            DS: DepthSlot<Self::Backend, D>,
        {
            Framebuffer::new(self, size, mipmaps, sampler)
        }
        /// Create a new shader stage.
        ///
        /// See the documentation of [`Stage::new`] for further details.
        fn new_shader_stage<R>(
            &mut self,
            ty: StageType,
            src: R,
        ) -> Result<Stage<Self::Backend>, StageError>
        where
            Self::Backend: Shader,
            R: AsRef<str>,
        {
            Stage::new(self, ty, src)
        }
        /// Create a new shader program.
        ///
        /// See the documentation of [`ProgramBuilder::new`] for further details.
        fn new_shader_program<Sem, Out, Uni>(&mut self) -> ProgramBuilder<Self, Sem, Out, Uni>
        where
            Self::Backend: Shader,
            Sem: Semantics,
        {
            ProgramBuilder::new(self)
        }
        /// Create a [`TessBuilder`].
        ///
        /// See the documentation of [`TessBuilder::new`] for further details.
        fn new_tess(&mut self) -> TessBuilder<Self::Backend, (), (), (), Interleaved>
        where
            Self::Backend: TessBackend<(), (), (), Interleaved>,
        {
            TessBuilder::new(self)
        }
        /// Create a [`TessBuilder`] with deinterleaved memory.
        ///
        /// See the documentation of [`TessBuilder::new`] for further details.
        fn new_deinterleaved_tess<V, W>(
            &mut self,
        ) -> TessBuilder<Self::Backend, V, (), W, Deinterleaved>
        where
            Self::Backend: TessBackend<V, (), W, Deinterleaved>,
            V: TessVertexData<Deinterleaved>,
            W: TessVertexData<Deinterleaved>,
        {
            TessBuilder::new(self)
        }
        /// Create a new texture.
        ///
        /// Feel free to have a look at the documentation of [`Texture::new`] for further details.
        fn new_texture<D, P>(
            &mut self,
            size: D::Size,
            mipmaps: usize,
            sampler: Sampler,
        ) -> Result<Texture<Self::Backend, D, P>, TextureError>
        where
            Self::Backend: TextureBackend<D, P>,
            D: Dimensionable,
            P: Pixel,
        {
            Texture::new(self, size, mipmaps, sampler)
        }
    }
}
pub mod depth_test {
    //! Depth test related features.
    /// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
    /// fragment’s depth that is already stored.
    pub enum DepthComparison {
        /// Depth test never succeeds.
        Never,
        /// Depth test always succeeds.
        Always,
        /// Depth test succeeds if `a == b`.
        Equal,
        /// Depth test succeeds if `a != b`.
        NotEqual,
        /// Depth test succeeds if `a < b`.
        Less,
        /// Depth test succeeds if `a <= b`.
        LessOrEqual,
        /// Depth test succeeds if `a > b`.
        Greater,
        /// Depth test succeeds if `a >= b`.
        GreaterOrEqual,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for DepthComparison {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for DepthComparison {
        #[inline]
        fn clone(&self) -> DepthComparison {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for DepthComparison {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&DepthComparison::Never,) => {
                    let mut debug_trait_builder = f.debug_tuple("Never");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::Always,) => {
                    let mut debug_trait_builder = f.debug_tuple("Always");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::Equal,) => {
                    let mut debug_trait_builder = f.debug_tuple("Equal");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::NotEqual,) => {
                    let mut debug_trait_builder = f.debug_tuple("NotEqual");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::Less,) => {
                    let mut debug_trait_builder = f.debug_tuple("Less");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::LessOrEqual,) => {
                    let mut debug_trait_builder = f.debug_tuple("LessOrEqual");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::Greater,) => {
                    let mut debug_trait_builder = f.debug_tuple("Greater");
                    debug_trait_builder.finish()
                }
                (&DepthComparison::GreaterOrEqual,) => {
                    let mut debug_trait_builder = f.debug_tuple("GreaterOrEqual");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for DepthComparison {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for DepthComparison {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for DepthComparison {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for DepthComparison {
        #[inline]
        fn eq(&self, other: &DepthComparison) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Whether or not depth writes should be performed when rendering.
    pub enum DepthWrite {
        /// Write values to depth buffers.
        On,
        /// Do not write values to depth buffers.
        Off,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for DepthWrite {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for DepthWrite {
        #[inline]
        fn clone(&self) -> DepthWrite {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for DepthWrite {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&DepthWrite::On,) => {
                    let mut debug_trait_builder = f.debug_tuple("On");
                    debug_trait_builder.finish()
                }
                (&DepthWrite::Off,) => {
                    let mut debug_trait_builder = f.debug_tuple("Off");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for DepthWrite {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for DepthWrite {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for DepthWrite {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for DepthWrite {
        #[inline]
        fn eq(&self, other: &DepthWrite) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
}
pub mod face_culling {
    //! Face culling is the operation of removing triangles if they’re facing the screen in a specific
    //! direction with a specific mode.
    /// Face culling setup.
    pub struct FaceCulling {
        /// Face culling order.
        pub order: FaceCullingOrder,
        /// Face culling mode.
        pub mode: FaceCullingMode,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for FaceCulling {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for FaceCulling {
        #[inline]
        fn clone(&self) -> FaceCulling {
            {
                let _: ::core::clone::AssertParamIsClone<FaceCullingOrder>;
                let _: ::core::clone::AssertParamIsClone<FaceCullingMode>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for FaceCulling {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                FaceCulling {
                    order: ref __self_0_0,
                    mode: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("FaceCulling");
                    let _ = debug_trait_builder.field("order", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("mode", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for FaceCulling {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for FaceCulling {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<FaceCullingOrder>;
                let _: ::core::cmp::AssertParamIsEq<FaceCullingMode>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for FaceCulling {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for FaceCulling {
        #[inline]
        fn eq(&self, other: &FaceCulling) -> bool {
            match *other {
                FaceCulling {
                    order: ref __self_1_0,
                    mode: ref __self_1_1,
                } => match *self {
                    FaceCulling {
                        order: ref __self_0_0,
                        mode: ref __self_0_1,
                    } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &FaceCulling) -> bool {
            match *other {
                FaceCulling {
                    order: ref __self_1_0,
                    mode: ref __self_1_1,
                } => match *self {
                    FaceCulling {
                        order: ref __self_0_0,
                        mode: ref __self_0_1,
                    } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                },
            }
        }
    }
    impl FaceCulling {
        /// Create a new [`FaceCulling`].
        pub fn new(order: FaceCullingOrder, mode: FaceCullingMode) -> Self {
            FaceCulling { order, mode }
        }
    }
    /// Default implementation of [`FaceCulling`].
    ///
    /// - Order is [`FaceCullingOrder::CCW`].
    /// - Mode is [`FaceCullingMode::Back`].
    impl Default for FaceCulling {
        fn default() -> Self {
            FaceCulling::new(FaceCullingOrder::CCW, FaceCullingMode::Back)
        }
    }
    /// Face culling order.
    ///
    /// The order determines how a triangle is determined to be discarded. If the triangle’s vertices
    /// wind up in the same direction as the `FaceCullingOrder`, it’s assigned the front side,
    /// otherwise, it’s the back side.
    pub enum FaceCullingOrder {
        /// Clockwise order.
        CW,
        /// Counter-clockwise order.
        CCW,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for FaceCullingOrder {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for FaceCullingOrder {
        #[inline]
        fn clone(&self) -> FaceCullingOrder {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for FaceCullingOrder {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&FaceCullingOrder::CW,) => {
                    let mut debug_trait_builder = f.debug_tuple("CW");
                    debug_trait_builder.finish()
                }
                (&FaceCullingOrder::CCW,) => {
                    let mut debug_trait_builder = f.debug_tuple("CCW");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for FaceCullingOrder {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for FaceCullingOrder {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for FaceCullingOrder {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for FaceCullingOrder {
        #[inline]
        fn eq(&self, other: &FaceCullingOrder) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Side to show and side to cull.
    pub enum FaceCullingMode {
        /// Cull the front side only.
        Front,
        /// Cull the back side only.
        Back,
        /// Always cull any triangle.
        Both,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for FaceCullingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for FaceCullingMode {
        #[inline]
        fn clone(&self) -> FaceCullingMode {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for FaceCullingMode {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&FaceCullingMode::Front,) => {
                    let mut debug_trait_builder = f.debug_tuple("Front");
                    debug_trait_builder.finish()
                }
                (&FaceCullingMode::Back,) => {
                    let mut debug_trait_builder = f.debug_tuple("Back");
                    debug_trait_builder.finish()
                }
                (&FaceCullingMode::Both,) => {
                    let mut debug_trait_builder = f.debug_tuple("Both");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for FaceCullingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for FaceCullingMode {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for FaceCullingMode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for FaceCullingMode {
        #[inline]
        fn eq(&self, other: &FaceCullingMode) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
}
pub mod framebuffer {
    //! Framebuffers and render targets.
    //!
    //! A framebuffer is a GPU object which responsibility is to hold renders — i.e. a rasterized
    //! scene. Currently, a framebuffer is the only support of rendering: you cannot render directly
    //! into a texture. Instead, you have to use a [`Framebuffer`], which automatically handles for
    //! you the texture creation, mapping and handling to receive renders.
    //!
    //! # Framebuffer creation
    //!
    //! Framebuffers are created via the [`Framebuffer::new`] method. Creating framebuffers require a
    //! bit of explanation as it highly depends on _refinement types_. When you create a new
    //! framebuffer, you are required to provide types to drive the creation and subsequent possible
    //! operations you will be able to perform. Framebuffers have three important type variables:
    //!
    //! - `D`, a type representing a dimension and that must implement [`Dimensionable`]. That types
    //!   gives information on what kind of sizes and offsets a framebuffer will operate on/with.
    //! - `CS`, a _color slot_. Color slots are described in the [backend::color_slot] module.
    //! - `DS`, a _depth slot_. Depth slots are described in the [backend::depth_slot] module.
    //!
    //! You are limited in which types you can choose — the list is visible as implementors of traits
    //! in [backend::color_slot] and [backend::depth_slot].
    //!
    //! Once a [`Framebuffer`] is created, you can do basically two main operations on it:
    //!
    //! - Render things to it.
    //! - Retreive color and depth slots to perform further operations.
    //!
    //! # Rendering to a framebuffer
    //!
    //! Rendering is pretty straightforward: you have to use a [`PipelineGate`] to initiate a render by
    //! passing a reference on your [`Framebuffer`]. Once the pipeline is done, the [`Framebuffer`]
    //! contains the result of the render.
    //!
    //! # Manipulating slots
    //!
    //! Slots’ types depend entirely on the types you choose in [`Framebuffer`]. The rule is that any
    //! type that implements [`ColorSlot`] will be associated another type: that other type (in this
    //! case, [`ColorSlot::ColorTextures`] will be the type you can use to manipulate textures. The
    //! same applies to [`DepthSlot`] with [`DepthSlot::DepthTexture`].
    //!
    //! You can access to the color slot via [`Framebuffer::color_slot`]. You can access to the depth
    //! slot via [`Framebuffer::depth_slot`]. Once you get textures from the color slots, you can use
    //! them as regular textures as input of next renders, for instance.
    //!
    //! ## Note on type generation
    //!
    //! Because framebuffers are highly subject to refinement typing, types are transformed at
    //! compile-time by using the type-system to provide you with a good and comfortable experience.
    //! If you use a single pixel format as color slot, for instance, you will get a single texture
    //! (whose pixel format will be the same as the type variable you set). The dimension of the
    //! texture will be set to the same as the framebuffer, too.
    //!
    //! Now if you use a tuple of pixel formats, you will get a tuple of textures, each having the
    //! correct pixel format. That feature allows to generate complex types by using a _pretty simple_
    //! input type. This is what we call _type constructors_ — type families in functional languages.
    //! All this look a bit magical but the type-system ensures it’s total and not as magic as you
    //! might think.
    //!
    //! [backend::color_slot]: crate::backend::color_slot
    //! [backend::depth_slot]: crate::backend::depth_slot
    //! [`PipelineGate`]: crate::pipeline::PipelineGate
    use std::error;
    use std::fmt;
    use crate::backend::color_slot::ColorSlot;
    use crate::backend::depth_slot::DepthSlot;
    use crate::backend::framebuffer::{Framebuffer as FramebufferBackend, FramebufferBackBuffer};
    use crate::context::GraphicsContext;
    use crate::texture::{Dim2, Dimensionable, Sampler, TextureError};
    /// Typed framebuffers.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [backend::framebuffer::Framebuffer].
    /// - `D` is the dimension type. It must implement [`Dimensionable`].
    /// - `CS` is the color slot type. It must implement [`ColorSlot`].
    /// - `DS` is the depth slot type. It must implement [`DepthSlot`].
    ///
    /// [backend::framebuffer::Framebuffer]: crate::backend::framebuffer::Framebuffer
    pub struct Framebuffer<B, D, CS, DS>
    where
        B: ?Sized + FramebufferBackend<D>,
        D: Dimensionable,
        CS: ColorSlot<B, D>,
        DS: DepthSlot<B, D>,
    {
        pub(crate) repr: B::FramebufferRepr,
        color_slot: CS::ColorTextures,
        depth_slot: DS::DepthTexture,
    }
    impl<B, D, CS, DS> Framebuffer<B, D, CS, DS>
    where
        B: ?Sized + FramebufferBackend<D>,
        D: Dimensionable,
        CS: ColorSlot<B, D>,
        DS: DepthSlot<B, D>,
    {
        /// Create a new [`Framebuffer`].
        ///
        /// The `mipmaps` argument allows to pass the number of _extra precision layers_ the texture will
        /// be created with. A precision layer contains the same image as the _base layer_ but in a lower
        /// resolution. Currently, the way the resolution is computed depends on the backend, but it is
        /// safe to assume that it’s logarithmic in base 2 — i.e. at each layer depth, the resolution
        /// is divided by 2 on each axis.
        ///
        /// # Errors
        ///
        /// It is possible that the [`Framebuffer`] cannot be created. The [`FramebufferError`] provides
        /// the reason why.
        ///
        /// # Notes
        ///
        /// You might be interested in the [`GraphicsContext::new_framebuffer`] function instead, which
        /// is the exact same function, but benefits from more type inference (based on `&mut C`).
        pub fn new<C>(
            ctx: &mut C,
            size: D::Size,
            mipmaps: usize,
            sampler: Sampler,
        ) -> Result<Self, FramebufferError>
        where
            C: GraphicsContext<Backend = B>,
        {
            unsafe {
                let mut repr = ctx
                    .backend()
                    .new_framebuffer::<CS, DS>(size, mipmaps, &sampler)?;
                let color_slot =
                    CS::reify_color_textures(ctx, size, mipmaps, &sampler, &mut repr, 0)?;
                let depth_slot = DS::reify_depth_texture(ctx, size, mipmaps, &sampler, &mut repr)?;
                let repr = B::validate_framebuffer(repr)?;
                Ok(Framebuffer {
                    repr,
                    color_slot,
                    depth_slot,
                })
            }
        }
        /// Get the size of the framebuffer.
        pub fn size(&self) -> D::Size {
            unsafe { B::framebuffer_size(&self.repr) }
        }
        /// Access the carried [`ColorSlot`].
        pub fn color_slot(&mut self) -> &mut CS::ColorTextures {
            &mut self.color_slot
        }
        /// Access the carried [`DepthSlot`].
        pub fn depth_slot(&mut self) -> &mut DS::DepthTexture {
            &mut self.depth_slot
        }
        /// Consume this framebuffer and return the carried slots.
        pub fn into_slots(self) -> (CS::ColorTextures, DS::DepthTexture) {
            (self.color_slot, self.depth_slot)
        }
        /// Consume this framebuffer and return the carried [`ColorSlot`].
        pub fn into_color_slot(self) -> CS::ColorTextures {
            self.color_slot
        }
        /// Consume this framebuffer and return the carried [`DepthSlot`].
        pub fn into_depth_slot(self) -> DS::DepthTexture {
            self.depth_slot
        }
    }
    impl<B> Framebuffer<B, Dim2, (), ()>
    where
        B: ?Sized + FramebufferBackend<Dim2> + FramebufferBackBuffer,
    {
        /// Get the _back buffer_ from the input context and the required resolution.
        pub fn back_buffer<C>(
            ctx: &mut C,
            size: <Dim2 as Dimensionable>::Size,
        ) -> Result<Self, FramebufferError>
        where
            C: GraphicsContext<Backend = B>,
        {
            unsafe { ctx.backend().back_buffer(size) }.map(|repr| Framebuffer {
                repr,
                color_slot: (),
                depth_slot: (),
            })
        }
    }
    /// Framebuffer error.
    #[non_exhaustive]
    pub enum FramebufferError {
        /// Cannot create the framebuffer on the GPU.
        CannotCreate,
        /// Texture error.
        ///
        /// This happen while creating / associating the color / depth slots.
        TextureError(TextureError),
        /// Incomplete error.
        ///
        /// This happens when finalizing the construction of the framebuffer.
        Incomplete(IncompleteReason),
        /// Cannot attach something to a framebuffer.
        UnsupportedAttachment,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for FramebufferError {
        #[inline]
        fn clone(&self) -> FramebufferError {
            match (&*self,) {
                (&FramebufferError::CannotCreate,) => FramebufferError::CannotCreate,
                (&FramebufferError::TextureError(ref __self_0),) => {
                    FramebufferError::TextureError(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&FramebufferError::Incomplete(ref __self_0),) => {
                    FramebufferError::Incomplete(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&FramebufferError::UnsupportedAttachment,) => {
                    FramebufferError::UnsupportedAttachment
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for FramebufferError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&FramebufferError::CannotCreate,) => {
                    let mut debug_trait_builder = f.debug_tuple("CannotCreate");
                    debug_trait_builder.finish()
                }
                (&FramebufferError::TextureError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("TextureError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&FramebufferError::Incomplete(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Incomplete");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&FramebufferError::UnsupportedAttachment,) => {
                    let mut debug_trait_builder = f.debug_tuple("UnsupportedAttachment");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for FramebufferError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for FramebufferError {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<TextureError>;
                let _: ::core::cmp::AssertParamIsEq<IncompleteReason>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for FramebufferError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for FramebufferError {
        #[inline]
        fn eq(&self, other: &FramebufferError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &FramebufferError::TextureError(ref __self_0),
                            &FramebufferError::TextureError(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &FramebufferError::Incomplete(ref __self_0),
                            &FramebufferError::Incomplete(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &FramebufferError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &FramebufferError::TextureError(ref __self_0),
                            &FramebufferError::TextureError(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &FramebufferError::Incomplete(ref __self_0),
                            &FramebufferError::Incomplete(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl FramebufferError {
        /// Cannot create the framebuffer on the GPU.
        pub fn cannot_create() -> Self {
            FramebufferError::CannotCreate
        }
        /// Texture error.
        pub fn texture_error(e: TextureError) -> Self {
            FramebufferError::TextureError(e)
        }
        /// Incomplete error.
        pub fn incomplete(e: IncompleteReason) -> Self {
            FramebufferError::Incomplete(e)
        }
        /// Cannot attach something to a framebuffer.
        pub fn unsupported_attachment() -> Self {
            FramebufferError::UnsupportedAttachment
        }
    }
    impl fmt::Display for FramebufferError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                FramebufferError::CannotCreate => {
                    f.write_str("cannot create the framebuffer on the GPU side")
                }
                FramebufferError::TextureError(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["framebuffer texture error: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                FramebufferError::Incomplete(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["incomplete framebuffer: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                FramebufferError::UnsupportedAttachment => {
                    f.write_str("unsupported framebuffer attachment")
                }
            }
        }
    }
    impl std::error::Error for FramebufferError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                FramebufferError::CannotCreate => None,
                FramebufferError::TextureError(e) => Some(e),
                FramebufferError::Incomplete(e) => Some(e),
                FramebufferError::UnsupportedAttachment => None,
            }
        }
    }
    impl From<TextureError> for FramebufferError {
        fn from(e: TextureError) -> Self {
            FramebufferError::TextureError(e)
        }
    }
    impl From<IncompleteReason> for FramebufferError {
        fn from(e: IncompleteReason) -> Self {
            FramebufferError::Incomplete(e)
        }
    }
    /// Reason a framebuffer is incomplete.
    pub enum IncompleteReason {
        /// Incomplete framebuffer.
        Undefined,
        /// Incomplete attachment (color / depth).
        IncompleteAttachment,
        /// An attachment was missing.
        MissingAttachment,
        /// Incomplete draw buffer.
        IncompleteDrawBuffer,
        /// Incomplete read buffer.
        IncompleteReadBuffer,
        /// Unsupported framebuffer.
        Unsupported,
        /// Incomplete multisample configuration.
        IncompleteMultisample,
        /// Incomplete layer targets.
        IncompleteLayerTargets,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for IncompleteReason {
        #[inline]
        fn clone(&self) -> IncompleteReason {
            match (&*self,) {
                (&IncompleteReason::Undefined,) => IncompleteReason::Undefined,
                (&IncompleteReason::IncompleteAttachment,) => {
                    IncompleteReason::IncompleteAttachment
                }
                (&IncompleteReason::MissingAttachment,) => IncompleteReason::MissingAttachment,
                (&IncompleteReason::IncompleteDrawBuffer,) => {
                    IncompleteReason::IncompleteDrawBuffer
                }
                (&IncompleteReason::IncompleteReadBuffer,) => {
                    IncompleteReason::IncompleteReadBuffer
                }
                (&IncompleteReason::Unsupported,) => IncompleteReason::Unsupported,
                (&IncompleteReason::IncompleteMultisample,) => {
                    IncompleteReason::IncompleteMultisample
                }
                (&IncompleteReason::IncompleteLayerTargets,) => {
                    IncompleteReason::IncompleteLayerTargets
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for IncompleteReason {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&IncompleteReason::Undefined,) => {
                    let mut debug_trait_builder = f.debug_tuple("Undefined");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::IncompleteAttachment,) => {
                    let mut debug_trait_builder = f.debug_tuple("IncompleteAttachment");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::MissingAttachment,) => {
                    let mut debug_trait_builder = f.debug_tuple("MissingAttachment");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::IncompleteDrawBuffer,) => {
                    let mut debug_trait_builder = f.debug_tuple("IncompleteDrawBuffer");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::IncompleteReadBuffer,) => {
                    let mut debug_trait_builder = f.debug_tuple("IncompleteReadBuffer");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::Unsupported,) => {
                    let mut debug_trait_builder = f.debug_tuple("Unsupported");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::IncompleteMultisample,) => {
                    let mut debug_trait_builder = f.debug_tuple("IncompleteMultisample");
                    debug_trait_builder.finish()
                }
                (&IncompleteReason::IncompleteLayerTargets,) => {
                    let mut debug_trait_builder = f.debug_tuple("IncompleteLayerTargets");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for IncompleteReason {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for IncompleteReason {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for IncompleteReason {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for IncompleteReason {
        #[inline]
        fn eq(&self, other: &IncompleteReason) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl fmt::Display for IncompleteReason {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                IncompleteReason::Undefined => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["incomplete reason"],
                    &match () {
                        () => [],
                    },
                )),
                IncompleteReason::IncompleteAttachment => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["incomplete attachment"],
                        &match () {
                            () => [],
                        },
                    ))
                }
                IncompleteReason::MissingAttachment => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["missing attachment"],
                    &match () {
                        () => [],
                    },
                )),
                IncompleteReason::IncompleteDrawBuffer => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["incomplete draw buffer"],
                        &match () {
                            () => [],
                        },
                    ))
                }
                IncompleteReason::IncompleteReadBuffer => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["incomplete read buffer"],
                        &match () {
                            () => [],
                        },
                    ))
                }
                IncompleteReason::Unsupported => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["unsupported"],
                    &match () {
                        () => [],
                    },
                )),
                IncompleteReason::IncompleteMultisample => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["incomplete multisample"],
                        &match () {
                            () => [],
                        },
                    ))
                }
                IncompleteReason::IncompleteLayerTargets => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["incomplete layer targets"],
                        &match () {
                            () => [],
                        },
                    ))
                }
            }
        }
    }
    impl error::Error for IncompleteReason {}
}
pub mod pipeline {
    //! Graphics pipelines.
    //!
    //! Graphics pipelines are the means used to describe — and hence perform — renders. They
    //! provide a way to describe how resources should be shared and used to produce a single
    //! pixel frame.
    //!
    //! # Pipelines and AST
    //!
    //! luminance has a very particular way of doing graphics. It represents a typical _graphics
    //! pipeline_ via a typed [AST] that is embedded into your code. As you might already know, when you
    //! write code, you’re actually creating an [AST]: expressions, assignments, bindings, conditions,
    //! function calls, etc. They all represent a typed tree that represents your program.
    //!
    //! luminance uses that property to create a dependency between resources your GPU needs to
    //! have in order to perform a render. It might be weird at first but you’ll see how simple and easy
    //! it is. If you want to perform a simple draw call of a triangle, you need several resources:
    //!
    //! - A [`Tess`] that represents the triangle. It holds three vertices.
    //! - A shader [`Program`], for shading the triangle with a constant color, for short and simple.
    //! - A [`Framebuffer`], to accept and hold the actual render.
    //! - A [`RenderState`], to state how the render should be performed.
    //! - And finally, a [`PipelineState`], which allows even more customization on how the pipeline
    //!   behaves
    //!
    //! There is a dependency _graph_ to represent how the resources must behave regarding each other:
    //!
    //! ```text
    //! (AST1)
    //!
    //! PipelineState ─── Framebuffer ─── Shader ─── RenderState ─── Tess
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
    //! PipelineState ─── Framebuffer ─── Shader ─┬─ RenderState ─┬─ Tess
    //!                                           │               │
    //!                                           │               └─ Tess
    //!                                           │
    //!                                           └─ RenderState ─── Tess
    //! ```
    //!
    //! That [AST] looks more complex. Imagine now that we want to shade one other triangle with
    //! another shader!
    //!
    //! ```text
    //! (AST3)
    //!
    //! PipelineState ─── Framebuffer ─┬─ Shader ─┬─ RenderState ─┬─ Tess
    //!                                │          │               │
    //!                                │          │               └─ Tess
    //!                                │          │
    //!                                │          └─ RenderState ─── Tess
    //!                                │
    //!                                └─ Shader ─── RenderState ─── Tess
    //! ```
    //!
    //! You can now clearly see the [AST]s and the relationships between objects. Those are encoded
    //! in luminance within your code directly: lambdas / closures.
    //!
    //! > If you have followed thoroughly, you might have noticed that you cannot, with such [AST]s,
    //! > shade a triangle with another shader but using the same render state as another node. That
    //! > was a decision that was needed to be made: how should we allow the [AST] to be shared?
    //! > In terms of graphics pipeline, luminance tries to do the best thing to minimize the number
    //! > of GPU context switches and CPU <=> GPU bandwidth congestion.
    //!
    //! # The lambda & closure design
    //!
    //! A function is a perfect candidate to modelize a dependency: the arguments of the function
    //! modelize the dependency — they will be provided _at some point in time_, but it doesn’t matter
    //! when while writing the function. We can then write code _depending_ on something without even
    //! knowing where it’s from.
    //!
    //! Using pseudo-code, here’s what the ASTs from above look like: (this is not a real luminance,
    //! excerpt, just a simplification).
    //!
    //! ```ignore
    //! // AST1
    //! pipeline(framebuffer, pipeline_state, || {
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
    //! See how simple it is to represent `AST1` with just closures? Rust’s lifetimes and existential
    //! quantification allow us to ensure that no resource will leak from the scope of each closures,
    //! hence enforcing memory and coherency safety.
    //!
    //! Now let’s try to tackle `AST2`.
    //!
    //! ```ignore
    //! // AST2
    //! pipeline(framebuffer, pipeline_state, || {
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
    //! pipeline(framebuffer, pipeline_state, || {
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
    //! The luminance equivalent is a bit more complex because it implies some objects that need
    //! to be introduced first.
    //!
    //! # PipelineGate and Pipeline
    //!
    //! A [`PipelineGate`] represents a whole [AST] as seen as just above. It is created by a
    //! [`GraphicsContext`] when you ask to create a pipeline gate. A [`PipelineGate`] is typically
    //! destroyed at the end of the current frame, but that’s not a general rule.
    //!
    //! Such an object gives you access, via the [`PipelineGate::pipeline`], to two other objects
    //! :
    //!
    //! - A [`ShadingGate`], explained below.
    //! - A [`Pipeline`].
    //!
    //! A [`Pipeline`] is a special object you can use to use some specific scarce resources, such as
    //! _textures_ and _buffers_. Those are treated a bit specifically on the GPU, so you have to use
    //! the [`Pipeline`] interface to deal with them.
    //!
    //! Creating a [`PipelineGate`] requires two resources: a [`Framebuffer`] to render to, and a
    //! [`PipelineState`], allowing to customize how the pipeline will perform renders at runtime.
    //!
    //! # ShadingGate
    //!
    //! When you create a pipeline, you’re also handed a [`ShadingGate`]. A [`ShadingGate`] is an object
    //! that allows you to create _shader_ nodes in the [AST] you’re building. You have no other way
    //! to go deeper in the [AST].
    //!
    //! That node will typically borrow a shader [`Program`] and will move you one level lower in the
    //! graph ([AST]). A shader [`Program`] is typically an object you create at initialization or at
    //! specific moment in time (i.e. you don’t create them each frame) that tells the GPU how vertices
    //! should be transformed; how primitives should be moved and generated, how tessellation occurs and
    //! how fragment (i.e. pixels) are computed / shaded — hence the name.
    //!
    //! At that level (i.e. in that closure), you are given two objects:
    //!
    //!   - A [`RenderGate`], discussed below.
    //!   - A [`ProgramInterface`], which has as type parameter the type of uniform your shader
    //!     [`Program`] defines.
    //!
    //! The [`ProgramInterface`] is the only way for you to access your _uniform interface_. More on
    //! this in the dedicated section. It also provides you with the [`ProgramInterface::query`]
    //! method, that allows you to perform _dynamic uniform lookup_.
    //!
    //! # RenderGate
    //!
    //! A [`RenderGate`] is the second to last gate you will be handling. It allows you to create
    //! _render state_ nodes in your [AST], creating a new level for you to render tessellations with
    //! an obvious, final gate: the [`TessGate`].
    //!
    //! The kind of object that node manipulates is [`RenderState`]. A [`RenderState`] — a bit like for
    //! [`PipelineGate`] with [`PipelineState`] — enables to customize how a render of a specific set
    //! of objects (i.e. tessellations) will occur. It’s a bit more specific to renders than pipelines.
    //!
    //! # TessGate
    //!
    //! The [`TessGate`] is the final gate you use in an [AST]. It’s used to create _tessellation
    //! nodes_. Those are used to render actual [`Tess`]. You cannot go any deeper in the [AST] at that
    //! stage.
    //!
    //! [`TessGate`]s don’t immediately use [`Tess`] as inputs. They use [`TessView`]. That type is
    //! a simple GPU view into a GPU tessellation ([`Tess`]). It can be obtained from a [`Tess`] via
    //! the [`View`] trait or built explicitly.
    //!
    //! [AST]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
    //! [`Tess`]: crate::tess::Tess
    //! [`Program`]: crate::shader::Program
    //! [`Framebuffer`]: crate::framebuffer::Framebuffer
    //! [`RenderState`]: crate::render_state::RenderState
    //! [`PipelineState`]: crate::pipeline::PipelineState
    //! [`ShadingGate`]: crate::shading_gate::ShadingGate
    //! [`RenderGate`]: crate::render_gate::RenderGate
    //! [`ProgramInterface`]: crate::shader::ProgramInterface
    //! [`ProgramInterface::query`]: crate::shader::ProgramInterface::query
    //! [`TessGate`]: crate::tess_gate::TessGate
    //! [`TessView`]: crate::tess::TessView
    //! [`View`]: crate::tess::View
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use crate::backend::color_slot::ColorSlot;
    use crate::backend::depth_slot::DepthSlot;
    use crate::backend::framebuffer::Framebuffer as FramebufferBackend;
    use crate::backend::pipeline::{
        Pipeline as PipelineBackend, PipelineBase, PipelineBuffer, PipelineTexture,
    };
    use crate::buffer::Buffer;
    use crate::context::GraphicsContext;
    use crate::framebuffer::Framebuffer;
    use crate::pixel::Pixel;
    use crate::shading_gate::ShadingGate;
    use crate::texture::Dimensionable;
    use crate::texture::Texture;
    /// Possible errors that might occur in a graphics [`Pipeline`].
    #[non_exhaustive]
    pub enum PipelineError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for PipelineError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    impl fmt::Display for PipelineError {
        fn fmt(&self, _: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            Ok(())
        }
    }
    impl error::Error for PipelineError {}
    /// The viewport being part of the [`PipelineState`].
    pub enum Viewport {
        /// The whole viewport is used. The position and dimension of the viewport rectangle are
        /// extracted from the framebuffer.
        Whole,
        /// The viewport is specific and the rectangle area is user-defined.
        Specific {
            /// The lower position on the X axis to start the viewport rectangle at.
            x: u32,
            /// The lower position on the Y axis to start the viewport rectangle at.
            y: u32,
            /// The width of the viewport.
            width: u32,
            /// The height of the viewport.
            height: u32,
        },
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Viewport {
        #[inline]
        fn clone(&self) -> Viewport {
            {
                let _: ::core::clone::AssertParamIsClone<u32>;
                let _: ::core::clone::AssertParamIsClone<u32>;
                let _: ::core::clone::AssertParamIsClone<u32>;
                let _: ::core::clone::AssertParamIsClone<u32>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Viewport {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Viewport {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Viewport::Whole,) => {
                    let mut debug_trait_builder = f.debug_tuple("Whole");
                    debug_trait_builder.finish()
                }
                (&Viewport::Specific {
                    x: ref __self_0,
                    y: ref __self_1,
                    width: ref __self_2,
                    height: ref __self_3,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("Specific");
                    let _ = debug_trait_builder.field("x", &&(*__self_0));
                    let _ = debug_trait_builder.field("y", &&(*__self_1));
                    let _ = debug_trait_builder.field("width", &&(*__self_2));
                    let _ = debug_trait_builder.field("height", &&(*__self_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Viewport {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Viewport {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<u32>;
                let _: ::core::cmp::AssertParamIsEq<u32>;
                let _: ::core::cmp::AssertParamIsEq<u32>;
                let _: ::core::cmp::AssertParamIsEq<u32>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Viewport {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                (&Viewport::Specific {
                    x: ref __self_0,
                    y: ref __self_1,
                    width: ref __self_2,
                    height: ref __self_3,
                },) => {
                    ::core::hash::Hash::hash(
                        &unsafe { ::core::intrinsics::discriminant_value(self) },
                        state,
                    );
                    ::core::hash::Hash::hash(&(*__self_0), state);
                    ::core::hash::Hash::hash(&(*__self_1), state);
                    ::core::hash::Hash::hash(&(*__self_2), state);
                    ::core::hash::Hash::hash(&(*__self_3), state)
                }
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Viewport {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Viewport {
        #[inline]
        fn eq(&self, other: &Viewport) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &Viewport::Specific {
                                x: ref __self_0,
                                y: ref __self_1,
                                width: ref __self_2,
                                height: ref __self_3,
                            },
                            &Viewport::Specific {
                                x: ref __arg_1_0,
                                y: ref __arg_1_1,
                                width: ref __arg_1_2,
                                height: ref __arg_1_3,
                            },
                        ) => {
                            (*__self_0) == (*__arg_1_0)
                                && (*__self_1) == (*__arg_1_1)
                                && (*__self_2) == (*__arg_1_2)
                                && (*__self_3) == (*__arg_1_3)
                        }
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &Viewport) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &Viewport::Specific {
                                x: ref __self_0,
                                y: ref __self_1,
                                width: ref __self_2,
                                height: ref __self_3,
                            },
                            &Viewport::Specific {
                                x: ref __arg_1_0,
                                y: ref __arg_1_1,
                                width: ref __arg_1_2,
                                height: ref __arg_1_3,
                            },
                        ) => {
                            (*__self_0) != (*__arg_1_0)
                                || (*__self_1) != (*__arg_1_1)
                                || (*__self_2) != (*__arg_1_2)
                                || (*__self_3) != (*__arg_1_3)
                        }
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    /// Various customization options for pipelines.
    #[non_exhaustive]
    pub struct PipelineState {
        /// Color to use when clearing buffers.
        pub clear_color: [f32; 4],
        /// Whether clearing color buffers.
        pub clear_color_enabled: bool,
        /// Whether clearing depth buffers.
        pub clear_depth_enabled: bool,
        /// Viewport to use when rendering.
        pub viewport: Viewport,
        /// Whether [sRGB](https://en.wikipedia.org/wiki/SRGB) should be enabled.
        pub srgb_enabled: bool,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for PipelineState {
        #[inline]
        fn clone(&self) -> PipelineState {
            match *self {
                PipelineState {
                    clear_color: ref __self_0_0,
                    clear_color_enabled: ref __self_0_1,
                    clear_depth_enabled: ref __self_0_2,
                    viewport: ref __self_0_3,
                    srgb_enabled: ref __self_0_4,
                } => PipelineState {
                    clear_color: ::core::clone::Clone::clone(&(*__self_0_0)),
                    clear_color_enabled: ::core::clone::Clone::clone(&(*__self_0_1)),
                    clear_depth_enabled: ::core::clone::Clone::clone(&(*__self_0_2)),
                    viewport: ::core::clone::Clone::clone(&(*__self_0_3)),
                    srgb_enabled: ::core::clone::Clone::clone(&(*__self_0_4)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for PipelineState {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                PipelineState {
                    clear_color: ref __self_0_0,
                    clear_color_enabled: ref __self_0_1,
                    clear_depth_enabled: ref __self_0_2,
                    viewport: ref __self_0_3,
                    srgb_enabled: ref __self_0_4,
                } => {
                    let mut debug_trait_builder = f.debug_struct("PipelineState");
                    let _ = debug_trait_builder.field("clear_color", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("clear_color_enabled", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("clear_depth_enabled", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("viewport", &&(*__self_0_3));
                    let _ = debug_trait_builder.field("srgb_enabled", &&(*__self_0_4));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Default for PipelineState {
        /// Default [`PipelineState`]:
        ///
        /// - Clear color: `[0, 0, 0, 1]`.
        /// - Color is always cleared.
        /// - Depth is always cleared.
        /// - The viewport uses the whole framebuffer’s.
        /// - sRGB encoding is disabled.
        fn default() -> Self {
            PipelineState {
                clear_color: [0., 0., 0., 1.],
                clear_color_enabled: true,
                clear_depth_enabled: true,
                viewport: Viewport::Whole,
                srgb_enabled: false,
            }
        }
    }
    impl PipelineState {
        /// Create a default [`PipelineState`].
        ///
        /// See the documentation of the [`Default`] for further details.
        pub fn new() -> Self {
            Self::default()
        }
        /// Get the clear color.
        pub fn clear_color(&self) -> [f32; 4] {
            self.clear_color
        }
        /// Set the clear color.
        pub fn set_clear_color(self, clear_color: [f32; 4]) -> Self {
            Self {
                clear_color,
                ..self
            }
        }
        /// Check whether the pipeline’s framebuffer’s color buffers will be cleared.
        pub fn is_clear_color_enabled(&self) -> bool {
            self.clear_color_enabled
        }
        /// Enable clearing color buffers.
        pub fn enable_clear_color(self, clear_color_enabled: bool) -> Self {
            Self {
                clear_color_enabled,
                ..self
            }
        }
        /// Check whether the pipeline’s framebuffer’s depth buffer will be cleared.
        pub fn is_clear_depth_enabled(&self) -> bool {
            self.clear_depth_enabled
        }
        /// Enable clearing depth buffers.
        pub fn enable_clear_depth(self, clear_depth_enabled: bool) -> Self {
            Self {
                clear_depth_enabled,
                ..self
            }
        }
        /// Get the viewport.
        pub fn viewport(&self) -> Viewport {
            self.viewport
        }
        /// Set the viewport.
        pub fn set_viewport(self, viewport: Viewport) -> Self {
            Self { viewport, ..self }
        }
        /// Check whether sRGB linearization is enabled.
        pub fn is_srgb_enabled(&self) -> bool {
            self.srgb_enabled
        }
        /// Enable sRGB linearization.
        pub fn enable_srgb(self, srgb_enabled: bool) -> Self {
            Self {
                srgb_enabled,
                ..self
            }
        }
    }
    /// A GPU pipeline handle.
    ///
    /// A [`Pipeline`] is a special object that is provided as soon as one enters a [`PipelineGate`].
    /// It is used to dynamically modify the behavior of the running graphics pipeline. That includes,
    /// for instance, obtaining _bound resources_, like buffers and textures, for subsequent uses in
    /// shader stages.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [`PipelineBase`].
    pub struct Pipeline<'a, B>
    where
        B: ?Sized + PipelineBase,
    {
        repr: B::PipelineRepr,
        _phantom: PhantomData<&'a mut ()>,
    }
    impl<'a, B> Pipeline<'a, B>
    where
        B: PipelineBase,
    {
        /// Bind a buffer.
        ///
        /// Once the buffer is bound, the [`BoundBuffer`] object has to be dropped / die in order to
        /// bind the buffer again.
        pub fn bind_buffer<T>(
            &'a self,
            buffer: &'a mut Buffer<B, T>,
        ) -> Result<BoundBuffer<'a, B, T>, PipelineError>
        where
            B: PipelineBuffer<T>,
            T: Copy,
        {
            unsafe {
                B::bind_buffer(&self.repr, &buffer.repr).map(|repr| BoundBuffer {
                    repr,
                    _phantom: PhantomData,
                })
            }
        }
        /// Bind a texture.
        ///
        /// Once the texture is bound, the [`BoundTexture`] object has to be dropped / die in order to
        /// bind the texture again.
        pub fn bind_texture<D, P>(
            &'a self,
            texture: &'a mut Texture<B, D, P>,
        ) -> Result<BoundTexture<'a, B, D, P>, PipelineError>
        where
            B: PipelineTexture<D, P>,
            D: Dimensionable,
            P: Pixel,
        {
            unsafe {
                B::bind_texture(&self.repr, &texture.repr).map(|repr| BoundTexture {
                    repr,
                    _phantom: PhantomData,
                })
            }
        }
    }
    /// Top-most node in a graphics pipeline.
    ///
    /// [`PipelineGate`] nodes represent the “entry-points” of graphics pipelines. They are used
    /// with a [`Framebuffer`] to render to and a [`PipelineState`] to customize the overall behavior
    /// of the pipeline.
    ///
    /// # Parametricity
    ///
    /// - `B`, the backend type.
    pub struct PipelineGate<'a, B>
    where
        B: ?Sized,
    {
        backend: &'a mut B,
    }
    impl<'a, B> PipelineGate<'a, B>
    where
        B: ?Sized,
    {
        /// Create a new [`PipelineGate`].
        pub fn new<C>(ctx: &'a mut C) -> Self
        where
            C: GraphicsContext<Backend = B>,
        {
            PipelineGate {
                backend: ctx.backend(),
            }
        }
        /// Enter a pipeline node.
        ///
        /// This method is the entry-point in a graphics pipeline. It takes a [`Framebuffer`] and a
        /// [`PipelineState`] and a closure that allows to go deeper in the pipeline (i.e. resource
        /// graph). The closure is passed a [`Pipeline`] for you to dynamically alter the pipeline and a
        /// [`ShadingGate`] to enter shading nodes.
        ///
        /// # Errors
        ///
        /// [`PipelineError`] might be thrown for various reasons, depending on the backend you use.
        /// However, this method doesn’t return [`PipelineError`] directly: instead, it returns
        /// `E: From<PipelineError>`. This allows you to inject your own error type in the argument
        /// closure, allowing for a grainer control of errors inside the pipeline.
        pub fn pipeline<E, D, CS, DS, F>(
            &mut self,
            framebuffer: &Framebuffer<B, D, CS, DS>,
            pipeline_state: &PipelineState,
            f: F,
        ) -> Render<E>
        where
            B: FramebufferBackend<D> + PipelineBackend<D>,
            D: Dimensionable,
            CS: ColorSlot<B, D>,
            DS: DepthSlot<B, D>,
            F: for<'b> FnOnce(Pipeline<'b, B>, ShadingGate<'b, B>) -> Result<(), E>,
            E: From<PipelineError>,
        {
            let render = || {
                unsafe {
                    self.backend
                        .start_pipeline(&framebuffer.repr, pipeline_state);
                }
                let pipeline = unsafe {
                    self.backend.new_pipeline().map(|repr| Pipeline {
                        repr,
                        _phantom: PhantomData,
                    })?
                };
                let shading_gate = ShadingGate {
                    backend: self.backend,
                };
                f(pipeline, shading_gate)
            };
            Render(render())
        }
    }
    /// Output of a [`PipelineGate`].
    ///
    /// This type is used as a proxy over `Result<(), E>`, which it defers to. It is needed so that
    /// you can seamlessly call the [`assume`] method
    ///
    /// [`assume`]: crate::pipeline::Render::assume
    pub struct Render<E>(Result<(), E>);
    impl<E> Render<E> {
        /// Turn a [`Render`] into a [`Result`].
        #[inline]
        pub fn into_result(self) -> Result<(), E> {
            self.0
        }
    }
    impl Render<PipelineError> {
        /// Assume the error type is [`PipelineError`].
        ///
        /// Most of the time, users will not provide their own error types for pipelines. Rust doesn’t
        /// have default type parameters for methods, so this function is needed to inform the type
        /// system to default the error type to [`PipelineError`].
        #[inline]
        pub fn assume(self) -> Self {
            self
        }
    }
    impl<E> From<Render<E>> for Result<(), E> {
        fn from(render: Render<E>) -> Self {
            render.0
        }
    }
    impl<E> Deref for Render<E> {
        type Target = Result<(), E>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<E> DerefMut for Render<E> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    /// Opaque buffer binding.
    ///
    /// This type represents a bound [`Buffer`] via [`BoundBuffer`]. It can be used along with a
    /// [`Uniform`] to customize a shader’s behavior.
    ///
    /// # Parametricity
    ///
    /// - `T` is the type of the carried item by the [`Buffer`].
    ///
    /// # Notes
    ///
    /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
    /// an opaque object.
    ///
    /// [`Uniform`]: crate::shader::Uniform
    pub struct BufferBinding<T> {
        binding: u32,
        _phantom: PhantomData<*const T>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for BufferBinding<T> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                BufferBinding {
                    binding: ref __self_0_0,
                    _phantom: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("BufferBinding");
                    let _ = debug_trait_builder.field("binding", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_phantom", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<T> BufferBinding<T> {
        /// Access the underlying binding value.
        ///
        /// # Notes
        ///
        /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
        pub fn binding(self) -> u32 {
            self.binding
        }
    }
    /// A _bound_ [`Buffer`].
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [`PipelineBuffer`].
    /// - `T` is the type of the carried item by the [`Buffer`].
    ///
    /// # Notes
    ///
    /// Once a [`Buffer`] is bound, it can be used and passed around to shaders. In order to do so,
    /// you will need to pass a [`BufferBinding`] to your [`ProgramInterface`]. That value is unique
    /// to each [`BoundBuffer`] and should always be asked — you shouldn’t cache them, for instance.
    ///
    /// Getting a [`BufferBinding`] is a cheap operation and is performed via the
    /// [`BoundBuffer::binding`] method.
    ///
    /// [`ProgramInterface`]: crate::shader::ProgramInterface
    pub struct BoundBuffer<'a, B, T>
    where
        B: PipelineBuffer<T>,
        T: Copy,
    {
        pub(crate) repr: B::BoundBufferRepr,
        _phantom: PhantomData<&'a T>,
    }
    impl<'a, B, T> BoundBuffer<'a, B, T>
    where
        B: PipelineBuffer<T>,
        T: Copy,
    {
        /// Obtain a [`BufferBinding`] object that can be used to refer to this bound buffer in shader
        /// stages.
        ///
        /// # Notes
        ///
        /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
        /// an opaque object.
        pub fn binding(&self) -> BufferBinding<T> {
            let binding = unsafe { B::buffer_binding(&self.repr) };
            BufferBinding {
                binding,
                _phantom: PhantomData,
            }
        }
    }
    /// Opaque texture binding.
    ///
    /// This type represents a bound [`Texture`] via [`BoundTexture`]. It can be used along with a
    /// [`Uniform`] to customize a shader’s behavior.
    ///
    /// # Parametricity
    ///
    /// - `D` is the dimension of the original texture. It must implement [`Dimensionable`] in most
    ///   useful methods.
    /// - `S` is the sampler type. It must implement [`SamplerType`] in most useful methods.
    ///
    /// # Notes
    ///
    /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
    /// an opaque object.
    ///
    /// [`Uniform`]: crate::shader::Uniform
    /// [`SamplerType`]: crate::pixel::SamplerType
    pub struct TextureBinding<D, S> {
        binding: u32,
        _phantom: PhantomData<*const (D, S)>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<D: ::core::fmt::Debug, S: ::core::fmt::Debug> ::core::fmt::Debug for TextureBinding<D, S> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                TextureBinding {
                    binding: ref __self_0_0,
                    _phantom: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("TextureBinding");
                    let _ = debug_trait_builder.field("binding", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_phantom", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<D, S> TextureBinding<D, S> {
        /// Access the underlying binding value.
        ///
        /// # Notes
        ///
        /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
        pub fn binding(self) -> u32 {
            self.binding
        }
    }
    /// A _bound_ [`Texture`].
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [`PipelineTexture`].
    /// - `D` is the dimension. It must implement [`Dimensionable`].
    /// - `P` is the pixel type. It must implement [`Pixel`].
    ///
    /// # Notes
    ///
    /// Once a [`Texture`] is bound, it can be used and passed around to shaders. In order to do so,
    /// you will need to pass a [`TextureBinding`] to your [`ProgramInterface`]. That value is unique
    /// to each [`BoundTexture`] and should always be asked — you shouldn’t cache them, for instance.
    ///
    /// Getting a [`TextureBinding`] is a cheap operation and is performed via the
    /// [`BoundTexture::binding`] method.
    ///
    /// [`ProgramInterface`]: crate::shader::ProgramInterface
    pub struct BoundTexture<'a, B, D, P>
    where
        B: PipelineTexture<D, P>,
        D: Dimensionable,
        P: Pixel,
    {
        pub(crate) repr: B::BoundTextureRepr,
        _phantom: PhantomData<&'a ()>,
    }
    impl<'a, B, D, P> BoundTexture<'a, B, D, P>
    where
        B: PipelineTexture<D, P>,
        D: Dimensionable,
        P: Pixel,
    {
        /// Obtain a [`TextureBinding`] object that can be used to refer to this bound texture in shader
        /// stages.
        ///
        /// # Notes
        ///
        /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
        /// an opaque object.
        pub fn binding(&self) -> TextureBinding<D, P::SamplerType> {
            let binding = unsafe { B::texture_binding(&self.repr) };
            TextureBinding {
                binding,
                _phantom: PhantomData,
            }
        }
    }
}
pub mod pixel {
    //! Pixel formats types and function manipulation.
    //!
    //! The [`Pixel`] trait is used to reify a pixel type at runtime via [`PixelFormat`]. It is made
    //! of several parts:
    //!
    //! - [`Pixel::Encoding`], an associated type, giving the type used to represent a single pixel.
    //! - [`Pixel::RawEncoding`], an associated typed that represents the encoding of underlying
    //!   values in each channel of a single pixel.
    //! - [`Pixel::SamplerType`], the type of sampler that is needed to be used to access this pixel
    //!   format on the GPU / in shaders.
    //! - [`Pixel::pixel_format`], a function returning the [`PixelFormat`], reified version of the
    //!   type at runtime.
    /// Reify a static pixel format at runtime.
    pub unsafe trait Pixel {
        /// Encoding of a single pixel. It should match the [`PixelFormat`] mapping.
        type Encoding: Copy;
        /// Raw encoding of a single pixel; i.e. that is, encoding of underlying values in contiguous
        /// texture memory, without taking into account channels. It should match the [`PixelFormat`]
        /// mapping.
        type RawEncoding: Copy;
        /// The type of sampler required to access this pixel format.
        type SamplerType: SamplerType;
        /// Reify to [`PixelFormat`].
        fn pixel_format() -> PixelFormat;
    }
    /// Constraint on [`Pixel`] for color ones.
    pub unsafe trait ColorPixel: Pixel {}
    /// Constraint on [`Pixel`] for depth ones.
    pub unsafe trait DepthPixel: Pixel {}
    /// Constaint on [`Pixel`] for renderable ones.
    pub unsafe trait RenderablePixel: Pixel {}
    /// Reify a static sample type at runtime.
    ///
    /// That trait is used to allow sampling with different types than the actual encoding of the
    /// texture as long as the [`Type`] remains the same.
    pub unsafe trait SamplerType {
        /// Underlying type of the sampler.
        fn sample_type() -> Type;
    }
    /// A `PixelFormat` gathers a `Type` along with a `Format`.
    pub struct PixelFormat {
        /// Encoding type of the pixel format.
        pub encoding: Type,
        /// Format of the pixel format.
        pub format: Format,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for PixelFormat {
        #[inline]
        fn clone(&self) -> PixelFormat {
            {
                let _: ::core::clone::AssertParamIsClone<Type>;
                let _: ::core::clone::AssertParamIsClone<Format>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for PixelFormat {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for PixelFormat {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                PixelFormat {
                    encoding: ref __self_0_0,
                    format: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("PixelFormat");
                    let _ = debug_trait_builder.field("encoding", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("format", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for PixelFormat {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for PixelFormat {
        #[inline]
        fn eq(&self, other: &PixelFormat) -> bool {
            match *other {
                PixelFormat {
                    encoding: ref __self_1_0,
                    format: ref __self_1_1,
                } => match *self {
                    PixelFormat {
                        encoding: ref __self_0_0,
                        format: ref __self_0_1,
                    } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &PixelFormat) -> bool {
            match *other {
                PixelFormat {
                    encoding: ref __self_1_0,
                    format: ref __self_1_1,
                } => match *self {
                    PixelFormat {
                        encoding: ref __self_0_0,
                        format: ref __self_0_1,
                    } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for PixelFormat {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for PixelFormat {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Type>;
                let _: ::core::cmp::AssertParamIsEq<Format>;
            }
        }
    }
    impl PixelFormat {
        /// Does a [`PixelFormat`] represent a color?
        pub fn is_color_pixel(self) -> bool {
            !match self.format {
                Format::Depth(_) => true,
                _ => false,
            }
        }
        /// Does a [`PixelFormat`] represent depth information?
        pub fn is_depth_pixel(self) -> bool {
            !self.is_color_pixel()
        }
        /// Return the number of canals.
        pub fn canals_len(self) -> usize {
            match self.format {
                Format::R(_) => 1,
                Format::RG(_, _) => 2,
                Format::RGB(_, _, _) => 3,
                Format::RGBA(_, _, _, _) => 4,
                Format::SRGB(_, _, _) => 3,
                Format::SRGBA(_, _, _, _) => 4,
                Format::Depth(_) => 1,
            }
        }
    }
    /// Pixel type.
    ///
    /// - Normalized integer types: [`NormIntegral`] and [`NormUnsigned`] represent integer types
    ///   (signed and unsigned, respectively). However, they are _normalized_ when used in shader
    ///   stages, i.e. fetching from them will yield a floating-point value. That value is
    ///   comprised between `0.0` and `1.0`.
    /// - Integer types: [`Integral`] and [`Unsigned`] allows to store signed and unsigned integers,
    ///   respectively.
    /// - Floating-point types: currently, only [`Floating`] is supported.
    pub enum Type {
        /// Normalized signed integral pixel type.
        NormIntegral,
        /// Normalized unsigned integral pixel type.
        NormUnsigned,
        /// Signed integral pixel type.
        Integral,
        /// Unsigned integral pixel type.
        Unsigned,
        /// Floating-point pixel type.
        Floating,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Type {
        #[inline]
        fn clone(&self) -> Type {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Type {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Type {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Type::NormIntegral,) => {
                    let mut debug_trait_builder = f.debug_tuple("NormIntegral");
                    debug_trait_builder.finish()
                }
                (&Type::NormUnsigned,) => {
                    let mut debug_trait_builder = f.debug_tuple("NormUnsigned");
                    debug_trait_builder.finish()
                }
                (&Type::Integral,) => {
                    let mut debug_trait_builder = f.debug_tuple("Integral");
                    debug_trait_builder.finish()
                }
                (&Type::Unsigned,) => {
                    let mut debug_trait_builder = f.debug_tuple("Unsigned");
                    debug_trait_builder.finish()
                }
                (&Type::Floating,) => {
                    let mut debug_trait_builder = f.debug_tuple("Floating");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Type {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Type {
        #[inline]
        fn eq(&self, other: &Type) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Type {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Type {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    /// Format of a pixel.
    ///
    /// Whichever the constructor you choose, the carried [`Size`]s represent how many bits are used to
    /// represent each channel.
    pub enum Format {
        /// Holds a red-only channel.
        R(Size),
        /// Holds red and green channels.
        RG(Size, Size),
        /// Holds red, green and blue channels.
        RGB(Size, Size, Size),
        /// Holds red, green, blue and alpha channels.
        RGBA(Size, Size, Size, Size),
        /// Holds a red, green and blue channels in sRGB colorspace.
        SRGB(Size, Size, Size),
        /// Holds a red, green and blue channels in sRGB colorspace, plus an alpha channel.
        SRGBA(Size, Size, Size, Size),
        /// Holds a depth channel.
        Depth(Size),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Format {
        #[inline]
        fn clone(&self) -> Format {
            {
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                let _: ::core::clone::AssertParamIsClone<Size>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Format {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Format {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Format::R(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("R");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&Format::RG(ref __self_0, ref __self_1),) => {
                    let mut debug_trait_builder = f.debug_tuple("RG");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&Format::RGB(ref __self_0, ref __self_1, ref __self_2),) => {
                    let mut debug_trait_builder = f.debug_tuple("RGB");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    let _ = debug_trait_builder.field(&&(*__self_2));
                    debug_trait_builder.finish()
                }
                (&Format::RGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),) => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    let _ = debug_trait_builder.field(&&(*__self_2));
                    let _ = debug_trait_builder.field(&&(*__self_3));
                    debug_trait_builder.finish()
                }
                (&Format::SRGB(ref __self_0, ref __self_1, ref __self_2),) => {
                    let mut debug_trait_builder = f.debug_tuple("SRGB");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    let _ = debug_trait_builder.field(&&(*__self_2));
                    debug_trait_builder.finish()
                }
                (&Format::SRGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),) => {
                    let mut debug_trait_builder = f.debug_tuple("SRGBA");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    let _ = debug_trait_builder.field(&&(*__self_2));
                    let _ = debug_trait_builder.field(&&(*__self_3));
                    debug_trait_builder.finish()
                }
                (&Format::Depth(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Depth");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Format {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Format {
        #[inline]
        fn eq(&self, other: &Format) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&Format::R(ref __self_0), &Format::R(ref __arg_1_0)) => {
                            (*__self_0) == (*__arg_1_0)
                        }
                        (
                            &Format::RG(ref __self_0, ref __self_1),
                            &Format::RG(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        (
                            &Format::RGB(ref __self_0, ref __self_1, ref __self_2),
                            &Format::RGB(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
                        ) => {
                            (*__self_0) == (*__arg_1_0)
                                && (*__self_1) == (*__arg_1_1)
                                && (*__self_2) == (*__arg_1_2)
                        }
                        (
                            &Format::RGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),
                            &Format::RGBA(
                                ref __arg_1_0,
                                ref __arg_1_1,
                                ref __arg_1_2,
                                ref __arg_1_3,
                            ),
                        ) => {
                            (*__self_0) == (*__arg_1_0)
                                && (*__self_1) == (*__arg_1_1)
                                && (*__self_2) == (*__arg_1_2)
                                && (*__self_3) == (*__arg_1_3)
                        }
                        (
                            &Format::SRGB(ref __self_0, ref __self_1, ref __self_2),
                            &Format::SRGB(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
                        ) => {
                            (*__self_0) == (*__arg_1_0)
                                && (*__self_1) == (*__arg_1_1)
                                && (*__self_2) == (*__arg_1_2)
                        }
                        (
                            &Format::SRGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),
                            &Format::SRGBA(
                                ref __arg_1_0,
                                ref __arg_1_1,
                                ref __arg_1_2,
                                ref __arg_1_3,
                            ),
                        ) => {
                            (*__self_0) == (*__arg_1_0)
                                && (*__self_1) == (*__arg_1_1)
                                && (*__self_2) == (*__arg_1_2)
                                && (*__self_3) == (*__arg_1_3)
                        }
                        (&Format::Depth(ref __self_0), &Format::Depth(ref __arg_1_0)) => {
                            (*__self_0) == (*__arg_1_0)
                        }
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &Format) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&Format::R(ref __self_0), &Format::R(ref __arg_1_0)) => {
                            (*__self_0) != (*__arg_1_0)
                        }
                        (
                            &Format::RG(ref __self_0, ref __self_1),
                            &Format::RG(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        (
                            &Format::RGB(ref __self_0, ref __self_1, ref __self_2),
                            &Format::RGB(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
                        ) => {
                            (*__self_0) != (*__arg_1_0)
                                || (*__self_1) != (*__arg_1_1)
                                || (*__self_2) != (*__arg_1_2)
                        }
                        (
                            &Format::RGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),
                            &Format::RGBA(
                                ref __arg_1_0,
                                ref __arg_1_1,
                                ref __arg_1_2,
                                ref __arg_1_3,
                            ),
                        ) => {
                            (*__self_0) != (*__arg_1_0)
                                || (*__self_1) != (*__arg_1_1)
                                || (*__self_2) != (*__arg_1_2)
                                || (*__self_3) != (*__arg_1_3)
                        }
                        (
                            &Format::SRGB(ref __self_0, ref __self_1, ref __self_2),
                            &Format::SRGB(ref __arg_1_0, ref __arg_1_1, ref __arg_1_2),
                        ) => {
                            (*__self_0) != (*__arg_1_0)
                                || (*__self_1) != (*__arg_1_1)
                                || (*__self_2) != (*__arg_1_2)
                        }
                        (
                            &Format::SRGBA(ref __self_0, ref __self_1, ref __self_2, ref __self_3),
                            &Format::SRGBA(
                                ref __arg_1_0,
                                ref __arg_1_1,
                                ref __arg_1_2,
                                ref __arg_1_3,
                            ),
                        ) => {
                            (*__self_0) != (*__arg_1_0)
                                || (*__self_1) != (*__arg_1_1)
                                || (*__self_2) != (*__arg_1_2)
                                || (*__self_3) != (*__arg_1_3)
                        }
                        (&Format::Depth(ref __self_0), &Format::Depth(ref __arg_1_0)) => {
                            (*__self_0) != (*__arg_1_0)
                        }
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    true
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Format {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Format {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
                let _: ::core::cmp::AssertParamIsEq<Size>;
            }
        }
    }
    impl Format {
        /// Size (in bytes) of a pixel that a format represents.
        pub fn size(self) -> usize {
            let bits = match self {
                Format::R(r) => r.bits(),
                Format::RG(r, g) => r.bits() + g.bits(),
                Format::RGB(r, g, b) => r.bits() + g.bits() + b.bits(),
                Format::RGBA(r, g, b, a) => r.bits() + g.bits() + b.bits() + a.bits(),
                Format::SRGB(r, g, b) => r.bits() + g.bits() + b.bits(),
                Format::SRGBA(r, g, b, a) => r.bits() + g.bits() + b.bits() + a.bits(),
                Format::Depth(d) => d.bits(),
            };
            bits / 8
        }
    }
    /// Size in bits a pixel channel can be.
    pub enum Size {
        /// 8-bit.
        Eight,
        /// 10-bit.
        Ten,
        /// 11-bit.
        Eleven,
        /// 16-bit.
        Sixteen,
        /// 32-bit.
        ThirtyTwo,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Size {
        #[inline]
        fn clone(&self) -> Size {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Size {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Size {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Size::Eight,) => {
                    let mut debug_trait_builder = f.debug_tuple("Eight");
                    debug_trait_builder.finish()
                }
                (&Size::Ten,) => {
                    let mut debug_trait_builder = f.debug_tuple("Ten");
                    debug_trait_builder.finish()
                }
                (&Size::Eleven,) => {
                    let mut debug_trait_builder = f.debug_tuple("Eleven");
                    debug_trait_builder.finish()
                }
                (&Size::Sixteen,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sixteen");
                    debug_trait_builder.finish()
                }
                (&Size::ThirtyTwo,) => {
                    let mut debug_trait_builder = f.debug_tuple("ThirtyTwo");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Size {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Size {
        #[inline]
        fn eq(&self, other: &Size) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Size {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Size {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl Size {
        /// Size (in bits).
        pub fn bits(self) -> usize {
            match self {
                Size::Eight => 8,
                Size::Ten => 10,
                Size::Eleven => 11,
                Size::Sixteen => 16,
                Size::ThirtyTwo => 32,
            }
        }
    }
    /// The normalized (signed) integral sampler type.
    pub struct NormIntegral;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormIntegral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormIntegral {
        #[inline]
        fn clone(&self) -> NormIntegral {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormIntegral {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormIntegral => {
                    let mut debug_trait_builder = f.debug_tuple("NormIntegral");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for NormIntegral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for NormIntegral {
        #[inline]
        fn eq(&self, other: &NormIntegral) -> bool {
            match *other {
                NormIntegral => match *self {
                    NormIntegral => true,
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for NormIntegral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for NormIntegral {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    unsafe impl SamplerType for NormIntegral {
        fn sample_type() -> Type {
            Type::NormIntegral
        }
    }
    /// The normalized unsigned integral samplre type.
    pub struct NormUnsigned;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormUnsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormUnsigned {
        #[inline]
        fn clone(&self) -> NormUnsigned {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormUnsigned {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormUnsigned => {
                    let mut debug_trait_builder = f.debug_tuple("NormUnsigned");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for NormUnsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for NormUnsigned {
        #[inline]
        fn eq(&self, other: &NormUnsigned) -> bool {
            match *other {
                NormUnsigned => match *self {
                    NormUnsigned => true,
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for NormUnsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for NormUnsigned {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    unsafe impl SamplerType for NormUnsigned {
        fn sample_type() -> Type {
            Type::NormUnsigned
        }
    }
    /// The (signed) integral sampler type.
    pub struct Integral;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Integral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Integral {
        #[inline]
        fn clone(&self) -> Integral {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Integral {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Integral => {
                    let mut debug_trait_builder = f.debug_tuple("Integral");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Integral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Integral {
        #[inline]
        fn eq(&self, other: &Integral) -> bool {
            match *other {
                Integral => match *self {
                    Integral => true,
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for Integral {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Integral {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    unsafe impl SamplerType for Integral {
        fn sample_type() -> Type {
            Type::Integral
        }
    }
    /// The unsigned integral sampler type.
    pub struct Unsigned;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Unsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Unsigned {
        #[inline]
        fn clone(&self) -> Unsigned {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Unsigned {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Unsigned => {
                    let mut debug_trait_builder = f.debug_tuple("Unsigned");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Unsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Unsigned {
        #[inline]
        fn eq(&self, other: &Unsigned) -> bool {
            match *other {
                Unsigned => match *self {
                    Unsigned => true,
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for Unsigned {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Unsigned {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    unsafe impl SamplerType for Unsigned {
        fn sample_type() -> Type {
            Type::Unsigned
        }
    }
    /// The floating sampler type.
    pub struct Floating;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Floating {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Floating {
        #[inline]
        fn clone(&self) -> Floating {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Floating {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Floating => {
                    let mut debug_trait_builder = f.debug_tuple("Floating");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Floating {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Floating {
        #[inline]
        fn eq(&self, other: &Floating) -> bool {
            match *other {
                Floating => match *self {
                    Floating => true,
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for Floating {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Floating {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    unsafe impl SamplerType for Floating {
        fn sample_type() -> Type {
            Type::Floating
        }
    }
    /// A red 8-bit signed integral pixel format.
    pub struct R8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R8I {
        #[inline]
        fn clone(&self) -> R8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R8I => {
                    let mut debug_trait_builder = f.debug_tuple("R8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R8I {
        type Encoding = i8;
        type RawEncoding = i8;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::R(Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for R8I {}
    unsafe impl RenderablePixel for R8I {}
    /// A red 8-bit signed integral pixel format, accessed as normalized floating pixels.
    pub struct NormR8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR8I {
        #[inline]
        fn clone(&self) -> NormR8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR8I => {
                    let mut debug_trait_builder = f.debug_tuple("NormR8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR8I {
        type Encoding = i8;
        type RawEncoding = i8;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::R(Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormR8I {}
    unsafe impl RenderablePixel for NormR8I {}
    /// A red 8-bit unsigned integral pixel format.
    pub struct R8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R8UI {
        #[inline]
        fn clone(&self) -> R8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R8UI => {
                    let mut debug_trait_builder = f.debug_tuple("R8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R8UI {
        type Encoding = u8;
        type RawEncoding = u8;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::R(Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for R8UI {}
    unsafe impl RenderablePixel for R8UI {}
    /// A red 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormR8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR8UI {
        #[inline]
        fn clone(&self) -> NormR8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR8UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormR8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR8UI {
        type Encoding = u8;
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::R(Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormR8UI {}
    unsafe impl RenderablePixel for NormR8UI {}
    /// A red 16-bit signed integral pixel format.
    pub struct R16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R16I {
        #[inline]
        fn clone(&self) -> R16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R16I => {
                    let mut debug_trait_builder = f.debug_tuple("R16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R16I {
        type Encoding = i16;
        type RawEncoding = i16;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::R(Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for R16I {}
    unsafe impl RenderablePixel for R16I {}
    /// A red 16-bit signed integral pixel format, accessed as normalized floating pixels.
    pub struct NormR16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR16I {
        #[inline]
        fn clone(&self) -> NormR16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR16I => {
                    let mut debug_trait_builder = f.debug_tuple("NormR16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR16I {
        type Encoding = i16;
        type RawEncoding = i16;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::R(Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormR16I {}
    unsafe impl RenderablePixel for NormR16I {}
    /// A red 16-bit unsigned integral pixel format.
    pub struct R16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R16UI {
        #[inline]
        fn clone(&self) -> R16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R16UI => {
                    let mut debug_trait_builder = f.debug_tuple("R16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R16UI {
        type Encoding = u16;
        type RawEncoding = u16;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::R(Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for R16UI {}
    unsafe impl RenderablePixel for R16UI {}
    /// A red 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormR16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR16UI {
        #[inline]
        fn clone(&self) -> NormR16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR16UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormR16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR16UI {
        type Encoding = u16;
        type RawEncoding = u16;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::R(Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormR16UI {}
    unsafe impl RenderablePixel for NormR16UI {}
    /// A red 32-bit signed integral pixel format.
    pub struct R32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R32I {
        #[inline]
        fn clone(&self) -> R32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R32I => {
                    let mut debug_trait_builder = f.debug_tuple("R32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R32I {
        type Encoding = i32;
        type RawEncoding = i32;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::R(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for R32I {}
    unsafe impl RenderablePixel for R32I {}
    /// A red 32-bit signed integral pixel format, accessed as normalized floating pixels.
    pub struct NormR32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR32I {
        #[inline]
        fn clone(&self) -> NormR32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR32I => {
                    let mut debug_trait_builder = f.debug_tuple("NormR32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR32I {
        type Encoding = i32;
        type RawEncoding = i32;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::R(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormR32I {}
    unsafe impl RenderablePixel for NormR32I {}
    /// A red 32-bit unsigned integral pixel format.
    pub struct R32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R32UI {
        #[inline]
        fn clone(&self) -> R32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R32UI => {
                    let mut debug_trait_builder = f.debug_tuple("R32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R32UI {
        type Encoding = u32;
        type RawEncoding = u32;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::R(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for R32UI {}
    unsafe impl RenderablePixel for R32UI {}
    /// A red 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormR32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormR32UI {
        #[inline]
        fn clone(&self) -> NormR32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormR32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormR32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormR32UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormR32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormR32UI {
        type Encoding = u32;
        type RawEncoding = u32;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::R(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormR32UI {}
    unsafe impl RenderablePixel for NormR32UI {}
    /// A red 32-bit floating pixel format.
    pub struct R32F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R32F {
        #[inline]
        fn clone(&self) -> R32F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R32F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R32F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R32F => {
                    let mut debug_trait_builder = f.debug_tuple("R32F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R32F {
        type Encoding = f32;
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::R(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for R32F {}
    unsafe impl RenderablePixel for R32F {}
    /// A red and green 8-bit signed integral pixel format.
    pub struct RG8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG8I {
        #[inline]
        fn clone(&self) -> RG8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG8I => {
                    let mut debug_trait_builder = f.debug_tuple("RG8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG8I {
        type Encoding = (i8, i8);
        type RawEncoding = i8;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RG(Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RG8I {}
    unsafe impl RenderablePixel for RG8I {}
    /// A red and green 8-bit integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG8I {
        #[inline]
        fn clone(&self) -> NormRG8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG8I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG8I {
        type Encoding = (i8, i8);
        type RawEncoding = i8;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RG(Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRG8I {}
    unsafe impl RenderablePixel for NormRG8I {}
    /// A red and green 8-bit unsigned integral pixel format.
    pub struct RG8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG8UI {
        #[inline]
        fn clone(&self) -> RG8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG8UI => {
                    let mut debug_trait_builder = f.debug_tuple("RG8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG8UI {
        type Encoding = (u8, u8);
        type RawEncoding = u8;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RG(Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RG8UI {}
    unsafe impl RenderablePixel for RG8UI {}
    /// A red and green 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG8UI {
        #[inline]
        fn clone(&self) -> NormRG8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG8UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG8UI {
        type Encoding = (u8, u8);
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RG(Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRG8UI {}
    unsafe impl RenderablePixel for NormRG8UI {}
    /// A red and green 16-bit signed integral pixel format.
    pub struct RG16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG16I {
        #[inline]
        fn clone(&self) -> RG16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG16I => {
                    let mut debug_trait_builder = f.debug_tuple("RG16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG16I {
        type Encoding = (i16, i16);
        type RawEncoding = i16;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RG(Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RG16I {}
    unsafe impl RenderablePixel for RG16I {}
    /// A red and green 16-bit integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG16I {
        #[inline]
        fn clone(&self) -> NormRG16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG16I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG16I {
        type Encoding = (i16, i16);
        type RawEncoding = i16;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RG(Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRG16I {}
    unsafe impl RenderablePixel for NormRG16I {}
    /// A red and green 16-bit unsigned integral pixel format.
    pub struct RG16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG16UI {
        #[inline]
        fn clone(&self) -> RG16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG16UI => {
                    let mut debug_trait_builder = f.debug_tuple("RG16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG16UI {
        type Encoding = (u16, u16);
        type RawEncoding = u16;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RG(Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RG16UI {}
    unsafe impl RenderablePixel for RG16UI {}
    /// A red and green 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG16UI {
        #[inline]
        fn clone(&self) -> NormRG16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG16UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG16UI {
        type Encoding = (u16, u16);
        type RawEncoding = u16;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RG(Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRG16UI {}
    unsafe impl RenderablePixel for NormRG16UI {}
    /// A red and green 32-bit signed integral pixel format.
    pub struct RG32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG32I {
        #[inline]
        fn clone(&self) -> RG32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG32I => {
                    let mut debug_trait_builder = f.debug_tuple("RG32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG32I {
        type Encoding = (i32, i32);
        type RawEncoding = i32;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RG(Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RG32I {}
    unsafe impl RenderablePixel for RG32I {}
    /// A red and green 32-bit signed integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG32I {
        #[inline]
        fn clone(&self) -> NormRG32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG32I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG32I {
        type Encoding = (i32, i32);
        type RawEncoding = i32;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RG(Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormRG32I {}
    unsafe impl RenderablePixel for NormRG32I {}
    /// A red and green 32-bit unsigned integral pixel format.
    pub struct RG32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG32UI {
        #[inline]
        fn clone(&self) -> RG32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG32UI => {
                    let mut debug_trait_builder = f.debug_tuple("RG32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG32UI {
        type Encoding = (u32, u32);
        type RawEncoding = u32;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RG(Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RG32UI {}
    unsafe impl RenderablePixel for RG32UI {}
    /// A red and green 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
    pub struct NormRG32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRG32UI {
        #[inline]
        fn clone(&self) -> NormRG32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRG32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRG32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRG32UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRG32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRG32UI {
        type Encoding = (u32, u32);
        type RawEncoding = u32;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RG(Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormRG32UI {}
    unsafe impl RenderablePixel for NormRG32UI {}
    /// A red and green 32-bit floating pixel format.
    pub struct RG32F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RG32F {
        #[inline]
        fn clone(&self) -> RG32F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RG32F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RG32F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RG32F => {
                    let mut debug_trait_builder = f.debug_tuple("RG32F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RG32F {
        type Encoding = (f32, f32);
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::RG(Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RG32F {}
    unsafe impl RenderablePixel for RG32F {}
    /// A red, green and blue 8-bit signed integral pixel format.
    pub struct RGB8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB8I {
        #[inline]
        fn clone(&self) -> RGB8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB8I => {
                    let mut debug_trait_builder = f.debug_tuple("RGB8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB8I {
        type Encoding = (i8, i8, i8);
        type RawEncoding = i8;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGB(Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RGB8I {}
    unsafe impl RenderablePixel for RGB8I {}
    /// A red, green and blue 8-bit signed integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB8I {
        #[inline]
        fn clone(&self) -> NormRGB8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB8I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB8I {
        type Encoding = (i8, i8, i8);
        type RawEncoding = i8;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGB(Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB8I {}
    unsafe impl RenderablePixel for NormRGB8I {}
    /// A red, green and blue 8-bit unsigned integral pixel format.
    pub struct RGB8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB8UI {
        #[inline]
        fn clone(&self) -> RGB8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB8UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGB8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB8UI {
        type Encoding = (u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGB(Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RGB8UI {}
    unsafe impl RenderablePixel for RGB8UI {}
    /// A red, green and blue 8-bit unsigned integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB8UI {
        #[inline]
        fn clone(&self) -> NormRGB8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB8UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB8UI {
        type Encoding = (u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGB(Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB8UI {}
    unsafe impl RenderablePixel for NormRGB8UI {}
    /// A red, green and blue 16-bit signed integral pixel format.
    pub struct RGB16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB16I {
        #[inline]
        fn clone(&self) -> RGB16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB16I => {
                    let mut debug_trait_builder = f.debug_tuple("RGB16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB16I {
        type Encoding = (i16, i16, i16);
        type RawEncoding = i16;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RGB16I {}
    unsafe impl RenderablePixel for RGB16I {}
    /// A red, green and blue 16-bit signed integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB16I {
        #[inline]
        fn clone(&self) -> NormRGB16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB16I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB16I {
        type Encoding = (i16, i16, i16);
        type RawEncoding = i16;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB16I {}
    unsafe impl RenderablePixel for NormRGB16I {}
    /// A red, green and blue 16-bit unsigned integral pixel format.
    pub struct RGB16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB16UI {
        #[inline]
        fn clone(&self) -> RGB16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB16UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGB16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB16UI {
        type Encoding = (u16, u16, u16);
        type RawEncoding = u16;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RGB16UI {}
    unsafe impl RenderablePixel for RGB16UI {}
    /// A red, green and blue 16-bit unsigned integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB16UI {
        #[inline]
        fn clone(&self) -> NormRGB16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB16UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB16UI {
        type Encoding = (u16, u16, u16);
        type RawEncoding = u16;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB16UI {}
    unsafe impl RenderablePixel for NormRGB16UI {}
    /// A red, green and blue 32-bit signed integral pixel format.
    pub struct RGB32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB32I {
        #[inline]
        fn clone(&self) -> RGB32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB32I => {
                    let mut debug_trait_builder = f.debug_tuple("RGB32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB32I {
        type Encoding = (i32, i32, i32);
        type RawEncoding = i32;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RGB32I {}
    unsafe impl RenderablePixel for RGB32I {}
    /// A red, green and blue 32-bit signed integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB32I {
        #[inline]
        fn clone(&self) -> NormRGB32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB32I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB32I {
        type Encoding = (i32, i32, i32);
        type RawEncoding = i32;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB32I {}
    unsafe impl RenderablePixel for NormRGB32I {}
    /// A red, green and blue 32-bit unsigned integral pixel format.
    pub struct RGB32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB32UI {
        #[inline]
        fn clone(&self) -> RGB32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB32UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGB32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB32UI {
        type Encoding = (u32, u32, u32);
        type RawEncoding = u32;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RGB32UI {}
    unsafe impl RenderablePixel for RGB32UI {}
    /// A red, green and blue 32-bit unsigned integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGB32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGB32UI {
        #[inline]
        fn clone(&self) -> NormRGB32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGB32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGB32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGB32UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGB32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGB32UI {
        type Encoding = (u32, u32, u32);
        type RawEncoding = u32;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for NormRGB32UI {}
    unsafe impl RenderablePixel for NormRGB32UI {}
    /// A red, green and blue 32-bit floating pixel format.
    pub struct RGB32F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGB32F {
        #[inline]
        fn clone(&self) -> RGB32F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGB32F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGB32F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGB32F => {
                    let mut debug_trait_builder = f.debug_tuple("RGB32F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGB32F {
        type Encoding = (f32, f32, f32);
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo),
            }
        }
    }
    unsafe impl ColorPixel for RGB32F {}
    unsafe impl RenderablePixel for RGB32F {}
    /// A red, green, blue and alpha 8-bit signed integral pixel format.
    pub struct RGBA8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA8I {
        #[inline]
        fn clone(&self) -> RGBA8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA8I => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA8I {
        type Encoding = (i8, i8, i8, i8);
        type RawEncoding = i8;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RGBA8I {}
    unsafe impl RenderablePixel for RGBA8I {}
    /// A red, green, blue and alpha 8-bit signed integral pixel format, accessed as normalized floating
    /// pixels.
    pub struct NormRGBA8I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA8I {
        #[inline]
        fn clone(&self) -> NormRGBA8I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA8I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA8I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA8I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA8I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA8I {
        type Encoding = (i8, i8, i8, i8);
        type RawEncoding = i8;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA8I {}
    unsafe impl RenderablePixel for NormRGBA8I {}
    /// A red, green, blue and alpha 8-bit unsigned integral pixel format.
    pub struct RGBA8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA8UI {
        #[inline]
        fn clone(&self) -> RGBA8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA8UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA8UI {
        type Encoding = (u8, u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for RGBA8UI {}
    unsafe impl RenderablePixel for RGBA8UI {}
    /// A red, green, blue and alpha 8-bit unsigned integral pixel format, accessed as normalized
    /// floating pixels.
    pub struct NormRGBA8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA8UI {
        #[inline]
        fn clone(&self) -> NormRGBA8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA8UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA8UI {
        type Encoding = (u8, u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA8UI {}
    unsafe impl RenderablePixel for NormRGBA8UI {}
    /// A red, green, blue and alpha 16-bit signed integral pixel format.
    pub struct RGBA16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA16I {
        #[inline]
        fn clone(&self) -> RGBA16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA16I => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA16I {
        type Encoding = (i16, i16, i16, i16);
        type RawEncoding = i16;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RGBA16I {}
    unsafe impl RenderablePixel for RGBA16I {}
    /// A red, green, blue and alpha 16-bit signed integral pixel format, accessed as normalized
    /// floating pixels.
    pub struct NormRGBA16I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA16I {
        #[inline]
        fn clone(&self) -> NormRGBA16I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA16I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA16I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA16I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA16I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA16I {
        type Encoding = (i16, i16, i16, i16);
        type RawEncoding = i16;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA16I {}
    unsafe impl RenderablePixel for NormRGBA16I {}
    /// A red, green, blue and alpha 16-bit unsigned integral pixel format.
    pub struct RGBA16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA16UI {
        #[inline]
        fn clone(&self) -> RGBA16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA16UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA16UI {
        type Encoding = (u16, u16, u16, u16);
        type RawEncoding = u16;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for RGBA16UI {}
    unsafe impl RenderablePixel for RGBA16UI {}
    /// A red, green, blue and alpha 16-bit unsigned integral pixel format, accessed as normalized
    /// floating pixels.
    pub struct NormRGBA16UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA16UI {
        #[inline]
        fn clone(&self) -> NormRGBA16UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA16UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA16UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA16UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA16UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA16UI {
        type Encoding = (u16, u16, u16, u16);
        type RawEncoding = u16;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA16UI {}
    unsafe impl RenderablePixel for NormRGBA16UI {}
    /// A red, green, blue and alpha 32-bit signed integral pixel format.
    pub struct RGBA32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA32I {
        #[inline]
        fn clone(&self) -> RGBA32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA32I => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA32I {
        type Encoding = (i32, i32, i32, i32);
        type RawEncoding = i32;
        type SamplerType = Integral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Integral,
                format: Format::RGBA(
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                ),
            }
        }
    }
    unsafe impl ColorPixel for RGBA32I {}
    unsafe impl RenderablePixel for RGBA32I {}
    /// A red, green, blue and alpha 32-bit signed integral pixel format, accessed as normalized
    /// floating pixels.
    pub struct NormRGBA32I;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA32I {
        #[inline]
        fn clone(&self) -> NormRGBA32I {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA32I {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA32I {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA32I => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA32I");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA32I {
        type Encoding = (i32, i32, i32, i32);
        type RawEncoding = i32;
        type SamplerType = NormIntegral;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormIntegral,
                format: Format::RGBA(
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                ),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA32I {}
    unsafe impl RenderablePixel for NormRGBA32I {}
    /// A red, green, blue and alpha 32-bit unsigned integral pixel format.
    pub struct RGBA32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA32UI {
        #[inline]
        fn clone(&self) -> RGBA32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA32UI => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA32UI {
        type Encoding = (u32, u32, u32, u32);
        type RawEncoding = u32;
        type SamplerType = Unsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Unsigned,
                format: Format::RGBA(
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                ),
            }
        }
    }
    unsafe impl ColorPixel for RGBA32UI {}
    unsafe impl RenderablePixel for RGBA32UI {}
    /// A red, green, blue and alpha 32-bit unsigned integral pixel format, accessed as normalized
    /// floating pixels.
    pub struct NormRGBA32UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NormRGBA32UI {
        #[inline]
        fn clone(&self) -> NormRGBA32UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NormRGBA32UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NormRGBA32UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NormRGBA32UI => {
                    let mut debug_trait_builder = f.debug_tuple("NormRGBA32UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for NormRGBA32UI {
        type Encoding = (u32, u32, u32, u32);
        type RawEncoding = u32;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::RGBA(
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                ),
            }
        }
    }
    unsafe impl ColorPixel for NormRGBA32UI {}
    unsafe impl RenderablePixel for NormRGBA32UI {}
    /// A red, green, blue and alpha 32-bit floating pixel format.
    pub struct RGBA32F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RGBA32F {
        #[inline]
        fn clone(&self) -> RGBA32F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for RGBA32F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RGBA32F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RGBA32F => {
                    let mut debug_trait_builder = f.debug_tuple("RGBA32F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for RGBA32F {
        type Encoding = (f32, f32, f32, f32);
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::RGBA(
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                    Size::ThirtyTwo,
                ),
            }
        }
    }
    unsafe impl ColorPixel for RGBA32F {}
    unsafe impl RenderablePixel for RGBA32F {}
    /// A red, green and blue pixel format in which:
    ///
    ///   - The red channel is on 11 bits.
    ///   - The green channel is on 11 bits, too.
    ///   - The blue channel is on 10 bits.
    pub struct R11G11B10F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for R11G11B10F {
        #[inline]
        fn clone(&self) -> R11G11B10F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for R11G11B10F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for R11G11B10F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                R11G11B10F => {
                    let mut debug_trait_builder = f.debug_tuple("R11G11B10F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for R11G11B10F {
        type Encoding = (f32, f32, f32, f32);
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::RGB(Size::Eleven, Size::Eleven, Size::Ten),
            }
        }
    }
    unsafe impl ColorPixel for R11G11B10F {}
    unsafe impl RenderablePixel for R11G11B10F {}
    /// An 8-bit unsigned integral red, green and blue pixel format in sRGB colorspace.
    pub struct SRGB8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for SRGB8UI {
        #[inline]
        fn clone(&self) -> SRGB8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for SRGB8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for SRGB8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                SRGB8UI => {
                    let mut debug_trait_builder = f.debug_tuple("SRGB8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for SRGB8UI {
        type Encoding = (u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::SRGB(Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for SRGB8UI {}
    unsafe impl RenderablePixel for SRGB8UI {}
    /// An 8-bit unsigned integral red, green and blue pixel format in sRGB colorspace, with linear alpha channel.
    pub struct SRGBA8UI;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for SRGBA8UI {
        #[inline]
        fn clone(&self) -> SRGBA8UI {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for SRGBA8UI {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for SRGBA8UI {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                SRGBA8UI => {
                    let mut debug_trait_builder = f.debug_tuple("SRGBA8UI");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for SRGBA8UI {
        type Encoding = (u8, u8, u8, u8);
        type RawEncoding = u8;
        type SamplerType = NormUnsigned;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::NormUnsigned,
                format: Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight),
            }
        }
    }
    unsafe impl ColorPixel for SRGBA8UI {}
    unsafe impl RenderablePixel for SRGBA8UI {}
    /// A depth 32-bit floating pixel format.
    pub struct Depth32F;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Depth32F {
        #[inline]
        fn clone(&self) -> Depth32F {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Depth32F {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Depth32F {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Depth32F => {
                    let mut debug_trait_builder = f.debug_tuple("Depth32F");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Pixel for Depth32F {
        type Encoding = f32;
        type RawEncoding = f32;
        type SamplerType = Floating;
        fn pixel_format() -> PixelFormat {
            PixelFormat {
                encoding: Type::Floating,
                format: Format::Depth(Size::ThirtyTwo),
            }
        }
    }
    unsafe impl DepthPixel for Depth32F {}
}
pub mod render_gate {
    //! Render gates.
    //!
    //! A render gate is a _pipeline node_ that allows to share [`RenderState`] for deeper nodes,
    //! which are used to render [`Tess`].
    //!
    //! [`Tess`]: crate::tess::Tess
    use crate::backend::render_gate::RenderGate as RenderGateBackend;
    use crate::render_state::RenderState;
    use crate::tess_gate::TessGate;
    /// A render gate.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    pub struct RenderGate<'a, B>
    where
        B: ?Sized,
    {
        pub(crate) backend: &'a mut B,
    }
    impl<'a, B> RenderGate<'a, B>
    where
        B: ?Sized + RenderGateBackend,
    {
        /// Enter a [`RenderGate`] and go deeper in the pipeline.
        pub fn render<'b, E, F>(&'b mut self, rdr_st: &RenderState, f: F) -> Result<(), E>
        where
            F: FnOnce(TessGate<'b, B>) -> Result<(), E>,
        {
            unsafe {
                self.backend.enter_render_state(rdr_st);
            }
            let tess_gate = TessGate {
                backend: self.backend,
            };
            f(tess_gate)
        }
    }
}
pub mod render_state {
    //! GPU render state.
    //!
    //! Such a state controls how the GPU must operate some fixed pipeline functionality, such as the
    //! blending, depth test or face culling operations.
    use crate::blending::{Blending, BlendingMode};
    use crate::depth_test::{DepthComparison, DepthWrite};
    use crate::face_culling::FaceCulling;
    /// GPU render state.
    ///
    /// You can get a default value with `RenderState::default` and set the operations you want with the
    /// various `RenderState::set_*` methods.
    pub struct RenderState {
        /// Blending configuration.
        blending: Option<BlendingMode>,
        /// Depth test configuration.
        depth_test: Option<DepthComparison>,
        /// Depth write configuration.
        depth_write: DepthWrite,
        /// Face culling configuration.
        face_culling: Option<FaceCulling>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for RenderState {
        #[inline]
        fn clone(&self) -> RenderState {
            match *self {
                RenderState {
                    blending: ref __self_0_0,
                    depth_test: ref __self_0_1,
                    depth_write: ref __self_0_2,
                    face_culling: ref __self_0_3,
                } => RenderState {
                    blending: ::core::clone::Clone::clone(&(*__self_0_0)),
                    depth_test: ::core::clone::Clone::clone(&(*__self_0_1)),
                    depth_write: ::core::clone::Clone::clone(&(*__self_0_2)),
                    face_culling: ::core::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for RenderState {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                RenderState {
                    blending: ref __self_0_0,
                    depth_test: ref __self_0_1,
                    depth_write: ref __self_0_2,
                    face_culling: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("RenderState");
                    let _ = debug_trait_builder.field("blending", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("depth_test", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("depth_write", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("face_culling", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for RenderState {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for RenderState {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Option<BlendingMode>>;
                let _: ::core::cmp::AssertParamIsEq<Option<DepthComparison>>;
                let _: ::core::cmp::AssertParamIsEq<DepthWrite>;
                let _: ::core::cmp::AssertParamIsEq<Option<FaceCulling>>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for RenderState {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for RenderState {
        #[inline]
        fn eq(&self, other: &RenderState) -> bool {
            match *other {
                RenderState {
                    blending: ref __self_1_0,
                    depth_test: ref __self_1_1,
                    depth_write: ref __self_1_2,
                    face_culling: ref __self_1_3,
                } => match *self {
                    RenderState {
                        blending: ref __self_0_0,
                        depth_test: ref __self_0_1,
                        depth_write: ref __self_0_2,
                        face_culling: ref __self_0_3,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                            && (*__self_0_3) == (*__self_1_3)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &RenderState) -> bool {
            match *other {
                RenderState {
                    blending: ref __self_1_0,
                    depth_test: ref __self_1_1,
                    depth_write: ref __self_1_2,
                    face_culling: ref __self_1_3,
                } => match *self {
                    RenderState {
                        blending: ref __self_0_0,
                        depth_test: ref __self_0_1,
                        depth_write: ref __self_0_2,
                        face_culling: ref __self_0_3,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                            || (*__self_0_3) != (*__self_1_3)
                    }
                },
            }
        }
    }
    impl RenderState {
        /// Override the blending configuration.
        pub fn set_blending<B>(self, blending: B) -> Self
        where
            B: Into<Option<Blending>>,
        {
            RenderState {
                blending: blending.into().map(|x| x.into()),
                ..self
            }
        }
        /// Override the blending configuration using separate blending.
        pub fn set_blending_separate(
            self,
            blending_rgb: Blending,
            blending_alpha: Blending,
        ) -> Self {
            RenderState {
                blending: Some(BlendingMode::Separate {
                    rgb: blending_rgb,
                    alpha: blending_alpha,
                }),
                ..self
            }
        }
        /// Blending configuration.
        pub fn blending(&self) -> Option<BlendingMode> {
            self.blending
        }
        /// Override the depth test configuration.
        pub fn set_depth_test<D>(self, depth_test: D) -> Self
        where
            D: Into<Option<DepthComparison>>,
        {
            let depth_test = depth_test.into();
            RenderState { depth_test, ..self }
        }
        /// Depth test configuration.
        pub fn depth_test(&self) -> Option<DepthComparison> {
            self.depth_test
        }
        /// Override the depth write configuration.
        pub fn set_depth_write(self, depth_write: DepthWrite) -> Self {
            RenderState {
                depth_write,
                ..self
            }
        }
        /// Depth write configuration.
        pub fn depth_write(&self) -> DepthWrite {
            self.depth_write
        }
        /// Override the face culling configuration.
        pub fn set_face_culling<FC>(self, face_culling: FC) -> Self
        where
            FC: Into<Option<FaceCulling>>,
        {
            RenderState {
                face_culling: face_culling.into(),
                ..self
            }
        }
        /// Face culling configuration.
        pub fn face_culling(&self) -> Option<FaceCulling> {
            self.face_culling
        }
    }
    impl Default for RenderState {
        /// The default `RenderState`.
        ///
        ///   - `blending`: `None`
        ///   - `depth_test`: `Some(DepthComparison::Less)`
        ///   - `depth_write`: `DepthWrite::On`
        ///   - `face_culling`: `None`
        fn default() -> Self {
            RenderState {
                blending: None,
                depth_test: Some(DepthComparison::Less),
                depth_write: DepthWrite::On,
                face_culling: None,
            }
        }
    }
}
pub mod shader {
    //! Shader stages, programs and uniforms.
    //!
    //! This module contains everything related to _shaders_. Shader programs — shaders, for short —
    //! are GPU binaries that run to perform a series of transformation. Typically run when a draw
    //! command is issued, they are responsible for:
    //!
    //! - Transforming vertex data. This is done in a _vertex shader_. Vertex data, such as the
    //!   positions, colors, UV coordinates, bi-tangents, etc. of each vertices will go through the
    //!   vertex shader and get transformed based on the code provided inside the stage. For example,
    //!   vertices can be projected on the screen with a perspective and view matrices.
    //! - Tessellating primitive patches. This is done in _tessellation shaders_.
    //! - Filtering, transforming again or even generating new vertices and primitives. This is done
    //!   by the _geometry shader_.
    //! - And finally, outputting a color for each _fragment_ covered by the objects you render. This
    //!   is done by the _fragment shader_.
    //!
    //! # Shader stages
    //!
    //! Right now, five shader stages  — [`Stage`] — are supported, ordered by usage in the graphics
    //! pipeline:
    //!
    //! 1. [`StageType::VertexShader`].
    //! 2. [`StageType::TessellationControlShader`].
    //! 3. [`StageType::TessellationEvaluationShader`].
    //! 4. [`StageType::GeometryShader`].
    //! 5. [`StageType::FragmentShader`].
    //!
    //! Those are not all mandatory: only the _vertex_ stage and _fragment_ stages are mandatory. If
    //! you want tessellation shaders, you have to provide both of them.
    //!
    //! Shader stages — [`Stage`] — are compiled independently at runtime by your GPU driver, and then
    //! _linked_ into a shader program. The creation of a [`Stage`] implies using an input string,
    //! representing the _source code_ of the stage. This is an opaque [`String`] that must represent
    //! a GLSL stage. The backend will transform the string into its own representation if needed.
    //!
    //! > For this version of the crate, the GLSL string must be at least 330-compliant. It is possible
    //! > that this changes in the future to be more flexible, but right now GLSL 150, for instance, is
    //! > not allowed.
    //!
    //! # Shader program
    //!
    //! A shader program — [`Program`] is akin to a binary program, but runs on GPU. It is invoked when
    //! you issue draw commands. It will run each stages you’ve put in it to transform vertices and
    //! rasterize fragments inside a framebuffer. Once this is done, the framebuffer will contain
    //! altered fragments by the final stage (fragment shader). If the shader program outputs several
    //! properties, we call that situation _MRT_ (Multiple Render Target) and the framebuffer must be
    //! configured to be able to receive those outputs — basically, it means that its _color slots_
    //! and/or _depth slots_ must adapt to the output of the shader program.
    //!
    //! Creating shader programs is done by gathering the [`Stage`] you want and _linking_ them. Some
    //! helper methods allow to create a shader [`Program`] directly from the string source for each
    //! stage, removing the need to build each stage individually.
    //!
    //! Shader programs are typed with three important piece of information:
    //!
    //! - The vertex [`Semantics`].
    //! - The render target outputs.
    //! - The [`UniformInterface`].
    //!
    //!
    //! # Vertex semantics
    //!
    //! When a shader program runs, it first executes the mandatory _vertex stage on a set of
    //! vertices. Those vertices have a given format — that is described by the [`Vertex`] trait.
    //! Because running a shader on an incompatible vertex would yield wrong results, both the
    //! vertices and the shader program must be tagged with a type which must implement the
    //! [`Semantics`] trait. More on that on the documentation of [`Semantics`].
    //!
    //! # Render target outputs
    //!
    //! A shader program, in its final mandatory _fragment stage_, will write values into the currently
    //! in-use framebuffer. The number of “channels” to write to represents the render targets.
    //! Typically, simple renders will simply write the color of a pixel — so only one render target.
    //! In that case, the type of the output of the shader program must match the color slot of the
    //! framebuffer it is used with.
    //!
    //! However, it is possible to write more data. For instance,
    //! [deferred shading](https://en.wikipedia.org/wiki/Deferred_shading) is a technique that requires
    //! to write several data to a framebuffer, called G-buffer (for geometry buffer): space
    //! coordinates, normals, tangents, bi-tangents, etc. In that case, your framebuffer must have
    //! a type matching the outputs of the fragment shader, too.
    //!
    //! # Shader customization
    //!
    //! A shader [`Program`] represents some code, in a binary form, that transform data. If you
    //! consider such code, it can adapt to the kind of data it receives, but the behavior is static.
    //! That means that it shouldn’t be possible to ask the program to do something else — shader
    //! programs don’t have a state as they must be spawned in parallel for your vertices, pixels, etc.
    //! However, there is a way to dynamically change what happens inside a shader program. That way
    //!
    //! The concept is similar to environment variables: you can declare, in your shader stages,
    //! _environment variables_ that will receive values from the host (i.e. on the Rust side). It is
    //! not possible to change those values while a draw command is issued: you have to change them
    //! in between draw commands. For this reason, those environment variables are often called
    //! _constant buffers_, _uniform_, _uniform buffers_, etc. by several graphics libraries. In our
    //! case, right now, we call them [`Uniform`].
    //!
    //! ## Uniforms
    //!
    //! A [`Uniform`] is parametric type that accepts the type of the value it will be able to change.
    //! For instance, `Uniform<f32>` represents a `f32` that can be changed in a shader program. That
    //! value can be set by the Rust program when the shader program is not currently in use — no
    //! draw commands.
    //!
    //! A [`Uniform`] is a _single_ variable that allows the most basic form of customization. It’s
    //! very similar to environment variables. You can declare several ones as you would declare
    //! several environment variables. More on that on the documentation of [`Uniform`].
    //!
    //! ## Uniform buffers
    //!
    //! > This section is under heavy rewriting, both the documentation and API.
    //!
    //! Sometimes, you will want to set and pass around rich and more complex data. Instead of a `f32`,
    //! you will want to pass a `struct`. This operation is currently supported but highly unsafe. The
    //! reason for this is that your GPU will expect a specific kind of memory layout for the types
    //! you use, and that also depends on the backend you use.
    //!
    //! Also, passing a lot of data is not very practical with default [`Uniform`] directly.
    //!
    //! In order to pass more data or `struct`s, you need to create a [`Buffer`]. That buffer will
    //! simply contain the data / object(s) you want to pass to the shader. It is then possible, via
    //! the use of a [`Pipeline`], to retrieve a [`BoundBuffer`], which can be used to get a
    //! [`BufferBinding`]. That [`BufferBinding`] can then be set on a
    //! `Uniform<BufferBinding<YourType>>`, telling your shader program where to grab the data — from
    //! the bound buffer.
    //!
    //! This way of doing is very practical and powerful but currently, in this version of the crate,
    //! very unsafe. A better API will be available in a next release to make all this simpler and
    //! safer.
    //!
    //! ## Uniform interfaces
    //!
    //! As with vertex semantics and render targets, the uniforms that can be used with a shader program
    //! are part of its type, too, and represented by a single type that must implement
    //! [`UniformInterface`]. That type can contain anything, but it is advised to just put [`Uniform`]
    //! fields in it. More on the [`UniformInterface`] documentation.
    //!
    //! [`Vertex`]: crate::vertex::Vertex
    //! [`Buffer`]: crate::buffer::Buffer
    //! [`Pipeline`]: crate::pipeline::Pipeline
    //! [`BoundBuffer`]: crate::pipeline::BoundBuffer
    //! [`BufferBinding`]: crate::pipeline::BufferBinding
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;
    use crate::backend::shader::{Shader, Uniformable};
    use crate::context::GraphicsContext;
    use crate::vertex::Semantics;
    /// A shader stage type.
    pub enum StageType {
        /// Vertex shader.
        VertexShader,
        /// Tessellation control shader.
        TessellationControlShader,
        /// Tessellation evaluation shader.
        TessellationEvaluationShader,
        /// Geometry shader.
        GeometryShader,
        /// Fragment shader.
        FragmentShader,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for StageType {
        #[inline]
        fn clone(&self) -> StageType {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for StageType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for StageType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&StageType::VertexShader,) => {
                    let mut debug_trait_builder = f.debug_tuple("VertexShader");
                    debug_trait_builder.finish()
                }
                (&StageType::TessellationControlShader,) => {
                    let mut debug_trait_builder = f.debug_tuple("TessellationControlShader");
                    debug_trait_builder.finish()
                }
                (&StageType::TessellationEvaluationShader,) => {
                    let mut debug_trait_builder = f.debug_tuple("TessellationEvaluationShader");
                    debug_trait_builder.finish()
                }
                (&StageType::GeometryShader,) => {
                    let mut debug_trait_builder = f.debug_tuple("GeometryShader");
                    debug_trait_builder.finish()
                }
                (&StageType::FragmentShader,) => {
                    let mut debug_trait_builder = f.debug_tuple("FragmentShader");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for StageType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for StageType {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for StageType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for StageType {
        #[inline]
        fn eq(&self, other: &StageType) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl fmt::Display for StageType {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                StageType::VertexShader => f.write_str("vertex shader"),
                StageType::TessellationControlShader => f.write_str("tessellation control shader"),
                StageType::TessellationEvaluationShader => {
                    f.write_str("tessellation evaluation shader")
                }
                StageType::GeometryShader => f.write_str("geometry shader"),
                StageType::FragmentShader => f.write_str("fragment shader"),
            }
        }
    }
    /// Errors that shader stages can emit.
    #[non_exhaustive]
    pub enum StageError {
        /// Occurs when a shader fails to compile.
        CompilationFailed(StageType, String),
        /// Occurs when you try to create a shader which type is not supported on the current hardware.
        UnsupportedType(StageType),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for StageError {
        #[inline]
        fn clone(&self) -> StageError {
            match (&*self,) {
                (&StageError::CompilationFailed(ref __self_0, ref __self_1),) => {
                    StageError::CompilationFailed(
                        ::core::clone::Clone::clone(&(*__self_0)),
                        ::core::clone::Clone::clone(&(*__self_1)),
                    )
                }
                (&StageError::UnsupportedType(ref __self_0),) => {
                    StageError::UnsupportedType(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for StageError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&StageError::CompilationFailed(ref __self_0, ref __self_1),) => {
                    let mut debug_trait_builder = f.debug_tuple("CompilationFailed");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&StageError::UnsupportedType(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("UnsupportedType");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl StageError {
        /// Occurs when a shader fails to compile.
        pub fn compilation_failed(ty: StageType, reason: impl Into<String>) -> Self {
            StageError::CompilationFailed(ty, reason.into())
        }
        /// Occurs when you try to create a shader which type is not supported on the current hardware.
        pub fn unsupported_type(ty: StageType) -> Self {
            StageError::UnsupportedType(ty)
        }
    }
    impl fmt::Display for StageError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                StageError::CompilationFailed(ref ty, ref r) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["", " compilation error: "],
                        &match (&ty, &r) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))
                }
                StageError::UnsupportedType(ty) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["unsupported "],
                    &match (&ty,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
            }
        }
    }
    impl error::Error for StageError {}
    impl From<StageError> for ProgramError {
        fn from(e: StageError) -> Self {
            ProgramError::StageError(e)
        }
    }
    /// Tessellation stages.
    ///
    /// - The `control` stage represents the _tessellation control stage_, which is invoked first.
    /// - The `evaluation` stage represents the _tessellation evaluation stage_, which is invoked after
    ///   the control stage has finished.
    ///
    /// # Parametricity
    ///
    /// - `S` is the representation of the stage. Depending on the interface you choose to create a
    ///   [`Program`], it might be a [`Stage`] or something akin to [`&str`] / [`String`].
    ///
    /// [`&str`]: str
    pub struct TessellationStages<'a, S>
    where
        S: ?Sized,
    {
        /// Tessellation control representation.
        pub control: &'a S,
        /// Tessellation evaluation representation.
        pub evaluation: &'a S,
    }
    /// Errors that a [`Program`] can generate.
    #[non_exhaustive]
    pub enum ProgramError {
        /// Creating the program failed.
        CreationFailed(String),
        /// A shader stage failed to compile or validate its state.
        StageError(StageError),
        /// Program link failed. You can inspect the reason by looking at the contained [`String`].
        LinkFailed(String),
        /// A program warning.
        Warning(ProgramWarning),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for ProgramError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&ProgramError::CreationFailed(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CreationFailed");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&ProgramError::StageError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("StageError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&ProgramError::LinkFailed(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("LinkFailed");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&ProgramError::Warning(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Warning");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ProgramError {
        /// Creating the program failed.
        pub fn creation_failed(reason: impl Into<String>) -> Self {
            ProgramError::CreationFailed(reason.into())
        }
        /// A shader stage failed to compile or validate its state.
        pub fn stage_error(e: StageError) -> Self {
            ProgramError::StageError(e)
        }
        /// Program link failed. You can inspect the reason by looking at the contained [`String`].
        pub fn link_failed(reason: impl Into<String>) -> Self {
            ProgramError::LinkFailed(reason.into())
        }
        /// A program warning.
        pub fn warning(w: ProgramWarning) -> Self {
            ProgramError::Warning(w)
        }
    }
    impl fmt::Display for ProgramError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                ProgramError::CreationFailed(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["cannot create shader program: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                ProgramError::StageError(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["shader program has stage error: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                ProgramError::LinkFailed(ref s) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["shader program failed to link: "],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                ProgramError::Warning(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["shader program warning: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
            }
        }
    }
    impl error::Error for ProgramError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                ProgramError::StageError(e) => Some(e),
                _ => None,
            }
        }
    }
    /// Program warnings, not necessarily considered blocking errors.
    pub enum ProgramWarning {
        /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
        /// type, etc. Check the [`UniformWarning`] type for more information.
        Uniform(UniformWarning),
        /// Some vertex attribute is ill-formed.
        VertexAttrib(VertexAttribWarning),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for ProgramWarning {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&ProgramWarning::Uniform(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Uniform");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&ProgramWarning::VertexAttrib(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("VertexAttrib");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl fmt::Display for ProgramWarning {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                ProgramWarning::Uniform(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["uniform warning: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                ProgramWarning::VertexAttrib(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["vertex attribute warning: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
            }
        }
    }
    impl error::Error for ProgramWarning {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                ProgramWarning::Uniform(e) => Some(e),
                ProgramWarning::VertexAttrib(e) => Some(e),
            }
        }
    }
    impl From<ProgramWarning> for ProgramError {
        fn from(e: ProgramWarning) -> Self {
            ProgramError::Warning(e)
        }
    }
    /// Warnings related to uniform issues.
    #[non_exhaustive]
    pub enum UniformWarning {
        /// Inactive uniform (not in use / no participation to the final output in shaders).
        Inactive(String),
        /// Type mismatch between the static requested type (i.e. the `T` in [`Uniform<T>`] for instance)
        /// and the type that got reflected from the backend in the shaders.
        ///
        /// The first [`String`] is the name of the uniform; the second one gives the type mismatch.
        ///
        /// [`Uniform<T>`]: crate::shader::Uniform
        TypeMismatch(String, UniformType),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for UniformWarning {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&UniformWarning::Inactive(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Inactive");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&UniformWarning::TypeMismatch(ref __self_0, ref __self_1),) => {
                    let mut debug_trait_builder = f.debug_tuple("TypeMismatch");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl UniformWarning {
        /// Create an inactive uniform warning.
        pub fn inactive<N>(name: N) -> Self
        where
            N: Into<String>,
        {
            UniformWarning::Inactive(name.into())
        }
        /// Create a type mismatch.
        pub fn type_mismatch<N>(name: N, ty: UniformType) -> Self
        where
            N: Into<String>,
        {
            UniformWarning::TypeMismatch(name.into(), ty)
        }
    }
    impl fmt::Display for UniformWarning {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                UniformWarning::Inactive(ref s) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["inactive ", " uniform"],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                UniformWarning::TypeMismatch(ref n, ref t) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["type mismatch for uniform ", ": "],
                        &match (&n, &t) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))
                }
            }
        }
    }
    impl From<UniformWarning> for ProgramWarning {
        fn from(e: UniformWarning) -> Self {
            ProgramWarning::Uniform(e)
        }
    }
    impl error::Error for UniformWarning {}
    /// Warnings related to vertex attributes issues.
    #[non_exhaustive]
    pub enum VertexAttribWarning {
        /// Inactive vertex attribute (not read).
        Inactive(String),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexAttribWarning {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&VertexAttribWarning::Inactive(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Inactive");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl VertexAttribWarning {
        /// Inactive vertex attribute (not read).
        pub fn inactive(attrib: impl Into<String>) -> Self {
            VertexAttribWarning::Inactive(attrib.into())
        }
    }
    impl fmt::Display for VertexAttribWarning {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                VertexAttribWarning::Inactive(ref s) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["inactive ", " vertex attribute"],
                        &match (&s,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
            }
        }
    }
    impl From<VertexAttribWarning> for ProgramWarning {
        fn from(e: VertexAttribWarning) -> Self {
            ProgramWarning::VertexAttrib(e)
        }
    }
    impl error::Error for VertexAttribWarning {}
    /// A GPU shader program environment variable.
    ///
    /// A uniform is a special variable that can be used to send data to a GPU. Several
    /// forms exist, but the idea is that `T` represents the data you want to send. Some exceptions
    /// exist that allow to pass _indirect_ data — such as [`BufferBinding`] to pass a buffer, or
    /// [`TextureBinding`] to pass a texture in order to fetch from it in a shader stage.
    ///
    /// You will never be able to store them by your own. Instead, you must use a [`UniformInterface`],
    /// which provides a _contravariant_ interface for you. Creation is `unsafe` and should be
    /// avoided. The [`UniformInterface`] is the only safe way to create those.
    ///
    /// # Parametricity
    ///
    /// - `T` is the type of data you want to be able to set in a shader program.
    ///
    /// [`BufferBinding`]: crate::pipeline::BufferBinding
    /// [`TextureBinding`]: crate::pipeline::TextureBinding
    pub struct Uniform<T>
    where
        T: ?Sized,
    {
        index: i32,
        _t: PhantomData<*const T>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for Uniform<T>
    where
        T: ?Sized,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Uniform {
                    index: ref __self_0_0,
                    _t: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Uniform");
                    let _ = debug_trait_builder.field("index", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_t", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<T> Uniform<T>
    where
        T: ?Sized,
    {
        /// Create a new [`Uniform`].
        ///
        /// # Safety
        ///
        /// This method must be used **only** by backends. If you end up using it,
        /// then you’re doing something wrong. Read on [`UniformInterface`] for further
        /// information.
        pub unsafe fn new(index: i32) -> Self {
            Uniform {
                index,
                _t: PhantomData,
            }
        }
        /// Retrieve the internal index.
        ///
        /// Even though that function is safe, you have no reason to use it. Read on
        /// [`UniformInterface`] for further details.
        pub fn index(&self) -> i32 {
            self.index
        }
    }
    /// Type of a uniform.
    ///
    /// This is an exhaustive list of possible types of value you can send to a shader program.
    /// A [`UniformType`] is associated to any type that can be considered sent via the
    /// [`Uniformable`] trait.
    pub enum UniformType {
        /// 32-bit signed integer.
        Int,
        /// 32-bit unsigned integer.
        UInt,
        /// 32-bit floating-point number.
        Float,
        /// Boolean.
        Bool,
        /// 2D signed integral vector.
        IVec2,
        /// 3D signed integral vector.
        IVec3,
        /// 4D signed integral vector.
        IVec4,
        /// 2D unsigned integral vector.
        UIVec2,
        /// 3D unsigned integral vector.
        UIVec3,
        /// 4D unsigned integral vector.
        UIVec4,
        /// 2D floating-point vector.
        Vec2,
        /// 3D floating-point vector.
        Vec3,
        /// 4D floating-point vector.
        Vec4,
        /// 2D boolean vector.
        BVec2,
        /// 3D boolean vector.
        BVec3,
        /// 4D boolean vector.
        BVec4,
        /// 2×2 floating-point matrix.
        M22,
        /// 3×3 floating-point matrix.
        M33,
        /// 4×4 floating-point matrix.
        M44,
        /// Signed integral 1D texture sampler.
        ISampler1D,
        /// Signed integral 2D texture sampler.
        ISampler2D,
        /// Signed integral 3D texture sampler.
        ISampler3D,
        /// Signed integral 1D array texture sampler.
        ISampler1DArray,
        /// Signed integral 2D array texture sampler.
        ISampler2DArray,
        /// Unsigned integral 1D texture sampler.
        UISampler1D,
        /// Unsigned integral 2D texture sampler.
        UISampler2D,
        /// Unsigned integral 3D texture sampler.
        UISampler3D,
        /// Unsigned integral 1D array texture sampler.
        UISampler1DArray,
        /// Unsigned integral 2D array texture sampler.
        UISampler2DArray,
        /// Floating-point 1D texture sampler.
        Sampler1D,
        /// Floating-point 2D texture sampler.
        Sampler2D,
        /// Floating-point 3D texture sampler.
        Sampler3D,
        /// Floating-point 1D array texture sampler.
        Sampler1DArray,
        /// Floating-point 2D array texture sampler.
        Sampler2DArray,
        /// Signed cubemap sampler.
        ICubemap,
        /// Unsigned cubemap sampler.
        UICubemap,
        /// Floating-point cubemap sampler.
        Cubemap,
        /// Buffer binding; used for UBOs.
        BufferBinding,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for UniformType {
        #[inline]
        fn clone(&self) -> UniformType {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for UniformType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for UniformType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&UniformType::Int,) => {
                    let mut debug_trait_builder = f.debug_tuple("Int");
                    debug_trait_builder.finish()
                }
                (&UniformType::UInt,) => {
                    let mut debug_trait_builder = f.debug_tuple("UInt");
                    debug_trait_builder.finish()
                }
                (&UniformType::Float,) => {
                    let mut debug_trait_builder = f.debug_tuple("Float");
                    debug_trait_builder.finish()
                }
                (&UniformType::Bool,) => {
                    let mut debug_trait_builder = f.debug_tuple("Bool");
                    debug_trait_builder.finish()
                }
                (&UniformType::IVec2,) => {
                    let mut debug_trait_builder = f.debug_tuple("IVec2");
                    debug_trait_builder.finish()
                }
                (&UniformType::IVec3,) => {
                    let mut debug_trait_builder = f.debug_tuple("IVec3");
                    debug_trait_builder.finish()
                }
                (&UniformType::IVec4,) => {
                    let mut debug_trait_builder = f.debug_tuple("IVec4");
                    debug_trait_builder.finish()
                }
                (&UniformType::UIVec2,) => {
                    let mut debug_trait_builder = f.debug_tuple("UIVec2");
                    debug_trait_builder.finish()
                }
                (&UniformType::UIVec3,) => {
                    let mut debug_trait_builder = f.debug_tuple("UIVec3");
                    debug_trait_builder.finish()
                }
                (&UniformType::UIVec4,) => {
                    let mut debug_trait_builder = f.debug_tuple("UIVec4");
                    debug_trait_builder.finish()
                }
                (&UniformType::Vec2,) => {
                    let mut debug_trait_builder = f.debug_tuple("Vec2");
                    debug_trait_builder.finish()
                }
                (&UniformType::Vec3,) => {
                    let mut debug_trait_builder = f.debug_tuple("Vec3");
                    debug_trait_builder.finish()
                }
                (&UniformType::Vec4,) => {
                    let mut debug_trait_builder = f.debug_tuple("Vec4");
                    debug_trait_builder.finish()
                }
                (&UniformType::BVec2,) => {
                    let mut debug_trait_builder = f.debug_tuple("BVec2");
                    debug_trait_builder.finish()
                }
                (&UniformType::BVec3,) => {
                    let mut debug_trait_builder = f.debug_tuple("BVec3");
                    debug_trait_builder.finish()
                }
                (&UniformType::BVec4,) => {
                    let mut debug_trait_builder = f.debug_tuple("BVec4");
                    debug_trait_builder.finish()
                }
                (&UniformType::M22,) => {
                    let mut debug_trait_builder = f.debug_tuple("M22");
                    debug_trait_builder.finish()
                }
                (&UniformType::M33,) => {
                    let mut debug_trait_builder = f.debug_tuple("M33");
                    debug_trait_builder.finish()
                }
                (&UniformType::M44,) => {
                    let mut debug_trait_builder = f.debug_tuple("M44");
                    debug_trait_builder.finish()
                }
                (&UniformType::ISampler1D,) => {
                    let mut debug_trait_builder = f.debug_tuple("ISampler1D");
                    debug_trait_builder.finish()
                }
                (&UniformType::ISampler2D,) => {
                    let mut debug_trait_builder = f.debug_tuple("ISampler2D");
                    debug_trait_builder.finish()
                }
                (&UniformType::ISampler3D,) => {
                    let mut debug_trait_builder = f.debug_tuple("ISampler3D");
                    debug_trait_builder.finish()
                }
                (&UniformType::ISampler1DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("ISampler1DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::ISampler2DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("ISampler2DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::UISampler1D,) => {
                    let mut debug_trait_builder = f.debug_tuple("UISampler1D");
                    debug_trait_builder.finish()
                }
                (&UniformType::UISampler2D,) => {
                    let mut debug_trait_builder = f.debug_tuple("UISampler2D");
                    debug_trait_builder.finish()
                }
                (&UniformType::UISampler3D,) => {
                    let mut debug_trait_builder = f.debug_tuple("UISampler3D");
                    debug_trait_builder.finish()
                }
                (&UniformType::UISampler1DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("UISampler1DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::UISampler2DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("UISampler2DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::Sampler1D,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sampler1D");
                    debug_trait_builder.finish()
                }
                (&UniformType::Sampler2D,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sampler2D");
                    debug_trait_builder.finish()
                }
                (&UniformType::Sampler3D,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sampler3D");
                    debug_trait_builder.finish()
                }
                (&UniformType::Sampler1DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sampler1DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::Sampler2DArray,) => {
                    let mut debug_trait_builder = f.debug_tuple("Sampler2DArray");
                    debug_trait_builder.finish()
                }
                (&UniformType::ICubemap,) => {
                    let mut debug_trait_builder = f.debug_tuple("ICubemap");
                    debug_trait_builder.finish()
                }
                (&UniformType::UICubemap,) => {
                    let mut debug_trait_builder = f.debug_tuple("UICubemap");
                    debug_trait_builder.finish()
                }
                (&UniformType::Cubemap,) => {
                    let mut debug_trait_builder = f.debug_tuple("Cubemap");
                    debug_trait_builder.finish()
                }
                (&UniformType::BufferBinding,) => {
                    let mut debug_trait_builder = f.debug_tuple("BufferBinding");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for UniformType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for UniformType {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for UniformType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for UniformType {
        #[inline]
        fn eq(&self, other: &UniformType) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl fmt::Display for UniformType {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                UniformType::Int => f.write_str("int"),
                UniformType::UInt => f.write_str("uint"),
                UniformType::Float => f.write_str("float"),
                UniformType::Bool => f.write_str("bool"),
                UniformType::IVec2 => f.write_str("ivec2"),
                UniformType::IVec3 => f.write_str("ivec3"),
                UniformType::IVec4 => f.write_str("ivec4"),
                UniformType::UIVec2 => f.write_str("uvec2"),
                UniformType::UIVec3 => f.write_str("uvec3"),
                UniformType::UIVec4 => f.write_str("uvec4"),
                UniformType::Vec2 => f.write_str("vec2"),
                UniformType::Vec3 => f.write_str("vec3"),
                UniformType::Vec4 => f.write_str("vec4"),
                UniformType::BVec2 => f.write_str("bvec2"),
                UniformType::BVec3 => f.write_str("bvec3"),
                UniformType::BVec4 => f.write_str("bvec4"),
                UniformType::M22 => f.write_str("mat2"),
                UniformType::M33 => f.write_str("mat3"),
                UniformType::M44 => f.write_str("mat4"),
                UniformType::ISampler1D => f.write_str("isampler1D"),
                UniformType::ISampler2D => f.write_str("isampler2D"),
                UniformType::ISampler3D => f.write_str("isampler3D"),
                UniformType::ISampler1DArray => f.write_str("isampler1DArray"),
                UniformType::ISampler2DArray => f.write_str("isampler2DArray"),
                UniformType::UISampler1D => f.write_str("usampler1D"),
                UniformType::UISampler2D => f.write_str("usampler2D"),
                UniformType::UISampler3D => f.write_str("usampler3D"),
                UniformType::UISampler1DArray => f.write_str("usampler1DArray"),
                UniformType::UISampler2DArray => f.write_str("usampler2DArray"),
                UniformType::Sampler1D => f.write_str("sampler1D"),
                UniformType::Sampler2D => f.write_str("sampler2D"),
                UniformType::Sampler3D => f.write_str("sampler3D"),
                UniformType::Sampler1DArray => f.write_str("sampler1DArray"),
                UniformType::Sampler2DArray => f.write_str("sampler2DArray"),
                UniformType::ICubemap => f.write_str("isamplerCube"),
                UniformType::UICubemap => f.write_str("usamplerCube"),
                UniformType::Cubemap => f.write_str("samplerCube"),
                UniformType::BufferBinding => f.write_str("buffer binding"),
            }
        }
    }
    /// A shader stage.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    ///
    /// [`&str`]: str
    pub struct Stage<B>
    where
        B: ?Sized + Shader,
    {
        repr: B::StageRepr,
    }
    impl<B> Stage<B>
    where
        B: ?Sized + Shader,
    {
        /// Create a new stage of type `ty` by compiling `src`.
        ///
        /// # Parametricity
        ///
        /// - `C` is the graphics context. `C::Backend` must implement the [`Shader`] trait.
        /// - `R` is the source code to use in the stage. It must implement [`AsRef<str>`].
        ///
        /// # Notes
        ///
        /// Feel free to consider using [`GraphicsContext::new_shader_stage`] for a simpler form of
        /// this method.
        ///
        /// [`AsRef<str>`]: AsRef
        pub fn new<C, R>(ctx: &mut C, ty: StageType, src: R) -> Result<Self, StageError>
        where
            C: GraphicsContext<Backend = B>,
            R: AsRef<str>,
        {
            unsafe {
                ctx.backend()
                    .new_stage(ty, src.as_ref())
                    .map(|repr| Stage { repr })
            }
        }
    }
    /// A builder of [`Uniform`].
    ///
    /// A [`UniformBuilder`] is an important type as it’s the only one that allows to safely create
    /// [`Uniform`] values.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement the [`Shader`] trait.
    pub struct UniformBuilder<'a, B>
    where
        B: ?Sized + Shader,
    {
        repr: B::UniformBuilderRepr,
        warnings: Vec<UniformWarning>,
        _a: PhantomData<&'a mut ()>,
    }
    impl<'a, B> UniformBuilder<'a, B>
    where
        B: ?Sized + Shader,
    {
        /// Ask the creation of a [`Uniform`], identified by its `name`.
        pub fn ask<T, N>(&mut self, name: N) -> Result<Uniform<T>, UniformWarning>
        where
            N: AsRef<str>,
            T: Uniformable<B>,
        {
            unsafe { B::ask_uniform(&mut self.repr, name.as_ref()) }
        }
        /// Ask the creation of a [`Uniform`], identified by its `name`.
        ///
        /// If the name is not found, an _unbound_ [`Uniform`] is returned (i.e. a [`Uniform`]) that does
        /// nothing.
        pub fn ask_or_unbound<T, N>(&mut self, name: N) -> Uniform<T>
        where
            N: AsRef<str>,
            T: Uniformable<B>,
        {
            match self.ask(name) {
                Ok(uniform) => uniform,
                Err(err) => {
                    self.warnings.push(err);
                    unsafe { B::unbound(&mut self.repr) }
                }
            }
        }
    }
    /// [`Uniform`] interface.
    ///
    /// When a type implements [`UniformInterface`], it means that it can be used as part of a shader
    /// [`Program`] type. When a [`Program`] is in use in a graphics pipeline, its [`UniformInterface`]
    /// is automatically provided to the user, giving them access to all the fields declared in. Then,
    /// they can pass data to shaders before issuing draw commands.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type. It must implement [`Shader`].
    /// - `E` is the environment type. Set by default to `()`, it allows to pass a mutable
    ///   object at construction-site of the [`UniformInterface`]. It can be useful to generate
    ///   events or customize the way the [`Uniform`] are built by doing some lookups in hashmaps, etc.
    ///
    /// # Notes
    ///
    /// Implementing this trait — especially [`UniformInterface::uniform_interface`] can be a bit
    /// overwhelming. It is highly recommended to use [luminance-derive]’s `UniformInterface`
    /// proc-macro, which will do that for you by scanning your type declaration.
    ///
    /// [luminance-derive]: https://crates.io/crates/luminance-derive
    pub trait UniformInterface<B, E = ()>: Sized
    where
        B: ?Sized + Shader,
    {
        /// Create a [`UniformInterface`] by constructing [`Uniform`]s with a [`UniformBuilder`] and an
        /// optional environment object.
        ///
        /// This method is the only place where `Self` should be created. In theory, you could create it
        /// the way you want (since the type is provided by you) but not all types make sense. You will
        /// likely want to have some [`Uniform`] objects in your type, and the [`UniformBuilder`] that is
        /// provided as argument is the only way to create them.
        fn uniform_interface<'a>(
            builder: &mut UniformBuilder<'a, B>,
            env: &mut E,
        ) -> Result<Self, UniformWarning>;
    }
    impl<B, E> UniformInterface<B, E> for ()
    where
        B: ?Sized + Shader,
    {
        fn uniform_interface<'a>(
            _: &mut UniformBuilder<'a, B>,
            _: &mut E,
        ) -> Result<Self, UniformWarning> {
            Ok(())
        }
    }
    /// A built program with potential warnings.
    ///
    /// The sole purpose of this type is to be destructured when a program is built.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    /// - `Sem` is the [`Semantics`] type.
    /// - `Out` is the render target type.
    /// - `Uni` is the [`UniformInterface`] type.
    pub struct BuiltProgram<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
    {
        /// Built program.
        pub program: Program<B, Sem, Out, Uni>,
        /// Potential warnings.
        pub warnings: Vec<ProgramError>,
    }
    impl<B, Sem, Out, Uni> BuiltProgram<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
    {
        /// Get the program and ignore the warnings.
        pub fn ignore_warnings(self) -> Program<B, Sem, Out, Uni> {
            self.program
        }
    }
    /// A [`Program`] uniform adaptation that has failed.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    /// - `Sem` is the [`Semantics`] type.
    /// - `Out` is the render target type.
    /// - `Uni` is the [`UniformInterface`] type.
    pub struct AdaptationFailure<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
    {
        /// Program used before trying to adapt.
        pub program: Program<B, Sem, Out, Uni>,
        /// Program error that prevented to adapt.
        pub error: ProgramError,
    }
    impl<B, Sem, Out, Uni> AdaptationFailure<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
    {
        pub(crate) fn new(program: Program<B, Sem, Out, Uni>, error: ProgramError) -> Self {
            AdaptationFailure { program, error }
        }
        /// Get the program and ignore the error.
        pub fn ignore_error(self) -> Program<B, Sem, Out, Uni> {
            self.program
        }
    }
    /// Interact with the [`UniformInterface`] carried by a [`Program`] and/or perform dynamic
    /// uniform lookup.
    ///
    /// This type allows to set — [`ProgramInterface::set`] – uniforms for a [`Program`].
    ///
    /// In the case where you don’t have a uniform interface or need to dynamically lookup uniforms,
    /// you can use the [`ProgramInterface::query`] method.
    ///
    /// # Parametricity
    ///
    /// `B` is the backend type.
    pub struct ProgramInterface<'a, B>
    where
        B: ?Sized + Shader,
    {
        pub(crate) program: &'a mut B::ProgramRepr,
    }
    impl<'a, B> ProgramInterface<'a, B>
    where
        B: ?Sized + Shader,
    {
        /// Set a value on a [`Uniform`].
        pub fn set<T>(&mut self, uniform: &Uniform<T>, value: T)
        where
            T: Uniformable<B>,
        {
            unsafe { T::update(value, self.program, uniform) };
        }
        /// Get back a [`UniformBuilder`] to dynamically access [`Uniform`] objects.
        pub fn query(&mut self) -> Result<UniformBuilder<'a, B>, ProgramError> {
            unsafe {
                B::new_uniform_builder(&mut self.program).map(|repr| UniformBuilder {
                    repr,
                    warnings: Vec::new(),
                    _a: PhantomData,
                })
            }
        }
    }
    /// A [`Program`] builder.
    ///
    /// This type allows to create shader programs without having to worry too much about the highly
    /// generic API.
    pub struct ProgramBuilder<'a, C, Sem, Out, Uni> {
        ctx: &'a mut C,
        _phantom: PhantomData<(Sem, Out, Uni)>,
    }
    impl<'a, C, Sem, Out, Uni> ProgramBuilder<'a, C, Sem, Out, Uni>
    where
        C: GraphicsContext,
        C::Backend: Shader,
        Sem: Semantics,
    {
        /// Create a new [`ProgramBuilder`] from a [`GraphicsContext`].
        pub fn new(ctx: &'a mut C) -> Self {
            ProgramBuilder {
                ctx,
                _phantom: PhantomData,
            }
        }
        /// Create a [`Program`] by linking [`Stage`]s and accessing a mutable environment variable.
        ///
        /// # Parametricity
        ///
        /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`Stage`] inside.
        /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
        /// - `E` is the mutable environment variable.
        ///
        /// # Notes
        ///
        /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
        /// a simpler interface.
        pub fn from_stages_env<'b, T, G, E>(
            &mut self,
            vertex: &'b Stage<C::Backend>,
            tess: T,
            geometry: G,
            fragment: &'b Stage<C::Backend>,
            env: &mut E,
        ) -> Result<BuiltProgram<C::Backend, Sem, Out, Uni>, ProgramError>
        where
            Uni: UniformInterface<C::Backend, E>,
            T: Into<Option<TessellationStages<'b, Stage<C::Backend>>>>,
            G: Into<Option<&'b Stage<C::Backend>>>,
        {
            let tess = tess.into();
            let geometry = geometry.into();
            unsafe {
                let mut repr = self.ctx.backend().new_program(
                    &vertex.repr,
                    tess.map(|stages| TessellationStages {
                        control: &stages.control.repr,
                        evaluation: &stages.evaluation.repr,
                    }),
                    geometry.map(|stage| &stage.repr),
                    &fragment.repr,
                )?;
                let warnings = C::Backend::apply_semantics::<Sem>(&mut repr)?
                    .into_iter()
                    .map(|w| ProgramError::Warning(w.into()))
                    .collect();
                let mut uniform_builder =
                    C::Backend::new_uniform_builder(&mut repr).map(|repr| UniformBuilder {
                        repr,
                        warnings: Vec::new(),
                        _a: PhantomData,
                    })?;
                let uni = Uni::uniform_interface(&mut uniform_builder, env)
                    .map_err(ProgramWarning::Uniform)?;
                let program = Program {
                    repr,
                    uni,
                    _sem: PhantomData,
                    _out: PhantomData,
                };
                Ok(BuiltProgram { program, warnings })
            }
        }
        /// Create a [`Program`] by linking [`Stage`]s.
        ///
        /// # Parametricity
        ///
        /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`Stage`] inside.
        /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
        ///
        /// # Notes
        ///
        /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
        /// a simpler interface.
        pub fn from_stages<'b, T, G>(
            &mut self,
            vertex: &'b Stage<C::Backend>,
            tess: T,
            geometry: G,
            fragment: &'b Stage<C::Backend>,
        ) -> Result<BuiltProgram<C::Backend, Sem, Out, Uni>, ProgramError>
        where
            Uni: UniformInterface<C::Backend>,
            T: Into<Option<TessellationStages<'b, Stage<C::Backend>>>>,
            G: Into<Option<&'b Stage<C::Backend>>>,
        {
            Self::from_stages_env(self, vertex, tess, geometry, fragment, &mut ())
        }
        /// Create a [`Program`] by linking [`&str`]s and accessing a mutable environment variable.
        ///
        /// # Parametricity
        ///
        /// - `C` is the graphics context.
        /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`&str`] inside.
        /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
        /// - `E` is the mutable environment variable.
        ///
        /// # Notes
        ///
        /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
        /// a simpler interface.
        ///
        /// [`&str`]: str
        pub fn from_strings_env<'b, T, G, E>(
            &mut self,
            vertex: &'b str,
            tess: T,
            geometry: G,
            fragment: &'b str,
            env: &mut E,
        ) -> Result<BuiltProgram<C::Backend, Sem, Out, Uni>, ProgramError>
        where
            Uni: UniformInterface<C::Backend, E>,
            T: Into<Option<TessellationStages<'b, str>>>,
            G: Into<Option<&'b str>>,
        {
            let vs_stage = Stage::new(self.ctx, StageType::VertexShader, vertex)?;
            let tess_stages = match tess.into() {
                Some(TessellationStages {
                    control,
                    evaluation,
                }) => {
                    let control_stage =
                        Stage::new(self.ctx, StageType::TessellationControlShader, control)?;
                    let evaluation_stage = Stage::new(
                        self.ctx,
                        StageType::TessellationEvaluationShader,
                        evaluation,
                    )?;
                    Some((control_stage, evaluation_stage))
                }
                None => None,
            };
            let tess_stages =
                tess_stages
                    .as_ref()
                    .map(|(ref control, ref evaluation)| TessellationStages {
                        control,
                        evaluation,
                    });
            let gs_stage = match geometry.into() {
                Some(geometry) => Some(Stage::new(self.ctx, StageType::GeometryShader, geometry)?),
                None => None,
            };
            let fs_stage = Stage::new(self.ctx, StageType::FragmentShader, fragment)?;
            Self::from_stages_env(
                self,
                &vs_stage,
                tess_stages,
                gs_stage.as_ref(),
                &fs_stage,
                env,
            )
        }
        /// Create a [`Program`] by linking [`&str`]s.
        ///
        /// # Parametricity
        ///
        /// - `C` is the graphics context.
        /// - `T` is an [`Option`] containing a [`TessellationStages`] with [`&str`] inside.
        /// - `G` is an [`Option`] containing a [`Stage`] inside (geometry shader).
        ///
        /// # Notes
        ///
        /// Feel free to look at the documentation of [`GraphicsContext::new_shader_program`] for
        /// a simpler interface.
        ///
        /// [`&str`]: str
        pub fn from_strings<'b, T, G>(
            &mut self,
            vertex: &'b str,
            tess: T,
            geometry: G,
            fragment: &'b str,
        ) -> Result<BuiltProgram<C::Backend, Sem, Out, Uni>, ProgramError>
        where
            Uni: UniformInterface<C::Backend>,
            T: Into<Option<TessellationStages<'b, str>>>,
            G: Into<Option<&'b str>>,
        {
            Self::from_strings_env(self, vertex, tess, geometry, fragment, &mut ())
        }
    }
    /// A shader program.
    ///
    /// Shader programs are GPU binaries that execute when a draw command is issued.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    /// - `Sem` is the [`Semantics`] type.
    /// - `Out` is the render target type.
    /// - `Uni` is the [`UniformInterface`] type.
    pub struct Program<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
    {
        pub(crate) repr: B::ProgramRepr,
        pub(crate) uni: Uni,
        _sem: PhantomData<*const Sem>,
        _out: PhantomData<*const Out>,
    }
    type ProgramResult<B, Sem, Out, Uni, Q = Uni> =
        Result<BuiltProgram<B, Sem, Out, Q>, AdaptationFailure<B, Sem, Out, Uni>>;
    impl<B, Sem, Out, Uni> Program<B, Sem, Out, Uni>
    where
        B: ?Sized + Shader,
        Sem: Semantics,
    {
        /// Create a new [`UniformInterface`] but keep the [`Program`] around without rebuilding it.
        ///
        /// # Parametricity
        ///
        /// - `Q` is the new [`UniformInterface`].
        pub fn adapt<Q>(self) -> ProgramResult<B, Sem, Out, Uni, Q>
        where
            Q: UniformInterface<B>,
        {
            self.adapt_env(&mut ())
        }
        /// Create a new [`UniformInterface`] but keep the [`Program`] around without rebuilding it, by
        /// using a mutable environment variable.
        ///
        /// # Parametricity
        ///
        /// - `Q` is the new [`UniformInterface`].
        /// - `E` is the mutable environment variable.
        pub fn adapt_env<Q, E>(mut self, env: &mut E) -> ProgramResult<B, Sem, Out, Uni, Q>
        where
            Q: UniformInterface<B, E>,
        {
            let mut uniform_builder: UniformBuilder<B> =
                match unsafe { B::new_uniform_builder(&mut self.repr) } {
                    Ok(repr) => UniformBuilder {
                        repr,
                        warnings: Vec::new(),
                        _a: PhantomData,
                    },
                    Err(e) => return Err(AdaptationFailure::new(self, e)),
                };
            let uni = match Q::uniform_interface(&mut uniform_builder, env) {
                Ok(uni) => uni,
                Err(e) => {
                    return Err(AdaptationFailure::new(
                        self,
                        ProgramWarning::Uniform(e).into(),
                    ))
                }
            };
            let warnings = uniform_builder
                .warnings
                .into_iter()
                .map(|w| ProgramError::Warning(w.into()))
                .collect();
            let program = Program {
                repr: self.repr,
                uni,
                _sem: PhantomData,
                _out: PhantomData,
            };
            Ok(BuiltProgram { program, warnings })
        }
        /// Re-create the [`UniformInterface`] but keep the [`Program`] around without rebuilding it.
        ///
        /// # Parametricity
        ///
        /// - `E` is the mutable environment variable.
        pub fn readapt_env<E>(self, env: &mut E) -> ProgramResult<B, Sem, Out, Uni>
        where
            Uni: UniformInterface<B, E>,
        {
            self.adapt_env(env)
        }
    }
}
pub mod shading_gate {
    //! Shading gates.
    //!
    //! A shading gate is a _pipeline node_ that allows to share shader [`Program`] for deeper nodes.
    //!
    //! [`Program`]: crate::shader::Program
    use crate::backend::shading_gate::ShadingGate as ShadingGateBackend;
    use crate::render_gate::RenderGate;
    use crate::shader::{Program, ProgramInterface, UniformInterface};
    use crate::vertex::Semantics;
    /// A shading gate.
    ///
    /// This is obtained after entering a [`PipelineGate`].
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type.
    ///
    /// [`PipelineGate`]: crate::pipeline::PipelineGate
    pub struct ShadingGate<'a, B>
    where
        B: ?Sized,
    {
        pub(crate) backend: &'a mut B,
    }
    impl<'a, B> ShadingGate<'a, B>
    where
        B: ?Sized + ShadingGateBackend,
    {
        /// Enter a [`ShadingGate`] by using a shader [`Program`].
        ///
        /// The argument closure is given two arguments:
        ///
        /// - A [`ProgramInterface`], that allows to pass values (via [`ProgramInterface::set`]) to the
        ///   in-use shader [`Program`] and/or perform dynamic lookup of uniforms.
        /// - A [`RenderGate`], allowing to create deeper nodes in the graphics pipeline.
        pub fn shade<E, Sem, Out, Uni, F>(
            &mut self,
            program: &mut Program<B, Sem, Out, Uni>,
            f: F,
        ) -> Result<(), E>
        where
            Sem: Semantics,
            Uni: UniformInterface<B>,
            F: for<'b> FnOnce(ProgramInterface<'b, B>, &'b Uni, RenderGate<'b, B>) -> Result<(), E>,
        {
            unsafe {
                self.backend.apply_shader_program(&program.repr);
            }
            let render_gate = RenderGate {
                backend: self.backend,
            };
            let program_interface = ProgramInterface {
                program: &mut program.repr,
            };
            f(program_interface, &program.uni, render_gate)
        }
    }
}
pub mod tess {
    //! Vertex sets.
    //!
    //! [`Tess`] is a type that represents the gathering of vertices and the way to connect / link
    //! them. A [`Tess`] has several intrinsic properties:
    //!
    //! - Its _primitive mode_ — [`Mode`]. That object tells the GPU how to connect the vertices.
    //! - A default number of vertex to render. When passing the [`Tess`] to the GPU for rendering,
    //!   it’s possible to specify the number of vertices to render or just let the [`Tess`] render
    //!   a default number of vertices (typically, the whole [`Tess`]).
    //! - A default number of _instances_, which allows for geometry instancing. Geometry instancing
    //!   is the fact of drawing with the same [`Tess`] (GPU buffers) several times, only changing the
    //!   instance index every time a new render is performed. This is done entirely on the GPU to
    //!   prevent bandwidth exhaustion. The index of the instance, in the shader stages, is often used
    //!   to pick material properties, matrices, etc. to customize each instances.
    //! - An indexed configuration, allowing to tell the GPU how to render the vertices by referring to
    //!   them via indices.
    //! - For indexed configuration, an optional _primitive restart index_ can be specified. That
    //!   index, when present in the indexed set, will make some primitive modes _“restart”_ and create
    //!   new primitives. More on this on the documentation of [`Mode`].
    //!
    //! # Tessellation creation
    //!
    //! [`Tess`] is not created directly. Instead, you need to use a [`TessBuilder`]. Tessellation
    //! builders make it easy to customize what a [`Tess`] will be made of before actually requesting
    //! the GPU to create them. They support a large number of possible situations:
    //!
    //! - _Attributeless_: when you only specify the [`Mode`] and number of vertices to render (and
    //!   optionally the number of instances). That will create a vertex set with no vertex data. Your
    //!   vertex shader will be responsible for creating the vertex attributes on the fly.
    //! - _Direct geometry_: when you pass vertices directly.
    //! - _Indexed geometry_: when you pass vertices and reference from with indices.
    //! - _Instanced geometry_: when you ask to use instances, making the graphics pipeline create
    //!   several instances of your vertex set on the GPU.
    //!
    //! # Tessellation views
    //!
    //! Once you have a [`Tess`] — created from [`TessBuilder::build`], you can now render it in a
    //! [`TessGate`]. In order to do so, you need a [`TessView`].
    //!
    //! A [`TessView`] is a temporary _view_ into a [`Tess`], describing what part of it should be
    //! drawn. Creating [`TessView`]s is a cheap operation, and can be done in two different ways:
    //!
    //! - By directly using the methods from [`TessView`].
    //! - By using the [`View`] trait.
    //!
    //! The [`View`] trait is a convenient way to create [`TessView`]. It provides the
    //! [`View::view`] and [`View::inst_view`] methods, which accept Rust’s range operators
    //! to create the [`TessView`]s in a more comfortable way.
    //!
    //! # Tessellation mapping
    //!
    //! Sometimes, you will want to edit tessellations in a dynamic way instead of re-creating new
    //! ones. That can be useful for streaming data of for using a small part of a big [`Tess`]. The
    //! [`Tess`] type has several methods to obtain subparts, allow you to map values and iterate over
    //! them via standard Rust slices. See these for further details:
    //!
    //! - [`Tess::vertices`] [`Tess::vertices_mut`] to map tessellations’ vertices.
    //! - [`Tess::indices`] [`Tess::indices_mut`] to map tessellations’ indices.
    //! - [`Tess::instances`] [`Tess::instances_mut`] to map tessellations’ instances.
    //!
    //! > Note: because of their slice nature, mapping a tessellation (vertices, indices or instances)
    //! > will not help you with resizing a [`Tess`], as this is not currently supported.
    //!
    //! [`TessGate`]: crate::tess_gate::TessGate
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;
    use std::ops::{
        Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    };
    use crate::backend::tess::{
        IndexSlice as IndexSliceBackend, InstanceSlice as InstanceSliceBackend,
        Tess as TessBackend, VertexSlice as VertexSliceBackend,
    };
    use crate::buffer::BufferError;
    use crate::context::GraphicsContext;
    use crate::vertex::{Deinterleave, Vertex, VertexDesc};
    /// Vertices can be connected via several modes.
    ///
    /// Some modes allow for _primitive restart_. Primitive restart is a cool feature that allows to
    /// _break_ the building of a primitive to _start over again_. For instance, when making a curve,
    /// you can imagine gluing segments next to each other. If at some point, you want to start a new
    /// curve, you have two choices:
    ///
    ///   - Either you stop your draw call and make another one.
    ///   - Or you just use the _primitive restart_ feature to ask to create another line from scratch.
    ///
    /// _Primitive restart_ should be used as much as possible as it will decrease the number of GPU
    /// commands you have to issue.
    ///
    /// That feature is encoded with a special _vertex index_. You can setup the value of the _primitive
    /// restart index_ with [`TessBuilder::set_primitive_restart_index`]. Whenever a vertex index is set
    /// to the same value as the _primitive restart index_, the value is not interpreted as a vertex
    /// index but just a marker / hint to start a new primitive.
    pub enum Mode {
        /// A single point.
        ///
        /// Points are left unconnected from each other and represent a _point cloud_. This is the typical
        /// primitive mode you want to do, for instance, particles rendering.
        Point,
        /// A line, defined by two points.
        ///
        /// Every pair of vertices are connected together to form a straight line.
        Line,
        /// A strip line, defined by at least two points and zero or many other ones.
        ///
        /// The first two vertices create a line, and every new vertex flowing in the graphics pipeline
        /// (starting from the third, then) well extend the initial line, making a curve composed of
        /// several segments.
        ///
        /// > This kind of primitive mode allows the usage of _primitive restart_.
        LineStrip,
        /// A triangle, defined by three points.
        Triangle,
        /// A triangle fan, defined by at least three points and zero or many other ones.
        ///
        /// Such a mode is easy to picture: a cooling fan is a circular shape, with blades.
        /// [`Mode::TriangleFan`] is kind of the same. The first vertex is at the center of the fan, then
        /// the second vertex creates the first edge of the first triangle. Every time you add a new
        /// vertex, a triangle is created by taking the first (center) vertex, the very previous vertex
        /// and the current vertex. By specifying vertices around the center, you actually create a
        /// fan-like shape.
        ///
        /// > This kind of primitive mode allows the usage of _primitive restart_.
        TriangleFan,
        /// A triangle strip, defined by at least three points and zero or many other ones.
        ///
        /// This mode is a bit different from [`Mode::TriangleFan`]. The first two vertices define the
        /// first edge of the first triangle. Then, for each new vertex, a new triangle is created by
        /// taking the very previous vertex and the last to very previous vertex. What it means is that
        /// every time a triangle is created, the next vertex will share the edge that was created to
        /// spawn the previous triangle.
        ///
        /// This mode is useful to create long ribbons / strips of triangles.
        ///
        /// > This kind of primitive mode allows the usage of _primitive restart_.
        TriangleStrip,
        /// A general purpose primitive with _n_ vertices, for use in tessellation shaders.
        /// For example, `Mode::Patch(3)` represents triangle patches, so every three vertices in the
        /// buffer form a patch.
        ///
        /// If you want to employ tessellation shaders, this is the only primitive mode you can use.
        Patch(usize),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Mode {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Mode {
        #[inline]
        fn clone(&self) -> Mode {
            {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Mode {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Mode::Point,) => {
                    let mut debug_trait_builder = f.debug_tuple("Point");
                    debug_trait_builder.finish()
                }
                (&Mode::Line,) => {
                    let mut debug_trait_builder = f.debug_tuple("Line");
                    debug_trait_builder.finish()
                }
                (&Mode::LineStrip,) => {
                    let mut debug_trait_builder = f.debug_tuple("LineStrip");
                    debug_trait_builder.finish()
                }
                (&Mode::Triangle,) => {
                    let mut debug_trait_builder = f.debug_tuple("Triangle");
                    debug_trait_builder.finish()
                }
                (&Mode::TriangleFan,) => {
                    let mut debug_trait_builder = f.debug_tuple("TriangleFan");
                    debug_trait_builder.finish()
                }
                (&Mode::TriangleStrip,) => {
                    let mut debug_trait_builder = f.debug_tuple("TriangleStrip");
                    debug_trait_builder.finish()
                }
                (&Mode::Patch(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Patch");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl fmt::Display for Mode {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                Mode::Point => f.write_str("point"),
                Mode::Line => f.write_str("line"),
                Mode::LineStrip => f.write_str("line strip"),
                Mode::Triangle => f.write_str("triangle"),
                Mode::TriangleStrip => f.write_str("triangle strip"),
                Mode::TriangleFan => f.write_str("triangle fan"),
                Mode::Patch(ref n) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["patch (", ")"],
                    &match (&n,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
            }
        }
    }
    /// Error that can occur while trying to map GPU tessellations to host code.
    #[non_exhaustive]
    pub enum TessMapError {
        /// The CPU mapping failed due to buffer errors.
        BufferMapError(BufferError),
        /// Vertex target type is not the same as the one stored in the buffer.
        VertexTypeMismatch(VertexDesc, VertexDesc),
        /// Index target type is not the same as the one stored in the buffer.
        IndexTypeMismatch(TessIndexType, TessIndexType),
        /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
        /// have any vertex attribute.
        ForbiddenAttributelessMapping,
        /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
        /// a single slice.
        ForbiddenDeinterleavedMapping,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for TessMapError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&TessMapError::BufferMapError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("BufferMapError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TessMapError::VertexTypeMismatch(ref __self_0, ref __self_1),) => {
                    let mut debug_trait_builder = f.debug_tuple("VertexTypeMismatch");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&TessMapError::IndexTypeMismatch(ref __self_0, ref __self_1),) => {
                    let mut debug_trait_builder = f.debug_tuple("IndexTypeMismatch");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    let _ = debug_trait_builder.field(&&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&TessMapError::ForbiddenAttributelessMapping,) => {
                    let mut debug_trait_builder = f.debug_tuple("ForbiddenAttributelessMapping");
                    debug_trait_builder.finish()
                }
                (&TessMapError::ForbiddenDeinterleavedMapping,) => {
                    let mut debug_trait_builder = f.debug_tuple("ForbiddenDeinterleavedMapping");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for TessMapError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for TessMapError {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<BufferError>;
                let _: ::core::cmp::AssertParamIsEq<VertexDesc>;
                let _: ::core::cmp::AssertParamIsEq<VertexDesc>;
                let _: ::core::cmp::AssertParamIsEq<TessIndexType>;
                let _: ::core::cmp::AssertParamIsEq<TessIndexType>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for TessMapError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for TessMapError {
        #[inline]
        fn eq(&self, other: &TessMapError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &TessMapError::BufferMapError(ref __self_0),
                            &TessMapError::BufferMapError(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &TessMapError::VertexTypeMismatch(ref __self_0, ref __self_1),
                            &TessMapError::VertexTypeMismatch(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        (
                            &TessMapError::IndexTypeMismatch(ref __self_0, ref __self_1),
                            &TessMapError::IndexTypeMismatch(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &TessMapError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &TessMapError::BufferMapError(ref __self_0),
                            &TessMapError::BufferMapError(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &TessMapError::VertexTypeMismatch(ref __self_0, ref __self_1),
                            &TessMapError::VertexTypeMismatch(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        (
                            &TessMapError::IndexTypeMismatch(ref __self_0, ref __self_1),
                            &TessMapError::IndexTypeMismatch(ref __arg_1_0, ref __arg_1_1),
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl TessMapError {
        /// The CPU mapping failed due to buffer errors.
        pub fn buffer_map_error(e: BufferError) -> Self {
            TessMapError::BufferMapError(e)
        }
        /// Vertex target type is not the same as the one stored in the buffer.
        pub fn vertex_type_mismatch(a: VertexDesc, b: VertexDesc) -> Self {
            TessMapError::VertexTypeMismatch(a, b)
        }
        /// Index target type is not the same as the one stored in the buffer.
        pub fn index_type_mismatch(a: TessIndexType, b: TessIndexType) -> Self {
            TessMapError::IndexTypeMismatch(a, b)
        }
        /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
        /// have any vertex attribute.
        pub fn forbidden_attributeless_mapping() -> Self {
            TessMapError::ForbiddenAttributelessMapping
        }
        /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
        /// a single slice.
        pub fn forbidden_deinterleaved_mapping() -> Self {
            TessMapError::ForbiddenDeinterleavedMapping
        }
    }
    impl fmt::Display for TessMapError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                TessMapError::BufferMapError(ref e) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["cannot map tessellation buffer: "],
                    &match (&e,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                TessMapError::VertexTypeMismatch(ref a, ref b) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[
                            "cannot map tessellation: vertex type mismatch between ",
                            " and ",
                        ],
                        &match (&a, &b) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Debug::fmt),
                            ],
                        },
                    ))
                }
                TessMapError::IndexTypeMismatch(ref a, ref b) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[
                            "cannot map tessellation: index type mismatch between ",
                            " and ",
                        ],
                        &match (&a, &b) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Debug::fmt),
                            ],
                        },
                    ))
                }
                TessMapError::ForbiddenAttributelessMapping => {
                    f.write_str("cannot map an attributeless buffer")
                }
                TessMapError::ForbiddenDeinterleavedMapping => {
                    f.write_str("cannot map a deinterleaved buffer as interleaved")
                }
            }
        }
    }
    impl From<BufferError> for TessMapError {
        fn from(e: BufferError) -> Self {
            TessMapError::buffer_map_error(e)
        }
    }
    impl error::Error for TessMapError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                TessMapError::BufferMapError(e) => Some(e),
                _ => None,
            }
        }
    }
    /// Possible errors that might occur when dealing with [`Tess`].
    #[non_exhaustive]
    pub enum TessError {
        /// Cannot create a tessellation.
        CannotCreate(String),
        /// Error related to attributeless tessellation and/or render.
        AttributelessError(String),
        /// Length incoherency in vertex, index or instance buffers.
        LengthIncoherency(usize),
        /// Internal error ocurring with a buffer.
        InternalBufferError(BufferError),
        /// Forbidden primitive mode by hardware.
        ForbiddenPrimitiveMode(Mode),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for TessError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&TessError::CannotCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CannotCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TessError::AttributelessError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("AttributelessError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TessError::LengthIncoherency(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("LengthIncoherency");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TessError::InternalBufferError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("InternalBufferError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TessError::ForbiddenPrimitiveMode(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("ForbiddenPrimitiveMode");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl TessError {
        /// Cannot create a tessellation.
        pub fn cannot_create(e: impl Into<String>) -> Self {
            TessError::CannotCreate(e.into())
        }
        /// Error related to attributeless tessellation and/or render.
        pub fn attributeless_error(e: impl Into<String>) -> Self {
            TessError::AttributelessError(e.into())
        }
        /// Length incoherency in vertex, index or instance buffers.
        pub fn length_incoherency(len: usize) -> Self {
            TessError::LengthIncoherency(len)
        }
        /// Internal error ocurring with a buffer.
        pub fn internal_buffer_error(e: BufferError) -> Self {
            TessError::InternalBufferError(e)
        }
        /// Forbidden primitive mode by hardware.
        pub fn forbidden_primitive_mode(mode: Mode) -> Self {
            TessError::ForbiddenPrimitiveMode(mode)
        }
    }
    impl fmt::Display for TessError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                TessError::CannotCreate(ref s) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Creation error: "],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                TessError::AttributelessError(ref s) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Attributeless error: "],
                        &match (&s,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                TessError::LengthIncoherency(ref s) => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Incoherent size for internal buffers: "],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                )),
                TessError::InternalBufferError(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["internal buffer error: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                TessError::ForbiddenPrimitiveMode(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["forbidden primitive mode: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
            }
        }
    }
    impl From<BufferError> for TessError {
        fn from(e: BufferError) -> Self {
            TessError::internal_buffer_error(e)
        }
    }
    impl error::Error for TessError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                TessError::InternalBufferError(e) => Some(e),
                _ => None,
            }
        }
    }
    /// Possible tessellation index types.
    pub enum TessIndexType {
        /// 8-bit unsigned integer.
        U8,
        /// 16-bit unsigned integer.
        U16,
        /// 32-bit unsigned integer.
        U32,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for TessIndexType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for TessIndexType {
        #[inline]
        fn clone(&self) -> TessIndexType {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for TessIndexType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&TessIndexType::U8,) => {
                    let mut debug_trait_builder = f.debug_tuple("U8");
                    debug_trait_builder.finish()
                }
                (&TessIndexType::U16,) => {
                    let mut debug_trait_builder = f.debug_tuple("U16");
                    debug_trait_builder.finish()
                }
                (&TessIndexType::U32,) => {
                    let mut debug_trait_builder = f.debug_tuple("U32");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for TessIndexType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for TessIndexType {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl ::core::marker::StructuralPartialEq for TessIndexType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for TessIndexType {
        #[inline]
        fn eq(&self, other: &TessIndexType) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl TessIndexType {
        /// Get the number of bytes that are needed to represent a type described by the variant.
        pub fn bytes(self) -> usize {
            match self {
                TessIndexType::U8 => 1,
                TessIndexType::U16 => 2,
                TessIndexType::U32 => 4,
            }
        }
    }
    /// Class of tessellation indices.
    ///
    /// Values which types implement this trait are allowed to be used to index tessellation in *indexed
    /// draw commands*.
    ///
    /// You shouldn’t have to worry too much about that trait. Have a look at the current implementors
    /// for an exhaustive list of types you can use.
    ///
    /// > Implementing this trait is `unsafe`.
    pub unsafe trait TessIndex: Copy {
        /// Type of the underlying index.
        ///
        /// You are limited in which types you can use as indexes. Feel free to have a look at the
        /// documentation of the [`TessIndexType`] trait for further information.
        ///
        /// `None` means that you disable indexing.
        const INDEX_TYPE: Option<TessIndexType>;
        /// Get and convert the index to [`u32`], if possible.
        fn try_into_u32(self) -> Option<u32>;
    }
    unsafe impl TessIndex for () {
        const INDEX_TYPE: Option<TessIndexType> = None;
        fn try_into_u32(self) -> Option<u32> {
            None
        }
    }
    /// Boop.
    unsafe impl TessIndex for u8 {
        const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U8);
        fn try_into_u32(self) -> Option<u32> {
            Some(self.into())
        }
    }
    /// Boop.
    unsafe impl TessIndex for u16 {
        const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U16);
        fn try_into_u32(self) -> Option<u32> {
            Some(self.into())
        }
    }
    /// Wuuuuuuha.
    unsafe impl TessIndex for u32 {
        const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U32);
        fn try_into_u32(self) -> Option<u32> {
            Some(self)
        }
    }
    /// Interleaved memory marker.
    pub enum Interleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Interleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Interleaved {
        #[inline]
        fn clone(&self) -> Interleaved {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Interleaved {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    impl ::core::marker::StructuralEq for Interleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Interleaved {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Interleaved {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    impl ::core::marker::StructuralPartialEq for Interleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Interleaved {
        #[inline]
        fn eq(&self, other: &Interleaved) -> bool {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    /// Deinterleaved memory marker.
    pub enum Deinterleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Deinterleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Deinterleaved {
        #[inline]
        fn clone(&self) -> Deinterleaved {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Deinterleaved {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    impl ::core::marker::StructuralEq for Deinterleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Deinterleaved {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Deinterleaved {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    impl ::core::marker::StructuralPartialEq for Deinterleaved {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Deinterleaved {
        #[inline]
        fn eq(&self, other: &Deinterleaved) -> bool {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    /// Vertex input data of a [`TessBuilder`].
    pub trait TessVertexData<S>: Vertex
    where
        S: ?Sized,
    {
        /// Vertex storage type.
        type Data;
        /// Coherent length of the vertices.
        ///
        /// Vertices length can be incohent for some implementations of [`TessVertexData::Data`],
        /// especially with deinterleaved memory.
        fn coherent_len(data: &Self::Data) -> Result<usize, TessError>;
    }
    impl<V> TessVertexData<Interleaved> for V
    where
        V: Vertex,
    {
        type Data = Vec<V>;
        fn coherent_len(data: &Self::Data) -> Result<usize, TessError> {
            Ok(data.len())
        }
    }
    impl<V> TessVertexData<Deinterleaved> for V
    where
        V: Vertex,
    {
        type Data = Vec<DeinterleavedData>;
        fn coherent_len(data: &Self::Data) -> Result<usize, TessError> {
            if data.is_empty() {
                Ok(0)
            } else {
                let len = data[0].len;
                if data[1..].iter().any(|a| a.len != len) {
                    Err(TessError::length_incoherency(len))
                } else {
                    Ok(len)
                }
            }
        }
    }
    /// Deinterleaved data.
    pub struct DeinterleavedData {
        raw: Vec<u8>,
        len: usize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for DeinterleavedData {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                DeinterleavedData {
                    raw: ref __self_0_0,
                    len: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("DeinterleavedData");
                    let _ = debug_trait_builder.field("raw", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("len", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for DeinterleavedData {
        #[inline]
        fn clone(&self) -> DeinterleavedData {
            match *self {
                DeinterleavedData {
                    raw: ref __self_0_0,
                    len: ref __self_0_1,
                } => DeinterleavedData {
                    raw: ::core::clone::Clone::clone(&(*__self_0_0)),
                    len: ::core::clone::Clone::clone(&(*__self_0_1)),
                },
            }
        }
    }
    impl DeinterleavedData {
        fn new() -> Self {
            DeinterleavedData {
                raw: Vec::new(),
                len: 0,
            }
        }
        /// Turn the [`DeinterleavedData`] into its raw representation.
        pub fn into_vec(self) -> Vec<u8> {
            self.raw
        }
    }
    /// [`Tess`] builder object.
    ///
    /// This type allows to create [`Tess`] via a _builder pattern_. You have several flavors of
    /// possible _vertex storage_ situations, as well as _data encoding_, described below.
    ///
    /// # Vertex storage
    ///
    /// ## Interleaved
    ///
    /// You can pass around interleaved vertices and indices. Those are encoded in `Vec<T>`. You
    /// typically want to use this when you already have the vertices and/or indices allocated somewhere,
    /// as the interface will use the input vector as a source of truth for lengths.
    ///
    /// ## Deinterleaved
    ///
    /// This is the same as interleaved data in terms of interface, but the `T` type is interpreted
    /// a bit differently. Here, the encoding is `(Vec<Field0>, Vec<Field1>, …)`, where `Field0`,
    /// `Field1` etc. are all the ordered fieds in `T`.
    ///
    /// That representation allows field-based operation later on [`Tess`], while it would be
    /// impossible with the interleaved version (you would need to get all the fields at once, since
    /// you would work on`T` directly and each of its fields).
    ///
    /// # Data encoding
    ///
    /// - Vectors: you can pass vectors as input data for both vertices and indices. Those will be
    ///   interpreted differently based on the vertex storage you chose for vertices, and the normal
    ///   way for indices.
    /// - Buffers: you can pass [`Buffer`] objects, too. Those are more flexible than vectors as you can
    ///   use all of the [`Buffer`] API before sending them to the builder.
    /// - Disabled: disabling means that no data will be passed to the GPU. You can disable independently
    ///   vertex data and/or index data.
    ///
    /// # Parametricity
    ///
    /// - `B` is the backend type
    /// - `V` is the vertex type.
    /// - `S` is the storage type.
    ///
    /// [`Buffer`]: crate::buffer::Buffer
    pub struct TessBuilder<'a, B, V, I = (), W = (), S = Interleaved>
    where
        B: ?Sized,
        V: TessVertexData<S>,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        backend: &'a mut B,
        vertex_data: Option<V::Data>,
        index_data: Vec<I>,
        instance_data: Option<W::Data>,
        mode: Mode,
        vert_nb: usize,
        inst_nb: usize,
        restart_index: Option<I>,
        _phantom: PhantomData<&'a mut ()>,
    }
    impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
    where
        B: ?Sized,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Set the [`Mode`] to connect vertices.
        ///
        /// Calling that function twice replace the previously set value.
        pub fn set_mode(mut self, mode: Mode) -> Self {
            self.mode = mode;
            self
        }
        /// Set the default number of vertices to render.
        ///
        /// Calling that function twice replace the previously set value.
        pub fn set_vertex_nb(mut self, vert_nb: usize) -> Self {
            self.vert_nb = vert_nb;
            self
        }
        /// Set the default number of instances to render.
        ///
        /// Calling that function twice replace the previously set value.
        pub fn set_instance_nb(mut self, inst_nb: usize) -> Self {
            self.inst_nb = inst_nb;
            self
        }
        /// Set the primitive restart index.
        ///
        /// Calling that function twice replace the previously set value.
        pub fn set_primitive_restart_index(mut self, restart_index: I) -> Self {
            self.restart_index = Some(restart_index);
            self
        }
    }
    impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
    where
        B: ?Sized,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Create a new default [`TessBuilder`].
        ///
        /// # Notes
        ///
        /// Feel free to use the [`GraphicsContext::new_tess`] method for a simpler method.
        ///
        /// [`GraphicsContext::new_tess`]: crate::context::GraphicsContext::new_tess
        pub fn new<C>(ctx: &'a mut C) -> Self
        where
            C: GraphicsContext<Backend = B>,
        {
            TessBuilder {
                backend: ctx.backend(),
                vertex_data: None,
                index_data: Vec::new(),
                instance_data: None,
                mode: Mode::Point,
                vert_nb: 0,
                inst_nb: 0,
                restart_index: None,
                _phantom: PhantomData,
            }
        }
    }
    impl<'a, B, V, W, S> TessBuilder<'a, B, V, (), W, S>
    where
        B: ?Sized,
        V: TessVertexData<S>,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Add indices to be bundled in the [`Tess`].
        ///
        /// Every time you call that function, the set of indices is replaced by the one you provided.
        /// The type of expected indices is ruled by the `II` type variable you chose.
        pub fn set_indices<I, X>(self, indices: X) -> TessBuilder<'a, B, V, I, W, S>
        where
            X: Into<Vec<I>>,
        {
            TessBuilder {
                backend: self.backend,
                vertex_data: self.vertex_data,
                index_data: indices.into(),
                instance_data: self.instance_data,
                mode: self.mode,
                vert_nb: self.vert_nb,
                inst_nb: self.inst_nb,
                restart_index: None,
                _phantom: PhantomData,
            }
        }
    }
    impl<'a, B, I, W> TessBuilder<'a, B, (), I, W, Interleaved>
    where
        B: ?Sized,
        I: TessIndex,
        W: TessVertexData<Interleaved>,
    {
        /// Add vertices to be bundled in the [`Tess`].
        ///
        /// Every time you call that function, the set of vertices is replaced by the one you provided.
        pub fn set_vertices<V, X>(self, vertices: X) -> TessBuilder<'a, B, V, I, W, Interleaved>
        where
            X: Into<Vec<V>>,
            V: TessVertexData<Interleaved, Data = Vec<V>>,
        {
            TessBuilder {
                backend: self.backend,
                vertex_data: Some(vertices.into()),
                index_data: self.index_data,
                instance_data: self.instance_data,
                mode: self.mode,
                vert_nb: self.vert_nb,
                inst_nb: self.inst_nb,
                restart_index: self.restart_index,
                _phantom: PhantomData,
            }
        }
    }
    impl<'a, B, I, V> TessBuilder<'a, B, V, I, (), Interleaved>
    where
        B: ?Sized,
        I: TessIndex,
        V: TessVertexData<Interleaved>,
    {
        /// Add instances to be bundled in the [`Tess`].
        ///
        /// Every time you call that function, the set of instances is replaced by the one you provided.
        pub fn set_instances<W, X>(self, instances: X) -> TessBuilder<'a, B, V, I, W, Interleaved>
        where
            X: Into<Vec<W>>,
            W: TessVertexData<Interleaved, Data = Vec<W>>,
        {
            TessBuilder {
                backend: self.backend,
                vertex_data: self.vertex_data,
                index_data: self.index_data,
                instance_data: Some(instances.into()),
                mode: self.mode,
                vert_nb: self.vert_nb,
                inst_nb: self.inst_nb,
                restart_index: self.restart_index,
                _phantom: PhantomData,
            }
        }
    }
    impl<'a, B, V, I, W> TessBuilder<'a, B, V, I, W, Deinterleaved>
    where
        B: ?Sized,
        V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
        I: TessIndex,
        W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
    {
        /// Add vertices to be bundled in the [`Tess`].
        ///
        /// Every time you call that function, the set of vertices is replaced by the one you provided.
        pub fn set_attributes<A, X>(mut self, attributes: X) -> Self
        where
            X: Into<Vec<A>>,
            V: Deinterleave<A>,
        {
            let build_raw = |deinterleaved: &mut Vec<DeinterleavedData>| {
                let boxed_slice = attributes.into().into_boxed_slice();
                let len = boxed_slice.len();
                let len_bytes = len * std::mem::size_of::<A>();
                let ptr = Box::into_raw(boxed_slice);
                let raw = unsafe { Vec::from_raw_parts(ptr as _, len_bytes, len_bytes) };
                deinterleaved[V::RANK] = DeinterleavedData { raw, len };
            };
            match self.vertex_data {
                Some(ref mut deinterleaved) => {
                    build_raw(deinterleaved);
                }
                None => {
                    let mut deinterleaved =
                        ::alloc::vec::from_elem(DeinterleavedData::new(), V::ATTR_COUNT);
                    build_raw(&mut deinterleaved);
                    self.vertex_data = Some(deinterleaved);
                }
            }
            self
        }
        /// Add instances to be bundled in the [`Tess`].
        ///
        /// Every time you call that function, the set of instances is replaced by the one you provided.
        pub fn set_instance_attributes<A, X>(mut self, attributes: X) -> Self
        where
            X: Into<Vec<A>>,
            W: Deinterleave<A>,
        {
            let build_raw = |deinterleaved: &mut Vec<DeinterleavedData>| {
                let boxed_slice = attributes.into().into_boxed_slice();
                let len = boxed_slice.len();
                let len_bytes = len * std::mem::size_of::<A>();
                let ptr = Box::into_raw(boxed_slice);
                let raw = unsafe { Vec::from_raw_parts(ptr as _, len_bytes, len_bytes) };
                deinterleaved[W::RANK] = DeinterleavedData { raw, len };
            };
            match self.instance_data {
                None => {
                    let mut deinterleaved =
                        ::alloc::vec::from_elem(DeinterleavedData::new(), W::ATTR_COUNT);
                    build_raw(&mut deinterleaved);
                    self.instance_data = Some(deinterleaved);
                }
                Some(ref mut deinterleaved) => {
                    build_raw(deinterleaved);
                }
            }
            self
        }
    }
    impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
    {
        /// Build a [`Tess`] if the [`TessBuilder`] has enough data and is in a valid state. What is
        /// needed is backend-dependent but most of the time, you will want to:
        ///
        /// - Set a [`Mode`].
        /// - Give vertex data and optionally indices, or give none of them (attributeless objects).
        /// - If you provide vertex data by submitting several sets with [`TessBuilder::set_attributes`]
        ///   and/or [`TessBuilder::set_instances`], do not forget that you must submit sets with the
        ///   same size. Otherwise, the GPU will not know what values use for missing attributes in
        ///   vertices.
        pub fn build(self) -> Result<Tess<B, V, I, W, S>, TessError> {
            let vert_nb = self.guess_vertex_len()?;
            let inst_nb = self.guess_instance_len()?;
            unsafe {
                self.backend
                    .build(
                        (self.vertex_data, vert_nb),
                        (self.instance_data, inst_nb),
                        self.index_data,
                        self.restart_index,
                        self.mode,
                    )
                    .map(|repr| Tess {
                        repr,
                        _phantom: PhantomData,
                    })
            }
        }
        fn guess_vertex_len(&self) -> Result<usize, TessError> {
            if self.vert_nb == 0 {
                if self.index_data.is_empty() {
                    match self.vertex_data {
                        Some(ref data) => V::coherent_len(data),
                        None => Ok(0),
                    }
                } else {
                    Ok(self.index_data.len())
                }
            } else {
                if self.index_data.is_empty() {
                    match self.vertex_data {
                        Some(ref data) => {
                            let coherent_len = V::coherent_len(data)?;
                            if self.vert_nb <= coherent_len {
                                Ok(self.vert_nb)
                            } else {
                                Err(TessError::length_incoherency(self.vert_nb))
                            }
                        }
                        None => Ok(self.vert_nb),
                    }
                } else {
                    if self.vert_nb <= self.index_data.len() {
                        Ok(self.vert_nb)
                    } else {
                        Err(TessError::length_incoherency(self.vert_nb))
                    }
                }
            }
        }
        fn guess_instance_len(&self) -> Result<usize, TessError> {
            if self.inst_nb == 0 {
                match self.instance_data {
                    Some(ref data) => W::coherent_len(data),
                    None => Ok(0),
                }
            } else {
                let coherent_len = self
                    .instance_data
                    .as_ref()
                    .ok_or_else(|| TessError::attributeless_error("missing number of instances"))
                    .and_then(W::coherent_len)?;
                if self.inst_nb <= coherent_len {
                    Ok(self.inst_nb)
                } else {
                    Err(TessError::length_incoherency(self.inst_nb))
                }
            }
        }
    }
    /// A GPU vertex set.
    ///
    /// Vertex set are the only way to represent space data. The dimension you choose is up to you, but
    /// people will typically want to represent objects in 2D or 3D. A _vertex_ is a point in such
    /// space and it carries _properties_ — called _“vertex attributes_”. Those attributes are
    /// completely free to use. They must, however, be compatible with the [`Semantics`] and [`Vertex`]
    /// traits.
    ///
    /// [`Tess`] are built out of [`TessBuilder`] and can be _sliced_ to edit their content in-line —
    /// by mapping the GPU memory region and access data via slices.
    ///
    /// [`Semantics`]: crate::vertex::Semantics
    /// [`TessGate`]: crate::tess_gate::TessGate
    pub struct Tess<B, V, I = (), W = (), S = Interleaved>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        pub(crate) repr: B::TessRepr,
        _phantom: PhantomData<*const S>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
        > ::core::fmt::Debug for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::TessRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Tess {
                    repr: ref __self_0_0,
                    _phantom: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Tess");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("_phantom", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S> Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Get the number of vertices.
        pub fn vert_nb(&self) -> usize {
            unsafe { B::tess_vertices_nb(&self.repr) }
        }
        /// Get the number of indices.
        pub fn inst_nb(&self) -> usize {
            unsafe { B::tess_instances_nb(&self.repr) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _index storage_.
        pub fn indices(&mut self) -> Result<Indices<B, V, I, W, S>, TessMapError>
        where
            B: IndexSliceBackend<V, I, W, S>,
        {
            unsafe { B::indices(&mut self.repr).map(|repr| Indices { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _index storage_.
        pub fn indices_mut(&mut self) -> Result<IndicesMut<B, V, I, W, S>, TessMapError>
        where
            B: IndexSliceBackend<V, I, W, S>,
        {
            unsafe { B::indices_mut(&mut self.repr).map(|repr| IndicesMut { repr }) }
        }
    }
    impl<B, V, I, W> Tess<B, V, I, W, Interleaved>
    where
        B: ?Sized + TessBackend<V, I, W, Interleaved>,
        V: TessVertexData<Interleaved>,
        I: TessIndex,
        W: TessVertexData<Interleaved>,
    {
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _vertex storage_.
        pub fn vertices(&mut self) -> Result<Vertices<B, V, I, W, Interleaved, V>, TessMapError>
        where
            B: VertexSliceBackend<V, I, W, Interleaved, V>,
        {
            unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _vertex storage_.
        pub fn vertices_mut(
            &mut self,
        ) -> Result<VerticesMut<B, V, I, W, Interleaved, V>, TessMapError>
        where
            B: VertexSliceBackend<V, I, W, Interleaved, V>,
        {
            unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _instance storage_.
        pub fn instances(&mut self) -> Result<Instances<B, V, I, W, Interleaved, V>, TessMapError>
        where
            B: InstanceSliceBackend<V, I, W, Interleaved, V>,
        {
            unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _instance storage_.
        pub fn instances_mut(
            &mut self,
        ) -> Result<InstancesMut<B, V, I, W, Interleaved, V>, TessMapError>
        where
            B: InstanceSliceBackend<V, I, W, Interleaved, V>,
        {
            unsafe { B::instances_mut(&mut self.repr).map(|repr| InstancesMut { repr }) }
        }
    }
    impl<B, V, I, W> Tess<B, V, I, W, Deinterleaved>
    where
        B: ?Sized + TessBackend<V, I, W, Deinterleaved>,
        V: TessVertexData<Deinterleaved>,
        I: TessIndex,
        W: TessVertexData<Deinterleaved>,
    {
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _vertex storage_.
        pub fn vertices<T>(
            &mut self,
        ) -> Result<Vertices<B, V, I, W, Deinterleaved, T>, TessMapError>
        where
            B: VertexSliceBackend<V, I, W, Deinterleaved, T>,
            V: Deinterleave<T>,
        {
            unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _vertex storage_.
        pub fn vertices_mut<T>(
            &mut self,
        ) -> Result<VerticesMut<B, V, I, W, Deinterleaved, T>, TessMapError>
        where
            B: VertexSliceBackend<V, I, W, Deinterleaved, T>,
            V: Deinterleave<T>,
        {
            unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _instance storage_.
        pub fn instances<T>(
            &mut self,
        ) -> Result<Instances<B, V, I, W, Deinterleaved, T>, TessMapError>
        where
            B: InstanceSliceBackend<V, I, W, Deinterleaved, T>,
            W: Deinterleave<T>,
        {
            unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
        }
        /// Slice the [`Tess`] in order to read its content via usual slices.
        ///
        /// This method gives access to the underlying _instance storage_.
        pub fn instances_mut<T>(
            &mut self,
        ) -> Result<InstancesMut<B, V, I, W, Deinterleaved, T>, TessMapError>
        where
            B: InstanceSliceBackend<V, I, W, Deinterleaved, T>,
            W: Deinterleave<T>,
        {
            unsafe { B::instances_mut(&mut self.repr).map(|repr| InstancesMut { repr }) }
        }
    }
    /// TODO
    pub struct Vertices<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::VertexSliceRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
            T: ::core::fmt::Debug,
        > ::core::fmt::Debug for Vertices<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::VertexSliceRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Vertices {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Vertices");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S, T> Deref for Vertices<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        type Target = [T];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    /// TODO
    pub struct VerticesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::VertexSliceMutRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
            T: ::core::fmt::Debug,
        > ::core::fmt::Debug for VerticesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::VertexSliceMutRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                VerticesMut {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("VerticesMut");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S, T> Deref for VerticesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        type Target = [T];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    impl<B, V, I, W, S, T> DerefMut for VerticesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.repr.deref_mut()
        }
    }
    /// TODO
    pub struct Indices<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::IndexSliceRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
        > ::core::fmt::Debug for Indices<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::IndexSliceRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Indices {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Indices");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S> Deref for Indices<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        type Target = [I];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    /// TODO
    pub struct IndicesMut<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::IndexSliceMutRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
        > ::core::fmt::Debug for IndicesMut<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::IndexSliceMutRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                IndicesMut {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("IndicesMut");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S> Deref for IndicesMut<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        type Target = [I];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    impl<B, V, I, W, S> DerefMut for IndicesMut<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.repr.deref_mut()
        }
    }
    /// TODO
    pub struct Instances<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::InstanceSliceRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
            T: ::core::fmt::Debug,
        > ::core::fmt::Debug for Instances<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::InstanceSliceRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Instances {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Instances");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S, T> Deref for Instances<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        type Target = [T];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    /// TODO
    pub struct InstancesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        repr: B::InstanceSliceMutRepr,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            B: ::core::fmt::Debug,
            V: ::core::fmt::Debug,
            I: ::core::fmt::Debug,
            W: ::core::fmt::Debug,
            S: ::core::fmt::Debug,
            T: ::core::fmt::Debug,
        > ::core::fmt::Debug for InstancesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
        B::InstanceSliceMutRepr: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                InstancesMut {
                    repr: ref __self_0_0,
                } => {
                    let mut debug_trait_builder = f.debug_struct("InstancesMut");
                    let _ = debug_trait_builder.field("repr", &&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<B, V, I, W, S, T> Deref for InstancesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        S: ?Sized,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
    {
        type Target = [T];
        fn deref(&self) -> &Self::Target {
            self.repr.deref()
        }
    }
    impl<B, V, I, W, S, T> DerefMut for InstancesMut<B, V, I, W, S, T>
    where
        B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.repr.deref_mut()
        }
    }
    /// Possible error that might occur while dealing with [`TessView`] objects.
    #[non_exhaustive]
    pub enum TessViewError {
        /// The view has incorrect size.
        ///
        /// data.
        IncorrectViewWindow {
            /// Capacity of data in the [`Tess`].
            capacity: usize,
            /// Requested start.
            start: usize,
            /// Requested number.
            nb: usize,
        },
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for TessViewError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&TessViewError::IncorrectViewWindow {
                    capacity: ref __self_0,
                    start: ref __self_1,
                    nb: ref __self_2,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("IncorrectViewWindow");
                    let _ = debug_trait_builder.field("capacity", &&(*__self_0));
                    let _ = debug_trait_builder.field("start", &&(*__self_1));
                    let _ = debug_trait_builder.field("nb", &&(*__self_2));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl fmt::Display for TessViewError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match self {
                TessViewError::IncorrectViewWindow {
                    capacity,
                    start,
                    nb,
                } => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[
                        "TessView incorrect window error: requested slice size ",
                        " starting at ",
                        ", but capacity is only ",
                    ],
                    &match (&nb, &start, &capacity) {
                        (arg0, arg1, arg2) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                        ],
                    },
                )),
            }
        }
    }
    impl error::Error for TessViewError {}
    /// A _view_ into a GPU tessellation.
    pub struct TessView<'a, B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Tessellation to render.
        pub(crate) tess: &'a Tess<B, V, I, W, S>,
        /// Start index (vertex) in the tessellation.
        pub(crate) start_index: usize,
        /// Number of vertices to pick from the tessellation.
        pub(crate) vert_nb: usize,
        /// Number of instances to render.
        pub(crate) inst_nb: usize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            'a,
            B: ::core::clone::Clone,
            V: ::core::clone::Clone,
            I: ::core::clone::Clone,
            W: ::core::clone::Clone,
            S: ::core::clone::Clone,
        > ::core::clone::Clone for TessView<'a, B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        #[inline]
        fn clone(&self) -> TessView<'a, B, V, I, W, S> {
            match *self {
                TessView {
                    tess: ref __self_0_0,
                    start_index: ref __self_0_1,
                    vert_nb: ref __self_0_2,
                    inst_nb: ref __self_0_3,
                } => TessView {
                    tess: ::core::clone::Clone::clone(&(*__self_0_0)),
                    start_index: ::core::clone::Clone::clone(&(*__self_0_1)),
                    vert_nb: ::core::clone::Clone::clone(&(*__self_0_2)),
                    inst_nb: ::core::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    impl<'a, B, V, I, W, S> TessView<'a, B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Create a view that is using the whole input [`Tess`].
        pub fn whole(tess: &'a Tess<B, V, I, W, S>) -> Self {
            TessView {
                tess,
                start_index: 0,
                vert_nb: tess.vert_nb(),
                inst_nb: tess.inst_nb(),
            }
        }
        /// Create a view that is using the whole input [`Tess`] with `inst_nb` instances.
        pub fn inst_whole(tess: &'a Tess<B, V, I, W, S>, inst_nb: usize) -> Self {
            TessView {
                tess,
                start_index: 0,
                vert_nb: tess.vert_nb(),
                inst_nb,
            }
        }
        /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
        /// of the vertices.
        pub fn sub(tess: &'a Tess<B, V, I, W, S>, vert_nb: usize) -> Result<Self, TessViewError> {
            let capacity = tess.vert_nb();
            if vert_nb > capacity {
                return Err(TessViewError::IncorrectViewWindow {
                    capacity,
                    start: 0,
                    nb: vert_nb,
                });
            }
            Ok(TessView {
                tess,
                start_index: 0,
                vert_nb,
                inst_nb: tess.inst_nb(),
            })
        }
        /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
        /// of the vertices, with `inst_nb` instances.
        pub fn inst_sub(
            tess: &'a Tess<B, V, I, W, S>,
            vert_nb: usize,
            inst_nb: usize,
        ) -> Result<Self, TessViewError> {
            let capacity = tess.vert_nb();
            if vert_nb > capacity {
                return Err(TessViewError::IncorrectViewWindow {
                    capacity,
                    start: 0,
                    nb: vert_nb,
                });
            }
            Ok(TessView {
                tess,
                start_index: 0,
                vert_nb,
                inst_nb,
            })
        }
        /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
        /// `nb` vertices.
        pub fn slice(
            tess: &'a Tess<B, V, I, W, S>,
            start: usize,
            nb: usize,
        ) -> Result<Self, TessViewError> {
            let capacity = tess.vert_nb();
            if start > capacity || nb + start > capacity {
                return Err(TessViewError::IncorrectViewWindow {
                    capacity,
                    start,
                    nb,
                });
            }
            Ok(TessView {
                tess,
                start_index: start,
                vert_nb: nb,
                inst_nb: tess.inst_nb(),
            })
        }
        /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
        /// `nb` vertices and `inst_nb` instances.
        pub fn inst_slice(
            tess: &'a Tess<B, V, I, W, S>,
            start: usize,
            nb: usize,
            inst_nb: usize,
        ) -> Result<Self, TessViewError> {
            let capacity = tess.vert_nb();
            if start > capacity || nb + start > capacity {
                return Err(TessViewError::IncorrectViewWindow {
                    capacity,
                    start,
                    nb,
                });
            }
            Ok(TessView {
                tess,
                start_index: start,
                vert_nb: nb,
                inst_nb,
            })
        }
    }
    impl<'a, B, V, I, W, S> From<&'a Tess<B, V, I, W, S>> for TessView<'a, B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn from(tess: &'a Tess<B, V, I, W, S>) -> Self {
            TessView::whole(tess)
        }
    }
    /// [`TessView`] helper trait.
    ///
    /// This trait helps to create [`TessView`] by allowing using the Rust range operators, such as
    ///
    /// - [`..`](https://doc.rust-lang.org/std/ops/struct.RangeFull.html); the full range operator.
    /// - [`a .. b`](https://doc.rust-lang.org/std/ops/struct.Range.html); the range operator.
    /// - [`a ..`](https://doc.rust-lang.org/std/ops/struct.RangeFrom.html); the range-from operator.
    /// - [`.. b`](https://doc.rust-lang.org/std/ops/struct.RangeTo.html); the range-to operator.
    /// - [`..= b`](https://doc.rust-lang.org/std/ops/struct.RangeToInclusive.html); the inclusive range-to operator.
    pub trait View<B, V, I, W, S, Idx>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        /// Slice a tessellation object and yields a [`TessView`] according to the index range.
        fn view(&self, idx: Idx) -> Result<TessView<B, V, I, W, S>, TessViewError>;
        /// Slice a tesselation object and yields a [`TessView`] according to the index range with as
        /// many instances as specified.
        fn inst_view(
            &self,
            idx: Idx,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError>;
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, RangeFull> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(&self, _: RangeFull) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            Ok(TessView::whole(self))
        }
        fn inst_view(
            &self,
            _: RangeFull,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            Ok(TessView::inst_whole(self, inst_nb))
        }
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, RangeTo<usize>> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(&self, to: RangeTo<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::sub(self, to.end)
        }
        fn inst_view(
            &self,
            to: RangeTo<usize>,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::inst_sub(self, to.end, inst_nb)
        }
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, RangeFrom<usize>> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(&self, from: RangeFrom<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::slice(self, from.start, self.vert_nb() - from.start)
        }
        fn inst_view(
            &self,
            from: RangeFrom<usize>,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::inst_slice(self, from.start, self.vert_nb() - from.start, inst_nb)
        }
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, Range<usize>> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(&self, range: Range<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::slice(self, range.start, range.end - range.start)
        }
        fn inst_view(
            &self,
            range: Range<usize>,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::inst_slice(self, range.start, range.end - range.start, inst_nb)
        }
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, RangeInclusive<usize>> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(
            &self,
            range: RangeInclusive<usize>,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            let start = *range.start();
            let end = *range.end();
            TessView::slice(self, start, end - start + 1)
        }
        fn inst_view(
            &self,
            range: RangeInclusive<usize>,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            let start = *range.start();
            let end = *range.end();
            TessView::inst_slice(self, start, end - start + 1, inst_nb)
        }
    }
    impl<B, V, I, W, S> View<B, V, I, W, S, RangeToInclusive<usize>> for Tess<B, V, I, W, S>
    where
        B: ?Sized + TessBackend<V, I, W, S>,
        V: TessVertexData<S>,
        I: TessIndex,
        W: TessVertexData<S>,
        S: ?Sized,
    {
        fn view(
            &self,
            to: RangeToInclusive<usize>,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::sub(self, to.end + 1)
        }
        fn inst_view(
            &self,
            to: RangeToInclusive<usize>,
            inst_nb: usize,
        ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
            TessView::inst_sub(self, to.end + 1, inst_nb)
        }
    }
}
pub mod tess_gate {
    //! Tessellation gates.
    //!
    //! A tessellation gate is a _pipeline node_ that allows to share [`Tess`] for deeper nodes.
    //!
    //! [`Tess`]: crate::tess::Tess
    use crate::backend::tess_gate::TessGate as TessGateBackend;
    use crate::tess::{TessIndex, TessVertexData, TessView};
    /// Tessellation gate.
    pub struct TessGate<'a, B>
    where
        B: ?Sized,
    {
        pub(crate) backend: &'a mut B,
    }
    impl<'a, B> TessGate<'a, B>
    where
        B: ?Sized,
    {
        /// Enter the [`TessGate`] by sharing a [`TessView`].
        pub fn render<'b, E, T, V, I, W, S>(&'b mut self, tess_view: T) -> Result<(), E>
        where
            B: TessGateBackend<V, I, W, S>,
            T: Into<TessView<'b, B, V, I, W, S>>,
            V: TessVertexData<S> + 'b,
            I: TessIndex + 'b,
            W: TessVertexData<S> + 'b,
            S: ?Sized + 'b,
        {
            let tess_view = tess_view.into();
            unsafe {
                self.backend.render(
                    &tess_view.tess.repr,
                    tess_view.start_index,
                    tess_view.vert_nb,
                    tess_view.inst_nb,
                );
                Ok(())
            }
        }
    }
}
pub mod texture {
    //! GPU textures.
    //!
    //! A GPU texture is an object that can be perceived as an array on the GPU. It holds several items
    //! and is a dimensional object. Textures are often associated with _images_, even though their
    //! usage in the graphics world can be much larger than that.
    //!
    //! Textures — [`Texture`] come in several flavors and the differences lie in:
    //!
    //! - The dimensionality: textures can be 1D, 2D, 3D, layered, cube maps, etc.
    //! - The encoding: the _items_ inside textures are called _pixels_ or _texels_. Those can be
    //!   encoded in several ways.
    //! - The usage: some textures will be used as images, others will be used to pass arbitrary data
    //!   around in shaders, etc.
    //!
    //! Whatever the flavor, textures have few even not no use outside of shaders. When a shader wants
    //! to use a texture, it can do it directly, by accessing each pixel by their position inside the
    //! texture (using a normalized coordinates system) or via the use of a [`Sampler`]. A [`Sampler`],
    //! as the name implies, is an object that tells the GPU how fetching (sampling) a texture should
    //! occur. Several options there too:
    //!
    //! - The GPU can fetch a pixel without sampler. It means that you have to pass the exact
    //!   coordinates of the pixel you want to access. This is useful when you store arbitrary
    //!   information, like UUID, velocities, etc.
    //! - The GPU can fetch a pixel with a floating-point coordinates system. How that system works
    //!   depends on the settings of [`Sampler`] you choose. For instance, you can always fetch the
    //!   _nearest_ pixel to where you sample, or you can ask the GPU to perform a linear
    //!   interpolation between all neighboring pixels, etc. [`Sampler`] allow way more than that, so
    //!   feel free to read their documentation.
    //!
    //! # Creating a texture
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;
    use crate::backend::texture::Texture as TextureBackend;
    use crate::context::GraphicsContext;
    use crate::depth_test::DepthComparison;
    use crate::pixel::{Pixel, PixelFormat};
    /// How to wrap texture coordinates while sampling textures?
    pub enum Wrap {
        /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
        /// every components.
        ClampToEdge,
        /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
        ///
        /// ```ignore
        /// // given the frac function returning the fractional part of a floating number:
        /// coord_ith = frac(coord_ith); // always between [0;1]
        /// ```
        Repeat,
        /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
        MirroredRepeat,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Wrap {
        #[inline]
        fn clone(&self) -> Wrap {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Wrap {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Wrap {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Wrap::ClampToEdge,) => {
                    let mut debug_trait_builder = f.debug_tuple("ClampToEdge");
                    debug_trait_builder.finish()
                }
                (&Wrap::Repeat,) => {
                    let mut debug_trait_builder = f.debug_tuple("Repeat");
                    debug_trait_builder.finish()
                }
                (&Wrap::MirroredRepeat,) => {
                    let mut debug_trait_builder = f.debug_tuple("MirroredRepeat");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Minification filter.
    pub enum MinFilter {
        /// Nearest interpolation.
        Nearest,
        /// Linear interpolation between surrounding pixels.
        Linear,
        /// This filter will select the nearest mipmap between two samples and will perform a nearest
        /// interpolation afterwards.
        NearestMipmapNearest,
        /// This filter will select the nearest mipmap between two samples and will perform a linear
        /// interpolation afterwards.
        NearestMipmapLinear,
        /// This filter will linearly interpolate between two mipmaps, which selected texels would have
        /// been interpolated with a nearest filter.
        LinearMipmapNearest,
        /// This filter will linearly interpolate between two mipmaps, which selected texels would have
        /// been linarily interpolated as well.
        LinearMipmapLinear,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for MinFilter {
        #[inline]
        fn clone(&self) -> MinFilter {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for MinFilter {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for MinFilter {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&MinFilter::Nearest,) => {
                    let mut debug_trait_builder = f.debug_tuple("Nearest");
                    debug_trait_builder.finish()
                }
                (&MinFilter::Linear,) => {
                    let mut debug_trait_builder = f.debug_tuple("Linear");
                    debug_trait_builder.finish()
                }
                (&MinFilter::NearestMipmapNearest,) => {
                    let mut debug_trait_builder = f.debug_tuple("NearestMipmapNearest");
                    debug_trait_builder.finish()
                }
                (&MinFilter::NearestMipmapLinear,) => {
                    let mut debug_trait_builder = f.debug_tuple("NearestMipmapLinear");
                    debug_trait_builder.finish()
                }
                (&MinFilter::LinearMipmapNearest,) => {
                    let mut debug_trait_builder = f.debug_tuple("LinearMipmapNearest");
                    debug_trait_builder.finish()
                }
                (&MinFilter::LinearMipmapLinear,) => {
                    let mut debug_trait_builder = f.debug_tuple("LinearMipmapLinear");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Magnification filter.
    pub enum MagFilter {
        /// Nearest interpolation.
        Nearest,
        /// Linear interpolation between surrounding pixels.
        Linear,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for MagFilter {
        #[inline]
        fn clone(&self) -> MagFilter {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for MagFilter {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for MagFilter {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&MagFilter::Nearest,) => {
                    let mut debug_trait_builder = f.debug_tuple("Nearest");
                    debug_trait_builder.finish()
                }
                (&MagFilter::Linear,) => {
                    let mut debug_trait_builder = f.debug_tuple("Linear");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Reify a type into a [`Dim`].
    pub trait Dimensionable {
        /// Size type of a dimension (used to caracterize dimensions’ areas).
        type Size: Copy;
        /// Offset type of a dimension (used to caracterize addition and subtraction of sizes, mostly).
        type Offset: Copy;
        /// Zero offset.
        const ZERO_OFFSET: Self::Offset;
        /// Dimension.
        fn dim() -> Dim;
        /// Width of the associated [`Dimensionable::Size`].
        fn width(size: Self::Size) -> u32;
        /// Height of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
        fn height(_: Self::Size) -> u32 {
            1
        }
        /// Depth of the associated [`Dimensionable::Size`]. If it doesn’t have one, set it to 1.
        fn depth(_: Self::Size) -> u32 {
            1
        }
        /// X offset.
        fn x_offset(offset: Self::Offset) -> u32;
        /// Y offset. If it doesn’t have one, set it to 0.
        fn y_offset(_: Self::Offset) -> u32 {
            1
        }
        /// Z offset. If it doesn’t have one, set it to 0.
        fn z_offset(_: Self::Offset) -> u32 {
            1
        }
        /// Amount of pixels this size represents.
        ///
        /// For 2D sizes, it represents the area; for 3D sizes, the volume; etc.
        /// For cubemaps, it represents the side length of the cube.
        fn count(size: Self::Size) -> usize;
    }
    /// Dimension of a texture.
    pub enum Dim {
        /// 1D.
        Dim1,
        /// 2D.
        Dim2,
        /// 3D.
        Dim3,
        /// Cubemap (i.e. a cube defining 6 faces — akin to 4D).
        Cubemap,
        /// 1D array.
        Dim1Array,
        /// 2D array.
        Dim2Array,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim {
        #[inline]
        fn clone(&self) -> Dim {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Dim::Dim1,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim1");
                    debug_trait_builder.finish()
                }
                (&Dim::Dim2,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim2");
                    debug_trait_builder.finish()
                }
                (&Dim::Dim3,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim3");
                    debug_trait_builder.finish()
                }
                (&Dim::Cubemap,) => {
                    let mut debug_trait_builder = f.debug_tuple("Cubemap");
                    debug_trait_builder.finish()
                }
                (&Dim::Dim1Array,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim1Array");
                    debug_trait_builder.finish()
                }
                (&Dim::Dim2Array,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim2Array");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl fmt::Display for Dim {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                Dim::Dim1 => f.write_str("1D"),
                Dim::Dim2 => f.write_str("2D"),
                Dim::Dim3 => f.write_str("3D"),
                Dim::Cubemap => f.write_str("cubemap"),
                Dim::Dim1Array => f.write_str("1D array"),
                Dim::Dim2Array => f.write_str("2D array"),
            }
        }
    }
    /// 1D dimension.
    pub struct Dim1;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim1 {
        #[inline]
        fn clone(&self) -> Dim1 {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim1 {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim1 {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Dim1 => {
                    let mut debug_trait_builder = f.debug_tuple("Dim1");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Dim1 {
        type Offset = u32;
        type Size = u32;
        const ZERO_OFFSET: Self::Offset = 0;
        fn dim() -> Dim {
            Dim::Dim1
        }
        fn width(w: Self::Size) -> u32 {
            w
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off
        }
        fn count(size: Self::Size) -> usize {
            size as usize
        }
    }
    /// 2D dimension.
    pub struct Dim2;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim2 {
        #[inline]
        fn clone(&self) -> Dim2 {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim2 {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim2 {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Dim2 => {
                    let mut debug_trait_builder = f.debug_tuple("Dim2");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Dim2 {
        type Offset = [u32; 2];
        type Size = [u32; 2];
        const ZERO_OFFSET: Self::Offset = [0, 0];
        fn dim() -> Dim {
            Dim::Dim2
        }
        fn width(size: Self::Size) -> u32 {
            size[0]
        }
        fn height(size: Self::Size) -> u32 {
            size[1]
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off[0]
        }
        fn y_offset(off: Self::Offset) -> u32 {
            off[1]
        }
        fn count([width, height]: Self::Size) -> usize {
            width as usize * height as usize
        }
    }
    /// 3D dimension.
    pub struct Dim3;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim3 {
        #[inline]
        fn clone(&self) -> Dim3 {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim3 {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim3 {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Dim3 => {
                    let mut debug_trait_builder = f.debug_tuple("Dim3");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Dim3 {
        type Offset = [u32; 3];
        type Size = [u32; 3];
        const ZERO_OFFSET: Self::Offset = [0, 0, 0];
        fn dim() -> Dim {
            Dim::Dim3
        }
        fn width(size: Self::Size) -> u32 {
            size[0]
        }
        fn height(size: Self::Size) -> u32 {
            size[1]
        }
        fn depth(size: Self::Size) -> u32 {
            size[2]
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off[0]
        }
        fn y_offset(off: Self::Offset) -> u32 {
            off[1]
        }
        fn z_offset(off: Self::Offset) -> u32 {
            off[2]
        }
        fn count([width, height, depth]: Self::Size) -> usize {
            width as usize * height as usize * depth as usize
        }
    }
    /// Cubemap dimension.
    pub struct Cubemap;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Cubemap {
        #[inline]
        fn clone(&self) -> Cubemap {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Cubemap {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Cubemap {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Cubemap => {
                    let mut debug_trait_builder = f.debug_tuple("Cubemap");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Cubemap {
        type Offset = ([u32; 2], CubeFace);
        type Size = u32;
        const ZERO_OFFSET: Self::Offset = ([0, 0], CubeFace::PositiveX);
        fn dim() -> Dim {
            Dim::Cubemap
        }
        fn width(s: Self::Size) -> u32 {
            s
        }
        fn height(s: Self::Size) -> u32 {
            s
        }
        fn depth(_: Self::Size) -> u32 {
            6
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off.0[0]
        }
        fn y_offset(off: Self::Offset) -> u32 {
            off.0[1]
        }
        fn z_offset(off: Self::Offset) -> u32 {
            match off.1 {
                CubeFace::PositiveX => 0,
                CubeFace::NegativeX => 1,
                CubeFace::PositiveY => 2,
                CubeFace::NegativeY => 3,
                CubeFace::PositiveZ => 4,
                CubeFace::NegativeZ => 5,
            }
        }
        fn count(size: Self::Size) -> usize {
            let size = size as usize;
            size * size
        }
    }
    /// Faces of a cubemap.
    pub enum CubeFace {
        /// The +X face of the cube.
        PositiveX,
        /// The -X face of the cube.
        NegativeX,
        /// The +Y face of the cube.
        PositiveY,
        /// The -Y face of the cube.
        NegativeY,
        /// The +Z face of the cube.
        PositiveZ,
        /// The -Z face of the cube.
        NegativeZ,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for CubeFace {
        #[inline]
        fn clone(&self) -> CubeFace {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for CubeFace {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for CubeFace {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&CubeFace::PositiveX,) => {
                    let mut debug_trait_builder = f.debug_tuple("PositiveX");
                    debug_trait_builder.finish()
                }
                (&CubeFace::NegativeX,) => {
                    let mut debug_trait_builder = f.debug_tuple("NegativeX");
                    debug_trait_builder.finish()
                }
                (&CubeFace::PositiveY,) => {
                    let mut debug_trait_builder = f.debug_tuple("PositiveY");
                    debug_trait_builder.finish()
                }
                (&CubeFace::NegativeY,) => {
                    let mut debug_trait_builder = f.debug_tuple("NegativeY");
                    debug_trait_builder.finish()
                }
                (&CubeFace::PositiveZ,) => {
                    let mut debug_trait_builder = f.debug_tuple("PositiveZ");
                    debug_trait_builder.finish()
                }
                (&CubeFace::NegativeZ,) => {
                    let mut debug_trait_builder = f.debug_tuple("NegativeZ");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// 1D array dimension.
    pub struct Dim1Array;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim1Array {
        #[inline]
        fn clone(&self) -> Dim1Array {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim1Array {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim1Array {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Dim1Array => {
                    let mut debug_trait_builder = f.debug_tuple("Dim1Array");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Dim1Array {
        type Offset = (u32, u32);
        type Size = (u32, u32);
        const ZERO_OFFSET: Self::Offset = (0, 0);
        fn dim() -> Dim {
            Dim::Dim1Array
        }
        fn width(size: Self::Size) -> u32 {
            size.0
        }
        fn height(size: Self::Size) -> u32 {
            size.1
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off.0
        }
        fn y_offset(off: Self::Offset) -> u32 {
            off.1
        }
        fn count((width, layer): Self::Size) -> usize {
            width as usize * layer as usize
        }
    }
    /// 2D dimension.
    pub struct Dim2Array;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Dim2Array {
        #[inline]
        fn clone(&self) -> Dim2Array {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Dim2Array {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Dim2Array {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Dim2Array => {
                    let mut debug_trait_builder = f.debug_tuple("Dim2Array");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl Dimensionable for Dim2Array {
        type Offset = ([u32; 2], u32);
        type Size = ([u32; 2], u32);
        const ZERO_OFFSET: Self::Offset = ([0, 0], 0);
        fn dim() -> Dim {
            Dim::Dim2Array
        }
        fn width(size: Self::Size) -> u32 {
            size.0[0]
        }
        fn height(size: Self::Size) -> u32 {
            size.0[1]
        }
        fn depth(size: Self::Size) -> u32 {
            size.1
        }
        fn x_offset(off: Self::Offset) -> u32 {
            off.0[0]
        }
        fn y_offset(off: Self::Offset) -> u32 {
            off.0[1]
        }
        fn z_offset(off: Self::Offset) -> u32 {
            off.1
        }
        fn count(([width, height], layer): Self::Size) -> usize {
            width as usize * height as usize * layer as usize
        }
    }
    /// A `Sampler` object gives hint on how a `Texture` should be sampled.
    pub struct Sampler {
        /// How should we wrap around the *r* sampling coordinate?
        pub wrap_r: Wrap,
        /// How should we wrap around the *s* sampling coordinate?
        pub wrap_s: Wrap,
        /// How should we wrap around the *t* sampling coordinate?
        pub wrap_t: Wrap,
        /// Minification filter.
        pub min_filter: MinFilter,
        /// Magnification filter.
        pub mag_filter: MagFilter,
        /// For depth textures, should we perform depth comparison and if so, how?
        pub depth_comparison: Option<DepthComparison>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Sampler {
        #[inline]
        fn clone(&self) -> Sampler {
            {
                let _: ::core::clone::AssertParamIsClone<Wrap>;
                let _: ::core::clone::AssertParamIsClone<Wrap>;
                let _: ::core::clone::AssertParamIsClone<Wrap>;
                let _: ::core::clone::AssertParamIsClone<MinFilter>;
                let _: ::core::clone::AssertParamIsClone<MagFilter>;
                let _: ::core::clone::AssertParamIsClone<Option<DepthComparison>>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Sampler {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Sampler {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Sampler {
                    wrap_r: ref __self_0_0,
                    wrap_s: ref __self_0_1,
                    wrap_t: ref __self_0_2,
                    min_filter: ref __self_0_3,
                    mag_filter: ref __self_0_4,
                    depth_comparison: ref __self_0_5,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Sampler");
                    let _ = debug_trait_builder.field("wrap_r", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("wrap_s", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("wrap_t", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("min_filter", &&(*__self_0_3));
                    let _ = debug_trait_builder.field("mag_filter", &&(*__self_0_4));
                    let _ = debug_trait_builder.field("depth_comparison", &&(*__self_0_5));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    /// Default value is as following:
    impl Default for Sampler {
        fn default() -> Self {
            Sampler {
                wrap_r: Wrap::ClampToEdge,
                wrap_s: Wrap::ClampToEdge,
                wrap_t: Wrap::ClampToEdge,
                min_filter: MinFilter::NearestMipmapLinear,
                mag_filter: MagFilter::Linear,
                depth_comparison: None,
            }
        }
    }
    /// Whether mipmaps should be generated.
    pub enum GenMipmaps {
        /// Mipmaps should be generated.
        ///
        /// Mipmaps are generated when creating textures but also when uploading texels, clearing, etc.
        Yes,
        /// Never generate mipmaps.
        No,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for GenMipmaps {
        #[inline]
        fn clone(&self) -> GenMipmaps {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for GenMipmaps {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for GenMipmaps {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&GenMipmaps::Yes,) => {
                    let mut debug_trait_builder = f.debug_tuple("Yes");
                    debug_trait_builder.finish()
                }
                (&GenMipmaps::No,) => {
                    let mut debug_trait_builder = f.debug_tuple("No");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for GenMipmaps {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for GenMipmaps {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for GenMipmaps {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for GenMipmaps {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for GenMipmaps {
        #[inline]
        fn eq(&self, other: &GenMipmaps) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Errors that might happen when working with textures.
    #[non_exhaustive]
    pub enum TextureError {
        /// A texture’s storage failed to be created.
        ///
        /// The carried [`String`] gives the reason of the failure.
        TextureStorageCreationFailed(String),
        /// Not enough pixel data provided for the given area asked.
        ///
        /// You must provide at least as many pixels as expected by the area in the texture you’re
        /// uploading to.
        NotEnoughPixels {
            /// Expected number of pixels in bytes.
            expected_bytes: usize,
            /// Provided number of pixels in bytes.
            provided_bytes: usize,
        },
        /// Unsupported pixel format.
        ///
        /// Sometimes, some hardware might not support a given pixel format (or the format exists on
        /// the interface side but doesn’t in the implementation). That error represents such a case.
        UnsupportedPixelFormat(PixelFormat),
        /// Cannot retrieve texels from a texture.
        ///
        /// That error might happen on some hardware implementations if the user tries to retrieve
        /// texels from a texture that doesn’t support getting its texels retrieved.
        CannotRetrieveTexels(String),
        /// Failed to upload texels.
        CannotUploadTexels(String),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for TextureError {
        #[inline]
        fn clone(&self) -> TextureError {
            match (&*self,) {
                (&TextureError::TextureStorageCreationFailed(ref __self_0),) => {
                    TextureError::TextureStorageCreationFailed(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&TextureError::NotEnoughPixels {
                    expected_bytes: ref __self_0,
                    provided_bytes: ref __self_1,
                },) => TextureError::NotEnoughPixels {
                    expected_bytes: ::core::clone::Clone::clone(&(*__self_0)),
                    provided_bytes: ::core::clone::Clone::clone(&(*__self_1)),
                },
                (&TextureError::UnsupportedPixelFormat(ref __self_0),) => {
                    TextureError::UnsupportedPixelFormat(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&TextureError::CannotRetrieveTexels(ref __self_0),) => {
                    TextureError::CannotRetrieveTexels(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&TextureError::CannotUploadTexels(ref __self_0),) => {
                    TextureError::CannotUploadTexels(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for TextureError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&TextureError::TextureStorageCreationFailed(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("TextureStorageCreationFailed");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TextureError::NotEnoughPixels {
                    expected_bytes: ref __self_0,
                    provided_bytes: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("NotEnoughPixels");
                    let _ = debug_trait_builder.field("expected_bytes", &&(*__self_0));
                    let _ = debug_trait_builder.field("provided_bytes", &&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&TextureError::UnsupportedPixelFormat(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("UnsupportedPixelFormat");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TextureError::CannotRetrieveTexels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CannotRetrieveTexels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&TextureError::CannotUploadTexels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CannotUploadTexels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for TextureError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for TextureError {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<PixelFormat>;
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for TextureError {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for TextureError {
        #[inline]
        fn eq(&self, other: &TextureError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &TextureError::TextureStorageCreationFailed(ref __self_0),
                            &TextureError::TextureStorageCreationFailed(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &TextureError::NotEnoughPixels {
                                expected_bytes: ref __self_0,
                                provided_bytes: ref __self_1,
                            },
                            &TextureError::NotEnoughPixels {
                                expected_bytes: ref __arg_1_0,
                                provided_bytes: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        (
                            &TextureError::UnsupportedPixelFormat(ref __self_0),
                            &TextureError::UnsupportedPixelFormat(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &TextureError::CannotRetrieveTexels(ref __self_0),
                            &TextureError::CannotRetrieveTexels(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &TextureError::CannotUploadTexels(ref __self_0),
                            &TextureError::CannotUploadTexels(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &TextureError) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &TextureError::TextureStorageCreationFailed(ref __self_0),
                            &TextureError::TextureStorageCreationFailed(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &TextureError::NotEnoughPixels {
                                expected_bytes: ref __self_0,
                                provided_bytes: ref __self_1,
                            },
                            &TextureError::NotEnoughPixels {
                                expected_bytes: ref __arg_1_0,
                                provided_bytes: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        (
                            &TextureError::UnsupportedPixelFormat(ref __self_0),
                            &TextureError::UnsupportedPixelFormat(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &TextureError::CannotRetrieveTexels(ref __self_0),
                            &TextureError::CannotRetrieveTexels(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &TextureError::CannotUploadTexels(ref __self_0),
                            &TextureError::CannotUploadTexels(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
                } else {
                    true
                }
            }
        }
    }
    impl TextureError {
        /// A texture’s storage failed to be created.
        pub fn texture_storage_creation_failed(reason: impl Into<String>) -> Self {
            TextureError::TextureStorageCreationFailed(reason.into())
        }
        /// Not enough pixel data provided for the given area asked.
        pub fn not_enough_pixels(expected_bytes: usize, provided_bytes: usize) -> Self {
            TextureError::NotEnoughPixels {
                expected_bytes,
                provided_bytes,
            }
        }
        /// Unsupported pixel format.
        pub fn unsupported_pixel_format(pf: PixelFormat) -> Self {
            TextureError::UnsupportedPixelFormat(pf)
        }
        /// Cannot retrieve texels from a texture.
        pub fn cannot_retrieve_texels(reason: impl Into<String>) -> Self {
            TextureError::CannotRetrieveTexels(reason.into())
        }
        /// Failed to upload texels.
        pub fn cannot_upload_texels(reason: impl Into<String>) -> Self {
            TextureError::CannotUploadTexels(reason.into())
        }
    }
    impl fmt::Display for TextureError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                TextureError::TextureStorageCreationFailed(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["texture storage creation failed: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                TextureError::NotEnoughPixels {
                    ref expected_bytes,
                    ref provided_bytes,
                } => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[
                        "not enough texels provided: expected ",
                        " bytes, provided ",
                        " bytes",
                    ],
                    &match (&expected_bytes, &provided_bytes) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                )),
                TextureError::UnsupportedPixelFormat(ref fmt) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["unsupported pixel format: "],
                        &match (&fmt,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))
                }
                TextureError::CannotRetrieveTexels(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["cannot retrieve texture\u{2019}s texels: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                TextureError::CannotUploadTexels(ref e) => {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["cannot upload texels to texture: "],
                        &match (&e,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
            }
        }
    }
    impl error::Error for TextureError {}
    /// GPU textures.
    pub struct Texture<B, D, P>
    where
        B: ?Sized + TextureBackend<D, P>,
        D: Dimensionable,
        P: Pixel,
    {
        pub(crate) repr: B::TextureRepr,
        size: D::Size,
        _phantom: PhantomData<*const P>,
    }
    impl<B, D, P> Texture<B, D, P>
    where
        B: ?Sized + TextureBackend<D, P>,
        D: Dimensionable,
        P: Pixel,
    {
        /// Create a new [`Texture`].
        ///
        /// `size` is the wished size of the [`Texture`].
        ///
        /// `mipmaps` is the number of extra mipmaps to allocate with the texture. `0` means that the
        /// texture will only be made of a _base level_.
        ///
        /// `sampler` is a [`Sampler`] object that will be used when sampling the texture from inside a
        /// shader, for instance.
        ///
        /// # Notes
        ///
        /// Feel free to have a look at the documentation of [`GraphicsContext::new_texture`] for a
        /// simpler interface.
        pub fn new<C>(
            ctx: &mut C,
            size: D::Size,
            mipmaps: usize,
            sampler: Sampler,
        ) -> Result<Self, TextureError>
        where
            C: GraphicsContext<Backend = B>,
        {
            unsafe {
                ctx.backend()
                    .new_texture(size, mipmaps, sampler)
                    .map(|repr| Texture {
                        repr,
                        size,
                        _phantom: PhantomData,
                    })
            }
        }
        /// Return the number of mipmaps.
        pub fn mipmaps(&self) -> usize {
            unsafe { B::mipmaps(&self.repr) }
        }
        /// Return the size of the texture.
        pub fn size(&self) -> D::Size {
            self.size
        }
        /// Clear the texture with a single pixel value.
        ///
        /// This function will assign the input pixel value to all the pixels in the rectangle described
        /// by `size` and `offset` in the texture.
        pub fn clear_part(
            &mut self,
            gen_mipmaps: GenMipmaps,
            offset: D::Offset,
            size: D::Size,
            pixel: P::Encoding,
        ) -> Result<(), TextureError> {
            unsafe { B::clear_part(&mut self.repr, gen_mipmaps, offset, size, pixel) }
        }
        /// Clear the texture with a single pixel value.
        ///
        /// This function will assign the input pixel value to all the pixels in the texture.
        pub fn clear(
            &mut self,
            gen_mipmaps: GenMipmaps,
            pixel: P::Encoding,
        ) -> Result<(), TextureError> {
            unsafe { B::clear(&mut self.repr, gen_mipmaps, self.size, pixel) }
        }
        /// Upload pixels to a region of the texture described by the rectangle made with `size` and
        /// `offset`.
        pub fn upload_part(
            &mut self,
            gen_mipmaps: GenMipmaps,
            offset: D::Offset,
            size: D::Size,
            texels: &[P::Encoding],
        ) -> Result<(), TextureError> {
            unsafe { B::upload_part(&mut self.repr, gen_mipmaps, offset, size, texels) }
        }
        /// Upload pixels to the whole texture.
        pub fn upload(
            &mut self,
            gen_mipmaps: GenMipmaps,
            texels: &[P::Encoding],
        ) -> Result<(), TextureError> {
            unsafe { B::upload(&mut self.repr, gen_mipmaps, self.size, texels) }
        }
        /// Upload raw data to a region of the texture described by the rectangle made with `size` and
        /// `offset`.
        pub fn upload_part_raw(
            &mut self,
            gen_mipmaps: GenMipmaps,
            offset: D::Offset,
            size: D::Size,
            texels: &[P::RawEncoding],
        ) -> Result<(), TextureError> {
            unsafe { B::upload_part_raw(&mut self.repr, gen_mipmaps, offset, size, texels) }
        }
        /// Upload raw data to the whole texture.
        pub fn upload_raw(
            &mut self,
            gen_mipmaps: GenMipmaps,
            texels: &[P::RawEncoding],
        ) -> Result<(), TextureError> {
            unsafe { B::upload_raw(&mut self.repr, gen_mipmaps, self.size, texels) }
        }
        /// Get a copy of all the pixels from the texture.
        pub fn get_raw_texels(&self) -> Result<Vec<P::RawEncoding>, TextureError>
        where
            P::RawEncoding: Copy + Default,
        {
            unsafe { B::get_raw_texels(&self.repr, self.size) }
        }
    }
}
pub mod vertex {
    //! Vertex formats, associated types and functions.
    //!
    //! A vertex is a type representing a point. It’s common to find vertex positions, normals, colors
    //! or even texture coordinates. Even though you’re free to use whichever type you want, you’re
    //! limited to a range of types and dimensions. See [`VertexAttribType`] and [`VertexAttribDim`]
    //! for further details.
    //!
    //! [`VertexAttribDim`]: crate::vertex::VertexAttribDim
    //! [`VertexAttribType`]: crate::vertex::VertexAttribType
    use std::fmt::Debug;
    /// A type that can be used as a [`Vertex`] has to implement that trait – it must provide an
    /// associated [`VertexDesc`] value via a function call. This associated value gives enough
    /// information on the types being used as attributes to reify enough memory data to align and, size
    /// and type buffers correctly.
    ///
    /// In theory, you should never have to implement that trait directly. Instead, feel free to use the
    /// [luminance-derive] [`Vertex`] proc-macro-derive instead.
    ///
    /// > Note: implementing this trait is `unsafe`.
    pub unsafe trait Vertex: Copy {
        /// Number of attributes.
        const ATTR_COUNT: usize;
        /// The associated vertex format.
        fn vertex_desc() -> VertexDesc;
    }
    unsafe impl Vertex for () {
        const ATTR_COUNT: usize = 0;
        fn vertex_desc() -> VertexDesc {
            Vec::new()
        }
    }
    /// TODO
    pub trait Deinterleave<T> {
        /// Rank of the type in the original type.
        const RANK: usize;
    }
    /// A [`VertexDesc`] is a list of [`VertexBufferDesc`]s.
    pub type VertexDesc = Vec<VertexBufferDesc>;
    /// A vertex attribute descriptor in a vertex buffer.
    ///
    /// Such a description is used to state what vertex buffers are made of and how they should be
    /// aligned / etc.
    pub struct VertexBufferDesc {
        /// Internal index of the attribute.
        ///
        /// That index is used as a mapping with vertex shaders to know how to fetch vertex attributes.
        pub index: usize,
        /// The name of the attribute.
        ///
        /// Such a name is used in vertex shaders to perform mapping.
        pub name: &'static str,
        /// Whether _vertex instancing_ should be used with that vertex attribute.
        pub instancing: VertexInstancing,
        /// Vertex attribute descriptor.
        pub attrib_desc: VertexAttribDesc,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for VertexBufferDesc {
        #[inline]
        fn clone(&self) -> VertexBufferDesc {
            {
                let _: ::core::clone::AssertParamIsClone<usize>;
                let _: ::core::clone::AssertParamIsClone<&'static str>;
                let _: ::core::clone::AssertParamIsClone<VertexInstancing>;
                let _: ::core::clone::AssertParamIsClone<VertexAttribDesc>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for VertexBufferDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexBufferDesc {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                VertexBufferDesc {
                    index: ref __self_0_0,
                    name: ref __self_0_1,
                    instancing: ref __self_0_2,
                    attrib_desc: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("VertexBufferDesc");
                    let _ = debug_trait_builder.field("index", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("name", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("instancing", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("attrib_desc", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for VertexBufferDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for VertexBufferDesc {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<&'static str>;
                let _: ::core::cmp::AssertParamIsEq<VertexInstancing>;
                let _: ::core::cmp::AssertParamIsEq<VertexAttribDesc>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for VertexBufferDesc {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                VertexBufferDesc {
                    index: ref __self_0_0,
                    name: ref __self_0_1,
                    instancing: ref __self_0_2,
                    attrib_desc: ref __self_0_3,
                } => {
                    ::core::hash::Hash::hash(&(*__self_0_0), state);
                    ::core::hash::Hash::hash(&(*__self_0_1), state);
                    ::core::hash::Hash::hash(&(*__self_0_2), state);
                    ::core::hash::Hash::hash(&(*__self_0_3), state)
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for VertexBufferDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for VertexBufferDesc {
        #[inline]
        fn eq(&self, other: &VertexBufferDesc) -> bool {
            match *other {
                VertexBufferDesc {
                    index: ref __self_1_0,
                    name: ref __self_1_1,
                    instancing: ref __self_1_2,
                    attrib_desc: ref __self_1_3,
                } => match *self {
                    VertexBufferDesc {
                        index: ref __self_0_0,
                        name: ref __self_0_1,
                        instancing: ref __self_0_2,
                        attrib_desc: ref __self_0_3,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                            && (*__self_0_3) == (*__self_1_3)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &VertexBufferDesc) -> bool {
            match *other {
                VertexBufferDesc {
                    index: ref __self_1_0,
                    name: ref __self_1_1,
                    instancing: ref __self_1_2,
                    attrib_desc: ref __self_1_3,
                } => match *self {
                    VertexBufferDesc {
                        index: ref __self_0_0,
                        name: ref __self_0_1,
                        instancing: ref __self_0_2,
                        attrib_desc: ref __self_0_3,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                            || (*__self_0_3) != (*__self_1_3)
                    }
                },
            }
        }
    }
    impl VertexBufferDesc {
        /// Create a new [`VertexBufferDesc`].
        pub fn new<S>(sem: S, instancing: VertexInstancing, attrib_desc: VertexAttribDesc) -> Self
        where
            S: Semantics,
        {
            let index = sem.index();
            let name = sem.name();
            VertexBufferDesc {
                index,
                name,
                instancing,
                attrib_desc,
            }
        }
    }
    /// Should vertex instancing be used for a vertex attribute?
    ///
    /// Enabling this is done per attribute but if you enable it for a single attribute of a struct, it
    /// should be enabled for all others (interleaved vertex instancing is not supported).
    pub enum VertexInstancing {
        /// Use vertex instancing.
        On,
        /// Disable vertex instancing.
        Off,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for VertexInstancing {
        #[inline]
        fn clone(&self) -> VertexInstancing {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for VertexInstancing {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexInstancing {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&VertexInstancing::On,) => {
                    let mut debug_trait_builder = f.debug_tuple("On");
                    debug_trait_builder.finish()
                }
                (&VertexInstancing::Off,) => {
                    let mut debug_trait_builder = f.debug_tuple("Off");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for VertexInstancing {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for VertexInstancing {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for VertexInstancing {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for VertexInstancing {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for VertexInstancing {
        #[inline]
        fn eq(&self, other: &VertexInstancing) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Vertex attribute format.
    ///
    /// Vertex attributes (such as positions, colors, texture UVs, normals, etc.) have all a specific
    /// format that must be passed to the GPU. This type gathers information about a single vertex
    /// attribute and is completly agnostic of the rest of the attributes used to form a vertex.
    ///
    /// A type is associated with a single value of type [`VertexAttribDesc`] via the [`VertexAttrib`]
    /// trait. If such an implementor exists for a type, it means that this type can be used as a vertex
    /// attribute.
    pub struct VertexAttribDesc {
        /// Type of the attribute. See [`VertexAttribType`] for further details.
        pub ty: VertexAttribType,
        /// Dimension of the attribute. It should be in 1–4. See [`VertexAttribDim`] for further details.
        pub dim: VertexAttribDim,
        /// Size in bytes that a single element of the attribute takes. That is, if your attribute has
        /// a dimension set to 2, then the unit size should be the size of a single element (not two).
        pub unit_size: usize,
        /// Alignment of the attribute. The best advice is to respect what Rust does, so it’s highly
        /// recommended to use `::std::mem::align_of` to let it does the job for you.
        pub align: usize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for VertexAttribDesc {
        #[inline]
        fn clone(&self) -> VertexAttribDesc {
            {
                let _: ::core::clone::AssertParamIsClone<VertexAttribType>;
                let _: ::core::clone::AssertParamIsClone<VertexAttribDim>;
                let _: ::core::clone::AssertParamIsClone<usize>;
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for VertexAttribDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexAttribDesc {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                VertexAttribDesc {
                    ty: ref __self_0_0,
                    dim: ref __self_0_1,
                    unit_size: ref __self_0_2,
                    align: ref __self_0_3,
                } => {
                    let mut debug_trait_builder = f.debug_struct("VertexAttribDesc");
                    let _ = debug_trait_builder.field("ty", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("dim", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("unit_size", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("align", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for VertexAttribDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for VertexAttribDesc {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<VertexAttribType>;
                let _: ::core::cmp::AssertParamIsEq<VertexAttribDim>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for VertexAttribDesc {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                VertexAttribDesc {
                    ty: ref __self_0_0,
                    dim: ref __self_0_1,
                    unit_size: ref __self_0_2,
                    align: ref __self_0_3,
                } => {
                    ::core::hash::Hash::hash(&(*__self_0_0), state);
                    ::core::hash::Hash::hash(&(*__self_0_1), state);
                    ::core::hash::Hash::hash(&(*__self_0_2), state);
                    ::core::hash::Hash::hash(&(*__self_0_3), state)
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for VertexAttribDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for VertexAttribDesc {
        #[inline]
        fn eq(&self, other: &VertexAttribDesc) -> bool {
            match *other {
                VertexAttribDesc {
                    ty: ref __self_1_0,
                    dim: ref __self_1_1,
                    unit_size: ref __self_1_2,
                    align: ref __self_1_3,
                } => match *self {
                    VertexAttribDesc {
                        ty: ref __self_0_0,
                        dim: ref __self_0_1,
                        unit_size: ref __self_0_2,
                        align: ref __self_0_3,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                            && (*__self_0_3) == (*__self_1_3)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &VertexAttribDesc) -> bool {
            match *other {
                VertexAttribDesc {
                    ty: ref __self_1_0,
                    dim: ref __self_1_1,
                    unit_size: ref __self_1_2,
                    align: ref __self_1_3,
                } => match *self {
                    VertexAttribDesc {
                        ty: ref __self_0_0,
                        dim: ref __self_0_1,
                        unit_size: ref __self_0_2,
                        align: ref __self_0_3,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                            || (*__self_0_3) != (*__self_1_3)
                    }
                },
            }
        }
    }
    impl VertexAttribDesc {
        /// Normalize a vertex attribute format’s type.
        pub fn normalize(self) -> Self {
            VertexAttribDesc {
                ty: self.ty.normalize(),
                ..self
            }
        }
    }
    /// Possible type of vertex attributes.
    pub enum VertexAttribType {
        /// An integral type.
        ///
        /// Typically, `i32` is integral but not `u32`.
        Integral(Normalized),
        /// An unsigned integral type.
        ///
        /// Typically, `u32` is unsigned but not `i32`.
        Unsigned(Normalized),
        /// A floating point integral type.
        Floating,
        /// A boolean integral type.
        Boolean,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for VertexAttribType {
        #[inline]
        fn clone(&self) -> VertexAttribType {
            {
                let _: ::core::clone::AssertParamIsClone<Normalized>;
                let _: ::core::clone::AssertParamIsClone<Normalized>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for VertexAttribType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexAttribType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&VertexAttribType::Integral(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Integral");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&VertexAttribType::Unsigned(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Unsigned");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&VertexAttribType::Floating,) => {
                    let mut debug_trait_builder = f.debug_tuple("Floating");
                    debug_trait_builder.finish()
                }
                (&VertexAttribType::Boolean,) => {
                    let mut debug_trait_builder = f.debug_tuple("Boolean");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for VertexAttribType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for VertexAttribType {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Normalized>;
                let _: ::core::cmp::AssertParamIsEq<Normalized>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for VertexAttribType {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                (&VertexAttribType::Integral(ref __self_0),) => {
                    ::core::hash::Hash::hash(
                        &unsafe { ::core::intrinsics::discriminant_value(self) },
                        state,
                    );
                    ::core::hash::Hash::hash(&(*__self_0), state)
                }
                (&VertexAttribType::Unsigned(ref __self_0),) => {
                    ::core::hash::Hash::hash(
                        &unsafe { ::core::intrinsics::discriminant_value(self) },
                        state,
                    );
                    ::core::hash::Hash::hash(&(*__self_0), state)
                }
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for VertexAttribType {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for VertexAttribType {
        #[inline]
        fn eq(&self, other: &VertexAttribType) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &VertexAttribType::Integral(ref __self_0),
                            &VertexAttribType::Integral(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &VertexAttribType::Unsigned(ref __self_0),
                            &VertexAttribType::Unsigned(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &VertexAttribType) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &VertexAttribType::Integral(ref __self_0),
                            &VertexAttribType::Integral(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &VertexAttribType::Unsigned(ref __self_0),
                            &VertexAttribType::Unsigned(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl VertexAttribType {
        /// Normalize a vertex attribute type if it’s integral.
        ///
        /// Return the normalized integer vertex attribute type if non-normalized. Otherwise, return the
        /// vertex attribute type directly.
        pub fn normalize(self) -> Self {
            match self {
                VertexAttribType::Integral(Normalized::No) => {
                    VertexAttribType::Integral(Normalized::Yes)
                }
                VertexAttribType::Unsigned(Normalized::No) => {
                    VertexAttribType::Unsigned(Normalized::Yes)
                }
                _ => self,
            }
        }
    }
    /// Whether an integral vertex type should be normalized when fetched from a shader program.
    ///
    /// The default implementation is not to normalize anything. You have to explicitly ask for
    /// normalized integers (that will, then, be accessed as floating vertex attributes).
    pub enum Normalized {
        /// Normalize integral values and expose them as floating-point values.
        Yes,
        /// Do not perform any normalization and hence leave integral values as-is.
        No,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Normalized {
        #[inline]
        fn clone(&self) -> Normalized {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Normalized {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Normalized {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Normalized::Yes,) => {
                    let mut debug_trait_builder = f.debug_tuple("Yes");
                    debug_trait_builder.finish()
                }
                (&Normalized::No,) => {
                    let mut debug_trait_builder = f.debug_tuple("No");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Normalized {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Normalized {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Normalized {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Normalized {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Normalized {
        #[inline]
        fn eq(&self, other: &Normalized) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Possible dimension of vertex attributes.
    pub enum VertexAttribDim {
        /// 1D.
        Dim1,
        /// 2D.
        Dim2,
        /// 3D.
        Dim3,
        /// 4D.
        Dim4,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for VertexAttribDim {
        #[inline]
        fn clone(&self) -> VertexAttribDim {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for VertexAttribDim {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VertexAttribDim {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&VertexAttribDim::Dim1,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim1");
                    debug_trait_builder.finish()
                }
                (&VertexAttribDim::Dim2,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim2");
                    debug_trait_builder.finish()
                }
                (&VertexAttribDim::Dim3,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim3");
                    debug_trait_builder.finish()
                }
                (&VertexAttribDim::Dim4,) => {
                    let mut debug_trait_builder = f.debug_tuple("Dim4");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for VertexAttribDim {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for VertexAttribDim {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for VertexAttribDim {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(
                    &unsafe { ::core::intrinsics::discriminant_value(self) },
                    state,
                ),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for VertexAttribDim {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for VertexAttribDim {
        #[inline]
        fn eq(&self, other: &VertexAttribDim) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
                let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    /// Class of vertex attributes.
    ///
    /// A vertex attribute type is always associated with a single constant of type [`VertexAttribDesc`],
    /// giving GPUs hints about how to treat them.
    pub unsafe trait VertexAttrib {
        /// The vertex attribute descriptor.
        const VERTEX_ATTRIB_DESC: VertexAttribDesc;
    }
    /// Vertex attribute semantics.
    ///
    /// Vertex attribute semantics are a mean to make shaders and vertex buffers talk to each other
    /// correctly. This is important for several reasons:
    ///
    ///   - The memory layout of your vertex buffers might be very different from an ideal case or even
    ///     the common case. Shaders don’t have any way to know where to pick vertex attributes from, so
    ///     a mapping is needed.
    ///   - Sometimes, a shader just need a few information from the vertex attributes. You then want to
    ///     be able to authorize _“gaps”_ in the semantics so that shaders can be used for several
    ///     varieties of vertex formats.
    ///
    /// Vertex attribute semantics are any type that can implement this trait. The idea is that
    /// semantics must be unique. The vertex position should have an index that is never used anywhere
    /// else in the vertex buffer. Because of the second point above, it’s also highly recommended
    /// (even though valid not to) to stick to the same index for a given semantics when you have
    /// several tessellations – that allows better composition with shaders. Basically, the best advice
    /// to follow: define your semantics once, and keep to them.
    ///
    /// > Note: feel free to use the [luminance-derive] crate to automatically derive this trait from
    /// > an `enum`.
    pub trait Semantics: Sized + Copy + Clone + Debug {
        /// Retrieve the semantics index of this semantics.
        fn index(&self) -> usize;
        /// Get the name of this semantics.
        fn name(&self) -> &'static str;
        /// Get all available semantics.
        fn semantics_set() -> Vec<SemanticsDesc>;
    }
    impl Semantics for () {
        fn index(&self) -> usize {
            0
        }
        fn name(&self) -> &'static str {
            ""
        }
        fn semantics_set() -> Vec<SemanticsDesc> {
            Vec::new()
        }
    }
    /// Semantics description.
    pub struct SemanticsDesc {
        /// Semantics index.
        pub index: usize,
        /// Name of the semantics (used in shaders).
        pub name: String,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for SemanticsDesc {
        #[inline]
        fn clone(&self) -> SemanticsDesc {
            match *self {
                SemanticsDesc {
                    index: ref __self_0_0,
                    name: ref __self_0_1,
                } => SemanticsDesc {
                    index: ::core::clone::Clone::clone(&(*__self_0_0)),
                    name: ::core::clone::Clone::clone(&(*__self_0_1)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for SemanticsDesc {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                SemanticsDesc {
                    index: ref __self_0_0,
                    name: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("SemanticsDesc");
                    let _ = debug_trait_builder.field("index", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("name", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for SemanticsDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for SemanticsDesc {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for SemanticsDesc {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                SemanticsDesc {
                    index: ref __self_0_0,
                    name: ref __self_0_1,
                } => {
                    ::core::hash::Hash::hash(&(*__self_0_0), state);
                    ::core::hash::Hash::hash(&(*__self_0_1), state)
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for SemanticsDesc {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for SemanticsDesc {
        #[inline]
        fn eq(&self, other: &SemanticsDesc) -> bool {
            match *other {
                SemanticsDesc {
                    index: ref __self_1_0,
                    name: ref __self_1_1,
                } => match *self {
                    SemanticsDesc {
                        index: ref __self_0_0,
                        name: ref __self_0_1,
                    } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &SemanticsDesc) -> bool {
            match *other {
                SemanticsDesc {
                    index: ref __self_1_0,
                    name: ref __self_1_1,
                } => match *self {
                    SemanticsDesc {
                        index: ref __self_0_0,
                        name: ref __self_0_1,
                    } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                },
            }
        }
    }
    /// Class of types that have an associated value which type implements [`Semantics`], defining
    /// vertex legit attributes.
    ///
    /// Vertex attribute types can be associated with only one semantics.
    pub trait HasSemantics {
        /// Type of the semantics.
        ///
        /// See the [`Semantics`] trait for further information.
        type Sem: Semantics;
        /// The aforementioned vertex semantics for the attribute type.
        const SEMANTICS: Self::Sem;
    }
    /// A local version of size_of that depends on the state of the std feature.
    #[inline(always)]
    const fn size_of<T>() -> usize {
        #[cfg(not(feature = "std"))]
        {
            ::core::mem::size_of::<T>()
        }
    }
    /// A local version of align_of that depends on the state of the std feature.
    #[inline(always)]
    const fn align_of<T>() -> usize {
        #[cfg(not(feature = "std"))]
        {
            ::core::mem::align_of::<T>()
        }
    }
    unsafe impl VertexAttrib for i8 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i8>(),
            align: crate::vertex::align_of::<i8>(),
        };
    }
    unsafe impl VertexAttrib for [i8; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i8>(),
            align: crate::vertex::align_of::<i8>(),
        };
    }
    unsafe impl VertexAttrib for [i8; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<i8>(),
            align: crate::vertex::align_of::<i8>(),
        };
    }
    unsafe impl VertexAttrib for [i8; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<i8>(),
            align: crate::vertex::align_of::<i8>(),
        };
    }
    unsafe impl VertexAttrib for [i8; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<i8>(),
            align: crate::vertex::align_of::<i8>(),
        };
    }
    unsafe impl VertexAttrib for i16 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i16>(),
            align: crate::vertex::align_of::<i16>(),
        };
    }
    unsafe impl VertexAttrib for [i16; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i16>(),
            align: crate::vertex::align_of::<i16>(),
        };
    }
    unsafe impl VertexAttrib for [i16; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<i16>(),
            align: crate::vertex::align_of::<i16>(),
        };
    }
    unsafe impl VertexAttrib for [i16; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<i16>(),
            align: crate::vertex::align_of::<i16>(),
        };
    }
    unsafe impl VertexAttrib for [i16; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<i16>(),
            align: crate::vertex::align_of::<i16>(),
        };
    }
    unsafe impl VertexAttrib for i32 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i32>(),
            align: crate::vertex::align_of::<i32>(),
        };
    }
    unsafe impl VertexAttrib for [i32; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<i32>(),
            align: crate::vertex::align_of::<i32>(),
        };
    }
    unsafe impl VertexAttrib for [i32; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<i32>(),
            align: crate::vertex::align_of::<i32>(),
        };
    }
    unsafe impl VertexAttrib for [i32; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<i32>(),
            align: crate::vertex::align_of::<i32>(),
        };
    }
    unsafe impl VertexAttrib for [i32; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Integral(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<i32>(),
            align: crate::vertex::align_of::<i32>(),
        };
    }
    unsafe impl VertexAttrib for u8 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u8>(),
            align: crate::vertex::align_of::<u8>(),
        };
    }
    unsafe impl VertexAttrib for [u8; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u8>(),
            align: crate::vertex::align_of::<u8>(),
        };
    }
    unsafe impl VertexAttrib for [u8; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<u8>(),
            align: crate::vertex::align_of::<u8>(),
        };
    }
    unsafe impl VertexAttrib for [u8; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<u8>(),
            align: crate::vertex::align_of::<u8>(),
        };
    }
    unsafe impl VertexAttrib for [u8; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<u8>(),
            align: crate::vertex::align_of::<u8>(),
        };
    }
    unsafe impl VertexAttrib for u16 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u16>(),
            align: crate::vertex::align_of::<u16>(),
        };
    }
    unsafe impl VertexAttrib for [u16; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u16>(),
            align: crate::vertex::align_of::<u16>(),
        };
    }
    unsafe impl VertexAttrib for [u16; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<u16>(),
            align: crate::vertex::align_of::<u16>(),
        };
    }
    unsafe impl VertexAttrib for [u16; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<u16>(),
            align: crate::vertex::align_of::<u16>(),
        };
    }
    unsafe impl VertexAttrib for [u16; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<u16>(),
            align: crate::vertex::align_of::<u16>(),
        };
    }
    unsafe impl VertexAttrib for u32 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u32>(),
            align: crate::vertex::align_of::<u32>(),
        };
    }
    unsafe impl VertexAttrib for [u32; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<u32>(),
            align: crate::vertex::align_of::<u32>(),
        };
    }
    unsafe impl VertexAttrib for [u32; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<u32>(),
            align: crate::vertex::align_of::<u32>(),
        };
    }
    unsafe impl VertexAttrib for [u32; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<u32>(),
            align: crate::vertex::align_of::<u32>(),
        };
    }
    unsafe impl VertexAttrib for [u32; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Unsigned(Normalized::No),
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<u32>(),
            align: crate::vertex::align_of::<u32>(),
        };
    }
    unsafe impl VertexAttrib for f32 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<f32>(),
            align: crate::vertex::align_of::<f32>(),
        };
    }
    unsafe impl VertexAttrib for [f32; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<f32>(),
            align: crate::vertex::align_of::<f32>(),
        };
    }
    unsafe impl VertexAttrib for [f32; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<f32>(),
            align: crate::vertex::align_of::<f32>(),
        };
    }
    unsafe impl VertexAttrib for [f32; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<f32>(),
            align: crate::vertex::align_of::<f32>(),
        };
    }
    unsafe impl VertexAttrib for [f32; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<f32>(),
            align: crate::vertex::align_of::<f32>(),
        };
    }
    unsafe impl VertexAttrib for f64 {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<f64>(),
            align: crate::vertex::align_of::<f64>(),
        };
    }
    unsafe impl VertexAttrib for [f64; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<f64>(),
            align: crate::vertex::align_of::<f64>(),
        };
    }
    unsafe impl VertexAttrib for [f64; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<f64>(),
            align: crate::vertex::align_of::<f64>(),
        };
    }
    unsafe impl VertexAttrib for [f64; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<f64>(),
            align: crate::vertex::align_of::<f64>(),
        };
    }
    unsafe impl VertexAttrib for [f64; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Floating,
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<f64>(),
            align: crate::vertex::align_of::<f64>(),
        };
    }
    unsafe impl VertexAttrib for bool {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Boolean,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<bool>(),
            align: crate::vertex::align_of::<bool>(),
        };
    }
    unsafe impl VertexAttrib for [bool; 1] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Boolean,
            dim: VertexAttribDim::Dim1,
            unit_size: crate::vertex::size_of::<bool>(),
            align: crate::vertex::align_of::<bool>(),
        };
    }
    unsafe impl VertexAttrib for [bool; 2] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Boolean,
            dim: VertexAttribDim::Dim2,
            unit_size: crate::vertex::size_of::<bool>(),
            align: crate::vertex::align_of::<bool>(),
        };
    }
    unsafe impl VertexAttrib for [bool; 3] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Boolean,
            dim: VertexAttribDim::Dim3,
            unit_size: crate::vertex::size_of::<bool>(),
            align: crate::vertex::align_of::<bool>(),
        };
    }
    unsafe impl VertexAttrib for [bool; 4] {
        const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
            ty: VertexAttribType::Boolean,
            dim: VertexAttribDim::Dim4,
            unit_size: crate::vertex::size_of::<bool>(),
            align: crate::vertex::align_of::<bool>(),
        };
    }
}
