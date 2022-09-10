use std::fmt::{Display, Error, Formatter};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

impl Display for Mode {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    match *self {
      Mode::Point => f.write_str("point"),
      Mode::Line => f.write_str("line"),
      Mode::LineStrip => f.write_str("line strip"),
      Mode::Triangle => f.write_str("triangle"),
      Mode::TriangleStrip => f.write_str("triangle strip"),
      Mode::TriangleFan => f.write_str("triangle fan"),
      Mode::Patch(ref n) => write!(f, "patch ({})", n),
    }
  }
}
