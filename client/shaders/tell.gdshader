shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled;
// A swing's tell: the tile it will land on, filling in from the middle as its wind-up runs out
// (CONCEPT.md: no hit lands without one).

instance uniform float progress = 0.0;  // 0 as the wind-up starts, 1 as it strikes
instance uniform vec3 tint = vec3(1.0, 0.2, 0.1);  // linear

varying vec2 local;

void vertex() {
	local = VERTEX.xz;  // -0.5 to 0.5 across the tile
}

void fragment() {
	float r = max(abs(local.x), abs(local.y)) * 2.0;  // 0 in the middle, 1 at the edge
	float fill = step(r, progress);
	float rim = smoothstep(0.8, 0.88, r) * (1.0 - smoothstep(0.94, 1.0, r));
	ALBEDO = tint;
	ALPHA = max(fill * 0.45, rim * 0.85);
}
