#version 450 core

layout (location = 0) uniform sampler2D text;

in vec2 uv;

out vec4 fragColor;

void main() {
    fragColor = texture(text, uv);
}
