#version 450 core

#define SCALE 1.0 / 1.0
#define COMPENSATE 10.0

layout (binding = 0) uniform sampler2D text;
layout (location = 0) uniform float uCurrentFrame;
layout (location = 1) uniform float uTotalFrames;

in vec2 uv;

out vec4 fragColor;

void main() {
    vec2 duv = vec2(0.0, uCurrentFrame);
    vec2 suv = (uv + duv) / vec2(1.0, uTotalFrames);

    vec4 foreground = texture(text, suv);

    fragColor.rgb = vec3(1.0 - foreground.a);
    fragColor.a = foreground.a;
}
