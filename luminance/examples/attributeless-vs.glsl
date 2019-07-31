out vec3 v_color;

const vec2[3] TRIANGLE_POS = vec2[](
  vec2(-.5, -.5),
  vec2( .5, -.5),
  vec2( 0.,  .5)
);

const vec3[3] TRIANGLE_COL = vec3[](
    vec3(1., 0., 0.),
    vec3(0., 1., 0.),
    vec3(0., 0., 1.)
);

void main() {
  gl_Position = vec4(TRIANGLE_POS[gl_VertexID], 0., 1.);
  v_color = TRIANGLE_COL[gl_VertexID];
}
