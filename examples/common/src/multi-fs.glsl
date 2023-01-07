in vec3 v_color;

out vec3 frag_color;
out float frag_white;

void main() {
  frag_color = v_color.rgb;
  frag_white = 1.;
}
