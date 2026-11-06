extends CanvasLayer
## What's drawn over the world: the state of play, the controls, what just happened, and a red
## flash when the dead get you.

const LINE_LIFE := 6.0  # seconds a line of news stays
const FLASH_LIFE := 0.6

const CONTROLS := "WASD walk  ·  Shift run  ·  click: walk there  ·  Space pause  ·  1 2 3 speed  ·  wheel zoom  ·  V low walls  ·  R new seed  ·  Esc quit"
const LEGEND := "On the ground, what the dead can see:  pale, not after you  ·  orange, heard something (?)  ·  red, sees you (!)\nRings: how far your footsteps carry."

var _state: Label
var _news: Label
var _paused: Label
var _problem: Label
var _flash: ColorRect
var _lines: Array[String] = []
var _ages: Array[float] = []
var _flash_left := 0.0


func _ready() -> void:
	_flash = ColorRect.new()
	_flash.color = Color(0.7, 0.0, 0.0, 0.0)
	_flash.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_flash)
	_flash.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_state = _label(20, Control.PRESET_TOP_LEFT)
	var legend := _label(14, Control.PRESET_BOTTOM_LEFT)
	legend.text = LEGEND + "\n" + CONTROLS
	legend.modulate = Color(1, 1, 1, 0.8)
	_news = _label(18, Control.PRESET_TOP_RIGHT)
	_news.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_paused = _label(40, Control.PRESET_CENTER_TOP)
	_paused.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_paused.text = "PAUSED"
	_paused.position.y += 60.0
	_paused.visible = false
	_problem = _label(26, Control.PRESET_CENTER)
	_problem.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER


func _process(delta: float) -> void:
	for i in range(_lines.size() - 1, -1, -1):
		_ages[i] += delta
		if _ages[i] > LINE_LIFE:
			_lines.remove_at(i)
			_ages.remove_at(i)
	_news.text = "\n".join(_lines)
	_flash_left = maxf(0.0, _flash_left - delta)
	_flash.color.a = 0.35 * _flash_left / FLASH_LIFE


## `tick` counts tenths of a second of game time; `speed` is the dial (1, 2, 5).
func show_state(world_seed: int, tick: int, speed: int, paused: bool, caught: int) -> void:
	_state.text = "DUSKHOLD  ·  build step 4: a patch of the Hushlands\nSeed %d  ·  %.1f s  ·  %d×  ·  caught %d" % [
		world_seed, tick / 10.0, speed, caught]
	_paused.visible = paused


## A line of news, at the top right; the newest last.
func say(line: String) -> void:
	_lines.append(line)
	_ages.append(0.0)
	if _lines.size() > 6:
		_lines.remove_at(0)
		_ages.remove_at(0)


func flash() -> void:
	_flash_left = FLASH_LIFE


func reset() -> void:
	_lines.clear()
	_ages.clear()
	_flash_left = 0.0


func show_problem(text: String) -> void:
	_problem.text = text


func _label(size: int, preset: Control.LayoutPreset) -> Label:
	var label := Label.new()
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_constant_override("outline_size", 6)
	label.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(label)
	label.set_anchors_and_offsets_preset(preset, Control.PRESET_MODE_MINSIZE, 16)
	# Grow away from the corner it's pinned to as the text changes.
	var right := preset in [Control.PRESET_TOP_RIGHT, Control.PRESET_BOTTOM_RIGHT]
	var bottom := preset in [Control.PRESET_BOTTOM_LEFT, Control.PRESET_BOTTOM_RIGHT]
	var centre := preset in [Control.PRESET_CENTER, Control.PRESET_CENTER_TOP]
	label.grow_horizontal = Control.GROW_DIRECTION_BOTH if centre else (
		Control.GROW_DIRECTION_BEGIN if right else Control.GROW_DIRECTION_END)
	label.grow_vertical = Control.GROW_DIRECTION_BOTH if preset == Control.PRESET_CENTER else (
		Control.GROW_DIRECTION_BEGIN if bottom else Control.GROW_DIRECTION_END)
	return label
