in vec3 v_color;

out vec4 frag;

void main() {
  frag = vec4(v_color, 1.);
}
