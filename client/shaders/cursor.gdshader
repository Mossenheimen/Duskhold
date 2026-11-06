shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled;
// The tile under the mouse: a thin square outline.

uniform vec4 color : source_color = vec4(1.0, 1.0, 1.0, 0.55);

varying vec2 local;

void vertex() {
	local = VERTEX.xz + 0.5;  // 0 to 1 across the tile
}

void fragment() {
	vec2 edge = min(local, 1.0 - local);
	float line = 1.0 - smoothstep(0.03, 0.05, min(edge.x, edge.y));
	if (line <= 0.0) {
		discard;
	}
	ALBEDO = color.rgb;
	ALPHA = color.a * line;
}
