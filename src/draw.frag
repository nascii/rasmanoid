precision mediump float;
precision mediump sampler2D;

uniform sampler2D textures[2];

varying vec2 v_tex_coords;
varying float v_tex_index;

vec4 fetch(int v_tex_index) {
    if (v_tex_index == 0) {
        return texture2D(textures[0], v_tex_coords);
    }

    if (v_tex_index == 1) {
        return texture2D(textures[1], v_tex_coords);
    }

    return vec4(0.);
}

void main() {
    gl_FragColor = fetch(int(v_tex_index));
}
