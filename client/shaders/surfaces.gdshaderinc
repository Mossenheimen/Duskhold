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
