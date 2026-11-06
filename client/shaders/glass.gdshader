shader_type spatial;
render_mode blend_mix, depth_draw_never, cull_disabled;
// Window glass: you can see through it, and so can they.

#include "res://shaders/cut_away.gdshaderinc"

uniform vec4 tint : source_color = vec4(0.65, 0.82, 0.92, 0.3);

void fragment() {
	if (cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX)) {
		discard;
	}
	ALBEDO = tint.rgb;
	ALPHA = tint.a;
	ROUGHNESS = 0.05;
	SPECULAR = 0.8;
}
