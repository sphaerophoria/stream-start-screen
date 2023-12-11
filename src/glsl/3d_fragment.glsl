#version 410
precision mediump float;

in vec2 uv;

uniform sampler2D tex;

out vec4 out_color;

void main() {
    vec2 adjusted_uv = uv;
    // Invert Y as texture is top to bottom, but UV coords are bottom to top
    adjusted_uv.y = 1.0 - adjusted_uv.y;
    out_color = texture(tex, adjusted_uv);
}
