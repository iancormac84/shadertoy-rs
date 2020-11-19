#version 450

layout (location = 0) in ivec4 position;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = position;
}
