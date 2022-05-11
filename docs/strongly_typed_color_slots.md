# Strongly typed color slots

> This document is a work-in-progress attempt at (re)designing color slots in a way that is easier and safer to work
> with.

<!-- vim-markdown-toc GFM -->

* [Context](#context)
* [Summary](#summary)
* [Analysis](#analysis)
  * [Typed color slots](#typed-color-slots)
  * [Similarity with vertex and semantics](#similarity-with-vertex-and-semantics)
* [First solution: revamp semantics](#first-solution-revamp-semantics)
  * [Semantics are just strings to integers maps](#semantics-are-just-strings-to-integers-maps)
  * [Compile-time tracking and indexed attributes](#compile-time-tracking-and-indexed-attributes)
  * [Back to semantics](#back-to-semantics)
  * [`ColorSlotData`](#colorslotdata)

<!-- vim-markdown-toc -->

# Context

The current way of declaring color slots is the following:

1. `()` for no color slot. This is not subject to change.
2. `P where P: ColorPixel + RenderablePixel`.
3. `(A, B) where A: ColorPixel + RenderablePixel, B: ColorPixel + RenderablePixel`.

(1.) is not going to change, but the two last points have to. At read site, we don’t really have a problem: we get
access to the textures with `.color_slot()`, like:

```rust
let (color0, color1) = fb.color_slot();

// do something with color0, like binding it to graphics pipeline
// do something with color1, like binding it to graphics pipeline
// etc.
```

The problem is mostly about the write path: indeed, a fragment shader will look like this:

```glsl
out vec4 color0;
out vec4 color1;

void main() {
  // …
}
```

What happens if we write this instead?:

```glsl
out vec4 color1;
out vec4 color0;

void main() {
  // …
}
```

Currently, there is no easy way to be resilient to that kind of problems (besides using an EDSL that makes it impossible
to write that code, but that’s for another topic).

# Summary

We are going to change how `Semantics` works by augmenting `luminance-derive`’s `Semantics` procedural macro with a
`namespace` keyword. That will allow to create semantics that will automatically implement `VertexAttrib` (for
`namespace = vertex`) or `ColorSlotAttrib` (for `namespace = color_slot`). We are also going to add `ColorSlotData` to
identify each attribute of a color slot. Finally, we will need to add a procedural macro to automatically implement
`ColorSlot`.

# Analysis

The first thing to see is that we have two incompatible ways to refer to something: in our code, we use a structural
way (tuples and ranks) while in GLSL, we use a nominal way (stuff has names that is a strong contract). What we
want is a way to express / tag color slots with names and ensure they map to fragment names.

Let’s write create a simple framebuffer:

```rust
let fb = ctx.new_framebuffer::<(Dim2, (RGBA32F, R32F))>().unwrap();
```

This creates a framebuffer with a color slot containing two data: RGBA 32-bit color and red 32-bit color, both floating.
If we want to write to that framebuffer, a fragment shader is required the following outputs:

```glsl
out vec4 color0;
out float color1;
```

The first problem is that both `color0` and `color1` here are meaningless for the rest of the pipelines. They only make
sense for the GLSL code.

The second problem is ranks: in Rust, we implicitly declare that the RGBA data has rank `0` while the red data has rank
`1`, and in GLSL we actually do the same, so it works. However, if we swap one pair without swapping the other, the
assumption fails.

What we need instead is a way to describe a color slot.

## Typed color slots

Besides `()`, color slots should have at least two informations:

- The types, like `RGBA32F` and `R32F`, so that we know what kind of textures to generate and (validate outputs of
shaders).
- The names, like `color0` and `color1`.

The idea is that, when creating a framebuffer, we want to find a way to ensure that a color slot data is correctly
named. So something like:

```rust
// the user-defined type
#[derive(ColorSlot)]
pub struct MyColorSlot {
  color0: RGBA32F,
  color1: R32F,
}

// in a function
let fb = ctx.new_framebuffer::<(Dim2, MyColorSlot)>().unwrap();
```

What would `#[derive(ColorSlot)]` do here is to implement `ColorSlot for MyColorSlot`. In order to do that, it needs to
generate a new type, `MyColorSlotTextures<B, D>` for instance, with the same names:

```rust
pub struct MyColorSlotTextures<B, D> {
  color0: Texture<B, D, RGBA32F>,
  color1: Texture<B, D, R32F>,
}
```

So now the user can use it like:

```rust
let fb = ctx.new_framebuffer::<(Dim2, MyColorSlot)>().unwrap();

let color0 = &fb.color_slot().color0;
let color1 = &fb.color_slot().color1;
```

Once we have this, we can probably do something on the backend regarding the outputs of fragments. In OpenGL, regarding
fragment outputs, locations are defined:

- In the shader, i.e.:
  ```glsl
  layout(location = 1) out vec4 color0;
  layout(location = 0) out float color1;
  ```
- Pre-link specification. We can tell OpenGL to bind things explicitly, using something like `glBindFragDataLocation`.
  That looks like a nice solution.
- Automatic assignment. If we don’t specify anything, it follows the natural order. It’s the current situation we use
  and it sucks.

## Similarity with vertex and semantics

Explicitly specifying the bindings looks like the best solution. However, we need to think about how to do that. The
problem comes from the fact we don’t have semantics / we use strings in the definition of the types. When we create a
fragment shader, we need to end up with a consistent mapping. The problem might be similar with vertex semantics,
actually:

- A `Tess` needs to enable vertex array attributes via indices for each attribute.
- Vertex shader inputs need to explicitly be assigned the right index.

The problem is the same, and the current situation for vertices is to use semantics. Maybe it’s time to remove them.

# First solution: revamp semantics

The concept of semantics (`Semantics`) is to create an `enum` type acting as a _sum type_. That type, unique in the
application, will allow different parts to talk “the same language.” For instance, we can tag the shader program with
the `enum` so that we know which kind of vertex inputs are possible, and we can use it in vertex arrays (`Tess`) as
well. Each variants represents a single “data slot.” For instance:

```rust
#[derive(Semantics)]
pub enum MyVertexSemantics {
  #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
  Position3,

  #[sem(name = "color", repr = "[f32; 3]", wrapper = "VertexColor")]
  Color,
}
```

`#[derive(Semantics)]` will simply implement the right traits to allow talking the same language in both shaders and
vertex data. Without `Semantics`, we are left with names, for instance:

```glsl
// vertex shader
in vec3 position;
in vec3 color;

void main() {
  // …
}
```

In order for this to work, we need to explicitly bind `position` to an index. That index was set using the `Semantics`,
but if we remove the `Semantics`, we need another indirection. It is the same situation for shader outputs: instead of
having the problem with shader data and vertex data, we have the problem with shader data and framebuffer data (color
slots).

## Semantics are just strings to integers maps

Indeed: semantics just map a name (which is mapped on the variant) to a unique integer value (that is automatically
generated by `luminance-derive` — if not using that crate, it comes from the `unsafe impl` of `Semantics`).

When we think about it, semantics could probably be something even simpler. We want a map, so something like:

```rust
"position" -> 0
"color" -> 1

"color0" -> 0
"color1" -> 1
```

We already see one need here: we need a protocol namespace. `0` and `1` are used twice, but they are used in a different
context: for vertex data, and for framebuffer color slot. A more important thing is that those attributes are unique to
the vertex data and to the framebuffer.

So what it means is that when we compile a shader program, we are going to explicitly set indices for inputs and outputs
(vertex attributes / color slots), and those should probably remain constant, so that we can use the same shader with
two different framebuffers as long as the color slots have the same “names.” How do we do that?

The first question is: do we want to track that at compile-time?

## Compile-time tracking and indexed attributes

If we want to track that at compile-time, we need to generate the name and this is probably going to be hard. However,
it’s not impossible. Using an `enum` is probably not the best thing, because it’s going to require people to write lots
of code that they are unlikely to understand why. We need to think of the optimal situation:

- People shouldn’t have to annotate anything in the shader code, especially since that will soon be an EDSL.
- People shouldnt feel friction when defining a `Tess` or a `Framebuffer`.

Users already have to define vertex types (implementing `Vertex`) and they are likely to have to define color slots
(implementing `ColorSlot`) and a depth/stencil slot (implementing `DepthStencilSlot`) as well. That’s already a lot of
types to create. When defining a `Vertex` type, users have to use field types that implement `VertexAttrib` and
`HasSemantics`. `VertexAttrib` ensures that the type is compatible and `HasSemantics` ensures that the type has an
associated mapping, basically.

So what it means is that instead of using that `Semantics` (vertex only) concept, we could add a new abstraction that
would benefit anything that requires an indexed name. The abstraction could be called `IndexedName`, and that would mean
that if a field implements `IndexedName`, it has an index (`usize`) and a name (`&'static str`).

Now, how do we declare indexed names? We can use a macro that automatically creates them for us, such as:

```rust
indexed_names! {
  vertex:
    "position": VertexPosition = V3<f32>;
    "color": VertexColor = f32;

  color_slot:
    "color0": Color0 = RGBA32F;
    "color1": Color1 = R32F;
}
```

Indexed names are generated as types behind the scenes, so that when we do something like:

```rust
#[derive(ColorSlot)]
pub struct MyColorSlot {
  color0: Color0,
  color1: Color1,
}
```

The problem with this solution is that it’s pretty ugly / confusing (the `indexed_names!` macro is basically an EDSL to
learn by itself; I don’t like that). Also, it’s very similar to the `enum`, but more obfuscated.

## Back to semantics

We can simply take the `Semantics` trait, remove it from the vertex module, and make it context-aware. What it means is
that semantics would be namespaced: one semantics set for vertices and one semantics set for color slots. The only thing
to do to enable that is to add another annotation keyword: `namespace`. We would have two namespaces for now:

- `namespace = vertex`: the `Semantics` is a vertex semantics. The effect is that each variant of the `Semantics` would
  also implement `VertexAttrib`.
- `namespace = color_slot`: the `Semantics` is a color slot semantics. The effect is that each variant of the
  `Semantics` would also implement `ColorSlotAttrib`.

## `ColorSlotData`

Color slots implement `ColorSlot`. That trait is used to reify textures.
