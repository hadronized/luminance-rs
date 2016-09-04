//! Shader programs related types and functions.
//!
//! A shader `Program` is an object representing several operations. It’s a streaming program that
//! will operate on vertices, vertex patches, primitives and/or fragments.
//!
//! > *Note: shader programs don’t have to run on all those objects; they can be ran only on
//! vertices and fragments, for instance*.
//!
//! Creating a shader program is very simple. You need shader `Stage`s representing each step of the
//! processing. Here’s the actual mapping between the shader stage types and the processing unit:
//!
//! - `Stage<TessellationControlShader>`: ran on **tessellation parameters** ;
//! - `Stage<TessellationEvaluationShader>`: ran on **patches** ;
//! - `Stage<VertexShader>`: ran on **vertices** ;
//! - `Stage<GeometryShader>`: ran on **primitives** ;
//! - `Stage<FragmentShader>`: ran on **screen fragments**.
//!
//! You *have* to provide at least a `Stage<VertexShader>` and a `Stage<FragmentShader>`. If you
//! want tessellation processing, you need to provide both a `Stage<TessellationControlShader>` and
//! a `Stage<TessellationEvaluationShader>`. If you want primitives processing, you need to add a
//! `Stage<GeometryShader>`.
//!
//! In order to customize the behavior of your shader programs, you have access to *uniforms*. For
//! more details about them, see the documentation for the type `Uniform` and `Uniformable`. When
//! creating a new shader program, you have to provide code to declare its *uniform interface*. The
//! *uniform interface* refers to a type of your own that will be kept by the shader program and
//! exposed to you when you’ll express the need to update its uniforms. That *uniform interface* is
//! created via a closure you pass. That closure takes as arguments a `ProgramProxy` used to
//! retrieve `Uniform`s from the program being constructed. That pattern, that can be a bit
//! overwhelming at first, is important to keep things safe and functional. Keep in mind that you
//! can make the closure fail, so that you can notify a `Uniform` lookup failure, for instance.
//!
//! You can create a `Program` with its `new` associated function.
//!
//! # Example
//!
//! ```
//! // assume we have a vertex shader `vs` and fragment shader `fs`
//! let program = Program::new(None, &vs, None, &fs, |get_uni| {
//!   let resolution: Uniform<[f32; 2]> = try!(get_uni("resolution"));
//!   let time: Uniform<f32> = try!(get_uni("time"));
//!
//!   Ok((resolution, time))
//! });
//! ```

use shader::stage::*;
use shader::uniform::{self, HasUniform, Uniform, Uniformable};

/// Trait to implement to provide shader program features.
pub trait HasProgram: HasStage + HasUniform {
  type Program;

  /// Create a new program by linking it with stages.
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Result<Self::Program, ProgramError>;
  /// Free a program.
  fn free_program(program: &mut Self::Program);
  /// Map a uniform name to its uniform representation. Can fail with `ProgramError`.
  fn map_uniform(program: &Self::Program, name: String, ty: uniform::Type, dim: uniform::Dim) -> (Self::U, Option<UniformWarning>);
  /// Bulk update of uniforms. The update closure should contain the code used to update as many
  /// uniforms as wished.
  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn();
}

/// A shader program with *uniform interface*.
#[derive(Debug)]
pub struct Program<C, T> where C: HasProgram {
  pub repr: C::Program,
  pub uniform_interface: T
}

impl<C, T> Drop for Program<C, T> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.repr)
  }
}

impl<C, T> Program<C, T> where C: HasProgram {
  /// Create a new `Program` by linking it with shader stages and by providing a function to build
  /// its *uniform interface*, which the `Program` will hold for you.
  ///
  /// The *uniform interface* is any type you want. The idea is to bake `Uniform<_>` in your type so
  /// that you can access them later. To do so, you’re given an object of type `ProgramProxy`, which
  /// has a function `uniform`. That function can be used to lookup uniforms so that you can build
  /// your *uniform interface*.
  ///
  /// Use the `update` function to access the *uniform interface* back.
  pub fn new<GetUni>(tess: Option<(&Stage<C, TessellationControlShader>, &Stage<C, TessellationEvaluationShader>)>, vertex: &Stage<C, VertexShader>, geometry: Option<&Stage<C, GeometryShader>>, fragment: &Stage<C, FragmentShader>, get_uni: GetUni) -> Result<Self, ProgramError>
      where GetUni: Fn(ProgramProxy<C>) -> Result<T, ProgramError> {
    let repr = try!(C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr));
    let uniform_interface = try!(get_uni(ProgramProxy::new(&repr)));

    Ok(Program {
      repr: repr,
      uniform_interface: uniform_interface
    })
  }

  pub fn update<F>(&self, f: F) where F: Fn(&T) {
    C::update_uniforms(&self.repr, || { f(&self.uniform_interface) })
  }
}

/// `Program` proxy used to map uniforms. When building a `Program`, as the `Program` doesn’t exist
/// yet, a `ProgramProxy` is passed to act as it was the `Program`.
///
/// Because `ProgramProxy` uses a ref to `Program`, it doesn’t own it and *must die before*.
#[derive(Debug)]
pub struct ProgramProxy<'a, C> where C: 'a + HasProgram {
  repr: &'a C::Program
}

impl<'a, C> ProgramProxy<'a, C> where C: HasProgram {
  fn new(program: &'a C::Program) -> Self {
    ProgramProxy {
      repr: program
    }
  }

  pub fn uniform<T>(&self, name: &str) -> (Uniform<C, T>, Option<UniformWarning>) where T: Uniformable<C> {
    // gather information about the uniform so that backend can proceed with runtime reification and
    // validation
    let ty = T::reify_type();
    let dim = T::dim();

    let (u, w) = C::map_uniform(self.repr, String::from(name), ty, dim);
    (Uniform::new(u), w)
  }
}

#[derive(Clone, Debug)]
pub enum ProgramError {
  LinkFailed(String),
}

#[derive(Clone, Debug)]
pub enum UniformWarning {
  Inactive(String),
  TypeMismatch(String)
}
