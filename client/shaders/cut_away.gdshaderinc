// Seeing yourself through what's in front of you, and walls lowered round you indoors. Shared by
// the wall, glass and kit shaders (looks.gdshader: a tree's crown, not its trunk).
//
// - See-through (the playtests: "Objects shouldn't disappear, like walls, when you walk behind
//   them"; "it doesnt work when your at corners, or half-way in doors. It also should show a
//   larger area"): what stands between the camera and the character, near it on screen, keeps a
//   lattice of itself (`see_through` of its pixels), so it's there and you see through it, fading
//   to whole at the hole's edge. "Between" is along the ground, toward the camera, with a little
//   behind you counted too: so a wall's foot counts as its top does (at a corner), and so does
//   the wall you stand in (half-way through a door).
// - Lowered indoors ("The low walls mechanic is nice. Perhaps use that automatically when you walk
//   into a building?"): while `low_reach` is above 0, walls within it of the character are
//   lowered to `low_height`, rising again past it; the client eases it in and out
//   (map_view.gd). V lowers every wall for good, by building them low (chunk_view.gd).

uniform vec3 focus = vec3(0.0);     // the character's chest, in world space
uniform float cut_radius = 2.6;     // the hole's radius on screen, in metres (1.7 until the third notes)
uniform float cut_softness = 0.9;   // the width of its edge, where the lattice fills in to whole
uniform float see_through = 0.35;   // what's left of a thing inside the hole: a lattice to see through
uniform float low_reach = 0.0;      // metres round the character that walls are lowered in (0: none)
uniform float low_height = 0.35;    // a lowered wall's top, as V's are

// Interleaved gradient noise (Jimenez 2014): a cheap, steady dither for the lattice.
float cut_dither(vec2 pixel) {
	return fract(52.9829189 * fract(dot(pixel, vec2(0.06711056, 0.00583715))));
}

// Whether a fragment at view-space position `v` is left out of the lattice. `view` is VIEW_MATRIX
// and `inv_view` INV_VIEW_MATRIX.
bool cut_away(vec3 v, vec2 pixel, mat4 view, mat4 inv_view) {
	vec3 world = (inv_view * vec4(v, 1.0)).xyz;
	// Below knee height nothing hides you: a wall's foot stays whole (so the wall reads as there),
	// and a lowered wall isn't dotted over the floor.
	if (world.y < 0.6) {
		return false;
	}
	// Toward the camera along the ground: its back axis, flattened.
	vec2 back = normalize(inv_view[2].xz + vec2(1e-5, 0.0));
	if (dot(world.xz - focus.xz, back) < -0.15) {
		return false;
	}
	vec3 f = (view * vec4(focus, 1.0)).xyz;
	float d = length(v.xy - f.xy);
	float keep = mix(see_through, 1.0, smoothstep(cut_radius - cut_softness, cut_radius, d));
	return cut_dither(pixel) > keep;
}

// The top a wall may reach at world point `p`: lowered within `low_reach` of the character
// indoors, rising over a metre and a half past it; out of reach (or outdoors), as built.
float wall_top(vec3 p) {
	if (low_reach <= 0.0) {
		return 1000.0;
	}
	float d = length(p.xz - focus.xz);
	return mix(low_height, 3.0, smoothstep(low_reach, low_reach + 1.5, d));
}
