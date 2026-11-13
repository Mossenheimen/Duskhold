extends CanvasLayer
## Your skills (K): each with its level and how far it is toward the next. Skills grow only by
## real use under real risk (the core decides; this only shows it). A skill at the free cap waits
## for a mastery: the "Master" button spends one, and the skill goes on to 10. Above them, your
## Strength and Fitness, which grow overnight from the day's exertion (`fitness.rs`).

const RISING := 0
const AT_CAP := 1
const MASTERED := 2
const TOP := 3

const Look = preload("res://look.gd")

const BAR := Look.SKILL
const CAPPED := Look.CAPPED
const GOLD := Look.GOLD

var _dusk: Node
var _names := PackedStringArray()
var _panel: PanelContainer
var _list: VBoxContainer
var _shown := PackedInt32Array()
var _shown_left := -1
var _shown_build := PackedInt32Array()  # Strength and Fitness, as last drawn
var _shown_talents := []  # your class's talents, taken and offered, as last drawn
var _texts := PackedStringArray()  # every talent's name and line (DuskWorld.talent_texts())


func _ready() -> void:
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", Look.panel_style())
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 3)
	_panel.add_child(_list)
	add_child(_panel)
	_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_LEFT, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	_panel.visible = false


## A new world: the skills' names come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.skill_names()
	_shown = PackedInt32Array()
	_shown_left = -1
	_shown_build = PackedInt32Array()
	_shown_talents = []
	_texts = dusk.talent_texts()


## Closed as the other book panel opens: one at a time (VISUALS.md §10, zone J).
func close() -> void:
	_panel.visible = false


func toggle() -> void:
	_panel.visible = not _panel.visible


func is_open() -> bool:
	return _panel.visible


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
	var talents := [_dusk.character_talents(), _dusk.talent_offer()]
	# Only the two stats: the day's work toward them changes with every breath.
	var build: PackedInt32Array = _dusk.character_build().slice(0, 2)
	if skills == _shown and left == _shown_left and talents == _shown_talents 			and build == _shown_build:
		return
	_shown = skills
	_shown_left = left
	_shown_talents = talents
	_shown_build = build
	_draw(skills, left, build)
	_draw_talents(talents[0], talents[1])


## Your class, the talents you've taken, and, with a pick to make, the three on offer.
func _draw_talents(taken: PackedInt32Array, offer: PackedInt32Array) -> void:
	var class_name_: String = _dusk.class_name()
	if class_name_ == "":
		return
	_list.add_child(_label(" ", 8))
	_list.add_child(_label("%s  ·  talents" % class_name_, Look.HEADING))
	for t in taken:
		_list.add_child(_label("%s: %s" % [_text(t, 0), _text(t, 1)], Look.SMALL))
	if offer.size() == 0:
		if taken.size() == 0:
			var hint := _label("Each level from 2 offers three to choose from.", Look.SMALL)
			hint.modulate = Look.TEXT_DIM
			_list.add_child(hint)
		return
	var choose := _label("Choose one:", Look.SMALL)
	choose.add_theme_color_override("font_color", GOLD)
	_list.add_child(choose)
	for t in offer:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var line := _label("%s: %s" % [_text(t, 0), _text(t, 1)], Look.SMALL)
		line.custom_minimum_size = Vector2(330, 0)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		row.add_child(line)
		var button := Button.new()
		button.text = "Choose"
		button.add_theme_font_size_override("font_size", Look.SMALL)
		button.focus_mode = Control.FOCUS_NONE  # keys stay with the game
		button.pressed.connect(func() -> void: _dusk.choose_talent(t))
		row.add_child(button)


## A talent's name (0) or line (1), by number.
func _text(talent: int, which: int) -> String:
	var i := talent * 2 + which
	return _texts[i] if i < _texts.size() else "?"


func _draw(skills: PackedInt32Array, left: int, build: PackedInt32Array) -> void:
	for child in _list.get_children():
		child.queue_free()
	_draw_build(build)
	var title := _label("Skills  ·  masteries left: %d" % left, Look.HEADING)
	_list.add_child(title)
	var hint := _label("They grow by real use under real risk.", Look.SMALL)
	hint.modulate = Look.TEXT_DIM
	_list.add_child(hint)
	for i in range(0, skills.size() - 2, 3):
		var skill := floori(i / 3.0)
		var level := skills[i]
		var progress := skills[i + 1]
		var state := skills[i + 2]
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var called := _label(_names[skill] if skill < _names.size() else "?", Look.BODY)
		called.custom_minimum_size = Vector2(90, 0)
		row.add_child(called)
		var number := _label(str(level), Look.BODY)
		number.custom_minimum_size = Vector2(22, 0)
		row.add_child(number)
		var colour := CAPPED if state == AT_CAP else (GOLD if state in [MASTERED, TOP] else BAR)
		row.add_child(_bar(progress / 1000.0, colour))
		match state:
			AT_CAP:
				if left > 0:
					var button := Button.new()
					button.text = "Master"
					button.add_theme_font_size_override("font_size", Look.SMALL)
					button.focus_mode = Control.FOCUS_NONE  # keys stay with the game
					button.pressed.connect(func() -> void: _dusk.master(skill))
					row.add_child(button)
				else:
					row.add_child(_label("at the cap", Look.SMALL))
			MASTERED:
				row.add_child(_label("mastered", Look.SMALL))
			TOP:
				row.add_child(_label("mastered, at the top", Look.SMALL))
		if state != RISING:
			called.add_theme_color_override("font_color", colour)


## Strength and Fitness, in thousandths of a level: each its level, and how far toward the next.
func _draw_build(build: PackedInt32Array) -> void:
	_list.add_child(_label("Body", Look.HEADING))
	var hint := _label("Strength and Fitness grow overnight from the day's exertion.", Look.SMALL)
	hint.modulate = Look.TEXT_DIM
	_list.add_child(hint)
	for k in 2:
		var value := build[k] if k < build.size() else 0
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var called := _label(["Strength", "Fitness"][k], Look.BODY)
		called.custom_minimum_size = Vector2(90, 0)
		row.add_child(called)
		var number := _label(str(floori(value / 1000.0)), Look.BODY)
		number.custom_minimum_size = Vector2(22, 0)
		row.add_child(number)
		var top := value >= 10000
		row.add_child(_bar(1.0 if top else (value % 1000) / 1000.0, GOLD if top else BAR))
	_list.add_child(_label(" ", 8))


func _label(text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	Look.sized(label, size)
	return label


func _bar(value: float, colour: Color) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.custom_minimum_size = Vector2(150, Look.SMALL)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.max_value = 1.0
	bar.value = value
	return Look.bar(bar, colour)
