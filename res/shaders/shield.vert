#version 450 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec4 aCol;
layout (location = 2) in float aRadius;
layout (location = 3) in int aIsFix;
layout (location = 4) in int aNumSides;
layout (location = 5) in vec2 aSides[4]; // locations [5,8]


out vec4 vshieldCol;
out float vradius;
out int visFix;
out int vnumSides;
out vec2 vsides[4];

void main() {
    // fix shield's position to center instead of top-left
    // this keeps uv coords (in the frag shader) centered however
    gl_Position = vec4(aPos + vec2(0.5), -0.7 , 1.0);

    vshieldCol = aCol;
    vradius = aRadius;
    visFix = aIsFix;
    vnumSides = aNumSides;
    vsides = aSides;
}
