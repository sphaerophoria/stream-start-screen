#version 410
precision mediump float;

in vec2 vert;

out vec4 out_color;

uniform sampler2D in_tex;
uniform float time;

void main() {
    vec2 adjusted_coord = vert.yx;
    adjusted_coord.y = 1.0 - adjusted_coord.y;
    adjusted_coord *= 2;
    adjusted_coord -= 0.5;

    vec4 background = vec4(
        0.0,
        (sin(adjusted_coord.y * 200.0 + time) + 1.0) / 10.0,
        0.0,
        1.0);

    if (adjusted_coord.x > 1.0 || adjusted_coord.y > 1.0 || adjusted_coord.x < 0.0 || adjusted_coord.y < 0) {
        out_color = background;
    } else {
        vec4 sampled = texture(in_tex, adjusted_coord);
        sampled.r = 0;
        sampled.b = 0;
        out_color = sampled + background;
    }
}
