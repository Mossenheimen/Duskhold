extends Node3D
## Engagement you can see (CONCEPT.md, combat): a line on the ground from each of the dead on
## you, thin while it's beside you and after you, thick while it has hold of you. Doorways
## matter because they cap the number of lines. From DuskWorld.character_threats().

const ENGAGED := Color(0.95, 0.25, 0.18, 0.55)
const HOLDING := Color(0.75, 0.05, 0.05, 0.9)

var _engaged := StandardMaterial3D.new()
var _holding := StandardMaterial3D.new()
var _lines: Array[MeshInstance3D] = []


func _init() -> void:
	for pair in [[_engaged, ENGAGED], [_holding, HOLDING]]:
		var material: StandardMaterial3D = pair[0]
		material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		material.albedo_color = pair[1]
		material.no_depth_test = false


## `threats` holds two numbers each (actor index, 1 if it holds you); `where` the actors' drawn
## places in 3D; `me` your index.
func show_threats(threats: PackedInt32Array, where: Array, me: int) -> void:
	var used := 0
	if me >= 0:
		var mine: Vector3 = where[me]
		for i in range(0, threats.size(), 2):
			var theirs: Vector3 = where[threats[i]]
			var holds := threats[i + 1] == 1
			var line := _line(used)
			used += 1
			var along := mine - theirs
			along.y = 0.0
			line.visible = true
			line.material_override = _holding if holds else _engaged
			line.position = (mine + theirs) / 2.0 + Vector3(0.0, 0.05, 0.0)
			line.rotation.y = atan2(along.x, along.z)
			line.scale = Vector3(0.2 if holds else 0.1, 1.0, along.length())
	for k in range(used, _lines.size()):
		_lines[k].visible = false


func _line(k: int) -> MeshInstance3D:
	while _lines.size() <= k:
		var bar := BoxMesh.new()
		bar.size = Vector3(1.0, 0.02, 1.0)
		var line := MeshInstance3D.new()
		line.mesh = bar
		line.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(line)
		_lines.append(line)
	return _lines[k]
