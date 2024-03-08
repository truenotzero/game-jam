#version 450 core 

in vec2 uv;
in vec4 shieldCol;

out vec4 fragCol;

// r is the rectangle's top right corner
// s is pushes the forcefield to the edge
float force_field(vec2 uv, float r, float s) {
    float radius = 0.6;
    return pow(length(max(abs(uv) - r + radius, 0.0)) / radius, s);
}

void main() {
    float ff = force_field(uv, 1.0, 8.0);

    if (ff >= 1.0) {
        ff = 0.0;
    }

    fragCol = vec4(shieldCol.xyz, ff);
}