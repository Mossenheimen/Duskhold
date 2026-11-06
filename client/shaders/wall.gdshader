shader_type spatial;
// Walls and doors, one MultiMesh: each instance's colour (linear) comes in COLOR.

#include "res://shaders/cut_away.gdshaderinc"

void fragment() {
	if (cut_away(VERTEX, FRAGCOORD.xy, VIEW_MATRIX)) {
		discard;
	}
	ALBEDO = COLOR.rgb;
	ROUGHNESS = 0.9;
}
