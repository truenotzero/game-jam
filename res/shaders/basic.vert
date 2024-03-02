#version 450 core

uniform mat4 uScreen;

// per vertex
layout (location = 0) in vec3 aPos;
// per instance
layout (location = 1) in mat3 aTransform;
layout (location = 4) in vec3 aCol;
layout (location = 5) in uint aFrame;

out flat vec3 color;

void main() {
    gl_Position = vec4(aTransform * aPos, 1.0);
    color = aCol;
}
