#version 450

layout(binding = 0) uniform Colors {
    vec4 triangleColor;
};

layout(location = 0) out vec4 outColor;

void main() {
    outColor = triangleColor;
}
