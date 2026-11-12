shader_type spatial;
// Walls, doors, fences, boards, furniture and trees, as MultiMeshes: each instance's colour
// (linear) comes in COLOR, and what it's made of in INSTANCE_CUSTOM.r, as a code over 16
// (chunk_view.gd's surfaces). Each surface's look is worked out from where a point is
// (surfaces.gdshaderinc): plaster over a stone plinth, wooden boards and their grain, stone,
// leaves, straw, cloth, iron, bark. 0 is plain colour.

#include "res://shaders/cut_away.gdshaderinc"
#include "res://shaders/surfaces.gdshaderinc"

// Cut away round the character, or not (what you must see whole: boards, trunks).
uniform bool cuts = true;

varying float surface;

void vertex() {
	surface = INSTANCE_CUSTOM.r * 16.0;
}

void fragment() {
	if (cuts && cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX)) {
		discard;
	}
	vec3 w = (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
	vec3 n = normalize((INV_VIEW_MATRIX * vec4(NORMAL, 0.0)).xyz);
	// The face's own coordinates: along a side and up it, or a top's ground plan.
	vec2 uv = abs(n.y) > 0.5 ? w.xz : vec2(abs(n.x) > abs(n.z) ? w.z : w.x, w.y);
	int kind = int(surface + 0.5);
	float shade = 1.0;
	float rough = 0.9;
	if (kind == 1) {
		// Plaster over a plinth of stone, damp-stained low down.
		if (w.y < 0.5) {
			vec2 b = sf_blocks(vec2(uv.x * 3.2, w.y * 6.0), 0.12);
			shade = mix(0.55 + 0.2 * b.x, 0.35, b.y);
		} else {
			float damp = smoothstep(1.1, 0.5, w.y) * sf_fbm(uv * 0.8);
			shade = 0.86 + 0.22 * sf_fbm(uv * 2.5) - 0.2 * damp;
		}
	} else if (kind == 2) {
		// Wood: boards and their grain.
		vec3 b = sf_boards(uv, 0.18, 1.2);
		shade = (0.84 + 0.24 * b.x) * (0.88 + 0.2 * b.z) * (1.0 - 0.4 * b.y);
	} else if (kind == 3) {
		// Stone: weathered, flecked.
		shade = 0.78 + 0.3 * sf_fbm(uv * 4.0 + w.y) + 0.1 * step(0.9, sf_hash(floor(uv * 14.0)));
	} else if (kind == 4) {
		// Leaves, in clumps with dark between.
		shade = 0.6 + 0.7 * sf_fbm(w.xz * 2.4 + w.y * 1.7);
	} else if (kind == 5) {
		// Straw: stalks standing up.
		shade = 0.82 + 0.3 * sf_noise(vec2(uv.x * 34.0, uv.y * 2.5));
	} else if (kind == 6) {
		// Cloth: a weave.
		shade = 0.92 + 0.08 * sin(uv.x * 140.0) * sin(uv.y * 140.0) + 0.06 * (sf_fbm(uv * 3.0) - 0.5);
	} else if (kind == 7) {
		// Iron: dark, drawn in streaks, with a little shine.
		shade = 0.85 + 0.2 * sf_noise(vec2(uv.x * 2.0, uv.y * 40.0));
		rough = 0.45;
	} else if (kind == 8) {
		// Bark: ridges up the trunk.
		shade = 0.75 + 0.4 * sf_noise(vec2((w.x + w.z) * 14.0, w.y * 2.2));
	}
	ALBEDO = COLOR.rgb * shade;
	ROUGHNESS = rough;
	METALLIC = kind == 7 ? 0.35 : 0.0;
}
