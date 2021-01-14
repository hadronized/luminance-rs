in vec2 co;
in uvec3 color;

out vec3 v_color;

void main() {
  gl_Position = vec4(co, 0., 1.);
  v_color = vec3(color.rgb);
}
