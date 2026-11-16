// Procedural surfaces: every look is worked out from where a point is, so nothing is loaded,
// nothing repeats, and a chunk's edge never shows a seam. Shared by the ground's shader and the
// walls' and furniture's.

float sf_hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

// Value noise: smooth, 0 to 1, a feature a unit.
float sf_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	vec2 u = f * f * (3.0 - 2.0 * f);
	float a = sf_hash(i);
	float b = sf_hash(i + vec2(1.0, 0.0));
	float c = sf_hash(i + vec2(0.0, 1.0));
	float d = sf_hash(i + vec2(1.0, 1.0));
	return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Four octaves of it: patches within patches.
float sf_fbm(vec2 p) {
	float v = 0.0;
	float a = 0.5;
	for (int k = 0; k < 4; k++) {
		v += a * sf_noise(p);
		p = p * 2.03 + vec2(17.1, 9.2);
		a *= 0.5;
	}
	return v;
}

// Courses of blocks (cobbles, a plinth's stones), `uv` in blocks, odd courses shifted half a block,
// their corners rounded by `round_`. The block's own shade (0 to 1) in x, and how near its mortar
// the point is (0 to 1) in y.
vec2 sf_blocks(vec2 uv, float round_) {
	uv.x += 0.5 * mod(floor(uv.y), 2.0);
	vec2 cell = floor(uv);
	vec2 f = fract(uv) - 0.5;
	vec2 d = max(abs(f) - (0.5 - round_), 0.0);
	float edge = length(d) / round_;
	return vec2(sf_hash(cell), smoothstep(0.55, 1.0, edge));
}

// Boards running along u, each `width` across and `len` long, their ends staggered: the board's
// own shade in x, its seams in y, its grain in z (each 0 to 1).
vec3 sf_boards(vec2 uv, float width, float len) {
	float row = floor(uv.y / width);
	float along = uv.x / len + sf_hash(vec2(row, 7.3));
	float board = floor(along);
	float shade = sf_hash(vec2(row, board));
	float across = fract(uv.y / width);
	float side = smoothstep(0.86, 0.98, abs(across - 0.5) * 2.0);
	float end = smoothstep(0.965, 1.0, fract(along));
	float grain = sf_noise(vec2(uv.x * 1.6, across * 9.0 + row * 3.1 + shade * 5.0));
	return vec3(shade, max(side, end), grain);
}

// A surface's look (VISUALS.md §6), by its code (codes.gd's Surface): how much it lightens or
// darkens its colour at `w`, a point on it, whose face's normal is `n` (both in the world, or in
// the thing's own space for what moves), and how rough and metallic it is there. Plaster sits on a
// plinth of stone under 0.5 m. One place for every surface: the walls' shader and the kit's
// (looks.gdshader) both draw with it.
float sf_surface(int kind, vec3 w, vec3 n, out float rough, out float metal) {
	// The face's own coordinates: along a side and up it, or a top's ground plan.
	vec2 uv = abs(n.y) > 0.5 ? w.xz : vec2(abs(n.x) > abs(n.z) ? w.z : w.x, w.y);
	float shade = 1.0;
	rough = 0.9;
	metal = 0.0;
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
		metal = 0.35;
	} else if (kind == 8) {
		// Bark: ridges up the trunk.
		shade = 0.75 + 0.4 * sf_noise(vec2((w.x + w.z) * 14.0, w.y * 2.2));
	}
	return shade;
}
