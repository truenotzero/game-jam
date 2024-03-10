#version 450 core

layout (location = 0) in vec4 aPos;
layout (location = 1) in vec2 aUV;

out vec2 uv;

void main() {
    uv = aUV;
    gl_Position = aPos;
}
