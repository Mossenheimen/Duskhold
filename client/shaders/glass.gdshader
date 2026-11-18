shader_type spatial;
render_mode blend_mix, depth_draw_never, cull_disabled, world_vertex_coords;
// Window glass: you can see through it, and so can they. Being see-through, it's drawn after the
// fog of war's pass over everything opaque (fog_of_war.gdshader), so it takes the fog itself:
// none of it where you've never seen, dim and grey where you only remember.

#include "res://shaders/cut_away.gdshaderinc"

uniform vec4 tint : source_color = vec4(0.65, 0.82, 0.92, 0.3);
// What you make out of the map, as the fog has it (DuskWorld.view_tiles()): 0 never seen, 1
// remembered, 2 seen now.
uniform sampler2D view : filter_linear, repeat_disable;
uniform vec2 map_size = vec2(32.0, 24.0);

varying float hidden;

void vertex() {
	// A pane stands over the sill: with its wall lowered indoors, it isn't drawn.
	float foot = MODEL_MATRIX[3].y - length(MODEL_MATRIX[1].xyz) * 0.5;
	hidden = foot >= wall_top(VERTEX) - 0.01 ? 1.0 : 0.0;
}

void fragment() {
	if (hidden > 0.5 || cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX, INV_VIEW_MATRIX)) {
		discard;
	}
	vec3 world = (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float seen = texture(view, world.xz / map_size).r * 255.0;
	if (seen < 0.5) {
		discard;
	}
	float now = clamp(seen - 1.0, 0.0, 1.0);
	vec3 grey = vec3(dot(tint.rgb, vec3(0.3, 0.59, 0.11))) * 0.3;
	ALBEDO = mix(grey, tint.rgb, now);
	ALPHA = tint.a * mix(0.5, 1.0, now);
	ROUGHNESS = 0.05;
	SPECULAR = mix(0.1, 0.8, now);
}
