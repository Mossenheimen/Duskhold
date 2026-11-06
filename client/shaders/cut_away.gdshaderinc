// Zomboid's cut-away walls: whatever stands between the camera and the character, close to it on
// screen, is cut away, so you can always see yourself. Shared by the wall and glass shaders.

uniform vec3 focus = vec3(0.0);    // the character's chest, in world space
uniform float cut_radius = 1.7;    // the hole's radius on screen, in metres
uniform float cut_softness = 0.5;  // the width of its dithered edge

// Interleaved gradient noise (Jimenez 2014): a cheap, steady dither for the hole's edge.
float cut_dither(vec2 pixel) {
	return fract(52.9829189 * fract(dot(pixel, vec2(0.06711056, 0.00583715))));
}

// Whether a fragment at view-space position `v` is cut away. `view` is VIEW_MATRIX.
bool cut_away(vec3 v, vec2 pixel, mat4 view) {
	vec3 f = (view * vec4(focus, 1.0)).xyz;
	// The camera looks down -z in view space: a bigger z is nearer the camera.
	if (v.z < f.z + 0.3) {
		return false;
	}
	float d = length(v.xy - f.xy);
	return smoothstep(cut_radius - cut_softness, cut_radius, d) < cut_dither(pixel);
}
