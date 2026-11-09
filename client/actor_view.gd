extends Node3D
## One actor, the character or a shambler. It faces the way the core says, and a mark over its
## head says what it knows of you: "?" it heard something, "!" it sees you. It leans in as it
## winds up a swing (the tell), lies down when knocked down or asleep ("z"), and stays down,
## darker, when dead.

const CHARACTER := Color(0.75, 0.32, 0.2)
const SHAMBLER := Color(0.56, 0.6, 0.5)
const CURIOUS := Color(1.0, 0.75, 0.25)
const HUNTING := Color(1.0, 0.22, 0.15)
const SLEEPING := Color(0.6, 0.72, 1.0)
const ASLEEP := 3

var _turn := 0.0
var _body: Node3D  # pivots at the feet
var _skin: StandardMaterial3D
var _colour := Color.WHITE
var _stoop := 0.0  # how far forward it stands, in degrees
var _pitch := 0.0
var _state := 0
var _mark: Label3D
var _alert := 0
var _pop := 0.0


## Kind 0 is the character, 1 a shambler (DuskWorld.actor_kinds()).
func setup(kind: int) -> void:
	var colour := CHARACTER if kind == 0 else SHAMBLER
	var tall := 1.55 if kind == 0 else 1.45
	_colour = colour
	_body = Node3D.new()
	_stoop = -14.0 if kind == 1 else 0.0  # the dead stoop
	_body.rotation_degrees.x = _stoop
	_pitch = _stoop
	add_child(_body)
	var body := _body
	var capsule := CapsuleMesh.new()
	capsule.radius = 0.22
	capsule.height = tall
	var torso := MeshInstance3D.new()
	torso.mesh = capsule
	torso.position.y = tall / 2.0
	_skin = _material(colour)
	torso.material_override = _skin
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
		_show_mark()
	_pop = maxf(0.0, _pop - delta * 4.0)
	_mark.scale = Vector3.ONE * (1.0 + _pop * 0.6)


## The mark over its head: "z" asleep, else what it knows of you.
func _show_mark() -> void:
	if _state == ASLEEP:
		_mark.text = "z"
		_mark.modulate = SLEEPING
	else:
		_mark.text = ["", "?", "!"][_alert]
		_mark.modulate = [Color.WHITE, CURIOUS, HUNTING][_alert]


func _material(colour: Color) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.albedo_color = colour
	material.roughness = 0.8
	return material


## Its state (0 standing, 1 knocked down, 2 dead, 3 asleep, from DuskWorld.actor_states()) and
## how far through a swing's wind-up it is (0 to 1; -1 when it isn't winding one up).
func show_pose(state: int, wind_up: float, delta: float) -> void:
	if state != _state:
		_state = state
		_skin.albedo_color = _colour.darkened(0.55) if state == 2 else _colour
		_mark.visible = state == 0 or state == ASLEEP
		_show_mark()
	var target := _stoop
	if state != 0:
		target = -88.0  # flat on the ground
	elif wind_up >= 0.0:
		target = _stoop - 22.0 * wind_up  # leaning in to strike
	_pitch = lerpf(_pitch, target, 1.0 - exp(-16.0 * delta)) if delta > 0.0 else target
	_body.rotation_degrees.x = _pitch
