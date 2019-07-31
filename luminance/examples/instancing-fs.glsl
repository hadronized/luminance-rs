in vec3 v_color;
in float v_instance_bias;

out vec4 frag;

uniform float t;

void main() {
  float q = v_instance_bias * 10. + t;
  frag = vec4(v_color * vec3(pow(cos(q), 2.), pow(sin(q), 2.), cos(q * .5)), 1.);
  frag = pow(frag, vec4(1./2.2));
}
