#version 450
layout (location = 0) in vec3 position;
layout (location = 1) in mat4 model_matrix;
layout (location = 5) in vec3 colour;

layout (location = 0) out vec4 aColor;

void main() {
    gl_Position = model_matrix * vec4(position, 1.0);
    aColor = vec4(colour, 1.0);
}