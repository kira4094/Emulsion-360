#version 140
uniform sampler2D tex;
uniform float bright_shade;
in vec2 v_tex_coords;
out vec4 f_color;
void main() {
    vec4 color = texture(tex, v_tex_coords);
    f_color = vec4(color.rgb * bright_shade, 1.0);
}
