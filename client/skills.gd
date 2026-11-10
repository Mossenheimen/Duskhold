extends CanvasLayer
## Your skills (K): each with its level and how far it is toward the next. Skills grow only by
## real use under real risk (the core decides; this only shows it). A skill at the free cap waits
## for a mastery: the "Master" button spends one, and the skill goes on to 10.

const RISING := 0
const AT_CAP := 1
const MASTERED := 2
const TOP := 3

const BAR := Color(0.55, 0.72, 0.9)
const CAPPED := Color(0.95, 0.85, 0.45)
const GOLD := Color(1.0, 0.78, 0.3)

var _dusk: Node
var _names := PackedStringArray()
var _panel: PanelContainer
var _list: VBoxContainer
var _shown := PackedInt32Array()
var _shown_left := -1


func _ready() -> void:
	_panel = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.06, 0.08, 0.72)
	style.set_content_margin_all(10)
	style.set_corner_radius_all(4)
	_panel.add_theme_stylebox_override("panel", style)
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 3)
	_panel.add_child(_list)
	add_child(_panel)
	_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_LEFT, Control.PRESET_MODE_MINSIZE, 16)
	_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	_panel.visible = false


## A new world: the skills' names come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.skill_names()
	_shown = PackedInt32Array()
	_shown_left = -1


func toggle() -> void:
	_panel.visible = not _panel.visible


## Each frame: redraw if anything changed (only while it's open).
func refresh(dead: bool) -> void:
	if _dusk == null:
		return
	if dead:
		_panel.visible = false
		return
	if not _panel.visible:
		return
	var skills: PackedInt32Array = _dusk.character_skills()
	var left: int = _dusk.masteries_left()
	if skills == _shown and left == _shown_left:
		return
	_shown = skills
	_shown_left = left
	_draw(skills, left)


func _draw(skills: PackedInt32Array, left: int) -> void:
	for child in _list.get_children():
		child.queue_free()
	var title := _label("Skills  ·  masteries left: %d" % left, 17)
	_list.add_child(title)
	var hint := _label("They grow by real use under real risk.", 12)
	hint.modulate = Color(1, 1, 1, 0.7)
	_list.add_child(hint)
	for i in range(0, skills.size() - 2, 3):
		var skill := floori(i / 3.0)
		var level := skills[i]
		var progress := skills[i + 1]
		var state := skills[i + 2]
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var called := _label(_names[skill] if skill < _names.size() else "?", 15)
		called.custom_minimum_size = Vector2(90, 0)
		row.add_child(called)
		var number := _label(str(level), 15)
		number.custom_minimum_size = Vector2(22, 0)
		row.add_child(number)
		var colour := CAPPED if state == AT_CAP else (GOLD if state in [MASTERED, TOP] else BAR)
		row.add_child(_bar(progress / 1000.0, colour))
		match state:
			AT_CAP:
				if left > 0:
					var button := Button.new()
					button.text = "Master"
					button.add_theme_font_size_override("font_size", 13)
					button.focus_mode = Control.FOCUS_NONE  # keys stay with the game
					button.pressed.connect(func() -> void: _dusk.master(skill))
					row.add_child(button)
				else:
					row.add_child(_label("at the cap", 12))
			MASTERED:
				row.add_child(_label("mastered", 12))
			TOP:
				row.add_child(_label("mastered, at the top", 12))
		if state != RISING:
			called.add_theme_color_override("font_color", colour)


func _label(text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	return label


func _bar(value: float, colour: Color) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.custom_minimum_size = Vector2(120, 10)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.show_percentage = false
	bar.max_value = 1.0
	bar.value = value
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var fill := StyleBoxFlat.new()
	fill.bg_color = colour
	var back := StyleBoxFlat.new()
	back.bg_color = Color(0, 0, 0, 0.5)
	bar.add_theme_stylebox_override("fill", fill)
	bar.add_theme_stylebox_override("background", back)
	return bar
