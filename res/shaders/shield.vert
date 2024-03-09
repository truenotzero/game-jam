#version 450 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aCol;
layout (location = 2) in float aRadius;
layout (location = 3) in int aNumSides;
layout (location = 4) in vec2 aSides[4]; // locations [4,7]


out vec3 vshieldCol;
out float vradius;
out int vnumSides;
out vec2 vsides[4];

void main() {
    // fix shield's position to center instead of top-left
    // this keeps uv coords (in the frag shader) centered however
    gl_Position = vec4(aPos + vec2(0.5, 0.5), -0.8, 1.0);

    vshieldCol = aCol;
    vradius = aRadius;
    vnumSides = aNumSides;
    vsides = aSides;
}
