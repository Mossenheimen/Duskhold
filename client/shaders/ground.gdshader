shader_type spatial;
// One chunk's ground: a texel a tile, coloured by the client from the core's tiles. A texel's
// alpha below 1 marks water, which shines a little.

uniform sampler2D ground : source_color, filter_nearest, repeat_disable;
uniform vec2 origin;  // the chunk's north-west corner, in tiles
uniform float size = 32.0;

varying vec2 tile;

void vertex() {
	tile = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xz;
}

void fragment() {
	vec4 texel = texture(ground, (tile - origin) / size);
	bool water = texel.a < 0.75;
	ALBEDO = texel.rgb;
	ROUGHNESS = water ? 0.12 : 0.95;
	SPECULAR = water ? 0.7 : 0.3;
}
