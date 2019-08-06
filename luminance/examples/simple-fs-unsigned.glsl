in vec3 v_color;

out uvec4 frag;

void main() {
  vec4 ffrag = vec4(v_color, 1.);
  ffrag = pow(ffrag, vec4(1./2.2));

  // convert the floating-point fragment output to unsigned integral
  ffrag *= 255.;
  frag = uvec4(ffrag);
}
