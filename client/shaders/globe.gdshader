shader_type canvas_item;
// A globe of what you have left, as Diablo's and Path of Exile's (VISUALS.md §11, Globe; hotbar.gd):
// a liquid in a glass ball, filled to `fill`, its surface stirring, lit from the upper left, in a
// bronze ring. Drawn whole from where each pixel is: no texture.

uniform float fill : hint_range(0.0, 1.0) = 1.0;
uniform vec4 liquid : source_color = vec4(0.75, 0.16, 0.14, 1.0);
uniform vec4 deep : source_color = vec4(0.2, 0.02, 0.02, 1.0);
uniform vec4 rim : source_color = vec4(0.79, 0.64, 0.4, 1.0);
// 0 to 1: how bright a low globe's throb is now (hotbar.gd sets it; none with less motion).
uniform float pulse = 0.0;
// 1: the surface stirs; 0: still (fewer flashes, less motion).
uniform float motion = 1.0;

void fragment() {
	vec2 p = UV * 2.0 - 1.0;
	float r = length(p);
	if (r > 1.0) {
		discard;
	}
	// The glass, near black where it's empty.
	vec3 colour = vec3(0.025, 0.02, 0.018);
	// The liquid, from the foot up to its surface, which stirs.
	float t = TIME * motion;
	float wave = 0.025 * sin(p.x * 6.0 + t * 2.0) + 0.015 * sin(p.x * 13.0 - t * 3.1);
	float surface = 1.0 - 2.0 * fill + wave * step(0.001, fill) * step(fill, 0.999);
	if (p.y > surface) {
		float depth = clamp((p.y - surface) * 0.7, 0.0, 1.0);
		vec3 body = mix(liquid.rgb, deep.rgb, depth);
		// Slow swirls in it.
		body *= 0.86 + 0.14 * sin(p.x * 7.0 + p.y * 5.0 + t * 1.3) * sin(p.y * 9.0 - t * 0.7);
		// Its surface catches the light.
		body += liquid.rgb * (1.0 - smoothstep(0.0, 0.05, p.y - surface)) * 0.7;
		colour = body * (1.0 + 0.35 * pulse);
	}
	// A ball: darker toward its edge.
	colour *= mix(1.0, 0.4, smoothstep(0.35, 0.86, r));
	// The glass's shine, at its upper left, and a faint one at its lower right.
	colour += vec3(1.0) * smoothstep(0.32, 0.0, length(p - vec2(-0.32, -0.42))) * 0.32;
	colour += vec3(1.0) * smoothstep(0.25, 0.0, length(p - vec2(0.38, 0.5))) * 0.06;
	// The bronze ring, lit from the upper left, with a dark line inside it.
	float ring = smoothstep(0.855, 0.875, r);
	float lit = 0.5 + 0.5 * dot(p / max(r, 0.001), normalize(vec2(-0.7, -0.7)));
	vec3 bronze = mix(rim.rgb * 0.3, rim.rgb * 1.25, lit);
	colour = mix(colour, bronze, ring);
	colour *= 1.0 - 0.7 * (smoothstep(0.83, 0.86, r) - smoothstep(0.86, 0.89, r));
	COLOR = vec4(colour, 1.0 - smoothstep(0.975, 1.0, r));
}
