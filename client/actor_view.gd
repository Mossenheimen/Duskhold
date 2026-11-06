extends Node3D
## One actor, the character or a shambler. It faces the way the core says, and a mark over its
## head says what it knows of you: "?" it heard something, "!" it sees you.

const CHARACTER := Color(0.75, 0.32, 0.2)
const SHAMBLER := Color(0.56, 0.6, 0.5)
const CURIOUS := Color(1.0, 0.75, 0.25)
const HUNTING := Color(1.0, 0.22, 0.15)

var _turn := 0.0
var _mark: Label3D
var _alert := 0
var _pop := 0.0


## Kind 0 is the character, 1 a shambler (DuskWorld.actor_kinds()).
func setup(kind: int) -> void:
	var colour := CHARACTER if kind == 0 else SHAMBLER
	var tall := 1.55 if kind == 0 else 1.45
	var body := Node3D.new()  # pivots at the feet
	if kind == 1:
		body.rotation_degrees.x = -14.0  # the dead stoop
	add_child(body)
	var capsule := CapsuleMesh.new()
	capsule.radius = 0.22
	capsule.height = tall
	var torso := MeshInstance3D.new()
	torso.mesh = capsule
	torso.position.y = tall / 2.0
	torso.material_override = _material(colour)
	body.add_child(torso)
	# A nose, to show which way it faces (forward is -z).
	var box := BoxMesh.new()
	box.size = Vector3(0.14, 0.1, 0.2)
	var nose := MeshInstance3D.new()
	nose.mesh = box
	nose.position = Vector3(0.0, tall - 0.28, -0.2)
	nose.material_override = _material(colour.darkened(0.4))
	body.add_child(nose)
	_mark = Label3D.new()
	_mark.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	_mark.no_depth_test = true
	_mark.font_size = 72
	_mark.pixel_size = 0.008
	_mark.outline_size = 16
	_mark.outline_modulate = Color(0.05, 0.05, 0.05)
	_mark.position.y = tall + 0.45
	add_child(_mark)


## Turns toward a facing, in eighths of a turn clockwise from north. A delta of 0 snaps.
func face(eighths: int, delta: float) -> void:
	var target := -eighths * PI / 4.0
	_turn = lerp_angle(_turn, target, 1.0 - exp(-14.0 * delta)) if delta > 0.0 else target
	rotation.y = _turn


## 0 unaware, 1 curious, 2 hunting (DuskWorld.actor_alert()). A new mark pops.
func show_alert(level: int, delta: float) -> void:
	if level != _alert:
		_alert = level
		_pop = 1.0
		_mark.text = ["", "?", "!"][level]
		_mark.modulate = [Color.WHITE, CURIOUS, HUNTING][level]
	_pop = maxf(0.0, _pop - delta * 4.0)
	_mark.scale = Vector3.ONE * (1.0 + _pop * 0.6)


func _material(colour: Color) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.albedo_color = colour
	material.roughness = 0.8
	return material
