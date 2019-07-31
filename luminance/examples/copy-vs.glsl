out vec2 v_uv;

vec2[4] CO = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  vec2 p = CO[gl_VertexID];

  gl_Position = vec4(p, 0., 1.);
  v_uv = p * .5 + .5;
}
