#version 410
precision mediump float;

in vec2 uv;
in vec3 normal;
in vec4 pos;

uniform sampler2D tex;
uniform sampler2D light_tex;
uniform mat4 view_pos_to_light_pos;

uniform vec3 light_dir = normalize(vec3(0.1, 0.9, 0.1));
uniform vec3 light_color = vec3(0.0, 0.0, 0.0);

out vec4 out_color;


float ndc_to_uv(float val) {
    // [-1, 1] -> [0, 1]
    return (val + 1.0) / 2.0;
}
vec2 ndc_to_uv(vec2 val) {
    // [-1, 1] -> [0, 1]
    return (val + 1.0) / 2.0;
}

void main() {
    vec2 adjusted_uv = uv;
    // Invert Y as texture is top to bottom, but UV coords are bottom to top
    adjusted_uv.y = 1.0 - adjusted_uv.y;

    vec4 light_pos = pos;
    light_pos = view_pos_to_light_pos * light_pos;

    vec4 light_tex_depth = texture(light_tex, clamp(ndc_to_uv(light_pos.xy), 0.0, 1.0));
    float lit_mul = (ndc_to_uv(light_pos.z - 0.01) < light_tex_depth.r) ? 1.0 : 0.0;

    out_color = texture(tex, adjusted_uv);
    vec3 ambient = out_color.xyz * 0.2 * light_color;
    vec3 diffuse = max(out_color.xyz * -dot(normal, light_dir) * lit_mul * light_color, 0.0);
    out_color.xyz = min(diffuse + ambient , vec3(1.0));
}
