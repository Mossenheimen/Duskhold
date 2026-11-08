extends CanvasLayer
## What's drawn over the world: the state of play, your body (health, endurance, wounds), the
## controls, what just happened, a red flash when the dead hurt you, and how you died.

const LINE_LIFE := 6.0  # seconds a line of news stays
const FLASH_LIFE := 0.6

const CONTROLS := "WASD walk  ·  Shift run  ·  click: walk there (on one of the dead beside you: swing)  ·  F swing  ·  Q shove  ·  E door  ·  Tab pack\nSpace pause  ·  1 2 3 speed  ·  wheel zoom  ·  V low walls  ·  R new seed  ·  Esc quit"
const LEGEND := "On the ground, what the dead can see:  pale, not after you  ·  orange, heard something (?)  ·  red, sees you (!)\nRings: how far a sound carries  ·  red lines: the dead on you (thick: it has hold of you)."

var _state: Label
var _news: Label
var _paused: Label
var _problem: Label
var _flash: ColorRect
var _lines: Array[String] = []
var _ages: Array[float] = []
var _flash_left := 0.0
var _health: ProgressBar
var _endurance: ProgressBar
var _breath: Label
var _wounds: Label
var _death: Label
var _held: Label
var _doing: Label
var _doing_bar: ProgressBar


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
	_death = _label(30, Control.PRESET_CENTER)
	_death.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_held = _label(30, Control.PRESET_CENTER_TOP)
	_held.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_held.position.y += 120.0
	_held.add_theme_color_override("font_color", Color(1.0, 0.35, 0.3))
	_build_action_bar()
	_build_body_panel()


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
func show_state(world_seed: int, tick: int, speed: int, paused: bool) -> void:
	_state.text = "DUSKHOLD  ·  build step 5: items\nSeed %d  ·  %.1f s  ·  %d×" % [
		world_seed, tick / 10.0, speed]
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
	_death.text = ""


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


## Your body, from DuskWorld.character_body() and character_wounds().
func show_body(body: PackedInt32Array, wounds: PackedStringArray) -> void:
	if body.size() < 5:
		return
	_health.value = body[0] / 10.0
	_endurance.value = body[1] / 10.0
	_breath.text = "Out of breath" if body[2] == 1 else ""
	_wounds.text = "\n".join(wounds)
	var holding := body[5] if body.size() > 5 and body[3] == 0 else 0
	match holding:
		0:
			_held.text = ""
		1:
			_held.text = "IT HAS HOLD OF YOU\nF struggle  ·  Q shove it off"
		_:
			_held.text = "%d HAVE HOLD OF YOU\nF struggle  ·  Q shove one off  ·  three drag you down" % holding


## The end: how you died, and how long you lasted.
func show_death(how: String, seconds: float) -> void:
	_death.text = "This is how you died.\n%s\nYou lasted %d minutes %d seconds.\n\nR to begin again, in a new patch." % [
		how, int(seconds) / 60, int(seconds) % 60]


func _build_body_panel() -> void:
	var panel := VBoxContainer.new()
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.position = Vector2(16, 76)
	add_child(panel)
	_health = _bar(panel, "Health", Color(0.75, 0.16, 0.14))
	_endurance = _bar(panel, "Endurance", Color(0.85, 0.68, 0.22))
	_breath = Label.new()
	_breath.add_theme_color_override("font_color", Color(1.0, 0.75, 0.3))
	_breath.add_theme_constant_override("outline_size", 6)
	_breath.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	panel.add_child(_breath)
	_wounds = Label.new()
	_wounds.add_theme_font_size_override("font_size", 15)
	_wounds.add_theme_color_override("font_color", Color(1.0, 0.82, 0.78))
	_wounds.add_theme_constant_override("outline_size", 6)
	_wounds.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	panel.add_child(_wounds)


func _bar(parent: Control, name: String, colour: Color) -> ProgressBar:
	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	parent.add_child(row)
	var label := Label.new()
	label.text = name
	label.custom_minimum_size = Vector2(96, 0)
	label.add_theme_constant_override("outline_size", 6)
	label.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	row.add_child(label)
	var bar := ProgressBar.new()
	bar.custom_minimum_size = Vector2(220, 16)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.show_percentage = false
	bar.max_value = 100.0
	bar.value = 100.0
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var fill := StyleBoxFlat.new()
	fill.bg_color = colour
	var back := StyleBoxFlat.new()
	back.bg_color = Color(0, 0, 0, 0.5)
	bar.add_theme_stylebox_override("fill", fill)
	bar.add_theme_stylebox_override("background", back)
	row.add_child(bar)
	return bar


## What you're doing now, with how far along it is (Zomboid's progress bar): "" for nothing.
func show_action(label: String, progress: float) -> void:
	_doing.text = label + "…" if label != "" else ""
	_doing_bar.visible = label != ""
	_doing_bar.value = progress * 100.0


func _build_action_bar() -> void:
	var box := VBoxContainer.new()
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(box)
	box.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM, Control.PRESET_MODE_MINSIZE, 140)
	box.grow_horizontal = Control.GROW_DIRECTION_BOTH
	box.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_doing = Label.new()
	_doing.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_doing.add_theme_constant_override("outline_size", 6)
	_doing.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	box.add_child(_doing)
	_doing_bar = ProgressBar.new()
	_doing_bar.custom_minimum_size = Vector2(200, 10)
	_doing_bar.show_percentage = false
	_doing_bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_doing_bar.visible = false
	var fill := StyleBoxFlat.new()
	fill.bg_color = Color(0.85, 0.85, 0.75)
	var back := StyleBoxFlat.new()
	back.bg_color = Color(0, 0, 0, 0.5)
	_doing_bar.add_theme_stylebox_override("fill", fill)
	_doing_bar.add_theme_stylebox_override("background", back)
	box.add_child(_doing_bar)
