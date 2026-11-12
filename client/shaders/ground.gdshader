shader_type spatial;
// One chunk's ground: a texel a tile, coloured by the client from the core's tiles, with the
// ground's kind in the texel's alpha, as (code + 0.5) / 16 (DuskWorld.chunk_tiles()'s codes).
// Each kind has a surface of its own, worked out from where a point is: grass in tufts and
// patches, dirt with pebbles, a cobbled road, plank floors, a bridge's planks across, and water
// that moves and shines a little.

#include "res://shaders/surfaces.gdshaderinc"

uniform sampler2D ground : source_color, filter_nearest, repeat_disable;
uniform vec2 origin;  // the chunk's north-west corner, in tiles
uniform float size = 32.0;

varying vec2 tile;

void vertex() {
	tile = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xz;
}

void fragment() {
	vec4 texel = texture(ground, (tile - origin) / size);
	int kind = int(texel.a * 16.0);
	bool water = kind == 4 || kind == 6;
	vec2 p = tile;
	float shade = 1.0;
	if (kind == 1) {
		// Grass: patches, and tufts running every which way.
		shade = 0.74 + 0.46 * sf_fbm(p * 0.9) + 0.18 * (sf_noise(p * vec2(22.0, 7.0)) - 0.5);
	} else if (kind == 2) {
		// Dirt, and its pebbles.
		shade = 0.78 + 0.34 * sf_fbm(p * 1.7) + 0.18 * step(0.92, sf_hash(floor(p * 9.0)));
	} else if (kind == 3) {
		// The road: cobbles, three across a tile and four along, in worn mortar.
		vec2 b = sf_blocks(p * vec2(3.0, 4.0), 0.18);
		shade = mix(0.82 + 0.3 * b.x, 0.55, b.y) * (0.92 + 0.12 * sf_fbm(p * 2.0));
	} else if (kind == 5) {
		// A floor: planks four to a tile, a tile and a half long.
		vec3 b = sf_boards(p, 0.25, 1.6);
		shade = (0.82 + 0.28 * b.x) * (0.9 + 0.16 * b.z) * (1.0 - 0.45 * b.y);
	} else if (kind == 7) {
		// A bridge: broad planks laid across it.
		vec3 b = sf_boards(p.yx, 0.34, 2.4);
		shade = (0.8 + 0.25 * b.x) * (0.9 + 0.16 * b.z) * (1.0 - 0.5 * b.y);
	} else if (water) {
		float swell = sin(p.x * 2.1 + TIME * 0.9 + sf_fbm(p * 0.8) * 5.0) * sin(p.y * 1.7 - TIME * 0.7);
		shade = 0.92 + 0.1 * swell;
	}
	ALBEDO = texel.rgb * shade;
	ROUGHNESS = water ? 0.12 : 0.95;
	SPECULAR = water ? 0.7 : 0.3;
}
