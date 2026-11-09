shader_type spatial;
render_mode unshaded, fog_disabled, depth_test_disabled, depth_draw_never, cull_disabled, shadows_disabled;
// The fog of war (CONCEPT.md: outside your sight you see what you remember, never a live monster),
// as one pass over the whole screen, drawn before the other see-through things (render priority):
// the sight cones, rings and tells go on top of it. Each pixel's tile comes from the depth buffer,
// and DuskWorld.view_tiles() says whether it's seen now (as it is), remembered (dim and grey), or
// never seen (black). Nothing here decides what's seen: the core did.

uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_nearest;
uniform sampler2D depth_texture : hint_depth_texture, repeat_disable, filter_nearest;
uniform sampler2D view : filter_linear, repeat_disable;
uniform vec2 map_size = vec2(32.0, 24.0);

void vertex() {
	POSITION = vec4(VERTEX.xy, 1.0, 1.0);  // the quad covers the screen
}

void fragment() {
	float depth = texture(depth_texture, SCREEN_UV).r;
	vec4 world = INV_VIEW_MATRIX * INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, depth, 1.0);
	vec2 tile = world.xz / world.w;
	// 0 never seen, 1 remembered, 2 seen: bytes, which the texture reads as n / 255.
	float seen = texture(view, tile / map_size).r * 255.0;
	vec3 colour = texture(screen_texture, SCREEN_UV).rgb;
	vec3 remembered = mix(colour, vec3(dot(colour, vec3(0.3, 0.59, 0.11))), 0.75) * 0.3;
	ALBEDO = seen < 1.0 ? remembered * seen : mix(remembered, colour, seen - 1.0);
}
