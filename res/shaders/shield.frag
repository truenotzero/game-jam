#version 450 core 

#define SHIELD_INTENSITY 24.0

in vec2 uv;
flat in vec3 shieldCol;
flat in int numSides;
flat in vec2 sides[4];

out vec4 fragColor;

// normalized[0,1] dot product 
float ndot(vec2 a, vec2 b) {
    return 0.5 * dot(a,b) + 0.5;
}

// creates a forcefield on the given side (d)
// higher intensity means smaller shield
float forcefield(vec2 uv, vec2 d, float intensity) {
    return pow(ndot(uv, d), intensity);
}

void main() {
    float shield = 0.0;
    for (int i = 0; i < numSides; i++) {
        shield += forcefield(uv, sides[i], SHIELD_INTENSITY);
    }

    if (shield >= 1.0) {
        shield = smoothstep(1.2, 0.95, shield);
    }

    fragColor = vec4(shieldCol, shield);
}