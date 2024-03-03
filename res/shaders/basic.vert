#version 450 core

layout(std140, binding = 0) uniform Common {
    mat4 uScreen;
};

// per vertex
layout (location = 0) in vec4 aPos;

out flat vec3 color;

void main() {
    gl_Position = uScreen * aPos;
    color = vec3(0.0, 1.0, 0.0);
}
