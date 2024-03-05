#version 450 core

layout(std140, binding = 0) uniform Common {
    mat4 uScreen;
};

// per vertex
layout (location = 0) in vec4 aPos;
// per instance
layout (location = 1) in mat4 aTransform;
layout (location = 5) in vec3 aCol;

out flat vec3 color;

void main() {
    gl_Position = uScreen * aTransform * aPos;
    color = aCol;
}
