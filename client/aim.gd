extends Node3D
## A readied spell, aimed (VISUALS.md §12, Effects): a faint ring round you as far as you can cast
## it, and on the tile under the mouse a ring as far as it would reach there. It's arcane where the
## spell can be cast, grey where it can't (too far, or a spot you can't see), and danger's red where
## a fireball would catch you too. Only the look: whether it can be cast, and whom it would catch,
## are the core's (DuskWorld.cast_check(), spell_catches()).

const Look = preload("res://look.gd")

var _range := _ring(0.08)
var _area := _ring(0.16)


func _init() -> void:
	add_child(_range)
	add_child(_area)
	visible = false


## The aim: `from` where you stand and `far` how far you can cast it, in tiles; `at` the tile aimed
## at and `reach` how far the spell would reach there (a Light, as far as it lights); `castable`,
## the core's yes; `catches_you`, a burst that would burn you too.
func show_aim(from: Vector3, far: float, at: Vector2i, reach: float, castable: bool,
		catches_you: bool) -> void:
	visible = true
	_place(_range, Vector3(from.x, 0.035, from.z), far, Color(Look.ARCANE, 0.35))
	var colour := Look.DANGER if catches_you else (Look.ARCANE if castable else Look.TEXT_DIM)
	_place(_area, Vector3(at.x + 0.5, 0.045, at.y + 0.5), maxf(reach, 0.45), Color(colour, 0.9))


func hide_aim() -> void:
	visible = false


## A ring on the ground: the noise ring's shader, held at its full reach.
static func _ring(width: float) -> MeshInstance3D:
	var ring := MeshInstance3D.new()
	ring.mesh = PlaneMesh.new()
	var material := ShaderMaterial.new()
	material.shader = preload("res://shaders/noise_ring.gdshader")
	material.set_shader_parameter("width", width)
	ring.material_override = material
	ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	ring.set_instance_shader_parameter("age", 0.3)  # spread out, not yet fading
	return ring


static func _place(ring: MeshInstance3D, at: Vector3, reach: float, colour: Color) -> void:
	ring.position = at
	var plane := ring.mesh as PlaneMesh
	var size := Vector2.ONE * (reach * 2.0 + 1.0)
	if plane.size != size:
		plane.size = size
	ring.set_instance_shader_parameter("reach", reach)
	(ring.material_override as ShaderMaterial).set_shader_parameter("color", colour)
