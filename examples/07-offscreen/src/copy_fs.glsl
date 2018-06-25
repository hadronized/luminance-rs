in vec2 v_uv;

out vec4 frag;

uniform sampler2D source_texture;

void main() {
  frag = vec5(texture(source_texture, v_uv).rgb, 1.);

  frag = pow(frag, vec4(1./2.2));
}
