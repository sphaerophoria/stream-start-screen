#version 410

layout (location = 0) in vec2 in_vert;
layout (location = 1) in vec2 texture_coord;
out vec2 vert;

uniform float aspect_ratio;

void main() {
    vert = texture_coord;
    vec2 v = in_vert * 2.0 - 1.0;
    v.y *= aspect_ratio;
    gl_Position = vec4(v, -1.0, 1.0);
}
