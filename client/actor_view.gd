extends Node3D
## One actor, the character or one of the dead. It faces the way the core says, and a mark over its
## head says what it knows of you: "?" it heard something, "!" it sees you. It leans in as it
## winds up a swing (the tell), lies down when knocked down or asleep ("z"), and stays down,
## darker, when dead. A body set alight glows and lights the ground round it; burnt, it's ash.

const CHARACTER := Color(0.75, 0.32, 0.2)
## Each kind's look, by DuskWorld.actor_kinds() (0 the character, 1 + n creature n): its colour,
## its height, and how far it stoops. The shambler is grey-green and stooped; the ghoul pale,
## lower and crouched, so a glance tells them apart; the revenant tall, upright and dark, the
## character's colour gone bad; the skeleton bone white and upright. A kind with no look here
## borrows the last.
const LOOKS := [
	[CHARACTER, 1.55, 0.0],
	[Color(0.56, 0.6, 0.5), 1.45, -14.0],
	[Color(0.82, 0.78, 0.7), 1.3, -28.0],
	[Color(0.45, 0.24, 0.2), 1.6, -6.0],  # a revenant: you, in your gear, gone dark
	[Color(0.88, 0.86, 0.78), 1.6, -3.0],  # a skeleton: bone white, tall and upright
	[Color(0.22, 0.25, 0.34), 1.8, 4.0],  # a wight: night-blue, the tallest, leaning back
	[Color(0.55, 0.38, 0.24), 1.2, 0.0, true],  # a deer: brown, on four legs (a beast)
	[Color(0.4, 0.4, 0.42), 0.95, -0.3, true],  # a wolf: grey, smaller, its head low and forward
]
const CURIOUS := Color(1.0, 0.75, 0.25)
const HUNTING := Color(1.0, 0.22, 0.15)
const SLEEPING := Color(0.6, 0.72, 1.0)
## Turned by the holy symbol (DuskWorld.actor_alert() 3): it flees you.
const TURNED := Color(1.0, 0.95, 0.7)
const ASLEEP := 3
## A body burning (DuskWorld.actor_remains() 1), burnt (2), and consecrated (4).
const BURNING := 1
const ASH := 2
const HALLOWED := 4
const FIRE := Color(1.0, 0.5, 0.18)
const CINDERS := Color(0.13, 0.12, 0.11)
## How far a burning body's light reaches, in tiles: rules.toml's [burning] light, plus a tile
## for the falloff (as the torch is drawn).
const FIRE_REACH := 5.0

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
var _remains := 0
var _fire: OmniLight3D  # made when it first catches
var _title: Label3D  # a rare's name, over its head
var _beast := false  # on four legs: it falls on its side


## A rare: its name over its head, in the gold of a named thing, and a little bigger than its
## kind, so you know it before it knows you.
func set_title(name: String) -> void:
	if _title == null:
		_title = Label3D.new()
		_title.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		_title.no_depth_test = true
		_title.font_size = 40
		_title.pixel_size = 0.008
		_title.outline_size = 10
		_title.outline_modulate = Color(0.05, 0.04, 0.02)
		_title.modulate = Color(1.0, 0.8, 0.35)
		_title.position.y = _mark.position.y + 0.45
		add_child(_title)
		_body.scale = Vector3.ONE * 1.12
	_title.text = name


## Kind 0 is the character, 1 + n creature n (DuskWorld.actor_kinds()).
func setup(kind: int) -> void:
	var look: Array = LOOKS[clampi(kind, 0, LOOKS.size() - 1)]
	var colour: Color = look[0]
	var tall: float = look[1]
	_colour = colour
	_body = Node3D.new()
	_stoop = look[2]
	_body.rotation_degrees.x = _stoop
	_pitch = _stoop
	add_child(_body)
	var body := _body
	if look.size() > 3 and look[3]:
		_beast = true
		_build_beast(colour, tall / 1.2, look[2])
		return
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


## 0 unaware, 1 curious, 2 hunting, 3 turned (DuskWorld.actor_alert()). A new mark pops.
func show_alert(level: int, delta: float) -> void:
	if level != _alert:
		_alert = level
		_pop = 1.0
		_show_mark()
	_pop = maxf(0.0, _pop - delta * 4.0)
	_mark.scale = Vector3.ONE * (1.0 + _pop * 0.6)


## The mark over its head: "z" asleep, else what it knows of you ("+" turned: it flees you).
func _show_mark() -> void:
	if _state == ASLEEP:
		_mark.text = "z"
		_mark.modulate = SLEEPING
	else:
		_mark.text = ["", "?", "!", "+"][_alert]
		_mark.modulate = [Color.WHITE, CURIOUS, HUNTING, TURNED][_alert]


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
	if _beast:
		# A beast doesn't lean in or pitch over: down, it rolls onto its side.
		var roll := 85.0 if state != 0 else 0.0
		var now := _body.rotation_degrees.z
		_body.rotation_degrees.z = lerpf(now, roll, 1.0 - exp(-16.0 * delta)) if delta > 0.0 else roll
		return
	_pitch = lerpf(_pitch, target, 1.0 - exp(-16.0 * delta)) if delta > 0.0 else target
	_body.rotation_degrees.x = _pitch


## What's become of its body (DuskWorld.actor_remains(): 0 nothing, 1 burning, 2 ash, 4 hallowed).
## Burning, it glows and throws a flickering light, with no shadows (a pile of them would cost too
## much); burnt, it's a low, dark smear; consecrated, it lies pale.
func show_remains(remains: int, _delta: float) -> void:
	if remains != _remains:
		_remains = remains
		if remains == HALLOWED:
			_skin.albedo_color = _colour.darkened(0.55).lerp(TURNED, 0.35)
		elif remains == BURNING:
			_skin.albedo_color = _colour.darkened(0.7)
			_skin.emission_enabled = true
			_skin.emission = FIRE
			if _fire == null:
				_fire = OmniLight3D.new()
				_fire.light_color = FIRE
				_fire.omni_range = FIRE_REACH
				_fire.omni_attenuation = 1.4
				_fire.position = Vector3(0.0, 0.5, 0.0)
				add_child(_fire)
			_fire.visible = true
		elif remains == ASH:
			_skin.emission_enabled = false
			_skin.albedo_color = CINDERS
			_body.scale = Vector3(1.15, 0.85, 0.35)  # lying, local z is up: flattened
			if _fire != null:
				_fire.visible = false
	if _remains == BURNING:
		# Each body flickers on its own beat (its place in the world), not all in step.
		var now := Time.get_ticks_msec() / 1000.0 + position.x * 0.37 + position.z * 0.61
		var flicker := 1.0 + 0.12 * sin(now * 11.0) + 0.07 * sin(now * 27.0)
		_fire.light_energy = 2.2 * flicker
		_skin.emission_energy_multiplier = 1.4 * flicker


## A beast's body (a LOOKS entry marked so, at its height over a deer's, its head lifted by the
## entry's third number): four legs, a long body and a raised head, so it
## reads as no person. Knocked down or dead, it rolls onto its side (`show_pose`).
func _build_beast(colour: Color, size: float, lift: float) -> void:
	var body := _body
	body.scale = Vector3.ONE * size  # a deer's is 1; a wolf's smaller
	_skin = _material(colour)
	var capsule := CapsuleMesh.new()
	capsule.radius = 0.2
	capsule.height = 1.0
	var torso := MeshInstance3D.new()
	torso.mesh = capsule
	torso.rotation_degrees.x = 90.0  # lying along its facing (-z)
	torso.position.y = 0.72
	torso.material_override = _skin
	body.add_child(torso)
	var parts := [
		[Vector3(0.12, 0.2, 0.3), Vector3(0.0, 1.05 + lift, -0.58 + lift * 0.4)],  # the head (a deer's high)
		[Vector3(0.1, 0.36, 0.12), Vector3(0.0, 0.88 + lift * 0.5, -0.45)],  # the neck
		[Vector3(0.07, 0.56, 0.07), Vector3(0.11, 0.28, -0.32)],  # the legs
		[Vector3(0.07, 0.56, 0.07), Vector3(-0.11, 0.28, -0.32)],
		[Vector3(0.07, 0.56, 0.07), Vector3(0.11, 0.28, 0.32)],
		[Vector3(0.07, 0.56, 0.07), Vector3(-0.11, 0.28, 0.32)],
	]
	for part in parts:
		var box := BoxMesh.new()
		box.size = part[0]
		var mesh := MeshInstance3D.new()
		mesh.mesh = box
		mesh.position = part[1]
		mesh.material_override = _skin
		body.add_child(mesh)
	var tail := BoxMesh.new()
	tail.size = Vector3(0.08, 0.1, 0.08)
	var scut := MeshInstance3D.new()
	scut.mesh = tail
	scut.position = Vector3(0.0, 0.8, 0.52)
	scut.material_override = _material(Color(0.92, 0.9, 0.85))  # the white scut
	body.add_child(scut)
	_mark = Label3D.new()
	_mark.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	_mark.no_depth_test = true
	_mark.font_size = 72
	_mark.pixel_size = 0.008
	_mark.outline_size = 16
	_mark.outline_modulate = Color(0.05, 0.05, 0.05)
	_mark.position.y = 1.6 * size
	add_child(_mark)
