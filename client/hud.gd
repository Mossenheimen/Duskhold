extends CanvasLayer
## What's drawn over the world (VISUALS.md §10): the status and your body down the top left
## (zones A and B); the news, and Zomboid's moodle column under it, down the top right (D, E);
## the one thing you must answer now at the top centre (C: held, paused, asleep); the act under
## way just under you (F); and the keys, on F1 (I). Then a red flash when the dead hurt you, a
## dark veil while you sleep, how you died, and who comes next. Every colour and size is a
## Look token.

const Look = preload("res://look.gd")

const LINE_LIFE := 6.0  # seconds a line of news stays
const FLASH_LIFE := 0.6
const NEWS_LINES := 6
const NEWS_WIDTH := 620.0
const LINE_HEIGHT := 28.0  # a line of BODY text, for the news's room

## Every key, for the F1 panel (zone I): the key, and what it does.
const KEYS := [
	["WASD", "walk (into an open window or a fence: climb it)"], ["Shift", "run"], ["Ctrl", "sneak"], ["Alt", "brace (shield)"],
	["click", "walk there; beside one of the dead, swing"], ["right-click", "shoot (bow)"],
	["F", "swing (at a shut window: smash it)"], ["Shift+F", "heavy swing: slower, may knock down"], ["Q", "shove"], ["E", "door or window"], ["B", "board up"], ["N", "pry a board off"],
	["H", "tend a fire"], ["J", "put it out"], ["G", "butcher a beast, or burn a body"],
	["O", "consecrate a body; at a shut door, ward it"], ["P", "pray at an altar"],
	["C", "second wind"], ["U", "turn the dead"], ["M", "mend"], ["Y", "pick a plant"],
	["I", "spells"], ["4 5 6 7", "ready a spell; then click the spot (right-click: let it go)"],
	["Tab", "your pack"], ["K", "Strength, Fitness, skills and talents"], ["T", "make things"], ["Z", "sleep"],
	["X", "drink water"], ["L", "light"], ["Space", "pause"], ["1 2 3", "speed"],
	["wheel", "zoom"], ["V", "low walls"], ["- =", "smaller or larger GUI"], ["F5", "save"],
	["R", "new world (dead: the next survivor)"], ["Esc", "save and quit"], ["F1", "these keys"],
]
const LEGEND := "On the ground, what the dead can see:  pale, not after you  ·  amber, heard something (?)  ·  red, sees you (!)\nRings: how far a sound carries  ·  red lines: the dead on you (thick: it has hold of you)."

## The moodles, by DuskWorld.character_moodles() code: a name for each level, what it does to
## you, and how bad its first level already is (0 mild, 1 bad, 2 dire; -1 a restful one).
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
	[["Chilly", "Cold", "Freezing"],
		["get under a roof, or by a fire, before it's worse",
			"shivering: your aim is worse, and everything costs more breath  ·  a roof, a fire",
			"nothing heals, health drains, no sleep  ·  get to a fire"], 0],
	[["Damp", "Wet", "Soaked"],
		["wet makes you colder  ·  dry off under a roof",
			"wet makes you colder  ·  dry off under a roof, faster by a fire",
			"much colder, and your clothes keep half their warmth  ·  dry off by a fire"], 0],
]
## How many of the dead have hold of you, in words, for the held banner.
const HOW_MANY := ["", "One", "Two", "Three", "Four", "Five"]

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
var _faith: ProgressBar  # a Cleric's (DuskWorld.faith()): hidden for a class without any
var _slots: ProgressBar  # a Wizard's spell slots (DuskWorld.character_slots()): hidden without
var _choice: Label  # who comes into the world: the classes to choose from
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
var _keys: PanelContainer  # every key, on F1 (zone I)
var _keys_hint: Label  # "F1: every key", for a world's first game hour


func _ready() -> void:
	# First, so it's under everything else the HUD draws.
	_dread = ColorRect.new()
	_dread.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var shade := ShaderMaterial.new()
	shade.shader = preload("res://shaders/dread.gdshader")
	shade.set_shader_parameter("tint", Look.DREAD)
	_dread.material = shade
	add_child(_dread)
	_dread.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_flash = ColorRect.new()
	_flash.color = Color(Look.HURT, 0.0)
	_flash.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_flash)
	_flash.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_veil = ColorRect.new()
	_veil.color = Look.VEIL
	_veil.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_veil.visible = false
	add_child(_veil)
	_veil.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_left_column()
	_build_right_column()
	# Zone C: the one thing you must answer now, at the top centre.
	_paused = _label(Look.BANNER, Control.PRESET_CENTER_TOP)
	_paused.text = "Paused"
	_paused.position.y += 110.0
	_paused.visible = false
	_asleep = _label(Look.BANNER, Control.PRESET_CENTER_TOP, Look.REST)  # above the sleeper
	_asleep.position.y += 150.0
	_held = _label(Look.BANNER, Control.PRESET_CENTER_TOP, Look.DANGER_TEXT)
	_held.position.y += 190.0
	_problem = _label(Look.HEADING, Control.PRESET_CENTER)
	_death = _label(Look.HEADING, Control.PRESET_CENTER)
	_choice = _label(Look.HEADING, Control.PRESET_CENTER)
	_build_action_bar()
	_build_keys()


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
		_flash.color = Color(Look.GOLD, 0.25 * _gold_left / FLASH_LIFE)
	else:
		_flash.color = Color(Look.HURT, 0.35 * _flash_left / FLASH_LIFE)


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
	_state.text = "DUSKHOLD  ·  new: the Wizard\n%s%s%s  ·  Seed %d  ·  %d×%s%s" % [
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
	if _lines.size() > NEWS_LINES:
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
	_choice.text = ""
	_veil.visible = false
	_asleep.text = ""
	_shown_moodles = PackedInt32Array()
	show_moodles(PackedInt32Array())


func show_problem(text: String) -> void:
	_problem.text = text


## Every key, on F1 (zone I), or not; and for a world's first game hour, a line saying so.
func show_keys(shown: bool, hint: bool) -> void:
	_keys.visible = shown
	_keys_hint.visible = hint and not shown


func keys_shown() -> bool:
	return _keys.visible


## Outlined text straight on the world, pinned to a corner or the middle of the screen and
## growing away from it as its text changes. Centred ones are centred line by line.
func _label(size: int, preset: Control.LayoutPreset, colour := Look.TEXT) -> Label:
	var label := Look.text(Label.new(), size, colour, true)
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(label)
	label.set_anchors_and_offsets_preset(preset, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	var centre := preset in [Control.PRESET_CENTER, Control.PRESET_CENTER_TOP]
	if centre:
		label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.grow_horizontal = Control.GROW_DIRECTION_BOTH if centre else Control.GROW_DIRECTION_END
	label.grow_vertical = Control.GROW_DIRECTION_BOTH if preset == Control.PRESET_CENTER \
		else Control.GROW_DIRECTION_END
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
			_held.text = "It has hold of you\nF struggle  ·  %s" % shove
		_:
			var many: String = HOW_MANY[holding] if holding < HOW_MANY.size() else str(holding)
			_held.text = "%s of them have hold of you\nF struggle  ·  %s  ·  three drag you down" % [
				many, shove]


## Your faith (DuskWorld.faith(): [now, the most], in points): a bar under endurance, for a class
## that has any.
func show_faith(faith: PackedInt32Array) -> void:
	var most := faith[1] if faith.size() > 1 else 0
	_faith.get_parent().visible = most > 0
	if most > 0:
		_faith.max_value = most
		_faith.value = faith[0]


## Your spell slots (DuskWorld.character_slots(): [left, the most]): a bar under endurance, for a
## class that casts.
func show_slots(slots: PackedInt32Array) -> void:
	var most := slots[1] if slots.size() > 1 else 0
	_slots.get_parent().visible = most > 0
	if most > 0:
		_slots.max_value = most
		_slots.value = slots[0]


## Who comes into the world: the classes to choose from, or "" once one's chosen.
func show_choice(text: String) -> void:
	_choice.text = text


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
		var colour := Look.REST if moodle[2] < 0 else Look.SEVERITY[mini(level - 1 + moodle[2], 2)]
		_moodle(names[level - 1], moodle[1][level - 1], colour)


## One moodle: a panel edged in its severity's colour, its name in that colour, what it does
## under it. (Icons on tinted tiles come later: VISUALS.md §11.)
func _moodle(heading: String, effect: String, colour: Color) -> void:
	var panel := PanelContainer.new()
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var style := Look.panel_style()
	style.border_color = colour
	style.set_border_width_all(0)
	style.border_width_left = 4
	style.content_margin_top = 4.0
	style.content_margin_bottom = 5.0
	panel.add_theme_stylebox_override("panel", style)
	var lines := VBoxContainer.new()
	lines.add_theme_constant_override("separation", 0)
	lines.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(lines)
	var title := Look.text(Label.new(), Look.BODY, colour)
	title.text = heading
	lines.add_child(title)
	var what := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	what.text = effect
	what.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	what.custom_minimum_size = Vector2(320, 0)
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


## Zones A and B, down the top left: the status, then your level, body bars, breath and wounds.
func _build_left_column() -> void:
	var column := VBoxContainer.new()
	column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_theme_constant_override("separation", 4)
	column.position = Vector2(Look.MARGIN, Look.MARGIN)
	add_child(column)
	_state = Look.text(Label.new(), Look.HEADING, Look.TEXT, true)
	column.add_child(_state)
	_level = Look.text(Label.new(), Look.BODY, Look.GOLD, true)
	column.add_child(_level)
	_health = _bar(column, "Health", Look.HEALTH)
	_endurance = _bar(column, "Endurance", Look.ENDURANCE)
	_faith = _bar(column, "Faith", Look.FAITH)
	_faith.get_parent().visible = false
	_slots = _bar(column, "Spells", Look.ARCANE)
	_slots.get_parent().visible = false
	_breath = Look.text(Label.new(), Look.BODY, Look.ALERT, true)
	column.add_child(_breath)
	_wounds = Look.text(Label.new(), Look.SMALL, Look.HURT_TEXT, true)
	column.add_child(_wounds)


## Zones D and E, down the top right: the news, with room kept for its lines so the moodles
## under it stay put (Zomboid's column), the text left-aligned in its box.
func _build_right_column() -> void:
	var column := VBoxContainer.new()
	column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_theme_constant_override("separation", 12)
	add_child(column)
	column.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	column.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_news = Look.text(Label.new(), Look.BODY, Look.TEXT, true)
	_news.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_news.custom_minimum_size = Vector2(NEWS_WIDTH, NEWS_LINES * LINE_HEIGHT)
	_news.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(_news)
	_moodles = VBoxContainer.new()
	_moodles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_moodles.add_theme_constant_override("separation", 4)
	_moodles.size_flags_horizontal = Control.SIZE_SHRINK_END
	column.add_child(_moodles)


func _bar(parent: Control, name: String, colour: Color) -> ProgressBar:
	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	parent.add_child(row)
	var label := Look.text(Label.new(), Look.BODY, Look.TEXT, true)
	label.text = name
	label.custom_minimum_size = Vector2(110, 0)
	row.add_child(label)
	var bar := Look.bar(ProgressBar.new(), colour)
	bar.custom_minimum_size = Vector2(240, 18)
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	bar.max_value = 100.0
	bar.value = 100.0
	row.add_child(bar)
	return bar


## What you're doing now, with how far along it is (Zomboid's progress bar): "" for nothing.
func show_action(label: String, progress: float) -> void:
	_doing.text = label + "…" if label != "" else ""
	_doing_bar.visible = label != ""
	_doing_bar.value = progress * 100.0


## Zone F: just under you, where your eyes already are.
func _build_action_bar() -> void:
	var box := VBoxContainer.new()
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(box)
	box.set_anchors_and_offsets_preset(Control.PRESET_CENTER, Control.PRESET_MODE_MINSIZE)
	box.position.y += 90.0
	box.grow_horizontal = Control.GROW_DIRECTION_BOTH
	box.grow_vertical = Control.GROW_DIRECTION_END
	_doing = Look.text(Label.new(), Look.BODY, Look.TEXT, true)
	_doing.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(_doing)
	_doing_bar = Look.bar(ProgressBar.new(), Look.YOURS)
	_doing_bar.custom_minimum_size = Vector2(240, 10)
	_doing_bar.visible = false
	box.add_child(_doing_bar)


## Zone I: every key in four columns, with what's drawn on the ground above them, over the
## middle of the screen while F1 has it open; and the line that says it's there.
func _build_keys() -> void:
	_keys_hint = _label(Look.SMALL, Control.PRESET_CENTER_BOTTOM, Look.TEXT_DIM)
	_keys_hint.text = "F1: every key"
	_keys_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_keys_hint.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_keys_hint.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_keys_hint.visible = false
	_keys = PanelContainer.new()
	_keys.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_keys.add_theme_stylebox_override("panel", Look.panel_style())
	var lines := VBoxContainer.new()
	lines.add_theme_constant_override("separation", 8)
	_keys.add_child(lines)
	var legend := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	legend.text = LEGEND
	lines.add_child(legend)
	var grid := GridContainer.new()
	grid.columns = 4
	grid.add_theme_constant_override("h_separation", 24)
	grid.add_theme_constant_override("v_separation", 2)
	lines.add_child(grid)
	for key: Array in KEYS:
		var cell := Look.text(Label.new(), Look.SMALL)
		cell.text = "[%s]  %s" % [key[0], key[1]]
		cell.custom_minimum_size = Vector2(300, 0)
		grid.add_child(cell)
	add_child(_keys)
	_keys.set_anchors_and_offsets_preset(Control.PRESET_CENTER, Control.PRESET_MODE_MINSIZE)
	_keys.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_keys.grow_vertical = Control.GROW_DIRECTION_BOTH
	_keys.visible = false
