extends CanvasLayer
## The bar (VISUALS.md §10, zone K; §11, the bar), as Diablo's: your powers on the number keys,
## then the belt's loops (potion.rs) on the keys after them, in one strip at the bottom centre. A
## slot shows what's on it (its glyph, its key, what a spell costs or how many potions the loop
## holds), dimmed while it can't be used now (not mana enough, no component, the Second Wind spent,
## nothing to mend, an empty loop), and outlined while its spell is readied. A click uses it, as
## its key does; hovering it shows what it is (a spell's numbers, a potion's card). A power dragged
## onto another swaps the two: the order is the player's, kept by class (settings.gd), and the
## number keys follow it. The core decides what can be done and does it; this shows it and asks.

const Look = preload("res://look.gd")
const Prompts = preload("res://prompts.gd")
const Codes = preload("res://codes.gd")
const Power = Codes.Power
const Spell = Codes.Spell

## The powers' slots (the number keys power_1 to power_4), then the belt's loops (belt_1 to belt_4).
const POWERS := 4
## The globes either side of the bar (VISUALS.md §11, Globe; Diablo's and Path of Exile's): across,
## in pixels at the base screen.
const GLOBE := 116.0
const GLOBE_SHADER := preload("res://shaders/globe.gdshader")
const LOOPS := 4
## The numbers a spell takes in DuskWorld.spell_info() (spells.gd's INFO), a potion in
## potion_info(), and a loop in character_belt().
const INFO := 10
const POTION := 4
const LOOP := 3
## A power's glyph (icons/powers/), by its code; a spell's by its kind.
const POWER_GLYPHS := {Power.TURN: "turn", Power.SECOND_WIND: "second_wind", Power.MEND: "mend"}
const SPELL_GLYPHS := {Spell.LIGHT: "light", Spell.SILENCE: "silence", Spell.FOG: "fog",
	Spell.FIREBALL: "fireball", Spell.BOLT: "firebolt", Spell.NOVA: "fire_nova"}
## What a power that isn't a spell is called, and what it does.
const POWER_TEXTS := {
	Power.TURN: ["Turn the dead", "Hold up the holy symbol: the dead near you flee, till a blow ends it. It costs faith."],
	Power.SECOND_WIND: ["Second Wind", "Your stamina back, and a little health: once between sleeps."],
	Power.MEND: ["Mending", "Faith closes your worst bleeding wound, and gives a little health back."],
}
## What each slot shows (`_shown`), by place: its glyph, the glyph's colour, its count, the
## count's colour, whether it can be used now, whether it's readied, its key, whether anything's
## there, and a potion's own icon once it's drawn (icons.gd), in place of the glyph.
enum { GLYPH, TINT, COUNT, COUNT_TINT, CAN, READIED, KEY, FILLED, ICON }

## Uses what's on a slot (main.gd's): the power on `k`, the potion in loop `k`.
var use_power: Callable
var use_loop: Callable
## Shows a slot's detail beside it, and takes it away (the HUD's hover_card and unhover_card).
var card: Callable
var uncard: Callable
## Where the bar's order is kept (settings.gd's bar_order and set_bar_order).
var settings: Node
## The items' icons (icons.gd): a loop shows its potion's own, drawn from its look.
var icons: Node

var _dusk: Node
var _base: PanelContainer  # the bar's frame
var _strip: HBoxContainer
var _globes: Array[ColorRect] = []  # health on the left; mana, faith or breath on the right
var _globe_words: Array[Label] = []  # each one's numbers: on hover, or always with numbers on
var _xp: ProgressBar  # how far into this level: thin, gold, along the bar's top
var _breath: ProgressBar  # your breath, thin, along its foot: when the right globe holds mana or faith
var _slots: Array[Button] = []
var _glyphs: Array[TextureRect] = []
var _caps: Array[Label] = []  # each slot's key
var _counts: Array[Label] = []  # a spell's cost, a loop's potions
var _textures := {}  # a glyph's name: its texture
var _powers: Array = []  # yours, in the bar's order: [code, spell] each
var _class := ""  # your class's name: what the order is kept under
var _spell_ids := PackedStringArray()
var _spell_names := PackedStringArray()
var _spell_lines := PackedStringArray()
var _item_names := PackedStringArray()
var _potions := PackedInt32Array()  # DuskWorld.potion_info()
var _belt := PackedInt32Array()  # DuskWorld.character_belt(), as last drawn
var _shown := []  # what the slots last showed: they're drawn again only when it changes
var _hovered := -1  # the slot under the mouse, -1 none


func _ready() -> void:
	# The bar stands in a frame of its own, as Diablo's does (art.gd's).
	var base := PanelContainer.new()
	_base = base
	var frame := Look.panel_style()
	frame.content_margin_left = 16.0
	frame.content_margin_right = 16.0
	frame.content_margin_top = 12.0
	frame.content_margin_bottom = 12.0
	base.add_theme_stylebox_override("panel", frame)
	base.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(base)
	# XP along the bar's top (the second playtest: "an xp bar for the character level on the main
	# ui"), and breath along its foot, thin, when the right globe holds mana or faith: kept apart,
	# as Diablo II keeps stamina from XP, since gold over amber reads as one bar.
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 6)
	column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	base.add_child(column)
	_xp = Look.bar(ProgressBar.new(), Look.GOLD)
	_xp.custom_minimum_size = Vector2(0, 8)
	_xp.max_value = 1.0
	_xp.step = 0.0
	_xp.mouse_filter = Control.MOUSE_FILTER_PASS  # for its tooltip
	# A faint gold edge, so the track reads as a bar even with no XP in it yet.
	var track: StyleBox = _xp.get_theme_stylebox("background")
	track.set("edge", Color(Look.GOLD, 0.35))
	track.set("width", 1.0)
	column.add_child(_xp)
	_strip = HBoxContainer.new()
	_strip.add_theme_constant_override("separation", 8)
	_strip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(_strip)
	_breath = Look.bar(ProgressBar.new(), Look.ENDURANCE)
	_breath.custom_minimum_size = Vector2(0, 6)
	_breath.max_value = 1.0
	_breath.step = 0.0
	_breath.mouse_filter = Control.MOUSE_FILTER_PASS
	_breath.visible = false
	column.add_child(_breath)
	for k in POWERS + LOOPS:
		if k == POWERS:
			# The belt's loops a little apart from the powers.
			var split := Control.new()
			split.custom_minimum_size = Vector2(16, 0)
			split.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_strip.add_child(split)
		_strip.add_child(_slot(k))
	base.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM,
		Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	base.grow_horizontal = Control.GROW_DIRECTION_BOTH
	base.grow_vertical = Control.GROW_DIRECTION_BEGIN
	for k in 2:
		_make_globe()
	visible = false


## A new world: the spells' ids, names and lines, the items' names and the potions, from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_spell_ids = dusk.spell_ids()
	_spell_names = dusk.spell_names()
	_spell_lines = dusk.spell_lines()
	_item_names = dusk.item_names()
	_potions = dusk.potion_info()
	_shown = []
	_unhover(_hovered)


## Each frame: the slots, drawn again only when what they show has changed. `here` is your tile
## (a spell is checked as if cast on it: what's too far is walked into reach, so only the rest
## dims it). Dead, there's no bar.
func refresh(dead: bool, readied: int, here: Vector2i) -> void:
	visible = not dead and _dusk != null
	if not visible:
		return
	_show_globes()
	_powers = _arrange()
	var info: PackedInt32Array = _dusk.spell_info()
	_belt = _dusk.character_belt()
	var now := []
	for k in POWERS:
		now.append(_power_shown(k, info, readied, here))
	for k in LOOPS:
		now.append(_loop_shown(k))
	if now == _shown:
		return
	_shown = now
	for k in now.size():
		_draw_slot(k, now[k])
	# What's hovered changed under the mouse: its detail with it.
	if _hovered >= 0:
		_hover(_hovered)


## The globes, each frame: health on the left; on the right mana (a Wizard's), else faith (a
## Cleric's), else your breath. Beside the bar's frame, their feet on its foot.
func _show_globes() -> void:
	var r := _base.get_rect()
	_globes[0].position = Vector2(r.position.x - GLOBE - 2.0, r.end.y - GLOBE + 4.0)
	_globes[1].position = Vector2(r.end.x + 2.0, r.end.y - GLOBE + 4.0)
	var body: PackedInt32Array = _dusk.character_body()
	var deeds: PackedInt32Array = _dusk.character_deeds()
	var most := deeds[4] / 10.0 if deeds.size() > 4 else 100.0
	_globe(0, body[0] / 10.0 if body.size() > 0 else 0.0, most, Look.HEALTH, "Health")
	var mana: PackedInt32Array = _dusk.character_mana()
	var faith: PackedInt32Array = _dusk.faith()
	var breath := body[1] / 10.0 if body.size() > 1 else 0.0
	_breath.value = breath / 100.0
	_breath.tooltip_text = "Stamina  ·  %d / 100" % roundi(breath)
	_breath.visible = true
	if mana.size() > 1 and mana[1] > 0:
		_globe(1, mana[0], mana[1], Look.ARCANE, "Mana")
	elif faith.size() > 1 and faith[1] > 0:
		_globe(1, faith[0], faith[1], Look.FAITH, "Faith")
	else:
		_globe(1, breath, 100.0, Look.ENDURANCE, "Stamina")
		_breath.visible = false
	# How far into this level (DuskWorld.character_deeds(): its XP, where the level began, the next).
	if deeds.size() > 3:
		var span := deeds[3] - deeds[2]
		_xp.value = 1.0 if deeds[3] < 0 or span <= 0 else clampf(float(deeds[1] - deeds[2]) / span, 0.0, 1.0)
		_xp.tooltip_text = "Level %d  ·  %d / %d XP" % [deeds[0], deeds[1], deeds[3]] if deeds[3] >= 0 else "Level %d" % deeds[0]


## Globe `k` filled to `now` of `most`, its liquid `colour`; low (under a quarter), it throbs, as a
## low bar does (none with less motion).
func _globe(k: int, now: float, most: float, colour: Color, word: String) -> void:
	var material := _globes[k].material as ShaderMaterial
	var fill := clampf(now / maxf(most, 1.0), 0.0, 1.0)
	material.set_shader_parameter("fill", fill)
	material.set_shader_parameter("liquid", colour)
	material.set_shader_parameter("deep", colour.darkened(0.82))
	material.set_shader_parameter("motion", 0.0 if Look.reduce_motion else 1.0)
	var low := fill > 0.0 and fill < 0.25 and not Look.reduce_motion
	material.set_shader_parameter("pulse",
		0.5 + 0.5 * sin(Time.get_ticks_msec() / 160.0) if low else 0.0)
	var words := _globe_words[k]
	words.text = "%s\n%d / %d" % [word, roundi(now), roundi(most)]
	words.visible = Look.show_numbers or bool(words.get_meta("hovered", false))


func _make_globe() -> void:
	var globe := ColorRect.new()
	globe.size = Vector2(GLOBE, GLOBE)
	var material := ShaderMaterial.new()
	material.shader = GLOBE_SHADER
	globe.material = material
	add_child(globe)
	var words := Look.text(Label.new(), Look.SMALL, Look.TEXT, true)
	words.set_anchors_preset(Control.PRESET_FULL_RECT)
	words.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	words.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	words.mouse_filter = Control.MOUSE_FILTER_IGNORE
	globe.add_child(words)
	globe.mouse_entered.connect(func() -> void: words.set_meta("hovered", true))
	globe.mouse_exited.connect(func() -> void: words.set_meta("hovered", false))
	_globes.append(globe)
	_globe_words.append(words)


## The power on key `k + 1`, in the bar's order: [its code, the spell's number], or [] for none.
func power_at(k: int) -> Array:
	if _dusk != null:
		_powers = _arrange()
	return _powers[k] if k >= 0 and k < _powers.size() else []


## The slot a spell is on (0 for the first key), -1 for none: the spells panel's keys.
func slot_of_spell(spell: int) -> int:
	for k in mini(_powers.size(), POWERS):
		if _powers[k][0] == Power.SPELL and _powers[k][1] == spell:
			return k
	return -1


## Swaps the powers on slots `a` and `b` (onto an empty one: to the last there is), and keeps
## the order for your class.
func swap(a: int, b: int) -> void:
	var keys := PackedStringArray()
	for p: Array in _powers:
		keys.append(_key(p[0], p[1]))
	b = mini(b, keys.size() - 1)
	if a < 0 or a >= keys.size() or a == b:
		return
	var moved := keys[a]
	keys[a] = keys[b]
	keys[b] = moved
	if settings != null:
		settings.set_bar_order(_class, keys)
	_powers = _arrange()
	_shown = []


## Each slot in words, for a script's news (main.gd prints them): "[1] Fireball, 20 mana",
## "[5] potion of life ×2", "[8] nothing".
func words() -> PackedStringArray:
	var out := PackedStringArray()
	for k in POWERS + LOOPS:
		var action := "power_%d" % (k + 1) if k < POWERS else "belt_%d" % (k - POWERS + 1)
		var said := "nothing"
		if k < POWERS and k < _powers.size():
			said = _detail(k)[0]
			var cost: String = _shown[k][COUNT] if k < _shown.size() else ""
			if cost != "":
				said += ", %s mana" % cost
		elif k >= POWERS and _belt.size() > (k - POWERS) * LOOP + 2 and _belt[(k - POWERS) * LOOP] >= 0:
			var i := (k - POWERS) * LOOP
			said = "%s ×%d" % [_item(_belt[i + 1]), _belt[i + 2]]
		out.append(Prompts.keyed("{%s} " % action) + said)
	return out


## Slot `k`: a button the size of a slot, its glyph in the middle, its key at the top left and its
## count at the bottom right. A click uses it, on letting go (a drag doesn't); a power's can be
## dragged onto another.
func _slot(k: int) -> Button:
	var slot := Button.new()
	slot.custom_minimum_size = Vector2(Look.SLOT, Look.SLOT)
	slot.focus_mode = Control.FOCUS_NONE
	_style(slot, false)
	var glyph := TextureRect.new()
	glyph.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	glyph.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	glyph.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.add_child(glyph)
	glyph.set_anchors_preset(Control.PRESET_FULL_RECT)
	var inset := (Look.SLOT - Look.SLOT_GLYPH) / 2.0
	glyph.offset_left = inset
	glyph.offset_top = inset
	glyph.offset_right = -inset
	glyph.offset_bottom = -inset
	var cap := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM, true)
	cap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	cap.position = Vector2(5, 0)
	slot.add_child(cap)
	var count := Look.text(Label.new(), Look.SMALL, Look.TEXT, true)
	count.mouse_filter = Control.MOUSE_FILTER_IGNORE
	count.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	slot.add_child(count)
	count.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_RIGHT, Control.PRESET_MODE_MINSIZE, 4)
	count.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	count.grow_vertical = Control.GROW_DIRECTION_BEGIN
	slot.pressed.connect(_use.bind(k))
	slot.mouse_entered.connect(_hover.bind(k))
	slot.mouse_exited.connect(_unhover.bind(k))
	if k < POWERS:
		slot.set_drag_forwarding(_drag.bind(k), _can_drop.bind(k), _drop.bind(k))
	_slots.append(slot)
	_glyphs.append(glyph)
	_caps.append(cap)
	_counts.append(count)
	return slot


## A slot's ground: the panel's, its edge brighter under the mouse; the readied one's in ARCANE.
func _style(slot: Button, readied: bool) -> void:
	var edge := Look.slot_style(Look.ARCANE, 2) if readied else Look.slot_style()
	var hover := edge if readied else Look.slot_style(Look.TEXT_DIM)
	for state: String in ["normal", "pressed", "disabled", "focus"]:
		slot.add_theme_stylebox_override(state, edge)
	slot.add_theme_stylebox_override("hover", hover)
	slot.add_theme_stylebox_override("hover_pressed", hover)


## Your powers in the bar's order: your class's kept order first, as far as they're still yours,
## then the rest in the core's (DuskWorld.character_powers()).
func _arrange() -> Array:
	_class = _dusk.class_name()
	var powers: PackedInt32Array = _dusk.character_powers()
	var all := []
	for i in range(0, powers.size() - 1, 2):
		all.append([powers[i], powers[i + 1]])
	var order: PackedStringArray = settings.bar_order(_class) if settings != null else PackedStringArray()
	# What the kept order doesn't name yet (a power new since it was kept: the second playtest's
	# fire spells) comes first, in your class's order, so it's on the bar; then the kept order.
	var out := []
	for p: Array in all:
		if not order.has(_key(p[0], p[1])):
			out.append(p)
	for key in order:
		for p: Array in all:
			if _key(p[0], p[1]) == key and not out.has(p):
				out.append(p)
	return out


## A power as the kept order names it: "turn", "second_wind", "mend", or "spell:" and its id.
func _key(code: int, spell: int) -> String:
	match code:
		Power.TURN:
			return "turn"
		Power.SECOND_WIND:
			return "second_wind"
		Power.MEND:
			return "mend"
	return "spell:" + (_spell_ids[spell] if spell >= 0 and spell < _spell_ids.size() else str(spell))


## What power slot `k` shows (`_shown`'s places): a spell's glyph in ARCANE with its mana; the
## Cleric's in HOLY, the Second Wind in ENDURANCE.
func _power_shown(k: int, info: PackedInt32Array, readied: int, here: Vector2i) -> Array:
	var key := Prompts.key("power_%d" % (k + 1))
	if k >= _powers.size():
		return ["", Look.TEXT, "", Look.TEXT, false, false, key, false]
	var code: int = _powers[k][0]
	var spell: int = _powers[k][1]
	match code:
		Power.TURN:
			return [POWER_GLYPHS[code], Look.HOLY, "", Look.TEXT, _dusk.turn_check() == 1, false, key, true]
		Power.MEND:
			return [POWER_GLYPHS[code], Look.HOLY, "", Look.TEXT, _dusk.heal_check() == 1, false, key, true]
		Power.SECOND_WIND:
			return [POWER_GLYPHS[code], Look.ENDURANCE, "", Look.TEXT, _dusk.can_second_wind(), false,
				key, true]
	var i := spell * INFO
	var kind: int = info[i] if i + 1 < info.size() else -1
	var mana: int = info[i + 1] if i + 1 < info.size() else 0
	# Too far or out of sight is no matter: it's walked into reach (spell.rs).
	var can: bool = _dusk.cast_check(spell, here.x, here.y) in [0, -5, -6]
	return [SPELL_GLYPHS.get(kind, ""), Look.ARCANE, str(mana) if mana > 0 else "", Look.ARCANE,
		can, spell == readied, key, true]


## What loop `k` shows: a flask in what it does, and how many; an empty loop's flask faint.
func _loop_shown(k: int) -> Array:
	var key := Prompts.key("belt_%d" % (k + 1))
	var i := k * LOOP
	if i + 2 >= _belt.size() or _belt[i] < 0:
		return ["potion", Look.TEXT, "", Look.TEXT, false, false, key, false]
	var def := _belt[i + 1]
	var effect := _potions[def * POTION] if def * POTION < _potions.size() else -1
	var tint: Color = Look.POTIONS[effect] if effect >= 0 and effect < Look.POTIONS.size() else Look.TEXT
	var can: bool = _dusk.use_ticks(_belt[i], def) >= 0
	var icon: Texture2D = icons.icon(def) if icons != null else null
	return ["potion", tint, str(_belt[i + 2]), Look.TEXT, can, false, key, true, icon]


func _draw_slot(k: int, shown: Array) -> void:
	# A potion's own icon when it's drawn (its own colours), else the glyph in its tint.
	var icon: Texture2D = shown[ICON] if shown.size() > ICON else null
	_glyphs[k].texture = icon if icon != null else _texture(shown[GLYPH])
	var tint: Color = Color.WHITE if icon != null else shown[TINT]
	# Dimmed while it can't be used; an empty slot's glyph only a trace.
	var alpha := 1.0 if shown[CAN] else (0.35 if shown[FILLED] else 0.12)
	_glyphs[k].modulate = Color(tint, tint.a * alpha)
	_caps[k].text = shown[KEY]
	_counts[k].text = shown[COUNT]
	_counts[k].add_theme_color_override("font_color", shown[COUNT_TINT] if shown[CAN] else Look.TEXT_DIM)
	_style(_slots[k], shown[READIED])


func _texture(glyph: String) -> Texture2D:
	if glyph == "":
		return null
	if not _textures.has(glyph):
		var path := "res://icons/potion.svg" if glyph == "potion" else "res://icons/powers/%s.svg" % glyph
		_textures[glyph] = load(path) as Texture2D
	return _textures[glyph]


func _use(k: int) -> void:
	if k < POWERS:
		use_power.call(k)
	else:
		use_loop.call(k - POWERS)


## Slot `k`'s detail: [its title, its lines (a card's: DuskWorld.card_lines(), three words a
## line), the title's colour], or [] for none. A spell's numbers as yours; a potion's own card.
func _detail(k: int) -> Array:
	var rows := []
	if k >= POWERS:
		var i := (k - POWERS) * LOOP
		if i + 2 >= _belt.size() or _belt[i] < 0:
			rows.append_array(["note", "A potion you take goes on your belt first: a loop for each kind, a few to a loop. Drink it with its key.", ""])
			return ["An empty loop of your belt", PackedStringArray(rows), Look.TEXT]
		var lines: PackedStringArray = _dusk.card_lines(2 + _belt[i])
		if lines.size() >= 3 and lines[0] == "title":
			return [lines[1], lines.slice(3), _shown[k][TINT] if k < _shown.size() else Look.TEXT]
		return [_item(_belt[i + 1]), lines, Look.TEXT]
	if k >= _powers.size():
		return []
	var code: int = _powers[k][0]
	if code != Power.SPELL:
		var texts: Array = POWER_TEXTS.get(code, ["", ""])
		rows.append_array(["note", texts[1], ""])
		return [texts[0], PackedStringArray(rows), Look.TEXT]
	var spell: int = _powers[k][1]
	var info: PackedInt32Array = _dusk.spell_info()
	var i := spell * INFO
	if i + INFO > info.size():
		return []
	rows.append_array(["kind", "Spell", "", "rule", "", ""])
	rows.append_array(["stat", "Mana", "none" if info[i + 1] == 0 else str(info[i + 1])])
	if info[i + 7] >= 0:
		rows.append_array(["stat", "Component", _item(info[i + 7])])
	rows.append_array(["stat", "Words", "%.1f s" % (info[i + 2] / 10.0)])
	rows.append_array(["stat", "Range", "%d tiles" % info[i + 3]])
	if info[i + 9] > 0:
		rows.append_array(["stat", "Damage", "%s to %s" % [_points(info[i + 8]), _points(info[i + 9])]])
	elif info[i + 6] > 0:
		rows.append_array(["stat", "Lasts", "%d s" % info[i + 6]])
	rows.append_array(["rule", "", ""])
	rows.append_array(["note", _spell_lines[spell] if spell < _spell_lines.size() else "", ""])
	rows.append_array(["note", "Ready it, then click the spot. Right-click lets it go.", ""])
	var called := _spell_names[spell] if spell < _spell_names.size() else "Spell"
	return [called, PackedStringArray(rows), Look.ARCANE]


func _hover(k: int) -> void:
	_hovered = k
	var shown := _detail(k)
	if shown.is_empty():
		uncard.call(_slots[k])
		return
	# The powers' detail to the left, the belt's to the right: never over the bar.
	card.call(_slots[k], shown[0], shown[1], k >= POWERS, shown[2])


func _unhover(k: int) -> void:
	if k >= 0 and k == _hovered:
		_hovered = -1
		uncard.call(_slots[k])


## A power picked up to move: what it is, and its glyph under the mouse.
func _drag(_at: Vector2, k: int) -> Variant:
	if k >= _powers.size():
		return null
	var preview := TextureRect.new()
	preview.texture = _glyphs[k].texture
	preview.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	preview.size = Vector2(Look.SLOT_GLYPH, Look.SLOT_GLYPH)
	preview.modulate = _glyphs[k].modulate
	_slots[k].set_drag_preview(preview)
	return {"bar": k}


func _can_drop(_at: Vector2, data: Variant, k: int) -> bool:
	return data is Dictionary and (data as Dictionary).has("bar") and k < POWERS


func _drop(_at: Vector2, data: Variant, k: int) -> void:
	swap(int((data as Dictionary)["bar"]), k)


func _item(def: int) -> String:
	return _item_names[def] if def >= 0 and def < _item_names.size() else "something"


## Tenths of a point, as the card writes them: "25", "33.7".
static func _points(tenths: int) -> String:
	return str(tenths / 10) if tenths % 10 == 0 else "%.1f" % (tenths / 10.0)
