//! Derive procedural macros for **luminance**.
//!
//! This crate exports several macros used to ease development with **luminance**. You are
//! strongly advised to read the documentation of [luminance] in the first place.
//!
//! # `Vertex`
//!
//! This macro allows to derive the [`Vertex`] trait for a custom `struct` type.
//!
//! [See the full documentation here](https://docs.rs/luminance-derive/latest/vertex/index.html)
//!
//! # `Semantics`
//!
//! This macro allows to derive the [`Semantics`] trait for a custom `enum` type.
//!
//! [See the full documentation here](https://docs.rs/luminance-derive/latest/semantics/index.html)
//!
//! # `UniformInterface`
//!
//! This macro allows to derive the [`UniformInterface`] trait for a custom `struct` type.
//!
//! [See the full documentation here](https://docs.rs/luminance-derive/latest/uniform_interface/index.html)
//!
//! [luminance]: https://docs.rs/luminance
//! [`Vertex`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Vertex.html
//! [`Semantics`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Semantics.html

#![deny(missing_docs)]
extern crate proc_macro;

mod attrib;
mod semantics;
mod uniform_interface;
mod vertex;

use crate::semantics::generate_enum_semantics_impl;
use crate::uniform_interface::generate_uniform_interface_impl;
use crate::vertex::generate_vertex_impl;
use proc_macro::TokenStream;
use syn::{self, Data, DeriveInput, parse_macro_input};

/// The [`Vertex`] derive proc-macro.
///
/// That proc-macro allows you to create custom vertex types easily without having to care about
/// implementing the required traits for your types to be usable with the rest of the crate.
///
/// # The `Vertex` trait
///
/// The [`Vertex`] trait must be implemented if you want to use a type as vertex (passed-in via
/// slices to [`Tess`]). Either you can decide to implement it on your own, or you could just let
/// this crate do the job for you.
///
/// > Important: the [`Vertex`] trait is `unsafe`, which means that all of its implementors must be
/// > as well. This is due to the fact that vertex formats include information about raw-level
/// > GPU memory and a bad implementation can have undefined behaviors.
///
/// # Deriving `Vertex`
///
/// You can derive the [`Vertex`] trait if your type follows these conditions:
///
///   - It must be a `struct` with named fields. This is just a temporary limitation that will get
///     dropped as soon as the crate is stable enough.
///   - Its fields must have a type that implements [`VertexAttrib`]. This is mandatory so that the
///     backend knows enough about the types used in the structure to correctly align memory, pick
///     the right types, etc.
///   - Its fields must have a type that implements [`HasSemantics`] as well. This trait is just a
///     type family that associates a single constant (i.e. the semantics) that the vertex attribute
///     uses.
///
/// Once all those requirements are met, you can derive [`Vertex`] pretty easily.
///
/// > Note: feel free to look at the [`Semantics`] proc-macro as well, that provides a way
/// > to generate semantics types in order to completely both implement [`Semantics`] for an
/// > `enum` of your choice, but also generate *field* types you can use when defining your vertex
/// > type.
///
/// ## Syntax
///
/// The syntax is the following:
///
/// ```rust
/// # use luminance_derive::{Vertex, Semantics};
///
/// // visit the Semantics proc-macro documentation for further details
/// #[derive(Clone, Copy, Debug, PartialEq, Semantics)]
/// pub enum Semantics {
///   #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
///   Position,
///   #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
///   Color
/// }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Vertex)] // just add Vertex to the list of derived traits
/// #[vertex(sem = "Semantics")] // specify the semantics to use for this type
/// struct MyVertex {
///   position: VertexPosition,
///   color: VertexColor
/// }
/// ```
///
/// > Note: the `Semantics` enum must be public because of the implementation of [`HasSemantics`]
/// > trait.
///
/// Besides the `Semantics`-related code, this will:
///
///   - Create a type called `MyVertex`, a struct that will hold a single vertex.
///   - Implement `Vertex for MyVertex`.
///
/// The proc-macro also supports an optional `#[vertex(instanced = "<bool>")]` struct attribute.
/// This attribute allows you to specify whether the fields are to be instanced or not. For more
/// about that, have a look at [`VertexInstancing`].
///
/// [`Vertex`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Vertex.html
#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_vertex_impl(di.ident, di.attrs.iter(), struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving Vertex")
  }
}

/// The [`Semantics`] derive proc-macro.
///
/// [`Semantics`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Semantics.html
#[proc_macro_derive(Semantics, attributes(sem))]
pub fn derive_semantics(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle enums
    Data::Enum(enum_) => {
      match generate_enum_semantics_impl(di.ident, enum_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only enums are currently supported for deriving VertexAttribSem")
  }
}

/// The [`UniformInterface`] derive proc-macro.
///
/// The procedural macro is very simple to use. You declare a struct as you would normally do:
///
/// ```
/// # use luminance::shader::program::Uniform;
/// # use luminance_derive::UniformInterface;
///
/// #[derive(Debug, UniformInterface)]
/// struct MyIface {
///   time: Uniform<f32>,
///   resolution: Uniform<[f32; 4]>
/// }
/// ```
///
/// The effect of this declaration is declaring the `MyIface` struct along with an effective
/// implementation of `UniformInterface` that will try to get the `"time"` and `"resolution"`
/// uniforms in the corresponding shader program. If any of the two uniforms fails to map (inactive
/// uniform, for instance), the whole struct cannot be generated, and an error is arisen (see
/// `UniformInterface::uniform_interface`’s documentation for further details).
///
/// If you don’t use a parameter in your shader, you might not want the whole interface to fail
/// building if that parameter cannot be mapped. You can do that via the `#[unbound]` field
/// attribute:
///
/// ```
/// # use luminance::shader::program::Uniform;
/// # use luminance_derive::UniformInterface;
///
/// #[derive(Debug, UniformInterface)]
/// struct MyIface {
///   #[uniform(unbound)]
///   time: Uniform<f32>, // if this field cannot be mapped, it’ll be ignored
///   resolution: Uniform<[f32; 4]>
/// }
/// ```
///
/// You can also change the default mapping with the `#[uniform(name = "string_mapping")]`
/// attribute. This changes the name that must be queried from the shader program for the mapping
/// to be complete:
///
/// ```
/// # use luminance::shader::program::Uniform;
/// # use luminance_derive::UniformInterface;
///
/// #[derive(Debug, UniformInterface)]
/// struct MyIface {
///   time: Uniform<f32>,
///   #[uniform(name = "res")]
///   resolution: Uniform<[f32; 4]> // maps "res" from the shader program
/// }
/// ```
///
/// Finally, you can mix both attributes if you want to change the mapping and have an unbound
/// uniform if it cannot be mapped:
///
/// ```
/// # use luminance::shader::program::Uniform;
/// # use luminance_derive::UniformInterface;
///
/// #[derive(Debug, UniformInterface)]
/// struct MyIface {
///   time: Uniform<f32>,
///   #[uniform(name = "res", unbound)]
///   resolution: Uniform<[f32; 4]> // must map "res" from the shader program and ignored otherwise
/// }
/// ```
///
/// [`UniformInterface`]: https://docs.rs/luminance/latest/luminance/shader/program/trait.UniformInterface.html
#[proc_macro_derive(UniformInterface, attributes(uniform))]
pub fn derive_uniform_interface(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_uniform_interface_impl(di.ident, struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving UniformInterface")
  }
}
