#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 outColor;

layout (push_constant) uniform ModelViewProjection {
    mat4 mvp;
} MVP;

void main() {
    outColor = normal;
    gl_Position = MVP.mvp * vec4(position, 1.0);
}