shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled;
// What the dead can see, tile by tile: DuskWorld.watched(), worked out by the rule they hunt by.
// Each texel is 0 where none can see, else 1 + the alert level of the most alert one that can.

uniform sampler2D watched : filter_nearest, repeat_disable;
uniform vec2 map_size = vec2(32.0, 24.0);
uniform vec3 unaware : source_color = vec3(0.96, 0.9, 0.6);
uniform vec3 curious : source_color = vec3(1.0, 0.62, 0.18);
uniform vec3 hunting : source_color = vec3(1.0, 0.16, 0.1);

varying vec2 tile;

void vertex() {
	tile = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xz;
}

void fragment() {
	int level = int(round(texture(watched, tile / map_size).r * 255.0));
	if (level == 0) {
		discard;
	}
	vec3 colour = level == 1 ? unaware : (level == 2 ? curious : hunting);
	vec2 cell = fract(tile);
	float inner = min(min(cell.x, cell.y), min(1.0 - cell.x, 1.0 - cell.y));
	float rim = 1.0 - smoothstep(0.02, 0.06, inner);
	ALBEDO = colour;
	ALPHA = (level == 1 ? 0.12 : 0.2) + 0.12 * rim;
}
