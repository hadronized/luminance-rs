in vec2 co;
in vec3 color;

out vec3 v_color;

uniform vec2 triangle_pos;
uniform float triangle_size;

void main() {
  gl_Position = vec4(co * triangle_size + triangle_pos, 0., 1.);
  v_color = color;
}
