#version 450 core

#define SCALE 1.0 / 1.0
#define LETTER_HEIGHT 10.0
#define LINE_HEIGHT 14.0

layout (binding = 0) uniform sampler2D text;
layout (location = 0) uniform float uCurrentFrame;
layout (location = 1) uniform float uTotalFrames;

in vec2 uv;

out vec4 fragColor;

void main() {
    vec2 suv = uv;
    if (uTotalFrames > 1.0) {
        // adjust for the whitespace above
        suv.y *= (LETTER_HEIGHT / LINE_HEIGHT);
    }

    vec2 duv = vec2(0.0, uCurrentFrame);
    suv = (suv + duv) / vec2(1.0, uTotalFrames);


    vec4 foreground = texture(text, suv);

    fragColor.rgb = vec3(1.0 - foreground.a);
    fragColor.a = foreground.a;
}
