#version 450 core

uniform vec3 uColor;

in flat vec3 color;

out vec4 fragColor;

void main() {
    fragColor = vec4(uColor, 1.0);
}
