#version 450 core

layout(std140, binding = 0) uniform Common {
    mat4 uScreen;
};

// per vertex
layout (location = 0) in vec4 aPos;

void main() {
    gl_Position = uScreen * aPos;
}
