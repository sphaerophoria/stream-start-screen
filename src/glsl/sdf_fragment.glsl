#version 410
precision mediump float;
in vec2 vert;
out vec4 color;

uniform sampler2D ourTexture;

float clamp(float x) {
  return x > 1.0 ? 1.0 : x < 0.0 ? 0.0 : x;
}

void main() {
    float val = texture(ourTexture, vert).r;

    if (val < 0.5) {
        discard;
    }

    val = (val - 0.5) * 50.0;
    float alpha = clamp((val - 0.5) * 0.09);

    color = vec4(1.0, 1.0, 1.0, alpha);
}
