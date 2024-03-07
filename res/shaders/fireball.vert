#version 450 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec4 aCol;
layout (location = 2) in float aRadius;

out vec4 vfireCol;
out float vradius;

void main() {
    gl_Position = vec4(aPos, -0.9, 1.0);

    vfireCol = aCol;
    vradius = aRadius;
}
