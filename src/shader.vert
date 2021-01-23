#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    // 0: -1 -1      0 0
    // 1: -1  3      0 4
    // 2:  3 -1      4 0
    int i = gl_VertexIndex;
    vec2 position = vec2((i / 2) * 4, (gl_VertexIndex & 1) * 4) - 1;
    gl_Position = vec4(position, 0.0, 1.0);
}
