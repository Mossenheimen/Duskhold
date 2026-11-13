extends "res://book_panel.gd"
## Spells (P): the ones you know, each with what it costs, how long its words take and how far it
## reaches, and a Ready button (or its number key: your powers). A readied spell is cast on the
## next spot you click (main.gd); right-click lets it go. The core decides what can be cast and
## does it; this only shows it and asks.

const Prompts = preload("res://prompts.gd")
const Power = preload("res://codes.gd").Power

## The numbers a spell takes in DuskWorld.spell_info().
const INFO := 8

## Readies a spell, by its number (main.gd's).
var ready_spell: Callable

var _names := PackedStringArray()
var _lines := PackedStringArray()
var _items := PackedStringArray()


## A new world: the spells' names and lines come from the core.
func start(dusk: Node) -> void:
	super(dusk)
	_names = dusk.spell_names()
	_lines = dusk.spell_lines()
	_items = dusk.item_names()


## Each frame: redraw if anything changed (only while it's open): what you know, your slots, and
## `readied`, the spell readied now (-1 none).
func refresh(dead: bool, readied: int) -> void:
	if not _awake(dead):
		return
	var now := [_dusk.character_spells(), _dusk.character_slots(), _dusk.spell_info(), readied]
	if _changed(now):
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
	if known.is_empty():
		_list.add_child(Look.line("Spells  ·  only a Wizard casts them", Look.HEADING))
		return
	var left := slots[0] if slots.size() > 1 else 0
	var most := slots[1] if slots.size() > 1 else 0
	_list.add_child(Look.line("Spells  ·  %d of %d slots  ·  all back with a night's sleep" % [left, most], Look.HEADING))
	for k in known.size():
		var spell := known[k]
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var lines := VBoxContainer.new()
		lines.add_theme_constant_override("separation", 0)
		lines.custom_minimum_size = Vector2(340, 0)
		row.add_child(lines)
		var key := _key_for(spell)
		var called := Look.line(key + (_names[spell] if spell < _names.size() else "?") + "  ·  " + describe(spell, info), Look.BODY)
		if spell == readied:
			called.add_theme_color_override("font_color", Look.ARCANE)
		lines.add_child(called)
		var what := Look.line(_lines[spell] if spell < _lines.size() else "", Look.SMALL)
		what.modulate = Look.TEXT_DIM
		what.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lines.add_child(what)
		var word := "Readied" if spell == readied else "Ready"
		row.add_child(Look.button(word, func() -> void: ready_spell.call(spell)))
	var hint := Look.line("Ready one, then click the spot to cast it on. Right-click lets it go. A blow breaks the words.", Look.SMALL)
	hint.modulate = Look.TEXT_DIM
	_list.add_child(hint)


## The number key a spell is on, as its cap and a space ("[2] "), by DuskWorld.character_powers();
## "" if it's on none (a fifth spell, say).
func _key_for(spell: int) -> String:
	var powers: PackedInt32Array = _dusk.character_powers()
	for p in range(0, mini(powers.size(), 8), 2):
		if powers[p] == Power.SPELL and powers[p + 1] == spell:
			return Prompts.keyed("{power_%d} " % (p / 2 + 1))
	return ""
