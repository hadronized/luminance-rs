in vec2 v_uv;
out vec4 frag;

uniform sampler2D tex;

void main() {
  frag = texture(tex, v_uv);
}
