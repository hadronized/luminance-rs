//! Backend interfacing.
//!
//! > Almost everything declared in this module and its submodules is `unsafe`. An end-user **is not
//! > supposed to implement any of this.** Library authors might use some traits from here, required
//! > by generic code, but no one but backend authors should implement any symbols from here.
//!
//! This module and its submodules form the _backend interface_ of luminance. Those are basically traits
//! exposing methods, associated types and associated constants that backend crates must implement in
//! order to be able to run luminance code on a specific technology, such as OpenGL, WebGL, Vulkan, or
//! even a software implementation. Anything that can implement all the traits in this module is a valid
//! backend implementation.
//!
//! # Conventions
//!
//! ## Constrained types
//!
//! When a public symbol, like [`ShaderData`], has type variables and the first one is `B`, it’s likely
//! to be the _backend type_. Very often, that type will be constrained by a trait. For instance,
//! for [`ShaderData`], it is constrained by [`ShaderDataBackend`]. That trait will provide the interface
//! that backends need to implement to support the API type — in our case, [`ShaderData`]. Implementing
//! such traits is `unsafe` and using their methods is `unsafe` too.
//!
//! > By convention, we split the names of the API and the backend traits, so `ShaderData` might refer to
//! > either the API type or the backend trait. For this reason, traits from the backend modules are
//! > often used as `NameBackend` instead of `Name` so that they don’t conflict with the API type.
//! > There are exceptions where the API type is not in scope, but don’t be surprised if you see
//! > `ShaderDataBackend` and there’s no such trait: it’s because the trait is `ShaderData` in
//! > `luminance::backend::shader::ShaderData`.
//!
//! ## Associated types and representation objects
//!
//! You will notice, if you have a look at backend traits, that most of them have associated types.
//! Most of them end with the `Repr` suffix. Those types are the concrete _representation_ of the
//! general concept. For [`ShaderData`], [`ShaderDataBackend::ShaderDataRepr`] must be provided by a backend
//! and used with the rest of the methods of the [`ShaderDataBackend`] trait. As with `unsafe` methods,
//! accessing a `Repr` object is not possible from the public interface — if you have found a way
//! to do without `unsafe`, this is a bug: please consider filing an issue.
//!
//! ## Relationship to `GraphicsContext`
//!
//! [`GraphicsContext`] is an important trait as it is the parent trait of _platform backends_ —
//! i.e. windowing. It makes it possible to use luminance with a wide variety of technologies,
//! systems and platforms.
//!
//! On the other side, traits defined in this module — and its submodules — are the traits
//! representing all concrete implementations of the code people will write with luminance. They
//! contain the concrete implementation of what it means to create a new [`ShaderData`] or set
//! a value at a given index in it, for instance.
//!
//! [`GraphicsContext`] has an associated type, [`GraphicsContext::Backend`], that maps the
//! type implementing [`GraphicsContext`] to a backend type. The graphics context doesn’t have to
//! ship its backend, as they’re loosely coupled: it’s possible to write a backend implementation
//! and use / share it as [`GraphicsContext::Backend`] in several system backends. You will probably
//! find many platform crates serving the same OpenGL 3.3 backend implementation, for instance. They
//! will then all use the same backend type.
//!
//! [`GraphicsContext::Backend`] is surjective — i.e. all backends have a [`GraphicsContext`]
//! mapped, which means that some backends are available via different graphics contexts. The
//! implication is that:
//!
//! - Given a [`GraphicsContext`], you immediately know its associated backend.
//! - Given a backend, there is no unique [`GraphicsContext`] you can map backwards, because
//!   several [`GraphicsContext`] might use that backend.
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
//! The rest of the backend crate is considered _specific_ and will not leak into the luminance API.
//! Some backends provide more features to tweak the backend. That is useful when a user knows they
//! will use that backend, but it also makes it harder for them to write code that is backend-agnostic.
//!
//! ### Mapping between features and traits
//!
//! The following traits must be implemented for the given features:
//!
//! - **Framebuffers**:
//!   - `luminance::backend::framebuffer::FramebufferBackBuffer`
//!   - `luminance::backend::framebuffer::Framebuffer`
//!   - `luminance::backend::texture::TextureBase`
//! - **Shaders**:
//!   - `luminance::backend::shader::Shader`
//!   - `luminance::backend::shader::ShaderData`
//!   - `luminance::backend::shader::Uniformable`
//! - **Tessellation**:
//!   - `luminance::backend::tess::IndexSlice`
//!   - `luminance::backend::tess::InstanceSlice`
//!   - `luminance::backend::tess::Tess`
//!   - `luminance::backend::tess::VertexSlice`
//! - **Texture**:
//!   - `luminance::backend::texture::Texture`
//!   - `luminance::backend::texture::TextureBase`
//! - **Pipelines and gates**:
//!   - `luminance::backend::framebuffer::Framebuffer`
//!   - `luminance::backend::pipeline::Pipeline`
//!   - `luminance::backend::pipeline::PipelineBase`
//!   - `luminance::backend::pipeline::PipelineShaderData`
//!   - `luminance::backend::pipeline::PipelineTexture`
//!   - `luminance::backend::render_gate::RenderGate`
//!   - `luminance::backend::shader::ShaderData`
//!   - `luminance::backend::shading_gate::ShadingGate`
//!   - `luminance::backend::tess_gate::TessGate`
//!   - `luminance::backend::texture::TextureBase`
//!   - `luminance::backend::texture::Texture`
//! - **Query**:
//!   - `luminance::backend::query::Query`
//!
//! [`ShaderData`]: crate::shader::ShaderData
//! [`ShaderDataBackend`]: crate::backend::shader::ShaderData
//! [`ShaderDataBackend::ShaderDataRepr`]: crate::backend::shader::ShaderData::ShaderDataRepr
//! [`GraphicsContext`]: crate::context::GraphicsContext
//! [`GraphicsContext::Backend`]: crate::context::GraphicsContext::Backend

#![allow(missing_docs)]

pub mod color_slot;
pub mod depth_stencil_slot;
pub mod framebuffer;
pub mod pipeline;
pub mod query;
pub mod render_gate;
pub mod shader;
pub mod shading_gate;
pub mod tess;
pub mod tess_gate;
pub mod texture;
