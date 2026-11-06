extends Node3D
## Noise rings (Mark of the Ninja's): each sound the core reports spreads out to how far it
## carries, then fades.

const LIFE := 0.9  # seconds of game time

var _material := ShaderMaterial.new()
var _rings: Array[MeshInstance3D] = []
var _ages: Array[float] = []


func _init() -> void:
	_material.shader = preload("res://shaders/noise_ring.gdshader")


## A sound at tile (x, y) that carries `reach` tiles.
func add(x: float, y: float, reach: float) -> void:
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE * (reach * 2.0 + 0.5)
	var ring := MeshInstance3D.new()
	ring.mesh = plane
	ring.material_override = _material
	ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	ring.position = Vector3(x + 0.5, 0.03, y + 0.5)
	ring.set_instance_shader_parameter("reach", reach)
	ring.set_instance_shader_parameter("age", 0.0)
	add_child(ring)
	_rings.append(ring)
	_ages.append(0.0)


## Ages the rings by `delta` seconds of game time; the old ones go.
func advance(delta: float) -> void:
	for i in range(_rings.size() - 1, -1, -1):
		_ages[i] += delta / LIFE
		if _ages[i] >= 1.0:
			_rings[i].queue_free()
			_rings.remove_at(i)
			_ages.remove_at(i)
		else:
			_rings[i].set_instance_shader_parameter("age", _ages[i])


func clear() -> void:
	for ring in _rings:
		ring.queue_free()
	_rings.clear()
	_ages.clear()
