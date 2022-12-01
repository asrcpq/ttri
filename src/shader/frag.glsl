#version 450
#extension GL_EXT_nonuniform_qualifier: enable

layout(location = 0) in vec4 f_color;
layout(location = 1) in vec2 f_tex_coord;
layout(location = 2) flat in int f_tex_layer;

layout(location = 0) out vec4 o_color;

layout(set = 1, binding = 0) uniform sampler2D tex[];

void main() {
	if (f_tex_layer >= 0) {
		o_color = texture(nonuniformEXT(tex[f_tex_layer]), f_tex_coord);
	} else {
		o_color = vec4(0.0, 0.0, 0.0, 1.0);
	}
	o_color.xyz = f_color.w * f_color.xyz + (1.0 - f_color.w) * o_color.xyz;
}
