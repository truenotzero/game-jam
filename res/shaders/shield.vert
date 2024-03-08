#version 450 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aCol;
layout (location = 2) in float aRadius;

out vec4 vshieldCol;
out float vradius;

void main() {
    // fix shield's position to center instead of top-left
    // this keeps uv coords (in the frag shader) centered however
    gl_Position = vec4(aPos + vec3(0.5, 0.5, 0.0), 1.0);

    vshieldCol = aCol;
    vradius = aRadius;
}
