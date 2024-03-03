#version 450 core

uniform mat4 uScreen;

// per vertex
layout (location = 0) in vec4 aPos;
// per instance
layout (location = 1) in mat4 aTransform;
layout (location = 5) in vec3 aCol;
layout (location = 6) in uint aFrame;

out flat vec3 color;

void main() {
    gl_Position = uScreen * aTransform * aPos;
    color = aCol;
}
