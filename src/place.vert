attribute vec2 a_position;
attribute vec2 a_tex_coords;
attribute float a_tex_index;

uniform vec2 u_shape;

varying vec2 v_tex_coords;
varying float v_tex_index;

void main() {
    v_tex_coords = a_tex_coords;
    v_tex_index = a_tex_index;

    gl_Position = vec4(2. * a_position / u_shape - 1., 0., 1.);
}
