shader_type canvas_item;
// Dread darkening the edges of the screen (CONCEPT.md: every sense is drawn). `amount` is your
// dread, from 0 (calm) to 1 (the most there is), eased from DuskWorld.character_dread(). Only the
// look: what dread does is the core's.

uniform float amount : hint_range(0.0, 1.0) = 0.0;
uniform vec3 tint : source_color = vec3(0.02, 0.0, 0.04);  // Look.DREAD, set by hud.gd

void fragment() {
	// Round, whatever the screen's shape: across stretched by its width over its height.
	vec2 size = 1.0 / SCREEN_PIXEL_SIZE;
	vec2 p = (UV - vec2(0.5)) * vec2(size.x / size.y, 1.0);
	float edge = smoothstep(0.3, 0.95, length(p));
	// A slow throb that quickens as it grows, like blood in the ears.
	float beat = 1.0 + 0.08 * amount * sin(TIME * (2.0 + 4.0 * amount));
	COLOR = vec4(tint, clamp(edge * amount * beat, 0.0, 0.92));
}
