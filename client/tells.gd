extends Node3D
## The tells: for every swing or shove winding up, the tile it will land on, filling in as the
## blow nears. Red for the dead's, pale for yours. From DuskWorld.actor_actions().

const THEIRS := Color(1.0, 0.18, 0.1)
const YOURS := Color(0.95, 0.92, 0.8)

var _material := ShaderMaterial.new()
var _marks: Array[MeshInstance3D] = []


func _init() -> void:
	_material.shader = preload("res://shaders/tell.gdshader")


## `actions` holds four numbers an actor (kind, wind-up per mille, x, y); `me` is your index.
func show_actions(actions: PackedInt32Array, me: int) -> void:
	var used := 0
	var actor := -1
	for i in range(0, actions.size(), 4):
		actor += 1
		var kind := actions[i]
		var progress := actions[i + 1]
		# Swings, shoves and blows on doors still winding up. Nothing else lands on a tile.
		if kind not in [1, 2, 4] or progress >= 1000:
			continue
		var mark := _mark(used)
		used += 1
		mark.visible = true
		mark.position = Vector3(actions[i + 2] + 0.5, 0.035, actions[i + 3] + 0.5)
		mark.set_instance_shader_parameter("progress", progress / 1000.0)
		var tint := YOURS if actor == me else THEIRS
		mark.set_instance_shader_parameter("tint", Vector3(tint.srgb_to_linear().r,
			tint.srgb_to_linear().g, tint.srgb_to_linear().b))
	for k in range(used, _marks.size()):
		_marks[k].visible = false


func _mark(k: int) -> MeshInstance3D:
	while _marks.size() <= k:
		var square := PlaneMesh.new()
		square.size = Vector2(0.94, 0.94)
		var mark := MeshInstance3D.new()
		mark.mesh = square
		mark.material_override = _material
		mark.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(mark)
		_marks.append(mark)
	return _marks[k]
