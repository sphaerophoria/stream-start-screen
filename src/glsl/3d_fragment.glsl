#version 410
precision mediump float;

in vec2 uv;
in vec3 normal;

uniform sampler2D tex;

uniform vec3 light_dir = normalize(vec3(0.1, 0.9, 0.1));
uniform vec3 light_color = vec3(1.0, 0.5, 0.0);

out vec4 out_color;

void main() {
    vec2 adjusted_uv = uv;
    // Invert Y as texture is top to bottom, but UV coords are bottom to top
    adjusted_uv.y = 1.0 - adjusted_uv.y;

    out_color = texture(tex, adjusted_uv);
    out_color.xyz = out_color.xyz * -dot(normal, light_dir) * light_color;
}
