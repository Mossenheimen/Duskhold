extends Node3D
## One actor, the character or one of the dead. It faces the way the core says, and a mark over its
## head says what it knows of you: "?" it heard something, "!" it sees you. It leans in as it
## winds up a swing (the tell), rears back for a heavy one, kneels to pray and bends to pick, and
## glows in a power's colour while it's called on (POSES). It lies down when knocked down or asleep
## ("z"), and stays down, darker, when dead. A body set alight glows and lights the ground round
## it; burnt, it's ash.

const Look = preload("res://look.gd")
const CHARACTER := Look.YOU
## Each kind's look, by DuskWorld.actor_kinds() (0 the character, 1 + n creature n): its colour,
## its height, and how far it stoops. The shambler is grey-green and stooped; the ghoul pale,
## lower and crouched, so a glance tells them apart; the revenant tall, upright and dark, the
## character's colour gone bad; the skeleton bone white and upright. A kind with no look here
## borrows the last.
const LOOKS := [
	[CHARACTER, 1.55, 0.0],
	[Look.SHAMBLER, 1.45, -14.0],
	[Look.GHOUL, 1.3, -28.0],
	[Look.REVENANT, 1.6, -6.0],  # a revenant: you, in your gear, gone dark
	[Look.SKELETON, 1.6, -3.0],  # a skeleton: bone white, tall and upright
	[Look.WIGHT, 1.8, 4.0],  # a wight: night-blue, the tallest, leaning back
	[Look.DEER, 1.2, 0.0, true],  # a deer: brown, on four legs (a beast)
	[Look.WOLF, 0.95, -0.3, true],  # a wolf: grey, smaller, its head low and forward
]
const SLEEPING := Look.REST
## Turned by the holy symbol (DuskWorld.actor_alert() 3): it flees you.
const TURNED := Look.HOLY
## An actor's state and what's become of a body, by name (codes.gd).
const State = preload("res://codes.gd").State
const Remains = preload("res://codes.gd").Remains
const Action = preload("res://codes.gd").Action
const Effects = preload("res://effects.gd")
## No glow: a colour with nothing to it.
const NO_GLOW := Color(0, 0, 0, 0)
## How a body holds itself through an act (codes.gd's Action), as it's done: [degrees it leans,
## back from its stoop (forward is -), how low it goes (1 standing, under 1 kneeling or
## crouching), the glow of a power (NO_GLOW for an everyday act)]. A swing and a heavy swing move
## through their wind-up instead (`show_pose`).
const POSES := {
	Action.CAST: [8.0, 1.0, Look.ARCANE],  # a Wizard's words, upright and leaning back
	Action.TURN: [14.0, 1.0, Look.HOLY],  # the holy symbol held up high
	Action.PRAY: [-14.0, 0.68, Look.HOLY],  # kneeling, head bowed
	Action.CONSECRATE: [-30.0, 0.72, Look.HOLY],  # kneeling over the body
	Action.WARD: [-10.0, 1.0, Look.HOLY],  # a hand on the door
	Action.HEAL: [-12.0, 1.0, Look.GOOD],  # Mending: bent over the wound
	Action.SECOND_WIND: [-28.0, 0.9, Look.ENDURANCE],  # hands on knees, catching its breath
	Action.PICK: [-35.0, 0.85, NO_GLOW],  # Foraging: bent to the plant
	Action.BUTCHER: [-25.0, 0.75, NO_GLOW],  # kneeling at the carcass
	Action.LAY_FIRE: [-20.0, 0.75, NO_GLOW],  # crouched at the hearth
	Action.LIGHT_FIRE: [-20.0, 0.75, NO_GLOW],
	Action.BANDAGE: [-15.0, 1.0, NO_GLOW],  # First Aid: bent to the wound
	Action.BOARD: [-8.0, 1.0, NO_GLOW],  # Carpentry: at the door, hammering
	Action.UNBOARD: [-8.0, 1.0, NO_GLOW],
	Action.CRAFT: [-15.0, 1.0, NO_GLOW],  # bent over what's being made
}
const FIRE := Look.EMBER
const CINDERS := Look.ASH
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
var _look := 0  # Look.version as the mark was coloured: colour-blind mode recolours it
var _remains := 0
var _fire: OmniLight3D  # made when it first catches
var _title: Label3D  # a rare's name, over its head
var _beast := false  # on four legs: it falls on its side
var _size := 1.0  # the body's own scale (a rare's is bigger)
var _low := 1.0  # how low it goes now: 1 standing, eased toward a pose's
var _glow := NO_GLOW  # a power's glow, while it's called on
var _aura: CPUParticles3D  # motes rising round it while it glows, made the first time


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
		_title.outline_modulate = Look.OUTLINE
		_title.modulate = Look.RARE
		_title.position.y = _mark.position.y + 0.45
		add_child(_title)
		_size = 1.12
		_body.scale = Vector3.ONE * _size
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
	_mark.outline_modulate = Look.OUTLINE
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
		_pop = 0.0 if Look.reduce_motion else 1.0
		_show_mark()
	elif _look != Look.version:
		_show_mark()
	_pop = maxf(0.0, _pop - delta * 4.0)
	_mark.scale = Vector3.ONE * (1.0 + _pop * 0.6)


## The mark over its head: "z" asleep, else what it knows of you ("+" turned: it flees you).
func _show_mark() -> void:
	if _state == State.ASLEEP:
		_mark.text = "z"
		_mark.modulate = SLEEPING
	else:
		_mark.text = ["", "?", "!", "+"][_alert]
		_mark.modulate = [Color.WHITE, Look.ALERT, Look.DANGER, TURNED][_alert]
	_look = Look.version


func _material(colour: Color) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.albedo_color = colour
	material.roughness = 0.8
	return material


## Its state (codes.gd's State, from DuskWorld.actor_states()) and the act it's doing (codes.gd's
## Action, from actor_actions()) with how far through its wind-up it is (0 to 1): its pose.
func show_pose(state: int, kind: int, progress: float, delta: float) -> void:
	if state != _state:
		_state = state
		_skin.albedo_color = _colour.darkened(0.55) if state == State.DEAD else _colour
		_mark.visible = state == State.STANDING or state == State.ASLEEP
		_show_mark()
	var target := _stoop
	var low := 1.0
	var glow := NO_GLOW
	if state != State.STANDING:
		target = -88.0  # flat on the ground
	elif kind == Action.SWING and progress < 1.0:
		target = _stoop - 22.0 * progress  # leaning in to strike
	elif kind == Action.HEAVY and progress < 1.0:
		# Its weight behind it: it rears back first, then throws itself in, further than a swing.
		target = _stoop + (16.0 * progress / 0.6 if progress < 0.6 else 16.0 - 46.0 * (progress - 0.6) / 0.4)
	elif POSES.has(kind):
		var pose: Array = POSES[kind]
		target = _stoop + pose[0]
		low = pose[1]
		glow = pose[2]
	if _beast:
		# A beast doesn't lean in or pitch over: down, it rolls onto its side.
		var roll := 85.0 if state != 0 else 0.0
		var now := _body.rotation_degrees.z
		_body.rotation_degrees.z = lerpf(now, roll, 1.0 - exp(-16.0 * delta)) if delta > 0.0 else roll
		return
	var blend := 1.0 - exp(-16.0 * delta) if delta > 0.0 else 1.0
	_pitch = lerpf(_pitch, target, blend)
	_body.rotation_degrees.x = _pitch
	if state == State.STANDING:
		_low = lerpf(_low, low, blend)
		_body.scale = Vector3(_size, _size * _low, _size)
	_show_glow(glow)


## A power's glow (a colour, or NO_GLOW): its body lit in it, gently throbbing (steady with less
## motion), and motes rising round it, both gone when the power's done.
func _show_glow(colour: Color) -> void:
	if colour == _glow:
		if colour.a > 0.0:
			var throb := 0.0 if Look.reduce_motion else 0.2 * sin(Time.get_ticks_msec() / 1000.0 * 6.0)
			_skin.emission_energy_multiplier = 0.45 + throb
		return
	_glow = colour
	if colour.a == 0.0:
		if _remains != Remains.BURNING:
			_skin.emission_enabled = false
		if _aura != null:
			_aura.emitting = false
		return
	_skin.emission_enabled = true
	_skin.emission = colour
	if _aura == null:
		_aura = CPUParticles3D.new()
		_aura.amount = 7 if Look.reduce_motion else 14
		_aura.lifetime = 0.9
		_aura.mesh = Effects.dot()
		_aura.material_override = Effects.mote()
		_aura.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		_aura.emission_shape = CPUParticles3D.EMISSION_SHAPE_SPHERE
		_aura.emission_sphere_radius = 0.35
		_aura.direction = Vector3.UP
		_aura.spread = 25.0
		_aura.initial_velocity_min = 0.4
		_aura.initial_velocity_max = 0.8
		_aura.gravity = Vector3(0, 0.3, 0)
		_aura.scale_amount_min = 0.12
		_aura.scale_amount_max = 0.22
		var fade := Gradient.new()
		fade.set_color(0, Color.WHITE)
		fade.set_color(1, Color(1, 1, 1, 0))
		_aura.color_ramp = fade
		_aura.position = Vector3(0, 0.9, 0)
		add_child(_aura)
	_aura.color = colour
	_aura.emitting = true


## What's become of its body (DuskWorld.actor_remains(): 0 nothing, 1 burning, 2 ash, 4 hallowed).
## Burning, it glows and throws a flickering light, with no shadows (a pile of them would cost too
## much); burnt, it's a low, dark smear; consecrated, it lies pale.
func show_remains(remains: int, _delta: float) -> void:
	if remains != _remains:
		_remains = remains
		if remains == Remains.HALLOWED:
			_skin.albedo_color = _colour.darkened(0.55).lerp(TURNED, 0.35)
		elif remains == Remains.BURNING:
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
		elif remains == Remains.ASH:
			_skin.emission_enabled = false
			_skin.albedo_color = CINDERS
			_body.scale = Vector3(1.15, 0.85, 0.35)  # lying, local z is up: flattened
			if _fire != null:
				_fire.visible = false
	if _remains == Remains.BURNING:
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
	scut.material_override = _material(Look.SCUT)  # the white scut
	body.add_child(scut)
	_mark = Label3D.new()
	_mark.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	_mark.no_depth_test = true
	_mark.font_size = 72
	_mark.pixel_size = 0.008
	_mark.outline_size = 16
	_mark.outline_modulate = Look.OUTLINE
	_mark.position.y = 1.6 * size
	add_child(_mark)
