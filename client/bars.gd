extends Control
## Health bars over what you fight (the first playtest: "I'd also like to see damage numbers and
## health bars during combat"; VISUALS.md §11, Health bar): over each of the dead you see that's
## hurt, and over every rare you see (it has a name over it already), a short bar of what it has
## left, as Path of Exile's: its health in `bar.health` on a dark track, in a bronze line. One
## control draws them all, each frame, over where each is drawn.

const Look = preload("res://look.gd")

const WIDTH := 56.0
const HEIGHT := 7.0
const OVER := 2.1  # metres over the feet: under the mark over a head
const BRONZE := Color("#8e6b3b")

var _bars: Array = []  # [where on screen, how full, a rare's]


func _init() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)


## The bars to draw now: for each actor its drawn place (`at`), whether you see it (`seen`, 1),
## whether it's a rare (`rares`, 1), and its health and the most it can have (`health`, two
## each), projected by `camera`. Characters (`kinds` 0) have none: yours is the globe.
func show_bars(camera: Camera3D, at: Array, kinds: PackedInt32Array, seen: PackedByteArray,
		rares: PackedInt32Array, health: PackedInt32Array) -> void:
	_bars.clear()
	for i in at.size():
		if i * 2 + 1 >= health.size() or i >= kinds.size() or kinds[i] == 0:
			continue
		var most := health[i * 2 + 1]
		var now := health[i * 2]
		var rare := i < rares.size() and rares[i] == 1
		if most <= 0 or i >= seen.size() or seen[i] != 1 or (now >= most and not rare):
			continue
		var feet: Vector3 = at[i]
		var over := feet + Vector3(0, OVER, 0)
		if camera.is_position_behind(over):
			continue
		_bars.append([camera.unproject_position(over), clampf(float(now) / most, 0.0, 1.0), rare])
	queue_redraw()


func _draw() -> void:
	for b: Array in _bars:
		var middle: Vector2 = b[0]
		var r := Rect2(middle - Vector2(WIDTH / 2.0, HEIGHT / 2.0), Vector2(WIDTH, HEIGHT))
		draw_rect(r.grow(1.0), Color(0, 0, 0, 0.85))
		draw_rect(r, Color(0.08, 0.02, 0.02, 0.9))
		var full: float = b[1]
		if full > 0.0:
			var fill := Rect2(r.position, Vector2(r.size.x * full, r.size.y))
			draw_rect(fill, Look.HEALTH)
			draw_rect(Rect2(fill.position, Vector2(fill.size.x, fill.size.y * 0.4)),
				Color(1, 1, 1, 0.2))
		draw_rect(r.grow(1.0), Look.GOLD if b[2] else BRONZE, false, 1.0)
