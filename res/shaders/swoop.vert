#version 450 core

#define Z -0.4


layout(std140, binding = 0) uniform Common {
    mat4 uScreen;
};

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aUV;

out vec2 uv;

void main() {
    uv = aUV;
    gl_Position = uScreen * vec4(aPos + vec2(0.5), Z, 1.0);
}
