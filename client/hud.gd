extends CanvasLayer
## What's drawn over the world: the state of play and the clock, your body (health, endurance,
## wounds), Zomboid's moodle column, the controls, what just happened, a red flash when the dead
## hurt you, a dark veil while you sleep, and how you died.

const LINE_LIFE := 6.0  # seconds a line of news stays
const FLASH_LIFE := 0.6

const CONTROLS := "WASD walk  ·  Shift run  ·  Ctrl sneak  ·  Alt brace (shield)  ·  right-click shoot (bow)  ·  click: walk there (on one of the dead beside you: swing)  ·  F swing  ·  Q shove  ·  E door\nB board up  ·  N pry a board off  ·  H tend a fire  ·  J put it out  ·  G burn a body  ·  C second wind  ·  Tab pack  ·  K skills and talents  ·  T make things  ·  Z sleep  ·  X drink water  ·  L light\nSpace pause  ·  1 2 3 speed  ·  wheel zoom  ·  V low walls  ·  F5 save  ·  R new world (dead: the next survivor)  ·  Esc save and quit"
const LEGEND := "On the ground, what the dead can see:  pale, not after you  ·  orange, heard something (?)  ·  red, sees you (!)\nRings: how far a sound carries  ·  red lines: the dead on you (thick: it has hold of you)."

## The moodles, by DuskWorld.character_moodles() code: a name for each level, what it does to
## you, and how bad its first level already is (0 mild, 1 bad, 2 dire; the colour).
const MOODLES := [
	[["Peckish", "Hungry", "Starving"],
		["eat something soon", "your breath comes back slower", "losing health"], 0],
	[["Thirsty", "Parched", "Dying of thirst"],
		["drink something soon", "your breath comes back slower", "losing health"], 0],
	[["Drowsy", "Tired", "Falling asleep"],
		["Z to sleep", "slower swings  ·  Z to sleep", "much slower swings  ·  Z to sleep"], 0],
	[["Hurt", "Badly hurt", "Near death"],
		["fed, watered and not bleeding, you heal", "sleep heals fastest", "sleep heals fastest"], 0],
	[["Bleeding", "Bleeding badly", "Bleeding heavily"],
		["bandage it: Tab  ·  ghouls smell it", "bandage them: Tab  ·  ghouls smell them",
			"bandage them: Tab  ·  ghouls smell them"], 1],
	[["Out of breath"], ["no running, weak swings"], 1],
	[["Heavy load"], ["no running"], 0],
	[["Held", "Held by two", "Held by three"], ["F struggle  ·  Q shove", "F struggle  ·  Q shove", "F struggle  ·  Q shove"], 2],
	[["Asleep"], ["Z to get up"], -1],
	[["Ghoul fever", "Weak with fever", "Dying of fever"],
		["thirstier, and nothing heals  ·  find a cure", "your breath comes back slowly  ·  find a cure",
			"the cure, today, or death"], 1],
	[["Sore", "In pain", "Agony"],
		["a bandage on the wound eases it: Tab", "everything costs more breath  ·  dress your wounds: Tab",
			"everything costs far more breath, and no sleep  ·  dress your wounds: Tab"], 0],
	[["Uneasy", "Afraid", "Frightened"],
		["your aim shakes  ·  light, food and rest ease it", "your aim shakes more  ·  get into the light",
			"you can't step nearer the dead  ·  back away, get to the light"], 0],
	[["Hurt arm", "Badly hurt arm"],
		["swings a little slower and less sure  ·  dress it: Tab",
			"swings slower and less sure  ·  dress it: Tab"], 0],
	[["Limping", "Lame"],
		["slower on your feet, and running costs more  ·  dress it: Tab",
			"much slower: a ghoul will run you down  ·  dress it: Tab"], 0],
	[["Head wound", "Bad head wound"],
		["your aim, and how far you see, a little worse  ·  dress it: Tab",
			"your aim and how far you see, worse  ·  dress it: Tab"], 0],
	[["Sneaking"], ["slower, quieter, seen only from nearer  ·  from behind, a sneak attack  ·  Ctrl to stand"], -1],
	[["Bracing"], ["blows from ahead may be blocked, for breath  ·  slower, no running  ·  let go of Alt"], -1],
]
const SEVERITY: Array[Color] = [Color(0.95, 0.85, 0.45), Color(1.0, 0.6, 0.25), Color(1.0, 0.3, 0.25)]
const RESTFUL := Color(0.6, 0.72, 1.0)

## When set (--news), every line of news is printed too: for runs with no window.
var echo := false

var _state: Label
var _news: Label
var _paused: Label
var _problem: Label
var _flash: ColorRect
var _veil: ColorRect
var _dread: ColorRect  # dread darkening the screen's edges
var _dread_shown := 0.0  # how much it shows now, eased toward your dread
var _asleep: Label
var _lines: Array[String] = []
var _ages: Array[float] = []
var _flash_left := 0.0
var _health: ProgressBar
var _endurance: ProgressBar
var _breath: Label
var _wounds: Label
var _level: Label
var _gold_left := 0.0  # the level-up glow
var _death: Label
var _held: Label
var _doing: Label
var _doing_bar: ProgressBar
var _moodles: VBoxContainer
var _shown_moodles := PackedInt32Array()


func _ready() -> void:
	# First, so it's under everything else the HUD draws.
	_dread = ColorRect.new()
	_dread.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var shade := ShaderMaterial.new()
	shade.shader = preload("res://shaders/dread.gdshader")
	_dread.material = shade
	add_child(_dread)
	_dread.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_flash = ColorRect.new()
	_flash.color = Color(0.7, 0.0, 0.0, 0.0)
	_flash.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_flash)
	_flash.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_veil = ColorRect.new()
	_veil.color = Color(0.02, 0.03, 0.08, 0.6)
	_veil.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_veil.visible = false
	add_child(_veil)
	_veil.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_asleep = _label(30, Control.PRESET_CENTER_TOP)  # above the sleeper, not on it
	_asleep.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_asleep.position.y += 110.0
	_asleep.add_theme_color_override("font_color", RESTFUL)
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
	_build_moodle_column()


func _process(delta: float) -> void:
	for i in range(_lines.size() - 1, -1, -1):
		_ages[i] += delta
		if _ages[i] > LINE_LIFE:
			_lines.remove_at(i)
			_ages.remove_at(i)
	_news.text = "\n".join(_lines)
	_flash_left = maxf(0.0, _flash_left - delta)
	_gold_left = maxf(0.0, _gold_left - delta)
	if _gold_left > 0.0:
		_flash.color = Color(1.0, 0.8, 0.3, 0.25 * _gold_left / FLASH_LIFE)
	else:
		_flash.color = Color(0.7, 0.0, 0.0, 0.35 * _flash_left / FLASH_LIFE)


## The seed, the clock (DuskWorld.clock()), the speed dial (1, 2, 5; more while you sleep), and
## the region's pressure (DuskWorld.pressure(): how many of the dead the patch holds, against
## how many it began with). Bodies that get up raise it; burning them lowers it. Then the
## Tolling's countdown (DuskWorld.tolling()), so you can build for it; the rain
## (DuskWorld.rain(): 0 dry, 1 rain, 2 heavy); and the building you're in (DuskWorld.place(),
## "" out of doors).
func show_state(world_seed: int, clock: PackedInt32Array, speed: int, paused: bool,
		pressure: PackedInt32Array, tolling: PackedInt32Array, rain: int, place: String) -> void:
	var dead := ""
	if pressure.size() >= 3 and pressure[0] > 0:
		dead = "  ·  the dead %d%%" % roundi(pressure[0] / 10.0)
	var weather: String = ["", "  ·  rain", "  ·  heavy rain"][clampi(rain, 0, 2)]
	var inside := "" if place.is_empty() else "  ·  in %s" % place
	_state.text = "DUSKHOLD  ·  new: bows and arrows\n%s%s%s  ·  Seed %d  ·  %d×%s%s" % [
		time_of_day(clock), inside, weather, world_seed, speed, dead, _countdown(tolling)]
	_paused.visible = paused


## "the Tolling in 2 days", "the Tolling tonight", "THE TOLLING": from DuskWorld.tolling()'s
## minutes until it begins, and whether it's on.
static func _countdown(tolling: PackedInt32Array) -> String:
	if tolling.size() < 2:
		return ""
	if tolling[1] == 1:
		return "  ·  THE TOLLING, till dawn"
	var minutes := tolling[0]
	if minutes < 24 * 60:
		return "  ·  the Tolling in %dh %02dm" % [minutes / 60, minutes % 60]
	var days := ceili(minutes / 1440.0)
	return "  ·  the Tolling in %d days" % days


## "Day 2  ·  06:45", from DuskWorld.clock().
static func time_of_day(clock: PackedInt32Array) -> String:
	if clock.size() < 3:
		return ""
	return "Day %d  ·  %02d:%02d" % [clock[0], clock[1], clock[2]]


## A line of news, at the top right; the newest last.
func say(line: String) -> void:
	if echo:
		print(line)
	_lines.append(line)
	_ages.append(0.0)
	if _lines.size() > 6:
		_lines.remove_at(0)
		_ages.remove_at(0)


func flash() -> void:
	_flash_left = FLASH_LIFE
	_gold_left = 0.0


## A warm glow, for something good: a level gained.
func flash_good() -> void:
	_gold_left = FLASH_LIFE * 2.0


## Your level and its XP, and the most health you can have (DuskWorld.character_deeds()).
func show_deeds(deeds: PackedInt32Array) -> void:
	if deeds.size() < 5:
		return
	_health.max_value = deeds[4] / 10.0
	var home := "" if deeds[5] >= 0 else "  ·  no home yet"
	if deeds[3] < 0:
		_level.text = "Level %d  ·  %d XP%s" % [deeds[0], deeds[1], home]
	else:
		_level.text = "Level %d  ·  %d / %d XP%s" % [deeds[0], deeds[1], deeds[3], home]


func reset() -> void:
	_lines.clear()
	_ages.clear()
	_flash_left = 0.0
	_death.text = ""
	_veil.visible = false
	_asleep.text = ""
	_shown_moodles = PackedInt32Array()
	show_moodles(PackedInt32Array())


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


## Your body, from DuskWorld.character_body() and character_wounds(). `can_shove` is the core's
## answer (DuskWorld.can_shove()): held with both hands full, there's only the struggle.
func show_body(body: PackedInt32Array, wounds: PackedStringArray, can_shove := true) -> void:
	if body.size() < 5:
		return
	_health.value = body[0] / 10.0
	_endurance.value = body[1] / 10.0
	_breath.text = "Out of breath" if body[2] == 1 else ""
	_wounds.text = "\n".join(wounds)
	var holding := body[5] if body.size() > 5 and body[3] == 0 else 0
	var shove := "Q shove it off" if holding == 1 else "Q shove one off"
	if not can_shove:
		shove = "no hand free to shove (weapon and light)"
	match holding:
		0:
			_held.text = ""
		1:
			_held.text = "IT HAS HOLD OF YOU\nF struggle  ·  %s" % shove
		_:
			_held.text = "%d HAVE HOLD OF YOU\nF struggle  ·  %s  ·  three drag you down" % [holding, shove]


## The moodle column, from DuskWorld.character_moodles(): rebuilt only when it changes.
func show_moodles(moodles: PackedInt32Array) -> void:
	if moodles == _shown_moodles:
		return
	_shown_moodles = moodles
	for child in _moodles.get_children():
		child.queue_free()
	for i in range(0, moodles.size() - 1, 2):
		var code := moodles[i]
		if code < 0 or code >= MOODLES.size():
			continue
		var moodle: Array = MOODLES[code]
		var names: Array = moodle[0]
		var level := clampi(moodles[i + 1], 1, names.size())
		var colour := RESTFUL if moodle[2] < 0 else SEVERITY[mini(level - 1 + moodle[2], 2)]
		_moodle(names[level - 1], moodle[1][level - 1], colour)


func _moodle(heading: String, effect: String, colour: Color) -> void:
	var panel := PanelContainer.new()
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.06, 0.08, 0.72)
	style.border_color = colour
	style.border_width_left = 4
	style.content_margin_left = 10.0
	style.content_margin_right = 8.0
	style.content_margin_top = 3.0
	style.content_margin_bottom = 4.0
	panel.add_theme_stylebox_override("panel", style)
	var lines := VBoxContainer.new()
	lines.add_theme_constant_override("separation", 0)
	lines.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(lines)
	var title := Label.new()
	title.text = heading
	title.add_theme_font_size_override("font_size", 17)
	title.add_theme_color_override("font_color", colour)
	lines.add_child(title)
	var what := Label.new()
	what.text = effect
	what.add_theme_font_size_override("font_size", 12)
	what.modulate = Color(1, 1, 1, 0.8)
	lines.add_child(what)
	_moodles.add_child(panel)


## Your dread (DuskWorld.character_dread(), per mille) darkens the screen's edges. It's eased
## toward, so a jump creeps in rather than snaps; dead, it lifts.
func show_dread(per_mille: int, delta: float) -> void:
	_dread_shown = move_toward(_dread_shown, per_mille / 1000.0, delta * 0.5)
	(_dread.material as ShaderMaterial).set_shader_parameter("amount", _dread_shown)


## Asleep, the world is veiled and the clock runs fast; awake, the veil lifts.
func show_asleep(asleep: bool, clock: PackedInt32Array) -> void:
	_veil.visible = asleep
	_asleep.text = "Asleep\n%s\n\nZ to get up" % time_of_day(clock) if asleep else ""


## The end, as a recap (CONCEPT.md, death and legacy): who, how they died, how long they lasted
## in game time (DuskWorld.character_lasted()), what they carried, and what becomes of the body.
func show_death(name: String, how: String, minutes: int, carried: String, rises: bool) -> void:
	var days := floori(minutes / 1440.0)
	var hours := floori((minutes % 1440) / 60.0)
	var lasted := "%d hours %d minutes" % [hours, minutes % 60]
	if days == 0 and hours == 0:
		lasted = "%d minutes" % minutes
	if days > 0:
		lasted = "%d day%s %d hours" % [days, "" if days == 1 else "s", hours]
	var who := name if name != "" else "You"
	var lines := ["%s lasted %s." % [who, lasted], how]
	if carried != "":
		lines.append("Carrying %s." % carried)
	if rises:
		lines.append("The body lies where it fell. Unburned, at midnight it rises as a revenant, in that gear.")
	lines.append("")
	lines.append("R: the next survivor comes to the abbey, in this world.   Shift+R: a new world.")
	_death.text = "\n".join(lines)
	_veil.visible = false
	_asleep.text = ""


func hide_death() -> void:
	_death.text = ""


func _build_body_panel() -> void:
	var panel := VBoxContainer.new()
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.position = Vector2(16, 76)
	add_child(panel)
	_level = Label.new()
	_level.add_theme_font_size_override("font_size", 15)
	_level.add_theme_color_override("font_color", Color(1.0, 0.85, 0.55))
	_level.add_theme_constant_override("outline_size", 6)
	_level.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	panel.add_child(_level)
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


## Zomboid's column: down the right side, under the news.
func _build_moodle_column() -> void:
	_moodles = VBoxContainer.new()
	_moodles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_moodles.add_theme_constant_override("separation", 4)
	add_child(_moodles)
	_moodles.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT, Control.PRESET_MODE_MINSIZE, 16)
	_moodles.position.y += 180.0
	_moodles.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_moodles.custom_minimum_size = Vector2(200, 0)


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
