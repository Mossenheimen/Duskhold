extends "res://book_panel.gd"
## Your skills (K): each with its level and how far it is toward the next. Skills grow only by
## real use under real risk (the core decides; this only shows it). A skill at the free cap waits
## for a mastery: the "Master" button spends one, and the skill goes on to 10. Above them, your
## Strength and Fitness, which grow overnight from the day's exertion (`fitness.rs`).

const RISING := 0
const AT_CAP := 1
const MASTERED := 2
const TOP := 3

const BAR := Look.SKILL
const CAPPED := Look.CAPPED
const GOLD := Look.GOLD

var _names := PackedStringArray()
var _texts := PackedStringArray()  # every talent's name and line (DuskWorld.talent_texts())


## A new world: the skills' and talents' names come from the core.
func start(dusk: Node) -> void:
	super(dusk)
	_names = dusk.skill_names()
	_texts = dusk.talent_texts()


## Each frame: redraw if anything changed (only while it's open): your skills, the masteries
## left, your talents taken and offered, and Strength and Fitness (only the two stats: the day's
## work toward them changes with every breath).
func refresh(dead: bool) -> void:
	if not _awake(dead):
		return
	var now := [_dusk.character_skills(), _dusk.masteries_left(),
		[_dusk.character_talents(), _dusk.talent_offer()], _dusk.character_build().slice(0, 2)]
	if _changed(now):
		_draw(now[0], now[1], now[3])
		_draw_talents(now[2][0], now[2][1])


## Your class, the talents you've taken, and, with a pick to make, the three on offer.
func _draw_talents(taken: PackedInt32Array, offer: PackedInt32Array) -> void:
	var class_name_: String = _dusk.class_name()
	if class_name_ == "":
		return
	_list.add_child(Look.line(" ", 8))
	_list.add_child(Look.line("%s  ·  talents" % class_name_, Look.HEADING))
	for t in taken:
		_list.add_child(Look.line("%s: %s" % [_text(t, 0), _text(t, 1)], Look.SMALL))
	if offer.size() == 0:
		if taken.size() == 0:
			var hint := Look.line("Each level from 2 offers three to choose from.", Look.SMALL)
			hint.modulate = Look.TEXT_DIM
			_list.add_child(hint)
		return
	var choose := Look.line("Choose one:", Look.SMALL)
	choose.add_theme_color_override("font_color", GOLD)
	_list.add_child(choose)
	for t in offer:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var line := Look.line("%s: %s" % [_text(t, 0), _text(t, 1)], Look.SMALL)
		line.custom_minimum_size = Vector2(330, 0)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		row.add_child(line)
		row.add_child(Look.button("Choose", func() -> void: _dusk.choose_talent(t)))


## A talent's name (0) or line (1), by number.
func _text(talent: int, which: int) -> String:
	var i := talent * 2 + which
	return _texts[i] if i < _texts.size() else "?"


func _draw(skills: PackedInt32Array, left: int, build: PackedInt32Array) -> void:
	_draw_build(build)
	_list.add_child(Look.line("Skills  ·  masteries left: %d" % left, Look.HEADING))
	var hint := Look.line("They grow by real use under real risk.", Look.SMALL)
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
		var called := Look.line(_names[skill] if skill < _names.size() else "?", Look.BODY)
		called.custom_minimum_size = Vector2(90, 0)
		row.add_child(called)
		var number := Look.line(str(level), Look.BODY)
		number.custom_minimum_size = Vector2(22, 0)
		row.add_child(number)
		var colour := CAPPED if state == AT_CAP else (GOLD if state in [MASTERED, TOP] else BAR)
		row.add_child(_bar(progress / 1000.0, colour))
		match state:
			AT_CAP:
				if left > 0:
					row.add_child(Look.button("Master", func() -> void: _dusk.master(skill)))
				else:
					row.add_child(Look.line("at the cap", Look.SMALL))
			MASTERED:
				row.add_child(Look.line("mastered", Look.SMALL))
			TOP:
				row.add_child(Look.line("mastered, at the top", Look.SMALL))
		if state != RISING:
			called.add_theme_color_override("font_color", colour)


## Strength and Fitness, in thousandths of a level: each its level, and how far toward the next.
func _draw_build(build: PackedInt32Array) -> void:
	_list.add_child(Look.line("Body", Look.HEADING))
	var hint := Look.line("Strength and Fitness grow overnight from the day's exertion.", Look.SMALL)
	hint.modulate = Look.TEXT_DIM
	_list.add_child(hint)
	for k in 2:
		var value := build[k] if k < build.size() else 0
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var called := Look.line(["Strength", "Fitness"][k], Look.BODY)
		called.custom_minimum_size = Vector2(90, 0)
		row.add_child(called)
		var number := Look.line(str(floori(value / 1000.0)), Look.BODY)
		number.custom_minimum_size = Vector2(22, 0)
		row.add_child(number)
		var top := value >= 10000
		row.add_child(_bar(1.0 if top else (value % 1000) / 1000.0, GOLD if top else BAR))
	_list.add_child(Look.line(" ", 8))


func _bar(value: float, colour: Color) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.custom_minimum_size = Vector2(150, Look.SMALL)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.max_value = 1.0
	bar.value = value
	return Look.bar(bar, colour)
