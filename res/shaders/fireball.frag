
#version 450 core

in vec2 uv;
in vec4 fireCol;
in float radius;

out vec4 fragCol;

float circle(vec2 p, float r) {
    return step(p.x * p.x + p.y * p.y, r * r);
}

float smoothcircle(vec2 p, float r, float pct) {
    float ir = r * (1.0 - pct);
    float n = smoothstep(ir * ir, r * r, p.x * p.x + p.y * p.y);
    return n;
}

void main() {
    vec4 col = vec4(smoothcircle(uv, 3 * radius, 1.0));
    col = 0.01 / col;
    col *= 1.0 - smoothcircle(uv, radius, 0.1);

    fragCol = fireCol * col;
}
