extends Node3D
## The spells lasting on their spots (DuskWorld.spell_zones(), the core's spell.rs): Light as a
## pale light over its spot, Silence as a ring as far as it reaches, Fog as a low cloud. Only the
## look: what each does is the core's.

const Look = preload("res://look.gd")

const LIGHT := 0
const SILENCE := 1
const FOG := 2
## Five numbers a zone: kind, x, y, reach, ticks left.
const ZONE := 5

var _ring := ShaderMaterial.new()
var _fog := StandardMaterial3D.new()
var _mote := StandardMaterial3D.new()
var _shown := PackedInt32Array()  # the zones drawn, without their ticks left
var _nodes: Array[Node3D] = []


func _init() -> void:
	# A Silence's edge: the noise ring's shader, held at its full reach.
	_ring.shader = preload("res://shaders/noise_ring.gdshader")
	_ring.set_shader_parameter("color", Color(Look.ARCANE, 0.85))
	_ring.set_shader_parameter("width", 0.14)
	_fog.albedo_color = Look.FOG
	_fog.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	_fog.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	_fog.cull_mode = BaseMaterial3D.CULL_DISABLED
	_mote.albedo_color = Look.ARCANE
	_mote.emission_enabled = true
	_mote.emission = Look.ARCANE
	_mote.emission_energy_multiplier = 3.0


## Each frame: redraw when the zones change.
func show_zones(zones: PackedInt32Array) -> void:
	var now := PackedInt32Array()
	for i in range(0, zones.size() - ZONE + 1, ZONE):
		now.append_array(zones.slice(i, i + 4))
	if now == _shown:
		return
	_shown = now
	clear()
	for i in range(0, now.size(), 4):
		var node := _make(now[i], now[i + 1], now[i + 2], now[i + 3])
		if node != null:
			add_child(node)
			_nodes.append(node)


func clear() -> void:
	for node in _nodes:
		node.queue_free()
	_nodes.clear()
	_shown = PackedInt32Array()


func _make(kind: int, x: int, y: int, reach: int) -> Node3D:
	var centre := Vector3(x + 0.5, 0.0, y + 0.5)
	match kind:
		LIGHT:
			var light := OmniLight3D.new()
			light.light_color = Look.ARCANE
			light.light_energy = 2.4
			light.omni_range = reach + 1.0
			light.omni_attenuation = 1.2
			light.shadow_enabled = true
			light.position = centre + Vector3(0.0, 1.6, 0.0)
			var mote := MeshInstance3D.new()
			var ball := SphereMesh.new()
			ball.radius = 0.12
			ball.height = 0.24
			mote.mesh = ball
			mote.material_override = _mote
			mote.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			light.add_child(mote)
			return light
		SILENCE:
			var plane := PlaneMesh.new()
			plane.size = Vector2.ONE * (reach * 2.0 + 0.5)
			var ring := MeshInstance3D.new()
			ring.mesh = plane
			ring.material_override = _ring
			ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			ring.position = centre + Vector3(0.0, 0.04, 0.0)
			ring.set_instance_shader_parameter("reach", float(reach))
			ring.set_instance_shader_parameter("age", 0.3)  # spread out, not yet fading
			return ring
		FOG:
			var cloud := MeshInstance3D.new()
			var drum := CylinderMesh.new()
			drum.top_radius = reach + 0.5
			drum.bottom_radius = reach + 0.5
			drum.height = 1.4
			cloud.mesh = drum
			cloud.material_override = _fog
			cloud.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			cloud.position = centre + Vector3(0.0, 0.7, 0.0)
			return cloud
	return null
