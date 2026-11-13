extends Node3D
## What a power or a spell looks like as it happens (VISUALS.md §12, Effects): short bursts of
## glowing motes in the colour of what they mean, a second or so each.
## - a spell, as it lands: a Light's pale motes rising, a Silence's ring drawing in, a Fog's puff,
##   a Fireball's bolt, flash, flames, embers and smoke;
## - the words broken: the motes fall away;
## - turning the dead: a holy wave out to as far as it reaches;
## - Mending, prayer and consecration: motes lifting from the body (good, holy);
## - a ward: holy motes on the door;
## - the Second Wind: a breath blown out round you;
## - a skill or a level rising: gold.
##
## And the fight's (main.gd's _fight_effect, where you'd see them):
## - a blow landing: blood, a little or (a wound) more; bone chips off the bloodless;
## - your shield taking a blow: sparks; armour turning one: a few;
## - a heavy blow felling one: dust round its feet; a sneak attack: a sharp burst;
## - bone turning a blade, or shattering under a blunt one: chips;
## - the dead getting up: grave dust; a revenant rising: its dark; a wight eating your light: smoke;
## - a door or the boards on it giving way: splinters.
##
## Only the look: the core says what happened (its events, read by main.gd). With fewer flashes
## and less motion (Look.reduce_motion), half as many motes and no flash of light.

const Look = preload("res://look.gd")
const Spell = preload("res://codes.gd").Spell

## How fast a fireball flies to its spot, in tiles a second: quick, but seen.
const BOLT_SPEED := 24.0

## A mote: a soft dot, drawn additively (light: magic, sparks, embers, glowing by itself) or
## blended and lit by the world's light (smoke, fog, dust, blood: dark at night unless a light
## reaches it). Made once, shared by every burst and every actor's glow (`mote`, `dot`).
static var _glow: StandardMaterial3D
static var _haze: StandardMaterial3D
static var _dot: QuadMesh
var _ring := ShaderMaterial.new()  # the noise ring's shader, for a wave or a ring drawing in


func _init() -> void:
	_ring.shader = preload("res://shaders/noise_ring.gdshader")
	_ring.set_shader_parameter("width", 0.35)


## A mote's material: glowing light, or (`haze`) blended smoke and fog. Each mote's colour is its
## particle's.
static func mote(haze := false) -> StandardMaterial3D:
	if _glow == null:
		# A dot that's brightest in its middle and gone at its edge, so motes read as light.
		var soft := Gradient.new()
		soft.set_color(0, Color.WHITE)
		soft.set_color(1, Color(1, 1, 1, 0))
		var texture := GradientTexture2D.new()
		texture.gradient = soft
		texture.fill = GradientTexture2D.FILL_RADIAL
		texture.fill_from = Vector2(0.5, 0.5)
		texture.fill_to = Vector2(1.0, 0.5)
		texture.width = 32
		texture.height = 32
		_glow = StandardMaterial3D.new()
		_haze = StandardMaterial3D.new()
		for material: StandardMaterial3D in [_glow, _haze]:
			material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			material.vertex_color_use_as_albedo = true
			material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			material.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
			material.billboard_keep_scale = true
			material.albedo_texture = texture
		_glow.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
		_haze.shading_mode = BaseMaterial3D.SHADING_MODE_PER_VERTEX  # lit, as the world is
	return _haze if haze else _glow


## A mote's shape: a unit square, scaled per mote.
static func dot() -> QuadMesh:
	if _dot == null:
		_dot = QuadMesh.new()
		_dot.size = Vector2.ONE
	return _dot


## A spell landing on tile (x, y): `kind` (codes.gd's Spell), `reach` how far it reaches, in
## tiles, and `from` where the caster stands (a fireball flies from there).
func spell(kind: int, x: int, y: int, reach: int, from: Vector3) -> void:
	var at := _tile(x, y)
	match kind:
		Spell.LIGHT:
			_burst(at + Vector3(0, 1.2, 0), Look.ARCANE, [28, 1.3, 1.2, 0.7, 0.5, 0.26])
		Spell.SILENCE:
			_ring_in(at, maxf(reach, 1.0), Look.ARCANE)
			_burst(at + Vector3(0, 0.5, 0), Look.ARCANE, [14, 0.8, 0.6, 0.0, maxf(reach, 1.0) * 0.6, 0.18])
		Spell.FOG:
			_burst(at + Vector3(0, 0.5, 0), Look.FOG, [18, 1.6, 1.4, 0.15, maxf(reach, 1.0) * 0.5, 1.4], true)
		Spell.FIREBALL:
			_fireball(from + Vector3(0, 1.1, 0), at, maxf(reach, 1.0))


## The words broken by a blow, at the caster: the gathered motes fall away.
func spell_broke(at: Vector3) -> void:
	_burst(at + Vector3(0, 1.1, 0), Look.ARCANE, [12, 0.7, 1.0, -3.0, 0.3, 0.16])


## Turning the dead: a holy wave from the Cleric out to `reach` tiles, and the symbol's light
## lifting off it.
func turn(at: Vector3, reach: float) -> void:
	_wave(at, reach, Look.HOLY, 0.9)
	_burst(at + Vector3(0, 1.4, 0), Look.HOLY, [20, 1.0, 1.4, 0.8, 0.2, 0.24])


## Motes lifting from a body: Mending's (Look.GOOD), a prayer's or a consecration's (Look.HOLY).
func rise(at: Vector3, colour: Color) -> void:
	_burst(at + Vector3(0, 0.6, 0), colour, [18, 1.3, 0.6, 1.1, 0.35, 0.22])


## A ward on the door or window on tile (x, y)'s `side` (0 north, 1 west): holy motes over it.
func ward(x: int, y: int, side: int) -> void:
	var on := Vector3(x + 0.5, 1.0, y) if side == 0 else Vector3(x, 1.0, y + 0.5)
	_burst(on, Look.HOLY, [22, 1.1, 0.5, 0.3, 0.45, 0.22])


## The Second Wind: a breath blown out round you, in breath's colour.
func second_wind(at: Vector3) -> void:
	var gust := _burst(at + Vector3(0, 0.9, 0), Look.ENDURANCE, [26, 0.8, 0.4, 0.5, 0.3, 0.24])
	gust.radial_accel_min = 5.0
	gust.radial_accel_max = 8.0


## A skill rising a level (or mastered): gold, round your feet and up. A level of your own: more,
## and a ring of it.
func gold(at: Vector3, level: bool) -> void:
	_burst(at + Vector3(0, 0.2, 0), Look.GOLD, [36 if level else 14, 1.2, 1.6, 0.4, 0.3, 0.22])
	if level:
		_wave(at, 1.6, Look.GOLD, 0.7)


## A blow landing on someone at `at`: blood flicked off it, falling, more for a wound; off one
## with no blood (`bloodless`: a skeleton), bone chips instead.
func hit(at: Vector3, bloodless: bool, wounded: bool) -> void:
	var n := 14 if wounded else 6
	if bloodless:
		bone(at, false)
	else:
		_burst(at + Vector3(0, 1.0, 0), Look.BLOOD, [n, 0.6, 2.2, -6.0, 0.15, 0.13], true)


## Your shield taking a blow: sparks off its face, toward `from` (the one who struck).
func block(at: Vector3, from: Vector3) -> void:
	var face := at.lerp(from, 0.35) + Vector3(0, 1.0, 0)
	_burst(face, Look.YOURS, [16, 0.35, 3.5, -5.0, 0.08, 0.1])


## Armour turning a blow: a few sparks off it.
func glance(at: Vector3) -> void:
	_burst(at + Vector3(0, 1.1, 0), Look.YOURS, [7, 0.3, 2.8, -5.0, 0.1, 0.09])


## A heavy blow felling one: dust thrown up round its feet as it goes down, and a ring of it.
func fell(at: Vector3) -> void:
	_burst(at + Vector3(0, 0.15, 0), Look.LITTER, [16, 0.9, 1.6, -0.5, 0.35, 0.5], true)
	_wave(at, 1.2, Look.LITTER, 0.5)


## A sneak attack: a sharp burst where it lands, out fast and gone.
func sneak_attack(at: Vector3) -> void:
	var cut := _burst(at + Vector3(0, 1.0, 0), Look.YOURS, [18, 0.3, 1.0, 0.0, 0.1, 0.12])
	cut.radial_accel_min = 14.0
	cut.radial_accel_max = 20.0


## Bone chips off one of the bloodless dead: a blade turned (a few), or bone shattered under a
## blunt blow (`shattered`: more).
func bone(at: Vector3, shattered: bool) -> void:
	_burst(at + Vector3(0, 1.0, 0), Look.SKELETON, [16 if shattered else 6, 0.7, 2.6, -7.0, 0.15, 0.11], true)


## The dead getting up again: grave dust rising off the body.
func got_up(at: Vector3) -> void:
	_burst(at + Vector3(0, 0.2, 0), Look.LITTER, [14, 1.4, 0.8, 0.5, 0.4, 0.5], true)


## A revenant rising from your old body: the dark coming off it, and a ring of it.
func risen(at: Vector3) -> void:
	_burst(at + Vector3(0, 0.6, 0), Look.REVENANT, [24, 1.6, 0.9, 0.9, 0.4, 0.4], true)
	_wave(at, 2.0, Look.REVENANT, 0.9)


## A wight putting out the light you held up: a curl of smoke where it was.
func light_eaten(at: Vector3) -> void:
	_burst(at + Vector3(0, 1.4, 0), Look.ASH, [12, 1.6, 0.4, 0.8, 0.1, 0.45], true)


## A door, or the boards on it, giving way at `on`: splinters, in the wood's colour.
func splinters(on: Vector3, wood: Color) -> void:
	_burst(on, wood, [18, 0.8, 2.6, -7.0, 0.3, 0.12], true)


## A fireball: a bolt from the hand to the spot; there, a flash, flames, embers and smoke as far as
## it reaches.
func _fireball(from: Vector3, to: Vector3, reach: float) -> void:
	var bolt := _burst(from, Look.FIRE, [12, 0.25, 0.3, 0.0, 0.08, 0.5], false, false)
	bolt.one_shot = false  # a trail, while it flies
	bolt.explosiveness = 0.0
	var flight := maxf(from.distance_to(to) / BOLT_SPEED, 0.08)
	var fly := create_tween()
	fly.tween_property(bolt, "position", to + Vector3(0, 0.6, 0), flight)
	fly.tween_callback(func() -> void:
		bolt.emitting = false
		get_tree().create_timer(0.4).timeout.connect(bolt.queue_free)
		_explode(to, reach))


func _explode(at: Vector3, reach: float) -> void:
	_burst(at + Vector3(0, 0.5, 0), Look.FIRE, [30, 0.5, 2.6, 0.5, reach * 0.45, 0.7])
	_burst(at + Vector3(0, 0.4, 0), Look.EMBER, [40, 1.0, 4.0, -3.0, reach * 0.3, 0.24])
	_burst(at + Vector3(0, 0.8, 0), Look.ASH, [14, 1.8, 0.8, 0.6, reach * 0.5, 1.6], true)
	_wave(at, reach, Look.EMBER, 0.45)
	if Look.reduce_motion:
		return
	var flash := OmniLight3D.new()
	flash.light_color = Look.FIRE
	flash.omni_range = reach + 4.0
	flash.light_energy = 6.0
	flash.position = at + Vector3(0, 1.2, 0)
	add_child(flash)
	var fade := create_tween()
	fade.tween_property(flash, "light_energy", 0.0, 0.45)
	fade.tween_callback(flash.queue_free)


## A ring out from `at` to `reach` tiles, over `life` seconds: the noise ring's way, in `colour`.
func _wave(at: Vector3, reach: float, colour: Color, life: float) -> void:
	var ring := _ring_mesh(at, reach, colour)
	var grow := create_tween()
	grow.tween_method(func(age: float) -> void: ring.set_instance_shader_parameter("age", age), 0.0, 1.0, life)
	grow.tween_callback(ring.queue_free)


## A ring at `reach` tiles drawing in to `at`, as a Silence takes hold.
func _ring_in(at: Vector3, reach: float, colour: Color) -> void:
	var ring := _ring_mesh(at, reach, colour)
	ring.set_instance_shader_parameter("age", 0.3)  # spread out, not yet fading
	var close := create_tween().set_parallel()
	close.tween_method(func(r: float) -> void: ring.set_instance_shader_parameter("reach", r), reach, 0.2, 0.6)
	close.tween_method(func(age: float) -> void: ring.set_instance_shader_parameter("age", age), 0.3, 0.95, 0.6)
	close.chain().tween_callback(ring.queue_free)


func _ring_mesh(at: Vector3, reach: float, colour: Color) -> MeshInstance3D:
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE * (reach * 2.0 + 1.0)
	var ring := MeshInstance3D.new()
	ring.mesh = plane
	var material := _ring.duplicate() as ShaderMaterial
	material.set_shader_parameter("color", Color(colour, 0.9))
	ring.material_override = material
	ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	ring.position = Vector3(at.x, 0.05, at.z)
	ring.set_instance_shader_parameter("reach", reach)
	ring.set_instance_shader_parameter("age", 0.0)
	add_child(ring)
	return ring


## A burst of motes at `at` in `colour`, fading out as they go: `shape` is [how many, how long
## they live, how fast they fly, what pulls them (up is +), how far round they start, how big
## each is]. `haze`: smoke or fog, blended rather than glowing. Gone once they've all faded,
## unless `goes` is false (the caller frees it: a fireball's trail).
func _burst(at: Vector3, colour: Color, shape: Array, haze := false, goes := true) -> CPUParticles3D:
	var motes := CPUParticles3D.new()
	motes.one_shot = true
	motes.explosiveness = 0.85
	motes.amount = maxi(1, int(shape[0]) / (2 if Look.reduce_motion else 1))
	motes.lifetime = shape[1]
	motes.mesh = dot()
	motes.material_override = mote(haze)
	motes.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	motes.emission_shape = CPUParticles3D.EMISSION_SHAPE_SPHERE
	motes.emission_sphere_radius = maxf(shape[4], 0.01)
	motes.direction = Vector3.UP
	motes.spread = 180.0
	motes.initial_velocity_min = float(shape[2]) * 0.4
	motes.initial_velocity_max = shape[2]
	motes.damping_min = float(shape[2]) * 0.5
	motes.damping_max = float(shape[2]) * 0.9
	motes.gravity = Vector3(0, shape[3], 0)
	motes.scale_amount_min = float(shape[5]) * 0.6
	motes.scale_amount_max = shape[5]
	var fade := Gradient.new()
	fade.set_color(0, Color(colour, 1.0))
	fade.set_color(1, Color(colour, 0.0))
	fade.add_point(0.25, Color(colour, 0.9))
	motes.color_ramp = fade
	motes.position = at
	add_child(motes)
	motes.emitting = true
	if goes:
		get_tree().create_timer(float(shape[1]) + 0.3).timeout.connect(motes.queue_free)
	return motes


## A tile's middle, on the ground.
static func _tile(x: int, y: int) -> Vector3:
	return Vector3(x + 0.5, 0.0, y + 0.5)
