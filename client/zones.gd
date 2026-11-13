extends Node3D
## The spells lasting on their spots (DuskWorld.spell_zones(), the core's spell.rs), alive while
## they last: Light as a pale light over its spot with motes drifting round it; Silence as a ring
## as far as it reaches, arcane dust rising off its edge; Fog as a low cloud with haze drifting in
## it. Each fades over its last seconds (FADE). Only the look: what each does is the core's.

const Look = preload("res://look.gd")
const Effects = preload("res://effects.gd")
## A spell's kind, by name (codes.gd).
const Spell = preload("res://codes.gd").Spell
## Five numbers a zone: kind, x, y, reach, ticks left.
const ZONE := 5
## A spell running out fades over its last this many ticks (3 s).
const FADE := 30.0
## How much of the fog token's own strength the cloud's body has: faint, under its drifting haze.
const FOG_BODY := 0.45

var _ring := ShaderMaterial.new()
var _mote := StandardMaterial3D.new()
var _shown := PackedInt32Array()  # the zones drawn, without their ticks left
var _nodes := []  # each drawn zone: [its node, its kind, its place in the zones given]


func _init() -> void:
	# A Silence's edge: the noise ring's shader, held at its full reach.
	_ring.shader = preload("res://shaders/noise_ring.gdshader")
	_ring.set_shader_parameter("color", Color(Look.ARCANE, 0.85))
	_ring.set_shader_parameter("width", 0.14)
	_mote.albedo_color = Look.ARCANE
	_mote.emission_enabled = true
	_mote.emission = Look.ARCANE
	_mote.emission_energy_multiplier = 3.0


## Each frame: redraw when the zones change; a zone running out fades.
func show_zones(zones: PackedInt32Array) -> void:
	var now := PackedInt32Array()
	for i in range(0, zones.size() - ZONE + 1, ZONE):
		now.append_array(zones.slice(i, i + 4))
	if now != _shown:
		clear()
		_shown = now
		for k in range(0, now.size() / 4):
			var node := _make(now[k * 4], now[k * 4 + 1], now[k * 4 + 2], now[k * 4 + 3])
			if node != null:
				add_child(node)
				_nodes.append([node, now[k * 4], k])
	for drawn: Array in _nodes:
		var at: int = drawn[2] * ZONE + 4
		var left := zones[at] if at < zones.size() else 0
		_fade(drawn[0], drawn[1], clampf(left / FADE, 0.0, 1.0))


func clear() -> void:
	for drawn: Array in _nodes:
		(drawn[0] as Node).queue_free()
	_nodes.clear()
	_shown = PackedInt32Array()


func _make(kind: int, x: int, y: int, reach: int) -> Node3D:
	var centre := Vector3(x + 0.5, 0.0, y + 0.5)
	match kind:
		Spell.LIGHT:
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
			# Motes drifting round it, slowly rising.
			light.add_child(_motes(Look.ARCANE, 10, 1.8, 0.6, 0.25, 0.15, 0.2, false))
			return light
		Spell.SILENCE:
			var plane := PlaneMesh.new()
			plane.size = Vector2.ONE * (reach * 2.0 + 0.5)
			var ring := MeshInstance3D.new()
			ring.mesh = plane
			ring.material_override = _ring
			ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			ring.position = centre + Vector3(0.0, 0.04, 0.0)
			ring.set_instance_shader_parameter("reach", float(reach))
			ring.set_instance_shader_parameter("age", 0.3)  # spread out, not yet fading
			# Arcane dust rising off its edge, all round.
			var dust := _motes(Look.ARCANE, maxi(8, reach * 7), 1.6, 0.0, 0.35, 0.1, 0.16, false)
			dust.emission_shape = CPUParticles3D.EMISSION_SHAPE_RING
			dust.emission_ring_axis = Vector3.UP
			dust.emission_ring_radius = float(reach)
			dust.emission_ring_inner_radius = maxf(reach - 0.2, 0.0)
			dust.emission_ring_height = 0.05
			ring.add_child(dust)
			return ring
		Spell.FOG:
			var cloud := MeshInstance3D.new()
			var drum := CylinderMesh.new()
			drum.top_radius = reach + 0.5
			drum.bottom_radius = reach + 0.5
			drum.height = 1.4
			cloud.mesh = drum
			# Its own material, to fade on its own; lit as the world is, so it's dark at night unless a
			# light reaches it, and faint, so the haze in it does the rest.
			var fog := StandardMaterial3D.new()
			fog.albedo_color = Color(Look.FOG, Look.FOG.a * FOG_BODY)
			fog.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			fog.cull_mode = BaseMaterial3D.CULL_DISABLED
			cloud.material_override = fog
			cloud.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			cloud.position = centre + Vector3(0.0, 0.7, 0.0)
			# Haze drifting in it: big, slow, soft puffs filling the drum.
			var haze := _motes(Color(Look.FOG, 0.6), maxi(10, reach * reach * 6), 3.0, 0.0, 0.15, 0.9, 1.5, true)
			haze.emission_shape = CPUParticles3D.EMISSION_SHAPE_RING
			haze.emission_ring_axis = Vector3.UP
			haze.emission_ring_radius = reach + 0.3
			haze.emission_ring_inner_radius = 0.0
			haze.emission_ring_height = 1.0
			cloud.add_child(haze)
			return cloud
	return null


## Motes that keep coming while the spell lasts: `amount` at a time, each living `life` seconds,
## starting within `spread` tiles, drifting up at `rise` tiles a second, `small` to `big` across.
## `haze`: blended (fog) rather than glowing.
func _motes(colour: Color, amount: int, life: float, spread: float, rise: float, small: float,
		big: float, haze: bool) -> CPUParticles3D:
	var motes := CPUParticles3D.new()
	motes.amount = maxi(1, amount / (2 if Look.reduce_motion else 1))
	motes.lifetime = life
	motes.mesh = Effects.dot()
	motes.material_override = Effects.mote(haze)
	motes.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	motes.emission_shape = CPUParticles3D.EMISSION_SHAPE_SPHERE
	motes.emission_sphere_radius = maxf(spread, 0.01)
	motes.direction = Vector3.UP
	motes.spread = 40.0
	motes.initial_velocity_min = rise * 0.5
	motes.initial_velocity_max = rise
	motes.gravity = Vector3.ZERO
	motes.scale_amount_min = small
	motes.scale_amount_max = big
	# Fading in, holding, fading out: a mote never pops.
	var fade := Gradient.new()
	fade.set_color(0, Color(colour, 0.0))
	fade.set_color(1, Color(colour, 0.0))
	fade.add_point(0.3, colour)
	fade.add_point(0.7, colour)
	motes.color_ramp = fade
	motes.emitting = true
	return motes


## A zone running out: `f` from 1 (plenty of time) down to 0 (gone). Its light dims, its ring and
## cloud thin, and its motes stop coming, so the last of them drift away.
func _fade(node: Node3D, kind: int, f: float) -> void:
	var motes := node.get_child(node.get_child_count() - 1) as CPUParticles3D
	if motes != null:
		motes.emitting = f > 0.4
	match kind:
		Spell.LIGHT:
			(node as OmniLight3D).light_energy = 2.4 * f
			(node.get_child(0) as Node3D).scale = Vector3.ONE * maxf(f, 0.05)
		Spell.SILENCE:
			(node as MeshInstance3D).set_instance_shader_parameter("age", lerpf(0.95, 0.3, f))
		Spell.FOG:
			var fog := (node as MeshInstance3D).material_override as StandardMaterial3D
			fog.albedo_color = Color(Look.FOG, Look.FOG.a * FOG_BODY * f)
