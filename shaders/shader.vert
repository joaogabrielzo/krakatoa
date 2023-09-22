#version 450

vec4 positions[3] = vec4[](vec4(0.0, -0.5, 0.0, 1.0), vec4(0.5, 0.5, 0.0, 1.0), vec4(-0.5, 0.5, 0.0, 1.0));

void main() {
    gl_PointSize = 10.0;
    gl_Position = positions[gl_VertexIndex];
}