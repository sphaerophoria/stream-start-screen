#version 410

in vec2 in_vert;

uniform float aspect_ratio;

void main() {
    vec2 v = in_vert * 2.0 - 1.0;
    v.y *= aspect_ratio;
    gl_Position = vec4(v, 0.0, 1.0);
}
