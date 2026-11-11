extends CanvasLayer
## Zomboid's inventory windows, as lists with weights (CONCEPT.md: no grid). On the right, your
## pack: what's in your hand, and what you carry, with its weight against what you can carry
## before you can't run. On the left, what's within your reach: each container beside you, and
## the floor, and water to drink from. Each line has its buttons: they ask the core for a timed
## action, which a blow breaks off. "Take all" and "Stash all finds" ask for one after another.
##
## Your stash (home.rs): a find (one that came out of the world) put into it pays its worth in
## XP, once. The pack marks finds with their worth; with your stash within reach, each line has
## a Stash button.
##
## A holdout's counter (holdout.rs), within reach, shows what it sells today and what it's short
## of; the pack's lines then have a Sell button (a find sold pays its XP too), and the weapon in
## your hand a Mend button.

const WEAPON := 1
const BANDAGE := 2
const FOOD := 3
const DRINK := 4
const LIGHT := 5
const CURE := 6
const OIL := 7
## Numbers a stack takes, and a container's header, in DuskWorld's arrays.
const STACK := 4
const HEADER := 5
## Why a container can't be your stash, by DuskWorld.stash_check().
const NO_STASH := {-1: "That's not a container within your reach.",
	-2: "A stash wants a roof over it.",
	-3: "You have no home yet: sleep a night through somewhere first.",
	-4: "That's too far from where you sleep to be your stash."}

## Says a line of news (the HUD's).
var say: Callable
## Burns the body beside you, or says why not (main.gd's G).
var burn: Callable

var _dusk: Node
var _names := PackedStringArray()
var _kinds := PackedInt32Array()
var _weights := PackedInt32Array()
var _worths := PackedInt32Array()
var _fixtures := PackedStringArray()
var _pack: VBoxContainer
var _reach: VBoxContainer
var _pack_panel: PanelContainer
var _reach_panel: PanelContainer
var _shown_pack := []  # what the pack panel shows now, to rebuild only when it changes
var _shown_reach := PackedInt32Array()
var _shown_water := false
var _queue: Array = []  # actions still to ask for: ["take" or "put", x, y, def]
var _shown_counter := []  # the counter within reach and what it sells, as last drawn


func _ready() -> void:
	_pack_panel = _panel(Control.PRESET_BOTTOM_RIGHT)
	_pack = _pack_panel.get_child(0)
	_reach_panel = _panel(Control.PRESET_BOTTOM_LEFT)
	_reach_panel.position.y -= 100.0  # above the legend
	_reach_panel.visible = false  # until something's within reach
	_reach = _reach_panel.get_child(0)


## A new world: the item names and the fixtures' come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.item_names()
	_kinds = dusk.item_kinds()
	_weights = dusk.item_weights()
	_worths = dusk.item_worths()
	_fixtures = dusk.fixture_names()
	_shown_pack = []
	_shown_reach = PackedInt32Array()
	_shown_water = false
	_shown_counter = []
	_draw_reach(_shown_reach, false, PackedInt32Array(), PackedInt32Array())
	_queue.clear()


func toggle() -> void:
	_pack_panel.visible = not _pack_panel.visible


## Each frame: redraw what changed, and ask for the next queued action once you're free.
## `busy`: you're in the middle of an action; `dead`: nothing more to do.
func refresh(busy: bool, dead: bool) -> void:
	if _dusk == null:
		return
	if dead:
		_pack_panel.visible = false
		_reach_panel.visible = false
		return
	var hands: PackedInt32Array = _dusk.character_hands()
	var items: PackedInt32Array = _dusk.character_items()
	var carried: PackedInt32Array = _dusk.character_load()
	var light: PackedInt32Array = _dusk.character_light()
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	var stash := _stash_in(reach)
	var counter: PackedInt32Array = _dusk.counter_in_reach()
	var coins: int = _dusk.coins()
	var pack := [hands, items, carried, light, stash, counter, coins]
	if pack != _shown_pack:
		_shown_pack = pack
		_draw_pack(hands, items, carried, light, stash, counter, coins)
	var water: bool = _dusk.water_in_reach()
	var sells := PackedInt32Array()
	if counter.size() >= 3:
		sells = _dusk.counter_sells(counter[0], counter[1])
	var at_counter := [counter, sells]
	if reach != _shown_reach or water != _shown_water or at_counter != _shown_counter:
		_shown_reach = reach
		_shown_water = water
		_shown_counter = at_counter
		_draw_reach(reach, water, counter, sells)
	if not busy and not _queue.is_empty():
		_next_queued(reach, items)


## Where your stash is, if it's within reach; else (-1, -1).
func _stash_in(reach: PackedInt32Array) -> Vector2i:
	var i := 0
	while i + HEADER <= reach.size():
		if reach[i + 4] == 1:
			return Vector2i(reach[i], reach[i + 1])
		i += HEADER + reach[i + 3] * STACK
	return Vector2i(-1, -1)


func _draw_pack(hands: PackedInt32Array, items: PackedInt32Array, carried: PackedInt32Array,
		light: PackedInt32Array, stash: Vector2i, counter: PackedInt32Array, coins: int) -> void:
	for child in _pack.get_children():
		child.queue_free()
	if carried.size() < 2:
		return
	var heavy := carried[0] > carried[1]
	var title := _line(_pack, "Pack  ·  %.1f / %.1f kg  ·  %d coin%s" % [carried[0] / 1000.0,
		carried[1] / 1000.0, coins, "  ·  too heavy to run" if heavy else ""], 17)
	if heavy:
		title.add_theme_color_override("font_color", Color(1.0, 0.55, 0.4))
	var trading := counter.size() >= 3
	if hands.size() >= 2:
		var row := _row(_pack, "In hand: %s (%d%%)" % [_name(hands[0]), roundi(hands[1] / 10.0)])
		_button(row, "Put away", func() -> void: _dusk.unwield())
		if trading:
			var cost: int = _dusk.mend_check(counter[0], counter[1])
			if cost > 0 or cost == -2:
				_button(row, "Mend", func() -> void: _mend(counter[0], counter[1]))
	else:
		_line(_pack, "In hand: nothing (bare fists)", 15)
	if light.size() >= 2:
		var row := _row(_pack, "Held up: %s, lit (%d%% left)" % [_name(light[0]), roundi(light[1] / 10.0)])
		_button(row, "Put out", func() -> void: _dusk.douse())
	var slot := -1
	for i in range(0, items.size(), STACK):
		slot += 1
		var def := items[i]
		var count := items[i + 1]
		var label := _describe(def, count, items[i + 3] == 1)
		if _kind(def) == LIGHT and items[i + 2] < 1000:
			label += "  ·  %d%% left" % roundi(items[i + 2] / 10.0)  # a part-burnt light
		var row := _row(_pack, label)
		match _kind(def):
			WEAPON:
				_button(row, "Wield", func() -> void: _dusk.wield(slot, def))
			BANDAGE:
				_button(row, "Use", func() -> void: _dusk.use_item(slot, def))
			FOOD:
				_button(row, "Eat", func() -> void: _dusk.use_item(slot, def))
			DRINK:
				_button(row, "Drink", func() -> void: _dusk.use_item(slot, def))
			LIGHT:
				_button(row, "Light", func() -> void: _dusk.light(slot, def))
			CURE:
				_button(row, "Take", func() -> void: _take_cure(slot, def))
			OIL:
				_button(row, "Burn a body", func() -> void: burn.call())
		if stash.x >= 0:
			_button(row, "Stash", func() -> void: _dusk.put_item(stash.x, stash.y, slot, def))
		if trading:
			var price: int = _dusk.sell_price(counter[0], counter[1], slot, def)
			if price >= 0:
				_button(row, "Sell · %d" % price, func() -> void: _dusk.sell(counter[0], counter[1], slot, def))
		_button(row, "Drop", func() -> void: _dusk.drop_item(slot, def))


## The cure, taken only with a fever to cure: the core won't start it otherwise, so say why.
func _take_cure(slot: int, def: int) -> void:
	if _dusk.character_fever()[0] == 0:
		say.call("You've no fever to cure. Keep it: a ghoul's bite brings one.")
	else:
		_dusk.use_item(slot, def)


func _draw_reach(reach: PackedInt32Array, water: bool, counter: PackedInt32Array,
		sells: PackedInt32Array) -> void:
	for child in _reach.get_children():
		child.queue_free()
	_reach_panel.visible = reach.size() > 0 or water or counter.size() >= 3
	if counter.size() >= 3:
		_draw_counter(counter, sells)
	if water:
		var row := _row(_reach, "Water")
		_button(row, "Drink", func() -> void: _dusk.drink_water())
	var i := 0
	while i + HEADER <= reach.size():
		var x := reach[i]
		var y := reach[i + 1]
		var fixture := reach[i + 2]
		var n := reach[i + 3]
		var mine := reach[i + 4] == 1
		i += HEADER
		var place := "On the ground" if fixture == 0 else _fixture_name(fixture).capitalize()
		if mine:
			place += "  ·  your stash"
		var header := _row(_reach, place + ("  (empty)" if n == 0 else ""))
		if mine:
			header.get_child(0).add_theme_color_override("font_color", Color(1.0, 0.82, 0.45))
			_button(header, "Stash all finds", func() -> void: _stash_finds(x, y))
		elif fixture != 0:
			_button(header, "Make it your stash", func() -> void: _mark_stash(x, y))
		if n > 1:
			var defs := []
			for k in n:
				defs.append(reach[i + k * STACK])
			_button(header, "Take all", func() -> void: _take_all(x, y, defs))
		for slot in n:
			var def := reach[i + slot * STACK]
			var count := reach[i + slot * STACK + 1]
			var found := reach[i + slot * STACK + 3] == 1
			var row := _row(_reach, "   " + _describe(def, count, found))
			_button(row, "Take", func() -> void: _dusk.take(x, y, slot, def))
		i += n * STACK


## A holdout's counter: what it sells today, at what price and how many are left, and what it's
## short of (selling it those pushes the dusk back).
func _draw_counter(counter: PackedInt32Array, sells: PackedInt32Array) -> void:
	var x := counter[0]
	var y := counter[1]
	var name: String = _dusk.holdout_name(counter[2])
	# "The abbey on the hill": only the first letter raised (capitalize() raises every word).
	var title := name.left(1).to_upper() + name.substr(1)
	var header := _line(_reach, title + "  ·  the counter", 16)
	header.add_theme_color_override("font_color", Color(0.85, 0.9, 1.0))
	var wants: PackedInt32Array = _dusk.counter_wants(x, y)
	if wants.size() > 0:
		var short := []
		for w in wants:
			short.append(_name(w))
		_line(_reach, "Short of: %s.  Sell it these to push the dusk back." % ", ".join(short), 13)
	for line in range(0, sells.size() / 4):
		var def := sells[line * 4]
		var price := sells[line * 4 + 1]
		var left := sells[line * 4 + 2]
		var can := sells[line * 4 + 3]
		var text := "   %s  ·  %d coin  ·  %s" % [_name(def), price,
			"none left today" if left == 0 else "%d left today" % left]
		var row := _row(_reach, text)
		_button(row, "Buy", func() -> void: _buy(x, y, line, can))


## "Buy": the core says whether you can, and this says why not.
func _buy(x: int, y: int, line: int, can: int) -> void:
	if can == 1:
		_dusk.buy(x, y, line)
	elif say.is_valid():
		say.call("None left today: more tomorrow." if can == -1 else "You haven't the coin.")


## "Mend": the weapon in your hand, made good as new for its price.
func _mend(x: int, y: int) -> void:
	var cost: int = _dusk.mend_check(x, y)
	if cost > 0:
		_dusk.mend(x, y)
		if say.is_valid():
			say.call("Mending it costs %d coin." % cost)
	elif say.is_valid():
		say.call("You haven't the coin to have it mended." if cost == -2 else "There's nothing to mend.")


## "Make it your stash": the core says whether it can be, and this says why not.
func _mark_stash(x: int, y: int) -> void:
	var why: int = _dusk.stash_check(x, y)
	if why == 0:
		_dusk.mark_stash(x, y)
	elif say.is_valid():
		say.call(NO_STASH.get(why, "It can't be your stash."))


## "Take all": one take after another, each asked for as the last one finishes.
func _take_all(x: int, y: int, defs: Array) -> void:
	for def in defs:
		_queue.append(["take", x, y, def])


## "Stash all finds": every find in the pack, put into the stash one after another.
func _stash_finds(x: int, y: int) -> void:
	var items: PackedInt32Array = _dusk.character_items()
	for i in range(0, items.size(), STACK):
		if items[i + 3] == 1:
			_queue.append(["put", x, y, items[i]])
	if _queue.is_empty() and say.is_valid():
		say.call("You're carrying no finds.")


## The next queued action: a take of the first stack on that tile that's still the item it was,
## or a put of the first find of that item in the pack.
func _next_queued(reach: PackedInt32Array, items: PackedInt32Array) -> void:
	var want: Array = _queue.pop_front()
	if want[0] == "put":
		for i in range(0, items.size(), STACK):
			if items[i] == want[3] and items[i + 3] == 1:
				_dusk.put_item(want[1], want[2], floori(i / float(STACK)), want[3])
				return
		return
	var i := 0
	while i + HEADER <= reach.size():
		var n := reach[i + 3]
		if reach[i] == want[1] and reach[i + 1] == want[2]:
			for slot in n:
				if reach[i + HEADER + slot * STACK] == want[3]:
					_dusk.take(want[1], want[2], slot, want[3])
					return
		i += HEADER + n * STACK


## "3 apples · 0.5 kg", and for a find, what it brings home.
func _describe(def: int, count: int, found := false) -> String:
	var grams := _weights[def] * count if def < _weights.size() else 0
	var label := _name(def) if count == 1 else "%s ×%d" % [_name(def), count]
	label = "%s  ·  %.1f kg" % [label, grams / 1000.0]
	var worth := _worths[def] * count if def < _worths.size() else 0
	if found and worth > 0:
		label += "  ·  a find: %d XP home" % worth
	return label


func _name(def: int) -> String:
	return _names[def] if def < _names.size() else "something"


func _kind(def: int) -> int:
	return _kinds[def] if def < _kinds.size() else 0


func _fixture_name(fixture: int) -> String:
	return _fixtures[fixture] if fixture < _fixtures.size() else "container"


func _panel(preset: Control.LayoutPreset) -> PanelContainer:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.06, 0.08, 0.72)
	style.set_content_margin_all(10)
	style.set_corner_radius_all(4)
	panel.add_theme_stylebox_override("panel", style)
	var list := VBoxContainer.new()
	list.add_theme_constant_override("separation", 2)
	panel.add_child(list)
	add_child(panel)
	panel.set_anchors_and_offsets_preset(preset, Control.PRESET_MODE_MINSIZE, 16)
	panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN if preset == Control.PRESET_BOTTOM_RIGHT \
		else Control.GROW_DIRECTION_END
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	return panel


func _line(parent: Control, text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	parent.add_child(label)
	return label


func _row(parent: Control, text: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	parent.add_child(row)
	var label := _line(row, text, 15)
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.custom_minimum_size = Vector2(250, 0)
	return row


func _button(row: HBoxContainer, text: String, pressed: Callable) -> void:
	var button := Button.new()
	button.text = text
	button.add_theme_font_size_override("font_size", 13)
	button.focus_mode = Control.FOCUS_NONE  # keys stay with the game (Space, WASD)
	button.pressed.connect(pressed)
	row.add_child(button)
