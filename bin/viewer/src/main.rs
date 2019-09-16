use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, Vector3, perspective};
use luminance::context::GraphicsContext;
use luminance::linear::M44;
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, Tess, TessBuilder, TessError, TessSliceIndex};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_glfw::{Action, GlfwSurface, Key, Surface as _, WindowDim, WindowEvent, WindowOpt};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;
use std::process::exit;
use std::time::Instant;
use try_guard::verify;
use wavefront_obj::obj;

const VS_STR: &str = include_str!("vs.glsl");
const FS_STR: &str = include_str!("fs.glsl");

const FOVY: Rad<f32> = Rad(std::f32::consts::PI / 2.);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  #[uniform(unbound)]
  projection: Uniform<M44>,
  #[uniform(unbound)]
  view: Uniform<M44>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum Semantics {
  #[sem(name = "position", repr = "[f32; 3]", wrapper = "VPos")]
  Position,
  #[sem(name = "normal", repr = "[f32; 3]", wrapper = "VNor")]
  Normal,
}

#[derive(Clone, Copy, Debug, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
  position: VPos,
  normal: VNor
}

type VertexIndex = u32;

struct Obj {
  vertices: Vec<Vertex>,
  indices: Vec<VertexIndex>,
}

impl Obj {
  fn into_tess<C>(self, ctx: &mut C) -> Result<Tess, TessError> where C: GraphicsContext {
    TessBuilder::new(ctx)
      .set_mode(Mode::Triangle)
      .add_vertices(self.vertices)
      .set_indices(self.indices)
      .build()
  }

  fn load<P>(path: P) -> Result<Self, String> where P: AsRef<Path> {
    let file_content = {
      let mut file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
      let mut content = String::new();
      file.read_to_string(&mut content).unwrap();
      content
    };
    let obj_set = obj::parse(file_content).map_err(|e| format!("cannot parse: {:?}", e))?;
    let objects = obj_set.objects;

    verify!(objects.len() == 1).ok_or_else(|| "expecting a single object".to_owned())?;

    let object = objects.into_iter().next().unwrap();

    verify!(object.geometry.len() == 1).ok_or_else(|| "expecting a single geometry".to_owned())?;

    let geometry = object.geometry.into_iter().next().unwrap();

    println!("loading {}", object.name);
    println!("{} vertices", object.vertices.len());
    println!("{} shapes", geometry.shapes.len());

    // build up vertices; for this to work, we remove duplicated vertices by putting them in a
    // map associating the vertex with its ID
    let mut vertex_cache: HashMap<obj::VTNIndex, VertexIndex> = HashMap::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<VertexIndex> = Vec::new();

    for shape in geometry.shapes {
      if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
        for key in &[a, b, c] {
          if let Some(vertex_index) = vertex_cache.get(key) {
            indices.push(*vertex_index);
          } else {
            let p = object.vertices[key.0];
            let n = object.normals[key.2.ok_or_else(|| "missing normal for a vertex".to_owned())?];
            let position = VPos::new([p.x as f32, p.y as f32, p.z as f32]);
            let normal = VNor::new([n.x as f32, n.y as f32, n.z as f32]);
            let vertex = Vertex::new(position, normal);
            let vertex_index = vertices.len() as VertexIndex;

            vertex_cache.insert(*key, vertex_index);
            vertices.push(vertex);
            indices.push(vertex_index);
          }
        }
      } else {
        return Err("unsupported non-triangle shape".to_owned());
      }
    }

    Ok(Obj { vertices, indices })
  }
}

fn main() {
  let surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default());

  match surface {
    Ok(surface) => {
      eprintln!("graphics surface created");
      main_loop(surface);
    }

    Err(e) => {
      eprintln!("cannot create graphics surface:\n{}", e);
      exit(1);
    }
  }
}

fn main_loop(mut surface: GlfwSurface) {
  let path = env::args().nth(1).expect("first argument must be the path of the .obj file to view");
  println!("loading {}", path);

  let mesh = Obj::load(path).unwrap().into_tess(&mut surface).unwrap();

  let start_t = Instant::now();
  let back_buffer = surface.back_buffer().unwrap();

  let program: Program<Semantics, (), ShaderInterface> = Program::from_strings(None, VS_STR, None, FS_STR).unwrap().ignore_warnings();

  let projection = perspective(FOVY, surface.width() as f32 / surface.height() as f32, Z_NEAR, Z_FAR);
  let view = Matrix4::<f32>::look_at(Point3::new(2., 2., 2.), Point3::origin(), Vector3::unit_y());

  'app: loop {
    // handle events
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
        _ => ()
      }
    }

    // rendering code goes here
    // get the current time and create a color based on the time
    let t = start_t.elapsed().as_millis() as f32 * 1e-3;
    let color = [t.cos(), t.sin(), 0.5, 1.];

    surface.pipeline_builder().pipeline(&back_buffer, color, |_, mut shd_gate| {
      shd_gate.shade(&program, |iface, mut rdr_gate| {
        iface.projection.update(projection.into());
        iface.view.update(view.into());

        rdr_gate.render(RenderState::default(), |mut tess_gate| {
          tess_gate.render(mesh.slice(..));
        });
      });
    });

    // swap buffer chains
    surface.swap_buffers();
  }
}

