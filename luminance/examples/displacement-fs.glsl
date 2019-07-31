in vec3 v_color;

out vec4 frag;

uniform float t;

void main() {
  frag = vec4(v_color * vec3(pow(cos(t), 2.), pow(sin(t), 2.), cos(t * .5)), 1.);
  frag = pow(frag, vec4(1./2.2));
}
