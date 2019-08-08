<!-- cargo-sync-readme start -->

# luminance windowing

This is the base, abstract crate for windowing common types and functions in luminance. The
`luminance` crate provides you with abstracting over OpenGL, but it doesn’t give you a way to
create an OpenGL context. This is due to the fact that creating and managing OpenGL contexts is
tightly related to the type of application you target. Typical PC applications might need
something like **GLFW** or **glutin** whilst others will directly bind to **X11** or **Windows
API**. Several crates – `luminance-*` – exist to solve that problem. They all provide a
different implementation for a simple need: opening an OpenGL context, opening a window and
manage events. In theory, you could even have a `luminance-gtk` or `luminance-qt` to embed
`luminance` in surfaces in those libraries.

# What’s included

This crate exposes several important types that all backends must use. Among them, you’ll find:

- `WindowDim`: abstraction over the dimension of a window and its mode (windowed, fullscreen, fullscreen
  restricted).
- `WindowOpt`: an opaque type giving access to hints to customize the window integration, such as whether
  the cursor should be hidden or not.

The `Device` trait must be implemented by a backend so that an application is completely
agnostic of the backend. This trait defines several basic methods that will help you to:

- Retrieve the dimension of the window / framebuffer.
- Iterate over the system events captured by your application.
- Draw and swap the buffer chain.

<!-- cargo-sync-readme end -->
