in vec2 v_uv;
out vec4 frag;

uniform sampler2D tex;

void main() {
  frag = vec4(texture(tex, v_uv).rgb, 1.);

  frag = pow(frag, vec4(1. / 2.2));
}
