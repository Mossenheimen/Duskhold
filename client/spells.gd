extends CanvasLayer
## Spells (I): the ones you know, each with what it costs, how long its words take and how far it
## reaches, and a Ready button (or its key, 4 to 7). A readied spell is cast on the next spot you
## click (main.gd); right-click lets it go. The core decides what can be cast and does it; this
## only shows it and asks.

const Look = preload("res://look.gd")

## The numbers a spell takes in DuskWorld.spell_info().
const INFO := 8

## Readies a spell, by its number (main.gd's).
var ready_spell: Callable

var _dusk: Node
var _panel: PanelContainer
var _list: VBoxContainer
var _names := PackedStringArray()
var _lines := PackedStringArray()
var _items := PackedStringArray()
var _shown := []  # what you know, your slots and what's readied, as last drawn


func _ready() -> void:
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", Look.panel_style())
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 3)
	_panel.add_child(_list)
	add_child(_panel)
	# Zone J, the left middle, where skills and making open too: one book panel at a time.
	_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_LEFT, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	_panel.visible = false


## A new world: the spells' names and lines come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.spell_names()
	_lines = dusk.spell_lines()
	_items = dusk.item_names()
	_shown = []


func close() -> void:
	_panel.visible = false


func toggle() -> void:
	_panel.visible = not _panel.visible


## Each frame: redraw if anything changed (only while it's open). `readied`: the spell readied
## now, -1 none.
func refresh(dead: bool, readied: int) -> void:
	if _dusk == null:
		return
	if dead:
		_panel.visible = false
		return
	if not _panel.visible:
		return
	var now := [_dusk.character_spells(), _dusk.character_slots(), _dusk.spell_info(), readied]
	if now == _shown:
		return
	_shown = now
	_draw(now[0], now[1], now[2], readied)


## What a spell costs, its words and its range: "2 slots and sulphur  ·  2.5 s  ·  8 tiles".
func describe(spell: int, info: PackedInt32Array) -> String:
	var i := spell * INFO
	if i + INFO > info.size():
		return ""
	var slots := info[i + 1]
	var cost := "a cantrip" if slots == 0 else ("1 slot" if slots == 1 else "%d slots" % slots)
	if info[i + 7] >= 0 and info[i + 7] < _items.size():
		cost += " and " + _items[info[i + 7]]
	return "%s  ·  %.1f s  ·  %d tiles" % [cost, info[i + 2] / 10.0, info[i + 3]]


func _draw(known: PackedInt32Array, slots: PackedInt32Array, info: PackedInt32Array,
		readied: int) -> void:
	for child in _list.get_children():
		child.queue_free()
	if known.is_empty():
		_list.add_child(_label("Spells  ·  only a Wizard casts them", Look.HEADING))
		return
	var left := slots[0] if slots.size() > 1 else 0
	var most := slots[1] if slots.size() > 1 else 0
	_list.add_child(_label("Spells  ·  %d of %d slots  ·  all back with a night's sleep" % [left, most], Look.HEADING))
	for k in known.size():
		var spell := known[k]
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var lines := VBoxContainer.new()
		lines.add_theme_constant_override("separation", 0)
		lines.custom_minimum_size = Vector2(340, 0)
		row.add_child(lines)
		var key := "%d  " % (k + 4) if k < 4 else ""
		var called := _label(key + (_names[spell] if spell < _names.size() else "?") + "  ·  " + describe(spell, info), Look.BODY)
		if spell == readied:
			called.add_theme_color_override("font_color", Look.ARCANE)
		lines.add_child(called)
		var what := _label(_lines[spell] if spell < _lines.size() else "", Look.SMALL)
		what.modulate = Look.TEXT_DIM
		what.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lines.add_child(what)
		var label := "Readied" if spell == readied else "Ready"
		row.add_child(_button(label, func() -> void: ready_spell.call(spell)))
	var hint := _label("Ready one, then click the spot to cast it on. Right-click lets it go. A blow breaks the words.", Look.SMALL)
	hint.modulate = Look.TEXT_DIM
	_list.add_child(hint)


func _button(text: String, pressed: Callable) -> Button:
	var button := Button.new()
	button.text = text
	button.add_theme_font_size_override("font_size", Look.SMALL)
	button.focus_mode = Control.FOCUS_NONE  # keys stay with the game
	button.pressed.connect(pressed)
	return button


func _label(text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	return label
