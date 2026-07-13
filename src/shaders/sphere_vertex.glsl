#version 140
uniform mat4 matrix;
uniform vec2 u_uv_offset;
uniform vec2 u_uv_scale;
in vec3 position;
in vec2 tex_coords;
out vec2 v_tex_coords;
void main() {
    gl_Position = matrix * vec4(position, 1.0);
    v_tex_coords = tex_coords * u_uv_scale + u_uv_offset;
}
