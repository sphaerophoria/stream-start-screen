#version 410

#define M_PI 3.1415926535897932384626433832795

in vec4 in_vert;
in vec2 in_uv;

uniform mat4 model = mat4(
    cos(M_PI / 4), -sin(M_PI / 4), 0, 0,
    sin(M_PI / 4), cos(M_PI / 4),  0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1
);
uniform mat4 view = mat4(
    cos(M_PI / 4), -sin(M_PI / 4), 0, 0,
    sin(M_PI / 4), cos(M_PI / 4),  0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1
);

out vec2 uv;

void main() {
    vec4 out_vert = in_vert;

    out_vert = inverse(view) * model * out_vert;

    out_vert.xy /= out_vert.z;
    gl_Position = out_vert;

    uv = in_uv;
}
