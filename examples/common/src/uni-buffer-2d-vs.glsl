in vec2 co;
in vec3 color;

out vec3 v_color;

uniform Positions {
  vec2[100] p;
} positions;

void main() {
  gl_Position = vec4(co + positions.p[gl_InstanceID], 0., 1.);
  v_color = color;
}
