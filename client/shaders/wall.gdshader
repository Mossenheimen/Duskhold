shader_type spatial;
render_mode world_vertex_coords;  // a wall's top lowered in the world's own heights (wall_top)
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
varying float hidden;

void vertex() {
	surface = INSTANCE_CUSTOM.r * 16.0;
	// Lowered indoors (cut_away.gdshaderinc's wall_top): a piece's top pressed down to it, and a
	// piece wholly above it (over a door, a window's lintel) not drawn, rather than flattened on
	// the wall's own top. Each piece is a unit box scaled: its foot from its middle and height.
	float top = wall_top(VERTEX);
	float foot = MODEL_MATRIX[3].y - length(MODEL_MATRIX[1].xyz) * 0.5;
	hidden = foot >= top - 0.01 ? 1.0 : 0.0;
	VERTEX.y = min(VERTEX.y, top);
}

void fragment() {
	if (hidden > 0.5) {
		discard;
	}
	if (cuts && cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX, INV_VIEW_MATRIX)) {
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
