shader_type spatial;
// Everything built from the kit (data/looks.toml, client/looks.gd): furniture, trees and plants,
// and whatever else is made of its parts. Each vertex carries its part's colour (linear) in COLOR,
// and in UV2 its surface (x: codes.gd's Surface) and whether it may be cut away (y: 1 if so). A
// MultiMesh instance's colour tints the whole of it (one tile's against the next's). The surfaces
// are surfaces.gdshaderinc's, worked out from where a point is: in the world for what stands
// still, so tiles join without seams, and in the thing's own space for what moves (`local`), so a
// pattern never slides over it.

#include "res://shaders/cut_away.gdshaderinc"
#include "res://shaders/surfaces.gdshaderinc"

// Cut away round the character where a part may be (furniture, crowns), or never.
uniform bool cuts = true;
// Patterns in the thing's own space: for what moves.
uniform bool local = false;
// What's done to the whole: a tint over its colours (darker: dead), a colour washed over them by
// its alpha (pale: hallowed), and a glow at an energy (a power, a body burning; 0: none).
uniform vec4 tint : source_color = vec4(1.0);
uniform vec4 wash : source_color = vec4(0.0);
uniform vec3 glow : source_color = vec3(0.0);
uniform float glow_energy = 0.0;

varying float surface;
varying float cuttable;
varying vec3 own;
varying vec3 own_normal;

void vertex() {
	surface = UV2.x;
	cuttable = UV2.y;
	own = VERTEX;
	own_normal = NORMAL;
}

void fragment() {
	if (cuts && cuttable > 0.5 && cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX, INV_VIEW_MATRIX)) {
		discard;
	}
	vec3 w = local ? own : (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
	vec3 n = local ? normalize(own_normal) : normalize((INV_VIEW_MATRIX * vec4(NORMAL, 0.0)).xyz);
	float rough;
	float metal;
	float shade = sf_surface(int(surface + 0.5), w, n, rough, metal);
	ALBEDO = mix(COLOR.rgb * shade * tint.rgb, wash.rgb, wash.a);
	ROUGHNESS = rough;
	METALLIC = metal;
	EMISSION = glow * glow_energy;
}
