#version 150 core

in vec4 a_Pos;
in vec2 a_Uv;
in vec3 a_Color;

uniform Transform {
    mat4 u_Transform;
};

out vec4 v_Color;
out vec2 v_Uv;

void main() {
    v_Color = vec4(a_Color, 1.0);
    v_Uv = a_Uv;
    gl_Position = a_Pos * u_Transform;
}
