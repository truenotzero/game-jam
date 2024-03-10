#version 450 core

#define RESOLUTION 256.0

layout (binding = 0) uniform sampler2D screen;
layout (location = 0) uniform int iTime;

in vec2 uv;
out vec4 fragColor;

float vignette(vec2 uv) {
    uv = 2.0 * uv - 1.0; 
    uv = abs(uv) - 0.0; // 'pushes' out the vignette, more negative = more far
    float b = max(uv.x, uv.y);
    b = max(b, 0.0); // prevent overflow ( = black box in the middle of the screen)
    return pow(b, 4.0); // control the vignette fade, higher = faster
}

void tonemap(inout vec3 frag) {
    // reinhard tonemapping
    frag = frag / (frag + vec3(1.0));
}

void gridify(inout vec3 frag) {
    vec2 res = RESOLUTION * uv;
    int x = int(res.x);
    int y = int(res.y);


    switch (y % 3) {
        case 0:
            frag.r *= 1.2;
            break;
        case 1:
            frag.g *= 1.8;
            break;
        case 2:
            frag.b *= 1.1;
            break;
    }
    //frag.rgb = vec3(sin(fTime));

    if ((iTime / 32 + x) % 2 == 0) {
        frag *= 1.2;
    }

}

vec2 warp(vec2 uv) {
    uv = 2.0 * uv - 1.0;

    float l = length(uv);
    float e = 1.0 * smoothstep(-0.1, 8.0, l); // lower value = less 'curvature'
    float s = 0.95; // controls the 'spread' of the curvature (lower = more spread towards edges)
    float p = pow(l, e);
    uv *= s * p;

    // map back to uv space
    // to sample texture
    return 0.5 * uv + 0.5;
}

void main() {
    vec2 wuv = warp(uv);
    fragColor = texture(screen, wuv);

    gridify(fragColor.rgb);
    tonemap(fragColor.rgb);

    float vig = 1.0 - vignette(uv);
    fragColor.rgb *= vig;
}
