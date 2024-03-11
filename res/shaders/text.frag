#version 450 core

#define SCALE 1.0 / 1.0

layout (location = 0) uniform sampler2D text;

in vec2 uv;

out vec4 fragColor;

void main() {
    vec4 foreground = texture(text, uv);

    fragColor.rgb = vec3(1.0 - foreground.a);
    fragColor.a = foreground.a;
}
