extends Control
## The body diagram (VISUALS.md §11; zone B, beside the bars): a front view, 118 px high, your
## right on the left as a doctor's chart has it. Each part takes its worst wound's tint (a scratch
## MILD, a laceration BAD, a bite WORST; unhurt, WHOLE); a bleeding part pulses in brightness once
## a second, and a bandaged one is wrapped in bands, so neither is told by colour alone. Hovering
## a part says its wounds (hud.gd's detail, from `detail`). What's drawn is
## DuskWorld.character_wound_parts().

const Look = preload("res://look.gd")

## Each part's box, by DuskWorld.part_names()'s order (the core's `Part`): head, torso, left arm,
## right arm, left hand, right hand, left leg, right leg, left foot, right foot. Your left is on
## the right. 2 px between parts, so each one's tint reads on its own.
const PARTS: Array[Rect2] = [
	Rect2(22, 0, 20, 20),  # head
	Rect2(20, 23, 24, 38),  # torso
	Rect2(46, 24, 8, 33),  # left arm
	Rect2(10, 24, 8, 33),  # right arm
	Rect2(45, 59, 10, 10),  # left hand
	Rect2(9, 59, 10, 10),  # right hand
	Rect2(33, 63, 10, 43),  # left leg
	Rect2(21, 63, 10, 43),  # right leg
	Rect2(33, 108, 13, 10),  # left foot
	Rect2(18, 108, 13, 10),  # right foot
]
const ROUND := [0, 4, 5]  # the head and the hands, drawn round in their boxes
const SIZE := Vector2(64, 118)
const PULSE := 0.4  # a bleeding part brightens toward white this much, once a second

## The part under the mouse (by PARTS), or -1 as it leaves.
signal hovered(part: int)

var _parts := PackedStringArray()  # DuskWorld.part_names()
var _kinds := PackedStringArray()  # DuskWorld.wound_names()
var _shown := PackedInt32Array()  # the wounds as last given (show)
var _worst: Array[int] = []  # by part: its worst wound's kind, -1 unhurt
var _bleeding: Array[bool] = []
var _bandaged: Array[bool] = []
var _under := -1
var _style := StyleBoxFlat.new()  # one box, its colour set part by part as they're drawn


func _init() -> void:
	custom_minimum_size = SIZE
	mouse_filter = Control.MOUSE_FILTER_STOP  # hovered for a part's wounds; a click walks nowhere
	_style.anti_aliasing = true
	mouse_exited.connect(_leave)
	show_wounds(PackedInt32Array())


## The names, once a world is made: the parts' and the kinds of wound.
func set_names(parts: PackedStringArray, kinds: PackedStringArray) -> void:
	_parts = parts
	_kinds = kinds


## Your wounds (DuskWorld.character_wound_parts(): part, kind, bleeding, bandaged, for each).
## True if they changed.
func show_wounds(wounds: PackedInt32Array) -> bool:
	if wounds == _shown and not _worst.is_empty():
		return false
	_shown = wounds
	_worst.assign(PARTS.map(func(_p: Rect2) -> int: return -1))
	_bleeding.assign(PARTS.map(func(_p: Rect2) -> bool: return false))
	_bandaged.assign(PARTS.map(func(_p: Rect2) -> bool: return false))
	for i in range(0, wounds.size() - 3, 4):
		var part := wounds[i]
		if part < 0 or part >= PARTS.size():
			continue
		_worst[part] = maxi(_worst[part], wounds[i + 1])
		_bleeding[part] = _bleeding[part] or wounds[i + 2] == 1
		_bandaged[part] = _bandaged[part] or wounds[i + 3] == 1
	queue_redraw()
	return true


## The worst-wounded part (the core lists the worst wound first), or -1 unhurt.
func worst_part() -> int:
	return _shown[0] if _shown.size() >= 4 else -1


## What hovering `part` says: [its name, its wounds a line each ("bite, bleeding"), or "unhurt"].
func detail(part: int) -> Array:
	var called := _parts[part] if part < _parts.size() else "?"
	var lines := PackedStringArray()
	for i in range(0, _shown.size() - 3, 4):
		if _shown[i] != part:
			continue
		var kind := _shown[i + 1]
		var line: String = _kinds[kind] if kind >= 0 and kind < _kinds.size() else "wound"
		if _shown[i + 2] == 1:
			line += ", bleeding"
		elif _shown[i + 3] == 1:
			line += ", bandaged"
		lines.append(line)
	if lines.is_empty():
		lines.append("unhurt")
	return [called.left(1).to_upper() + called.substr(1), "\n".join(lines)]


func _process(_delta: float) -> void:
	if _bleeding.has(true):
		queue_redraw()  # the pulse


func _draw() -> void:
	var pulse := 0.5 + 0.5 * sin(Time.get_ticks_msec() / 1000.0 * TAU)
	for p in PARTS.size():
		var box := PARTS[p]
		var colour := Look.WHOLE if _worst[p] < 0 else Look.SEVERITY[mini(_worst[p], 2)]
		if _bleeding[p]:
			colour = colour.lerp(Color.WHITE, PULSE * pulse)
		_style.bg_color = colour
		_style.set_corner_radius_all(roundi(box.size.x / 2.0) if p in ROUND else 3)
		draw_style_box(_style, box)
		if _bandaged[p]:
			_bands(box, p in ROUND)


## A bandage's bands across a part: one through a round part, two across a long one; each a pale
## band on a dark edge, so it shows on any tint.
func _bands(box: Rect2, is_round: bool) -> void:
	for at: float in ([0.5] if is_round else [0.33, 0.67]):
		var y := box.position.y + box.size.y * at
		var from := Vector2(box.position.x, y)
		var to := Vector2(box.end.x, y)
		draw_line(from, to, Look.OUTLINE, 4.0)
		draw_line(from, to, Look.DRESSING, 2.0)


func _gui_input(event: InputEvent) -> void:
	var motion := event as InputEventMouseMotion
	if motion == null:
		return
	var part := -1
	for p in PARTS.size():
		if PARTS[p].grow(1.0).has_point(motion.position):
			part = p
			break
	if part != _under:
		_under = part
		hovered.emit(part)


func _leave() -> void:
	if _under != -1:
		_under = -1
		hovered.emit(-1)
