#version 450

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec4 normal;

layout(location = 0) out vec4 outColor;

layout (push_constant) uniform ModelViewProjection {
    mat4 mvp;
} MVP;

void main() {
    outColor = color;
    gl_Position = MVP.mvp * position;
}