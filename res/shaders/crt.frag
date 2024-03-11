#version 450 core

#define RESOLUTION 256.0

layout (binding = 0) uniform sampler2D screen;
layout (location = 0) uniform uint iTime;
layout (location = 1) uniform float brightness;

in vec2 uv;
out vec4 fragColor;

float vignette(vec2 uv) {
    uv = 2.0 * uv - 1.0; 
    uv = abs(uv) - 0.0; // 'pushes' out the vignette, more negative = more far
    float b = max(uv.x, uv.y);
    b = max(b, 0.0); // prevent overflow ( = black box in the middle of the screen)
    return pow(b, 8.0); // control the vignette fade, higher = faster
}

void add_scan_lines(inout vec3 frag) {
    vec2 res = RESOLUTION * uv;
    int x = int(res.x);
    int y = int(res.y);


    float pop = 1.15;
    switch (y % 3) {
        case 0:
            frag.r *= pop;
            break;
        case 1:
            frag.g *= pop;
            break;
        case 2:
            frag.b *= pop;
            break;
    }
    //frag.rgb = vec3(sin(fTime));

    if ((iTime / 32 + y) % 2 == 0) {
        frag *= 1.05;
    }

}

// monitor-like warp
vec2 warp(vec2 uv) {
    uv = 2.0 * uv - 1.0;

    float l = length(uv);
    float e = 1.0 * smoothstep(-0.1, 8.0, l); // lower value = less 'curvature'
    float s = 0.99; // controls the 'spread' of the curvature (lower = more spread towards edges)
    float p = pow(l, e);
    uv *= s * p;

    // map back to uv space
    // to sample texture
    return 0.5 * uv + 0.5;
}

void main() {
    vec2 wuv = warp(uv);
    fragColor = texture(screen, wuv);

    add_scan_lines(fragColor.rgb);

    float vig = 1.0 - vignette(uv);
    fragColor.rgb *= vig;
    fragColor.a = brightness;
}
