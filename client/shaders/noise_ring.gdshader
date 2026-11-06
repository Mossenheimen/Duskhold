shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled;
// A noise ring (Mark of the Ninja's): a sound spreads out to how far it carries in the open, then
// fades. (Walls muffle it on the way; the ring shows its reach in the open.)

instance uniform float age = 0.0;    // 0 to 1 over the ring's life
instance uniform float reach = 3.0;  // how far the sound carries, in tiles
uniform vec4 color : source_color = vec4(0.9, 0.95, 1.0, 0.8);
uniform float width = 0.1;           // the ring's width, in tiles

varying vec2 offset;

void vertex() {
	offset = VERTEX.xz;  // from the sound's tile centre, in tiles (the node isn't scaled)
}

void fragment() {
	float spread = 1.0 - pow(1.0 - min(age / 0.3, 1.0), 3.0);  // out fast, then hold
	float d = abs(length(offset) - spread * reach);
	float ring = 1.0 - smoothstep(width * 0.5, width, d);
	ALBEDO = color.rgb;
	ALPHA = color.a * ring * (1.0 - age * age);
}
