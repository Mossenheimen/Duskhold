extends Node3D
## What lies on the floor: a small bundle on each tile with something dropped there, so you can
## find it again. From DuskWorld.floor_items(); what it is, the pack's "within reach" panel says.

const Look = preload("res://look.gd")

var _shown := PackedInt32Array()
var _bundles: Array[MeshInstance3D] = []
var _material := StandardMaterial3D.new()
var _mesh := BoxMesh.new()


func _init() -> void:
	_material.albedo_color = Look.LITTER
	_material.roughness = 0.9
	_mesh.size = Vector3(0.32, 0.14, 0.24)


## `tiles` holds (x, y) pairs; nothing is redrawn unless they changed.
func show_floors(tiles: PackedInt32Array) -> void:
	if tiles == _shown:
		return
	_shown = tiles
	var used := 0
	for i in range(0, tiles.size(), 2):
		while _bundles.size() <= used:
			var bundle := MeshInstance3D.new()
			bundle.mesh = _mesh
			bundle.material_override = _material
			add_child(bundle)
			_bundles.append(bundle)
		var bundle := _bundles[used]
		used += 1
		bundle.visible = true
		bundle.position = Vector3(tiles[i] + 0.5, 0.07, tiles[i + 1] + 0.5)
		bundle.rotation.y = float((tiles[i] * 7 + tiles[i + 1] * 13) % 12) * 0.5
	for k in range(used, _bundles.size()):
		_bundles[k].visible = false
