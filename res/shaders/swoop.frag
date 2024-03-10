#version 450 core

#define SWOOP_COLOR vec3(1.0)
#define SWOOP_BLUR 0.5
#define SWOOP_SHARPEN 1.25

in vec2 uv;

out vec4 fragColor;

// s controls blur/antialias, higher = blurrier
// r controls resharpen, higher = sharper
float swoop(vec2 uv, float s, float r) {
    float x = uv.x;
    float y = uv.y;
    
    // parabola: y = a(x - x0)^2 + y0
    float ly = 0.65;
    float l = -1.8 * x * x + ly; // leading edge
    float cy = -0.15;
    float c = -0.75 * x * x + cy; // closing edge
    // step(e, x) -> x < e / e > x -> 0
    /*
    if (y < l && y > c) {
        return 1.0;
    } else {
        return 0.0;
    }
    */
    float q = smoothstep(l - s, l + s, y) + smoothstep(y - s, y + s, c);
    q = 1.0 - q;
    q = pow(q, r); 
    return q;
}

void main() {
    float s = swoop(2.0 * uv - 1.0, SWOOP_BLUR, SWOOP_SHARPEN);
    fragColor = vec4(SWOOP_COLOR, s);
}
