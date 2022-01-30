#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 outColor;

layout (push_constant) uniform PushConstants {
    mat4 model;
} PC;

layout(set = 0, binding = 0) uniform CameraBuffer {
    mat4 view;
    mat4 proj;
} CB;

void main() {
    outColor = normal;
    mat4 mvp = CB.proj * CB.view * PC.model;
    gl_Position = mvp * vec4(position, 1.0);
}