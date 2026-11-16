shader_type spatial;
// Walls, doors, fences and boards, as MultiMeshes: each instance's colour (linear) comes in COLOR,
// and what it's made of in INSTANCE_CUSTOM.r, as a code over 16 (codes.gd's Surface). Each
// surface's look is worked out from where a point is (surfaces.gdshaderinc's sf_surface): plaster
// over a stone plinth, wooden boards and their grain. 0 is plain colour. Furniture, trees and
// plants are the kit's (looks.gdshader).

#include "res://shaders/cut_away.gdshaderinc"
#include "res://shaders/surfaces.gdshaderinc"

// Cut away round the character, or not (what you must see whole: boards).
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
	float rough;
	float metal;
	float shade = sf_surface(int(surface + 0.5), w, n, rough, metal);
	ALBEDO = COLOR.rgb * shade;
	ROUGHNESS = rough;
	METALLIC = metal;
}
