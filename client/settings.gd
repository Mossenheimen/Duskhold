extends CanvasLayer
## Settings (VISUALS.md §13, and §10's Settings screen): the GUI's size, colour-blind mode, high
## contrast, fewer flashes and less motion, numbers on the bars, and every key. On F10 (and from
## the menu, once there is one: §16, slice 9), over the world dimmed and paused. A change shows at
## once. The player's own game keeps them in user://settings.cfg; a trial run keeps nothing.
##
## The keys' defaults are here (VISUALS.md §10, The keys), and their remapping: a new key takes the
## place of an action's first, and a key taken from another action leaves that one without it.

const Look = preload("res://look.gd")
const Prompts = preload("res://prompts.gd")

const FILE := "user://settings.cfg"
## The GUI's sizes, over the screen's own (- and =, or the screen's buttons).
const GUI_SCALES := [0.75, 0.85, 1.0, 1.15, 1.3, 1.5, 1.75, 2.0]

## Every key's default (VISUALS.md §10, The keys). The left hand stays on WASD: what's pressed in a
## fight or a hurry is within reach of it (Shift, C, F, Q, E, R, Z, the number row). E does
## whatever is beside you, as in Zomboid, so the world's verbs need no keys of their own. The
## right hand's letters only open panels; the F keys are the game's own. A key held with Ctrl is
## KEY_MASK_CTRL | its key.
const KEYS := {
	# Moving and fighting.
	"walk_up": [KEY_W, KEY_UP],
	"walk_down": [KEY_S, KEY_DOWN],
	"walk_left": [KEY_A, KEY_LEFT],
	"walk_right": [KEY_D, KEY_RIGHT],
	"run": [KEY_SHIFT],
	"sneak": [KEY_C],
	"swing": [KEY_F],
	"shove": [KEY_Q],
	# Here: E for whatever's beside you, held for all of it.
	"use": [KEY_E],
	"light": [KEY_R],  # dead, the next survivor
	"sleep": [KEY_Z],
	# Your powers: a class's own, its talents', its spells.
	"power_1": [KEY_1],
	"power_2": [KEY_2],
	"power_3": [KEY_3],
	"power_4": [KEY_4],
	# The panels.
	"pack": [KEY_TAB, KEY_I],
	"craft": [KEY_B],
	"skills": [KEY_K],
	"spells": [KEY_P],
	"keys": [KEY_F1],
	"settings": [KEY_F10],
	# The game.
	"pause": [KEY_SPACE],
	"speed_1": [KEY_F2],
	"speed_2": [KEY_F3],
	"speed_3": [KEY_F4],
	"save": [KEY_F5],
	"low_walls": [KEY_V],
	"gui_smaller": [KEY_MINUS],
	"gui_larger": [KEY_EQUAL],
	"new_world": [KEY_MASK_CTRL | KEY_N],
	"quit": [KEY_ESCAPE],
}
## What the screen offers to remap, in its order: [the action, its name]. Not Esc: it's the way
## out, of this screen too.
const REMAP := [
	["walk_up", "Walk up"], ["walk_left", "Walk left"], ["walk_down", "Walk down"],
	["walk_right", "Walk right"], ["run", "Run (held)"], ["sneak", "Sneak"], ["swing", "Swing"],
	["shove", "Shove"], ["use", "Use what's here"], ["light", "Light; dead, the next"],
	["sleep", "Sleep"], ["power_1", "Power 1"], ["power_2", "Power 2"], ["power_3", "Power 3"],
	["power_4", "Power 4"], ["pack", "Your pack"], ["craft", "Making"], ["skills", "Skills"],
	["spells", "Spells"], ["keys", "The keys"], ["settings", "Settings"], ["pause", "Pause"],
	["speed_1", "Speed 1×"], ["speed_2", "Speed 2×"], ["speed_3", "Speed 5×"], ["save", "Save"],
	["low_walls", "Low walls"], ["gui_smaller", "Smaller GUI"], ["gui_larger", "Larger GUI"],
	["new_world", "A new world"],
]
## The settings that are on or off: [its name, what it's called, what it does].
const TOGGLES := [
	["colour_blind", "Colour-blind mode", "the dead's calm, alert and danger in sky blue, yellow and vermillion"],
	["high_contrast", "High contrast", "panels nearly solid, and every text outlined"],
	["reduce_motion", "Fewer flashes, less motion", "no flash at the screen's edge, no breathing dread, nothing pops"],
	["show_numbers", "Numbers on the bars", "health, endurance, faith and spells, always (else on hover)"],
]
const NAME_WIDTH := 330.0  # a setting's name, before its button

## A key was remapped, or the keys put back: what shows them builds itself again.
signal keys_changed
## The screen opened or closed: the world waits while it's open.
signal shown(open: bool)

## The player's own game keeps what's set here; a trial run doesn't.
var keep := false
var gui_scale := 1.0

var _gui_size: Label
var _toggles := {}  # a setting's name: its button
var _key_grid: GridContainer
var _key_buttons := {}  # an action: its button
var _hint: Label
var _waiting := ""  # the action whose new key is awaited; "" for none
var _modifier := KEY_NONE  # a modifier pressed while waiting: bound alone if it's let go first


## The keys at their defaults, into the InputMap: once, at the start.
static func add_keys() -> void:
	for action: String in KEYS:
		if not InputMap.has_action(action):
			InputMap.add_action(action)
			_bind(action, KEYS[action])


## `action` on `codes` (KEY_MASK_CTRL | a key, for one held with Ctrl), in place of its keys.
static func _bind(action: String, codes: Array) -> void:
	InputMap.action_erase_events(action)
	for code: int in codes:
		var press := InputEventKey.new()
		press.physical_keycode = (code & KEY_CODE_MASK) as Key
		press.ctrl_pressed = code & KEY_MASK_CTRL != 0
		InputMap.action_add_event(action, press)


## `action`'s keys, as KEYS has them.
static func codes_of(action: String) -> Array:
	var codes := []
	for event in InputMap.action_get_events(action):
		var press := event as InputEventKey
		if press != null:
			codes.append(int(press.physical_keycode) | (KEY_MASK_CTRL if press.ctrl_pressed else 0))
	return codes


## Whether a setting is on (TOGGLES' names).
static func is_on(setting: String) -> bool:
	match setting:
		"colour_blind":
			return Look.colour_blind
		"high_contrast":
			return Look.high_contrast
		"reduce_motion":
			return Look.reduce_motion
		_:
			return Look.show_numbers


func _ready() -> void:
	layer = 10  # over the HUD and its panels
	var dim := ColorRect.new()
	dim.color = Look.DIM
	dim.mouse_filter = Control.MOUSE_FILTER_STOP  # the world behind it takes no clicks
	add_child(dim)
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	var panel := PanelContainer.new()
	# A screen over the dimmed world: its ground nearly solid whatever high contrast says, so the
	# world's prompts and banners don't show through.
	var style := Look.panel_style()
	style.bg_color = Look.PANEL_SOLID
	style.remove_meta("panel")
	style.set_content_margin_all(24.0)
	panel.add_theme_stylebox_override("panel", style)
	add_child(panel)
	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 8)
	panel.add_child(rows)
	var title := Look.text(Label.new(), Look.TITLE)
	title.text = "Settings"
	rows.add_child(title)
	# The GUI's size.
	rows.add_child(_heading("The GUI"))
	var sizes := _row(rows, "GUI size")
	sizes.add_child(_button("−", func() -> void: scale_gui(-1), 44.0))
	_gui_size = Look.text(Label.new(), Look.BODY)
	_gui_size.custom_minimum_size = Vector2(70, 0)
	_gui_size.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	sizes.add_child(_gui_size)
	sizes.add_child(_button("+", func() -> void: scale_gui(1), 44.0))
	sizes.add_child(_dim_text(Prompts.keyed("also {gui_smaller} and {gui_larger}, in the game")))
	# What's on or off.
	rows.add_child(_heading("Seeing"))
	for toggle: Array in TOGGLES:
		var row := _row(rows, toggle[1])
		var setting: String = toggle[0]
		var button := _button("", func() -> void: set_on(setting, not is_on(setting)), 80.0)
		_toggles[setting] = button
		row.add_child(button)
		row.add_child(_dim_text(toggle[2]))
	# The keys.
	var keys := HBoxContainer.new()
	keys.add_theme_constant_override("separation", 16)
	rows.add_child(keys)
	keys.add_child(_heading("Keys"))
	_hint = _dim_text("")
	_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_hint.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	keys.add_child(_hint)
	keys.add_child(_button("Back to the defaults", _defaults))
	_key_grid = GridContainer.new()
	_key_grid.columns = 6
	_key_grid.add_theme_constant_override("h_separation", 10)
	_key_grid.add_theme_constant_override("v_separation", 4)
	rows.add_child(_key_grid)
	# Done.
	var done := HBoxContainer.new()
	done.alignment = BoxContainer.ALIGNMENT_END
	rows.add_child(done)
	done.add_child(_button(Prompts.keyed("Done  {quit}"), close, 120.0))
	panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER, Control.PRESET_MODE_MINSIZE)
	panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	visible = false


## What the player's own game kept: the GUI's size, what's on or off, and any keys remapped.
func load_kept() -> void:
	var file := ConfigFile.new()
	if file.load(FILE) != OK:
		return
	gui_scale = float(file.get_value("gui", "scale", 1.0))
	get_window().content_scale_factor = gui_scale
	for toggle: Array in TOGGLES:
		_turn(toggle[0], bool(file.get_value("look", toggle[0], false)))
	if file.has_section("keys"):
		for action in file.get_section_keys("keys"):
			if InputMap.has_action(action):
				_bind(action, file.get_value("keys", action, []))
	_restyle()


## --look=: settings turned on for a trial run's pictures (TOGGLES' names, by commas). Not kept.
func turn_on(names: String) -> void:
	for setting in names.split(",", false):
		_turn(setting.strip_edges(), true)
	_restyle()


## A setting turned on or off, at once, and kept.
func set_on(setting: String, on: bool) -> void:
	_turn(setting, on)
	_restyle()
	_refresh()
	_save()


## The GUI a step smaller or larger; its new size, in percent. Kept.
func scale_gui(step: int) -> int:
	var at := GUI_SCALES.find(gui_scale)
	if at < 0:
		at = GUI_SCALES.find(1.0)
	gui_scale = GUI_SCALES[clampi(at + step, 0, GUI_SCALES.size() - 1)]
	get_window().content_scale_factor = gui_scale
	_refresh()
	_save()
	return roundi(gui_scale * 100.0)


func open() -> void:
	_waiting = ""
	_hint.text = "Click a key, then press the new one."
	_build_keys()
	_refresh()
	visible = true
	shown.emit(true)


func close() -> void:
	if not visible:
		return
	visible = false
	_waiting = ""
	shown.emit(false)


func toggle() -> void:
	if visible:
		close()
	else:
		open()


func is_open() -> bool:
	return visible


## While a key is awaited, every key comes here first, and goes no further. Esc keeps the old one.
## A modifier alone (Shift, Ctrl, Alt) is bound when it's let go; Ctrl held with a key goes with it.
func _input(event: InputEvent) -> void:
	if not visible or _waiting == "":
		return
	var key := event as InputEventKey
	if key == null or key.echo:
		return
	get_viewport().set_input_as_handled()
	var code := key.physical_keycode
	if code == KEY_ESCAPE:
		if key.pressed:
			_hint.text = "Kept as it was."
			_waiting = ""
			_build_keys()
	elif code in [KEY_SHIFT, KEY_CTRL, KEY_ALT, KEY_META]:
		if key.pressed:
			_modifier = code
		elif _modifier == code:
			_rebind(_waiting, code)
	elif key.pressed:
		_rebind(_waiting, code | (KEY_MASK_CTRL if key.ctrl_pressed else 0))


## `action` on `code`, in place of its first key; any other action with that key loses it.
func _rebind(action: String, code: int) -> void:
	var lost := PackedStringArray()
	for row: Array in REMAP:
		var other: String = row[0]
		var theirs := codes_of(other)
		if other != action and code in theirs:
			theirs.erase(code)
			_bind(other, theirs)
			lost.append(row[1])
	# Its first key goes; a second (the arrows, for walking) stays.
	var mine := codes_of(action)
	mine.erase(code)
	if mine.is_empty():
		mine.append(code)
	else:
		mine[0] = code
	_bind(action, mine)
	_waiting = ""
	_hint.text = "%s: %s." % [_name(action), Prompts.keyed("{%s}" % action, false)]
	if not lost.is_empty():
		_hint.text += " %s no longer has it." % " and ".join(lost)
	_build_keys()
	_save()
	keys_changed.emit()


## Every key back to its default.
func _defaults() -> void:
	for row: Array in REMAP:
		_bind(row[0], KEYS[row[0]])
	_waiting = ""
	_hint.text = "Every key is back to its default."
	_build_keys()
	_save()
	keys_changed.emit()


## The keys' rows: each action's name and its keys on a button, three to a line.
func _build_keys() -> void:
	for child in _key_grid.get_children():
		_key_grid.remove_child(child)
		child.queue_free()
	_key_buttons.clear()
	for row: Array in REMAP:
		var action: String = row[0]
		var called := Look.text(Label.new(), Look.SMALL)
		called.text = row[1]
		called.custom_minimum_size = Vector2(170, 0)
		called.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		_key_grid.add_child(called)
		var keys := "press a key…" if action == _waiting else _keys_of(action)
		var button := _button(keys, _await.bind(action), 130.0)
		_key_buttons[action] = button
		_key_grid.add_child(button)


## A key's button was clicked: the next key pressed is its new one.
func _await(action: String) -> void:
	_waiting = action
	_modifier = KEY_NONE
	_hint.text = "The new key for %s. Esc keeps the old one." % _name(action)
	_build_keys()


func _keys_of(action: String) -> String:
	return Prompts.keyed("{%s|all}" % action, false) if not Prompts.keys(action).is_empty() else "—"


func _name(action: String) -> String:
	for row: Array in REMAP:
		if row[0] == action:
			return row[1]
	return action


## A setting's value, in Look (where what draws reads it); the window's theme for high contrast.
func _turn(setting: String, on: bool) -> void:
	match setting:
		"colour_blind":
			Look.set_look(on, Look.high_contrast)
		"high_contrast":
			Look.set_look(Look.colour_blind, on)
		"reduce_motion":
			Look.reduce_motion = on
			Look.version += 1
		"show_numbers":
			Look.show_numbers = on
			Look.version += 1


## What's built already, as a setting now has it: the panels' ground, and every text outlined in
## high contrast (a theme on the window, so what's built later has it too).
func _restyle() -> void:
	get_window().theme = Look.contrast() if Look.high_contrast else null
	Look.restyle(get_tree().root)


func _refresh() -> void:
	_gui_size.text = "%d%%" % roundi(gui_scale * 100.0)
	for setting: String in _toggles:
		(_toggles[setting] as Button).text = "On" if is_on(setting) else "Off"


func _save() -> void:
	if not keep:
		return
	var file := ConfigFile.new()
	file.load(FILE)  # whatever else is kept there
	file.set_value("gui", "scale", gui_scale)
	for toggle: Array in TOGGLES:
		file.set_value("look", toggle[0], is_on(toggle[0]))
	if file.has_section("keys"):
		file.erase_section("keys")
	for row: Array in REMAP:
		var now := codes_of(row[0])
		if now != KEYS[row[0]]:
			file.set_value("keys", row[0], now)
	file.save(FILE)


func _heading(text: String) -> Label:
	var heading := Look.text(Label.new(), Look.BODY)
	heading.add_theme_font_override("font", Look.SANS_MEDIUM)
	heading.text = text
	return heading


## A setting's line: its name, then what's added.
func _row(parent: Control, called: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	parent.add_child(row)
	var label := Look.text(Label.new(), Look.BODY)
	label.text = called
	label.custom_minimum_size = Vector2(NAME_WIDTH, 0)
	row.add_child(label)
	return row


func _dim_text(text: String) -> Label:
	var label := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	label.text = text
	label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	return label


## A button at least `width` wide, and 32 px high: a click target's least (§11).
func _button(text: String, pressed: Callable, width := 0.0) -> Button:
	return Look.button(text, pressed, Vector2(width, 32))
