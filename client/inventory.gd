extends CanvasLayer
## The inventory, as Path of Exile 2's (the user, 2026-11-16: "an inventory system that looks like
## PoE 2"; BACKLOG.md #7; VISUALS.md §10, zone H, and §11, Grid). I or Tab opens it, and Esc or
## either key again shuts it. On the right, your pack: what you hold and wear round a figure, your
## belt under it, then your pockets' grid and the bag's on your back (pack.rs), each thing drawn
## from its own look (icons.gd) over the cells it takes up. On the left, what's within your reach,
## a tab each: a container's grid, what lies on the ground, a holdout's counter. The world goes on
## while it's open: moving a thing from one grid to another is an act, and takes its time.
##
## Click a thing to pick it up, and click where it goes to put it down: in its own grid it moves at
## once (`arrange`); between your pockets and your bag, out of a container or into one, it's an act
## (`shift_to`, `take_to`, `put_to`); on the figure it's taken in hand, put on or belted; out on the
## world it's dropped. Where it would go shows green, or red where it can't: the core's own checks
## say which. Right-click uses a thing as its button did (bag.gd's `use`), or takes it from what's
## within reach; Ctrl-click moves it across: into the container open on the left, sold at the
## counter, dropped on the ground, or between your pockets and your bag; out of a container into
## your pack. Hovering a thing shows its card.

const Look = preload("res://look.gd")
const Prompts = preload("res://prompts.gd")
const Bag = preload("res://bag.gd")
const Item = preload("res://codes.gd").Item

## Pixels a cell takes up at the base screen of 1920 × 1080: about Path of Exile 2's.
const CELL := 50
## Numbers a stack takes in DuskWorld's arrays, a place's header in stores_in_reach(), a loop in
## character_belt(), and a bounty in counter_bounties().
const STACK := 4
const HEADER := 5
const LOOP := 3
const BOUNTY := 5
## The belt's loops (hotbar.gd's).
const LOOPS := 4
## What's within reach opens under zones A and B (the status line, the bars and the body), so your
## health stays in sight while you loot.
const REACH_TOP := 250
## The pockets' and the bag's grids side by side are twelve cells and a gap across, as Path of
## Exile 2's are; the ground and a counter are laid out as wide.
const GAP := 20
## The height of the line over each grid (its name and weight; the bag's Drop button): the same
## for both, so the pockets' grid and the bag's start level (the third notes: "Bag grid is
## slightly offset from pocket grid": the button made the bag's line the taller).
const HEAD := 34
const ACROSS := 12
## The figure (Path of Exile 2's paper doll): what's in hand to the left, the off arm to the right,
## the head over the body, the bag on your back beside the head, arms and legs either side of the
## body's foot, and the belt's loops under it. Each slot: its name, its cells, its top-left.
const FIGURE := Vector2(580, 320)
const SLOTS := {
	"hand": ["In hand", Vector2i(2, 4), Vector2(0, 0)],
	"head": ["Head", Vector2i(2, 2), Vector2(240, 0)],
	"back": ["Back", Vector2i(2, 2), Vector2(360, 0)],
	"off": ["Off arm", Vector2i(2, 4), Vector2(480, 0)],
	"torso": ["Body", Vector2i(2, 3), Vector2(240, 110)],
	"arms": ["Arms", Vector2i(2, 2), Vector2(120, 160)],
	"legs": ["Legs", Vector2i(2, 2), Vector2(360, 160)],
}
const BELT_AT := Vector2(190, 270)
## Where a piece of armour shows: the slot of the first place it covers (DuskWorld.item_covers()'
## bits), and the rest of what it covers dimmed.
const PLACES := [[1, "head"], [2, "torso"], [4, "arms"], [8, "legs"]]
## A thing held is drawn this faint where it was, and under the mouse.
const FAINT := 0.35

## Says a line of news (the HUD's).
var say: Callable
## The HUD's card beside what's hovered, and away again (hud.gd's hover_card, unhover_card).
var card: Callable
var uncard: Callable
## The rules for what a thing's buttons do (bag.gd), and the items' icons (icons.gd).
var bag: Node
var icons: Node

var _dusk: Node
var _names := PackedStringArray()
var _kinds := PackedInt32Array()
var _weights := PackedInt32Array()
var _worths := PackedInt32Array()
var _covers := PackedInt32Array()
var _fixtures := PackedStringArray()
var _open := false
var _root: Control  # under the panels, over the world: while you hold a thing, a click on it drops it
var _pack_panel: PanelContainer
var _pack_box: VBoxContainer
var _reach_panel: PanelContainer
var _reach_box: VBoxContainer
var _floating: TextureRect  # what you hold, under the mouse
var _floating_word: Label  # its name, till its icon's drawn
var _held := {}  # what's picked up (`_pick`), or nothing
var _equip_after := {}  # a thing taken from within reach to be put on as it lands (_mind_equip_after)
var _shown_pack := []  # what each side last showed: it's drawn again only when that changes
var _shown_reach := []
var _tab := ""  # the place open on the left: a store's "x,y", or "counter"
var _counter := Vector2i(-1, -1)  # the counter within reach, if the left shows it
var _grids: Array = []  # the grids drawn now
var _slots: Array = []  # the figure's slots drawn now
var _tiles: Array = []  # the things drawn now, each with its "what" in its meta
var _waiting := {}  # what waits for its icon: definition -> [[TextureRect, Label], ...]


func _ready() -> void:
	layer = 2  # over the HUD's panels; the HUD's cards (3), the menu (9) and the settings over it
	visible = false
	_root = Control.new()
	_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_root.gui_input.connect(_on_world_input)
	add_child(_root)
	_pack_panel = _panel()
	_pack_panel.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT,
		Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_pack_panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_pack_box = _pack_panel.get_child(0)
	_reach_panel = _panel()
	_reach_panel.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT,
		Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_reach_panel.offset_top = REACH_TOP
	_reach_box = _reach_panel.get_child(0)
	_floating = TextureRect.new()
	_floating.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_floating.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_floating.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_floating.z_index = 2
	_floating.visible = false
	_root.add_child(_floating)
	_floating_word = Look.text(Label.new(), Look.SMALL, Look.TEXT, true)
	_floating_word.set_anchors_preset(Control.PRESET_FULL_RECT)
	_floating_word.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_floating_word.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_floating_word.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_floating.add_child(_floating_word)


## A new world, or the next survivor: what the items are, and nothing held.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.item_names()
	_kinds = dusk.item_kinds()
	_weights = dusk.item_weights()
	_worths = dusk.item_worths()
	_covers = dusk.item_covers()
	_fixtures = dusk.fixture_names()
	_waiting.clear()
	_let_go()
	_shown_pack = []
	_shown_reach = []
	if not icons.drawn.is_connected(_on_drawn):
		icons.drawn.connect(_on_drawn)


func toggle() -> void:
	if _open:
		close()
	else:
		open()


func open() -> void:
	if _dusk == null:
		return
	_open = true
	visible = true
	_shown_pack = []
	_shown_reach = []


func close() -> void:
	_open = false
	visible = false
	_let_go()
	if uncard.is_valid():
		uncard.call(null)


func is_open() -> bool:
	return _open


## How much of the screen's right the inventory takes while it's open (the HUD keeps its news and
## moodles to the left of it: hud.gd's make_room), 0 while it's shut.
func room_on_right() -> float:
	return _pack_panel.size.x + Look.MARGIN if _open else 0.0


## Esc: what you hold goes back where it was, else the inventory shuts. False if it wasn't open.
func back() -> bool:
	if not _open:
		return false
	if not _held.is_empty():
		_let_go()
	else:
		close()
	return true


## Each frame: what changed drawn again, and what you hold under the mouse. Dead, it shuts.
func refresh(dead: bool) -> void:
	if dead and _open:
		close()
	if not _open or _dusk == null:
		return
	var pack := _pack_state()
	if pack != _shown_pack:
		_shown_pack = pack
		_draw_pack()
		_mind_equip_after()
	var reach := _reach_state()
	if reach != _shown_reach:
		_shown_reach = reach
		_draw_reach()
	_mind_the_hold()


# --- What's shown: the core's numbers, each side's in one array, to see when it changes ---

func _pack_state() -> Array:
	return [_dusk.character_items(), _dusk.character_cells(), _dusk.character_bagged(),
		_dusk.character_belt(), _dusk.character_hands(), _dusk.character_light(),
		_dusk.character_shield(), _dusk.character_worn(), _dusk.character_labels(),
		_dusk.character_rarity(), _dusk.worn_labels(), _dusk.shield_label(),
		_dusk.character_bag(), _dusk.character_pockets(), _dusk.character_load(), _dusk.coins()]


func _reach_state() -> Array:
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	var laid := []
	var i := 0
	while i + HEADER <= reach.size():
		laid.append([_dusk.store_cells(reach[i], reach[i + 1]),
			_dusk.store_rarity(reach[i], reach[i + 1]), _dusk.store_labels(reach[i], reach[i + 1])])
		i += HEADER + reach[i + 3] * STACK
	var counter: PackedInt32Array = _dusk.counter_in_reach()
	var dealing := []
	if counter.size() >= 3:
		var x := counter[0]
		var y := counter[1]
		dealing = [_dusk.counter_sells(x, y), _dusk.counter_wants(x, y),
			_dusk.counter_bounties(x, y), _dusk.mend_check(x, y), _dusk.uncurse_check(x, y),
			_readings(x, y), _dusk.coins()]
	return [reach, laid, counter, dealing, _dusk.water_in_reach(), _dusk.fire_beside(), _tab]


## What the counter on (x, y) would read for you (DuskWorld.identify_check()): [slot, price] for
## each thing of yours it would, the one in hand's slot -1.
func _readings(x: int, y: int) -> Array:
	var out := []
	var n: int = _dusk.character_items().size() / STACK
	for slot in range(-1, n):
		var price: int = _dusk.identify_check(x, y, slot)
		if price != -1:
			out.append([slot, price])
	return out


# --- The right: your pack ---

func _draw_pack() -> void:
	_forget(_pack_box)
	var items: PackedInt32Array = _shown_pack[0]
	var cells: PackedInt32Array = _shown_pack[1]
	var bagged: PackedInt32Array = _shown_pack[2]
	var belt: PackedInt32Array = _shown_pack[3]
	var labels: PackedStringArray = _shown_pack[8]
	var rarity: PackedInt32Array = _shown_pack[9]
	var bag_on: PackedInt32Array = _shown_pack[12]
	var pockets: PackedInt32Array = _shown_pack[13]
	var carried: PackedInt32Array = _shown_pack[14]
	var coins: int = _shown_pack[15]
	# The title: what it all weighs against what you can carry and still run, and your coin.
	var title := HBoxContainer.new()
	_pack_box.add_child(title)
	var heading := Look.text(Label.new(), Look.HEADING)
	heading.text = "Your pack"
	heading.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title.add_child(heading)
	if carried.size() >= 2:
		var heavy := carried[0] > carried[1]
		var weight := Look.text(Label.new(), Look.BODY, Look.BAD if heavy else Look.TEXT)
		weight.text = "%.1f / %.1f kg%s  ·  %d coin" % [carried[0] / 1000.0, carried[1] / 1000.0,
			", too heavy to run" if heavy else "", coins]
		title.add_child(weight)
	_pack_box.add_child(Look.rule())
	# The figure, what you hold and wear on it, and the belt.
	var figure := Control.new()
	figure.custom_minimum_size = FIGURE
	figure.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var middle := CenterContainer.new()
	middle.add_child(figure)
	_pack_box.add_child(middle)
	var on := _on_you(labels, rarity, bag_on)
	for id: String in SLOTS:
		var s: Array = SLOTS[id]
		figure.add_child(_slot(id, s[0], s[1], s[2], on.get(id, {})))
	for k in LOOPS:
		var i := k * LOOP
		var thing := {}
		if i + 2 < belt.size() and belt[i] >= 0:
			var slot := belt[i]
			thing = {"from": "belt", "slot": slot, "def": belt[i + 1], "count": belt[i + 2],
				"cond": items[slot * STACK + 2] if slot * STACK + 2 < items.size() else 1000,
				"rarity": rarity[2 + slot] if 2 + slot < rarity.size() else 0,
				"label": labels[2 + slot] if 2 + slot < labels.size() else _name(belt[i + 1])}
		figure.add_child(_slot("belt%d" % (k + 1), Prompts.key("belt_%d" % (k + 1)), Vector2i.ONE,
			BELT_AT + Vector2(k * CELL, 0), thing))
	# The pockets' grid, and the bag's beside it.
	var stacks := _stacks(items, cells, bagged, labels, rarity)
	var grids := HBoxContainer.new()
	grids.add_theme_constant_override("separation", GAP)
	_pack_box.add_child(grids)
	var lost := []
	for into_bag in [false, true]:
		var column := VBoxContainer.new()
		column.add_theme_constant_override("separation", 4)
		grids.add_child(column)
		var size := Vector2i(pockets[1], pockets[2]) if pockets.size() >= 3 else Vector2i(4, 5)
		var head := HBoxContainer.new()
		head.custom_minimum_size = Vector2(0, HEAD)
		column.add_child(head)
		var said := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
		said.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		said.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		head.add_child(said)
		if not into_bag:
			said.text = "Pockets  ·  %.1f kg" % (pockets[0] / 1000.0 if pockets.size() >= 3 else 0.0)
		elif bag_on.size() >= 6:
			size = Vector2i(bag_on[3], bag_on[4])
			said.text = "%s  ·  %.1f kg in it, %d%% lighter" % [_name(bag_on[0]).capitalize(),
				bag_on[2] / 1000.0, roundi(bag_on[5] / 10.0)]
			var drop := Look.button("Drop", func() -> void: _dusk.drop_bag())
			drop.size_flags_vertical = Control.SIZE_SHRINK_CENTER
			head.add_child(drop)
		else:
			said.text = "No bag on your back: one holds more"
			continue
		var grid := _grid("bag" if into_bag else "pockets", size)
		column.add_child(grid)
		for s: Dictionary in stacks:
			if s.bagged != into_bag or s.loop > 0:
				continue
			if s.cell.x < 0:
				lost.append(s)
			else:
				grid.add_thing(_tile(s), s.cell, icons.cells(s.def))
	# What has no place in a grid yet (from before the grids, or given past what fits): a click
	# picks it up, to put down in its own grid.
	if not lost.is_empty():
		var line := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
		line.text = "Without a place: put each where it goes"
		_pack_box.add_child(line)
		var strip := HFlowContainer.new()
		strip.add_theme_constant_override("h_separation", 4)
		_pack_box.add_child(strip)
		for s: Dictionary in lost:
			var tile := _tile(s)
			tile.custom_minimum_size = Vector2(icons.cells(s.def) * CELL)
			tile.gui_input.connect(_on_loose_input.bind(s))
			strip.add_child(tile)
	var hint := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	hint.text = "Click: pick up, put down  ·  Right-click: use  ·  Ctrl-click: move across"
	_pack_box.add_child(hint)


## Your pack's stacks, in order: each a "what" (`_tile`), where it is and how it stands.
func _stacks(items: PackedInt32Array, cells: PackedInt32Array, bagged: PackedInt32Array,
		labels: PackedStringArray, rarity: PackedInt32Array) -> Array:
	var belted: PackedInt32Array = _dusk.character_belted()
	var out := []
	for slot in range(items.size() / STACK):
		var i := slot * STACK
		out.append({"from": "pack", "slot": slot, "def": items[i], "count": items[i + 1],
			"cond": items[i + 2], "found": items[i + 3] == 1,
			"cell": Vector2i(cells[slot * 2], cells[slot * 2 + 1]) if slot * 2 + 1 < cells.size() else Vector2i(-1, -1),
			"bagged": slot < bagged.size() and bagged[slot] == 1,
			"loop": belted[slot] if slot < belted.size() else 0,
			"rarity": rarity[2 + slot] if 2 + slot < rarity.size() else 0,
			"label": labels[2 + slot] if 2 + slot < labels.size() else _name(items[i])})
	return out


## What's on you, by the figure's slot: in hand, on the off arm (a shield, or a light held up),
## each piece worn where the first place it covers is (the rest of what it covers dimmed), and the
## bag on your back.
func _on_you(labels: PackedStringArray, rarity: PackedInt32Array,
		bag_on: PackedInt32Array) -> Dictionary:
	var on := {}
	var hands: PackedInt32Array = _shown_pack[4]
	var light: PackedInt32Array = _shown_pack[5]
	var shield: PackedInt32Array = _shown_pack[6]
	var worn: PackedInt32Array = _shown_pack[7]
	var worn_labels: PackedStringArray = _shown_pack[10]
	var shield_label: PackedStringArray = _shown_pack[11]
	if hands.size() >= 2:
		on["hand"] = {"from": "hand", "def": hands[0], "cond": hands[1], "count": 1,
			"rarity": rarity[0] if rarity.size() > 0 else 0,
			"label": labels[0] if labels.size() > 0 else _name(hands[0])}
	if shield.size() >= 2:
		on["off"] = {"from": "shield", "def": shield[0], "cond": shield[1], "count": 1,
			"rarity": int(shield_label[1]) if shield_label.size() >= 2 else 0,
			"label": shield_label[0] if shield_label.size() >= 2 else _name(shield[0])}
	elif light.size() >= 2:
		on["off"] = {"from": "light", "def": light[0], "cond": light[1], "count": 1,
			"rarity": rarity[1] if rarity.size() > 1 else 0,
			"label": labels[1] if labels.size() > 1 else _name(light[0])}
	var piece := 0
	for i in range(0, worn.size(), STACK):
		var def := worn[i]
		var thing := {"from": "worn", "piece": piece, "def": def, "cond": worn[i + 2], "count": 1,
			"rarity": int(worn_labels[piece * 2 + 1]) if piece * 2 + 1 < worn_labels.size() else 0,
			"label": worn_labels[piece * 2] if piece * 2 < worn_labels.size() else _name(def)}
		var first := true
		for place: Array in PLACES:
			if def < _covers.size() and (_covers[def] & place[0]) != 0:
				var shown := thing.duplicate()
				shown["dim"] = not first
				on[place[1]] = shown
				first = false
		piece += 1
	if bag_on.size() >= 6:
		on["back"] = {"from": "back", "def": bag_on[0], "cond": bag_on[1], "count": 1,
			"rarity": 0, "label": _name(bag_on[0]), "in": bag_on}
	return on


## One of the figure's slots: its name, faint, while it's empty; else what's in it, over the whole
## slot. A click on it takes off what's there, or puts on what you hold.
func _slot(id: String, word: String, cells: Vector2i, at: Vector2, thing: Dictionary) -> Panel:
	var slot := Panel.new()
	slot.position = at
	slot.size = Vector2(cells * CELL)
	slot.add_theme_stylebox_override("panel", _slot_style(0))
	slot.mouse_filter = Control.MOUSE_FILTER_STOP
	slot.set_meta("slot", id)
	slot.set_meta("what", thing)
	if thing.is_empty():
		var empty := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
		empty.text = word
		empty.set_anchors_preset(Control.PRESET_FULL_RECT)
		empty.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		empty.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		empty.mouse_filter = Control.MOUSE_FILTER_IGNORE
		slot.add_child(empty)
	else:
		var tile := _tile(thing)
		tile.set_anchors_preset(Control.PRESET_FULL_RECT)
		slot.add_child(tile)
	slot.gui_input.connect(_on_slot_input.bind(slot))
	slot.mouse_entered.connect(_on_slot_hover.bind(slot, true))
	slot.mouse_exited.connect(_on_slot_hover.bind(slot, false))
	_slots.append(slot)
	return slot


# --- The left: what's within reach ---

func _draw_reach() -> void:
	_forget(_reach_box)
	var reach: PackedInt32Array = _shown_reach[0]
	var counter: PackedInt32Array = _shown_reach[2]
	var water: bool = _shown_reach[4]
	var places := {}  # tab -> [x, y, fixture, n, mine, where its stacks start, its layout]
	var tabs: Array[String] = []
	var i := 0
	var k := 0
	while i + HEADER <= reach.size():
		var key := "%d,%d" % [reach[i], reach[i + 1]]
		places[key] = [reach[i], reach[i + 1], reach[i + 2], reach[i + 3], reach[i + 4] == 1,
			i + HEADER, _shown_reach[1][k]]
		tabs.append(key)
		i += HEADER + reach[i + 3] * STACK
		k += 1
	_counter = Vector2i(counter[0], counter[1]) if counter.size() >= 3 else Vector2i(-1, -1)
	if counter.size() >= 3:
		tabs.append("counter")
	_reach_panel.visible = not tabs.is_empty() or water
	if not _reach_panel.visible:
		return
	if not tabs.has(_tab):
		_tab = tabs[0] if not tabs.is_empty() else ""
	# The tabs: a place each, as Path of Exile's stash has.
	if tabs.size() > 1:
		var row := HFlowContainer.new()
		row.add_theme_constant_override("h_separation", 6)
		row.add_theme_constant_override("v_separation", 6)
		_reach_box.add_child(row)
		for key in tabs:
			var b := Look.button(_tab_name(key, places),
				func() -> void: _open_tab(key))
			b.toggle_mode = true
			b.button_pressed = key == _tab
			row.add_child(b)
	if water:
		var drink := HBoxContainer.new()
		_reach_box.add_child(drink)
		var said := Look.text(Label.new(), Look.BODY)
		said.text = "Water"
		said.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		drink.add_child(said)
		drink.add_child(Look.button("Drink", func() -> void: _dusk.drink_water()))
	if _tab == "counter":
		_draw_counter(counter)
	elif places.has(_tab):
		_draw_place(places[_tab], reach)


func _tab_name(key: String, places: Dictionary) -> String:
	if key == "counter":
		return "The counter"
	var p: Array = places[key]
	var called := "On the ground" if p[2] == 0 else _fixture_name(p[2]).capitalize()
	return called + (" (your stash)" if p[4] else "")


func _open_tab(key: String) -> void:
	_tab = key
	_shown_reach = []


## A container's grid, or what lies on the ground laid out as one, with what can be done there:
## take it all, make it your stash, stash your finds in it.
func _draw_place(p: Array, reach: PackedInt32Array) -> void:
	var x: int = p[0]
	var y: int = p[1]
	var fixture: int = p[2]
	var n: int = p[3]
	var mine: bool = p[4]
	var start: int = p[5]
	var cells: PackedInt32Array = p[6][0]
	var rarity: PackedInt32Array = p[6][1]
	var labels: PackedStringArray = p[6][2]
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	_reach_box.add_child(head)
	var title := Look.text(Label.new(), Look.HEADING, Look.GOLD if mine else Look.TEXT)
	title.text = ("On the ground" if fixture == 0 else _fixture_name(fixture).capitalize()) + \
		("  ·  your stash" if mine else "") + ("  (empty)" if n == 0 else "") + bag.fire_line(x, y)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(title)
	_reach_box.add_child(Look.rule())
	if n > 1:
		var defs := []
		for s in n:
			defs.append(reach[start + s * STACK])
		head.add_child(Look.button("Take all", func() -> void: bag.take_all(x, y, defs)))
	if mine:
		head.add_child(Look.button("Stash all finds", func() -> void: bag.stash_finds(x, y)))
	elif fixture != 0 and _dusk.stash_check(x, y) == 0:
		# Only where it can be (the second playtest: "obnoxious when you see it everywhere").
		head.add_child(Look.button("Make it your stash", func() -> void: bag.mark_stash(x, y)))
	var stacks := []
	for s in n:
		var i := start + s * STACK
		stacks.append({"from": "store", "at": Vector2i(x, y), "slot": s, "def": reach[i],
			"count": reach[i + 1], "cond": reach[i + 2], "found": reach[i + 3] == 1,
			"cell": Vector2i(cells[s * 2], cells[s * 2 + 1]) if s * 2 + 1 < cells.size() else Vector2i(-1, -1),
			"rarity": rarity[s] if s < rarity.size() else 0,
			"label": labels[s] if s < labels.size() else _name(reach[i])})
	var size: PackedInt32Array = _dusk.store_grid(x, y)
	var grid: Grid
	if size.size() >= 2:
		grid = _grid("store", Vector2i(size[0], size[1]), Vector2i(x, y))
		var lost := []
		for s: Dictionary in stacks:
			if s.cell.x < 0:
				lost.append(s)
			else:
				grid.add_thing(_tile(s), s.cell, icons.cells(s.def))
		_reach_box.add_child(grid)
		# Past what its grid holds (a tool's, or an older save's): laid out under it.
		if not lost.is_empty():
			_reach_box.add_child(_laid_out_grid("floor", lost, Vector2i(x, y)))
	else:
		_reach_box.add_child(_laid_out_grid("floor", stacks, Vector2i(x, y)))
	var hint := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	hint.text = "Right-click or Ctrl-click: take  ·  Ctrl-click a thing of yours: put it here" \
		if fixture != 0 else "Right-click or Ctrl-click: take  ·  Ctrl-click a thing of yours: drop it"
	_reach_box.add_child(hint)


## Things with no cells of their own (on the ground, on a counter) laid out as a grid twelve
## across: each where it first fits, row by row.
func _laid_out_grid(kind: String, things: Array, at: Vector2i) -> Grid:
	var taken := {}
	var rows := 2
	var places := []
	for t: Dictionary in things:
		var c: Vector2i = icons.cells(t.def)
		var found := Vector2i(-1, -1)
		var y := 0
		while found.x < 0:
			for x in range(0, ACROSS - c.x + 1):
				if _free(taken, Vector2i(x, y), c):
					found = Vector2i(x, y)
					break
			y += 1
		for dy in c.y:
			for dx in c.x:
				taken[found + Vector2i(dx, dy)] = true
		places.append(found)
		rows = maxi(rows, found.y + c.y)
	var grid := _grid(kind, Vector2i(ACROSS, rows), at)
	for k in things.size():
		var t: Dictionary = things[k]
		t["cell"] = places[k]
		grid.add_thing(_tile(t), places[k], icons.cells(t.def))
	return grid


func _free(taken: Dictionary, at: Vector2i, c: Vector2i) -> bool:
	for dy in c.y:
		for dx in c.x:
			if taken.has(at + Vector2i(dx, dy)):
				return false
	return true


## A holdout's counter, as Path of Exile's vendors: what it sells today as a grid (a click buys
## one), what it's short of, what it would mend, read or lift for you, and the bounties posted;
## Ctrl-click a thing of yours to sell it.
func _draw_counter(counter: PackedInt32Array) -> void:
	var x := counter[0]
	var y := counter[1]
	var dealing: Array = _shown_reach[3]
	var sells: PackedInt32Array = dealing[0]
	var wants: PackedInt32Array = dealing[1]
	var bounties: PackedInt32Array = dealing[2]
	var mend: int = dealing[3]
	var lift: int = dealing[4]
	var readings: Array = dealing[5]
	var holdout: String = _dusk.holdout_name(counter[2])
	var title := Look.text(Label.new(), Look.HEADING, Look.HOLY)
	title.text = holdout.left(1).to_upper() + holdout.substr(1) + "  ·  the counter"
	_reach_box.add_child(title)
	if wants.size() > 0:
		var short := []
		for w in wants:
			short.append(_name(w))
		var line := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
		line.text = "Short of: %s. Sell it these to push the dusk back." % ", ".join(short)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		line.custom_minimum_size = Vector2(ACROSS * CELL, 0)
		_reach_box.add_child(line)
	var services := HFlowContainer.new()
	services.add_theme_constant_override("h_separation", 6)
	services.add_theme_constant_override("v_separation", 6)
	_reach_box.add_child(services)
	if mend > 0 or mend == -2:
		services.add_child(Look.button("Mend what's in your hand · %d coin" % maxi(mend, 0),
			func() -> void: bag.mend(x, y)))
	if lift != -1:
		services.add_child(Look.button("Lift the curse · %d coin" % maxi(lift, 0),
			func() -> void: bag.uncurse(x, y)))
	var labels: PackedStringArray = _dusk.character_labels()
	for r: Array in readings:
		var slot: int = r[0]
		# DuskWorld.character_labels(): what's in hand first, then the light, then the pack's.
		var index := 0 if slot < 0 else slot + 2
		var called := labels[index] if index < labels.size() else "it"
		services.add_child(Look.button("Read the %s · %d coin" % [called, maxi(r[1], 0)],
			func() -> void: bag.read(x, y, slot)))
	for b in range(0, bounties.size() / BOUNTY):
		var who := bounties[b * BOUNTY]
		var state := bounties[b * BOUNTY + 3]
		var row := HBoxContainer.new()
		_reach_box.add_child(row)
		var line := Look.text(Label.new(), Look.SMALL)
		line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		line.text = "Bounty: %s  ·  %d coin  ·  %s" % [_dusk.actor_name(who),
			bounties[b * BOUNTY + 4], ["last heard of about (%d, %d)" % [bounties[b * BOUNTY + 1],
			bounties[b * BOUNTY + 2]], "put down", "paid"][clampi(state, 0, 2)]]
		row.add_child(line)
		if state == 1:
			row.add_child(Look.button("Claim", func() -> void: _dusk.claim(x, y, who)))
	var wares := []
	for line in range(0, sells.size() / 4):
		wares.append({"from": "ware", "at": Vector2i(x, y), "line": line, "def": sells[line * 4],
			"price": sells[line * 4 + 1], "left": sells[line * 4 + 2], "can": sells[line * 4 + 3],
			"count": 1, "label": _name(sells[line * 4]), "dim": sells[line * 4 + 2] == 0})
	if not wares.is_empty():
		_reach_box.add_child(_laid_out_grid("wares", wares, Vector2i(x, y)))
	var hint := Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	hint.text = "Click: buy one  ·  Ctrl-click a thing of yours: sell it"
	_reach_box.add_child(hint)


# --- Things, grids and slots ---

## A thing drawn over its cells (VISUALS.md §11, Item tile): its icon (its name till that's
## drawn), on a ground in its magic's colour; how many, at the top left; a gold dot for a find; a
## ware's price at the bottom right. `what` says what it is and where (`from`: "pack", "store",
## "belt", "hand", "shield", "light", "worn", "back", "ware").
func _tile(what: Dictionary) -> Panel:
	var tile := Panel.new()
	tile.mouse_filter = Control.MOUSE_FILTER_PASS
	tile.add_theme_stylebox_override("panel", _tile_style(what.get("rarity", 0)))
	tile.set_meta("what", what)
	var def: int = what.def
	var picture := TextureRect.new()
	picture.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT, Control.PRESET_MODE_MINSIZE, 3)
	picture.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	picture.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	picture.mouse_filter = Control.MOUSE_FILTER_IGNORE
	tile.add_child(picture)
	var word := Label.new()
	word.text = what.get("label", _name(def))
	word.add_theme_font_size_override("font_size", 14)
	word.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT, Control.PRESET_MODE_MINSIZE, 2)
	word.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	word.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	word.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	word.clip_text = true
	word.mouse_filter = Control.MOUSE_FILTER_IGNORE
	tile.add_child(word)
	var texture: Texture2D = icons.icon(def)
	if texture != null:
		picture.texture = texture
		word.visible = false
	else:
		if not _waiting.has(def):
			_waiting[def] = []
		_waiting[def].append([picture, word])
	var count: int = what.get("count", 1)
	if count > 1:
		var many := Look.text(Label.new(), Look.SMALL, Look.TEXT, true)
		many.text = str(count)
		many.position = Vector2(4, 0)
		many.mouse_filter = Control.MOUSE_FILTER_IGNORE
		tile.add_child(many)
	if what.get("found", false):
		var dot := Panel.new()
		var gold := StyleBoxFlat.new()
		gold.bg_color = Look.GOLD
		gold.set_corner_radius_all(4)
		dot.add_theme_stylebox_override("panel", gold)
		dot.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
		dot.offset_left = -12
		dot.offset_top = 4
		dot.offset_right = -4
		dot.offset_bottom = 12
		dot.mouse_filter = Control.MOUSE_FILTER_IGNORE
		tile.add_child(dot)
	if what.has("price"):
		var price := Look.text(Label.new(), Look.SMALL, Look.GOLD, true)
		price.text = str(what.price)
		price.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_RIGHT)
		price.grow_horizontal = Control.GROW_DIRECTION_BEGIN
		price.grow_vertical = Control.GROW_DIRECTION_BEGIN
		price.mouse_filter = Control.MOUSE_FILTER_IGNORE
		tile.add_child(price)
	if what.get("dim", false) or _is_held(what):
		tile.modulate = Color(1, 1, 1, FAINT)
	tile.mouse_entered.connect(_hover_tile.bind(tile))
	tile.mouse_exited.connect(_unhover_tile.bind(tile))
	_tiles.append(tile)
	return tile


## A thing's ground over its cells (art.gd's Glow): its rarity's colour welling up behind its icon,
## edged in it, so rarity shows before the name; a plain thing's a faint warm glow, unedged.
func _tile_style(rarity: int) -> StyleBox:
	var style := Look.Art.Glow.new()
	var tint: Color = bag.rarity_colour(rarity)
	if tint.a > 0.0:
		style.colour = Color(tint, 0.55)
		style.edge = Color(tint, 0.7)
	else:
		style.colour = Color(Look.HEADING_GOLD, 0.16)
	return style


## A slot's ground: a cell's, its edge faint; where what you hold would go, green, or red where it
## won't (`state` 1 or -1).
func _slot_style(state: int) -> StyleBox:
	var style := Look.Art.Well.new()
	if state != 0:
		style.edge = Look.GOOD if state > 0 else Look.DANGER
	return style


func _grid(kind: String, cells: Vector2i, at := Vector2i(-1, -1)) -> Grid:
	var grid := Grid.new()
	grid.kind = kind
	grid.cells = cells
	grid.at = at
	grid.px = CELL
	grid.ground = Look.CELL_GROUND
	grid.edge = Look.CELL_EDGE
	grid.well = Look.Art.Well.new()
	grid.good = Color(Look.GOOD, 0.35)
	grid.bad = Color(Look.DANGER, 0.4)
	grid.custom_minimum_size = Vector2(cells * CELL)
	grid.clicked = _on_grid_click
	grid.moved = _on_grid_move
	grid.mouse_exited.connect(func() -> void: grid.set_mark(Rect2i(), 0))
	_grids.append(grid)
	return grid


## The things a panel shows forgotten, before it's drawn again.
func _forget(box: VBoxContainer) -> void:
	for child in box.get_children():
		box.remove_child(child)
		child.queue_free()
	_tiles = _tiles.filter(func(t: Control) -> bool: return t.is_inside_tree())
	_grids = _grids.filter(func(g: Control) -> bool: return g.is_inside_tree())
	_slots = _slots.filter(func(s: Control) -> bool: return s.is_inside_tree())


func _panel() -> PanelContainer:
	var panel := PanelContainer.new()
	var style := Look.panel_style()
	# Solid, as Path of Exile's are: the news and the moodles under it don't show through.
	style.bg_color = Look.PANEL_SOLID
	style.remove_meta("panel")
	panel.add_theme_stylebox_override("panel", style)
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 8)
	panel.add_child(box)
	_root.add_child(panel)
	return panel


func _on_drawn(def: int) -> void:
	for pair: Array in _waiting.get(def, []):
		if is_instance_valid(pair[0]):
			(pair[0] as TextureRect).texture = icons.icon(def)
			(pair[1] as Label).visible = false
	_waiting.erase(def)
	if not _held.is_empty() and _held.def == def:
		_floating.texture = icons.icon(def)
		_floating_word.visible = false


# --- Picking up and putting down ---

func _pick(what: Dictionary) -> void:
	if uncard.is_valid():
		uncard.call(null)
	_held = what.duplicate()
	var size := Vector2(icons.cells(what.def) * CELL)
	_floating.size = size
	_floating.texture = icons.icon(what.def)
	_floating_word.text = what.get("label", _name(what.def))
	_floating_word.visible = _floating.texture == null
	_floating.modulate = Color(1, 1, 1, 0.9)
	_floating.visible = true
	_root.mouse_filter = Control.MOUSE_FILTER_STOP
	for tile: Control in _tiles:
		if is_instance_valid(tile) and _is_held(tile.get_meta("what")):
			tile.modulate = Color(1, 1, 1, FAINT)
	_mind_the_hold()


## What you hold goes back where it was: nothing moved.
func _let_go() -> void:
	_held = {}
	if _floating != null:
		_floating.visible = false
	if _root != null:
		_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	for tile: Control in _tiles:
		if is_instance_valid(tile) and not tile.get_meta("what").get("dim", false):
			tile.modulate = Color.WHITE
	for grid: Grid in _grids:
		if is_instance_valid(grid):
			grid.set_mark(Rect2i(), 0)
	for slot: Panel in _slots:
		if is_instance_valid(slot):
			slot.add_theme_stylebox_override("panel", _slot_style(0))


func _is_held(what: Dictionary) -> bool:
	if _held.is_empty() or what.get("from", "") != _held.from:
		return false
	return what.get("slot", -1) == _held.get("slot", -1) and what.def == _held.def \
		and what.get("at", Vector2i(-1, -1)) == _held.get("at", Vector2i(-1, -1))


## Each frame you hold a thing: it follows the mouse, centred on it; and if it's gone from where
## you took it (used up, taken, a blow), it's let go of.
func _mind_the_hold() -> void:
	if _held.is_empty():
		return
	if not _still_there():
		_let_go()
		return
	_floating.position = _root.get_global_mouse_position() - _floating.size / 2.0


func _still_there() -> bool:
	var slot: int = _held.get("slot", -1)
	if _held.from == "pack" or _held.from == "belt":
		var items: PackedInt32Array = _shown_pack[0] if not _shown_pack.is_empty() else PackedInt32Array()
		return slot * STACK < items.size() and items[slot * STACK] == _held.def
	if _held.from == "store":
		var at: Vector2i = _held.at
		var reach: PackedInt32Array = _shown_reach[0] if not _shown_reach.is_empty() else PackedInt32Array()
		var i := 0
		while i + HEADER <= reach.size():
			if reach[i] == at.x and reach[i + 1] == at.y:
				return slot < reach[i + 3] and reach[i + HEADER + slot * STACK] == _held.def
			i += HEADER + reach[i + 3] * STACK
		return false
	return true


## Where what you hold would go in `grid` with the mouse at `local`: its top-left cell, the thing
## centred on the mouse as Path of Exile's is, kept inside the grid.
func _target(grid: Grid, local: Vector2) -> Vector2i:
	var c: Vector2i = icons.cells(_held.def)
	var at := Vector2i(roundi(local.x / CELL - c.x / 2.0), roundi(local.y / CELL - c.y / 2.0))
	return Vector2i(clampi(at.x, 0, maxi(grid.cells.x - c.x, 0)),
		clampi(at.y, 0, maxi(grid.cells.y - c.y, 0)))


## How many of what you hold would go onto `cell` of `grid`, by the core's own checks; 0 for none.
func _check(grid: Grid, cell: Vector2i) -> int:
	var from: String = _held.from
	var slot: int = _held.get("slot", -1)
	var def: int = _held.def
	match grid.kind:
		"pockets", "bag":
			var into_bag := grid.kind == "bag"
			if from == "pack" and _held.bagged == into_bag:
				return _dusk.arrange_check(slot, def, cell.x, cell.y)
			if from == "pack":
				return _dusk.shift_to_check(slot, def, cell.x, cell.y)
			if from == "store":
				var at: Vector2i = _held.at
				return _dusk.take_to_check(at.x, at.y, slot, def, into_bag, cell.x, cell.y)
			if from == "belt":
				return _dusk.belt_check(slot, def)
		"store":
			if from == "pack":
				return _dusk.put_check(grid.at.x, grid.at.y, slot, def, cell.x, cell.y)
			if from == "store" and _held.at == grid.at:
				return _dusk.arrange_store_check(grid.at.x, grid.at.y, slot, def, cell.x, cell.y)
		"floor":
			return _held.count if from == "pack" else 0
		"wares":
			if from == "pack" and _dusk.sell_price(grid.at.x, grid.at.y, slot, def) >= 0:
				return 1
	return 0


## What you hold put down on `cell` of `grid`, if the core says it can go there; else why not.
func _put_down(grid: Grid, cell: Vector2i) -> void:
	var h := _held
	if _check(grid, cell) <= 0:
		_say_why_not(grid)
		return
	var slot: int = h.get("slot", -1)
	var def: int = h.def
	match grid.kind:
		"pockets", "bag":
			var into_bag := grid.kind == "bag"
			if h.from == "pack" and h.bagged == into_bag:
				_dusk.arrange(slot, def, cell.x, cell.y)
			elif h.from == "pack":
				_dusk.shift_to(slot, def, cell.x, cell.y)
			elif h.from == "store":
				_dusk.take_to(h.at.x, h.at.y, slot, def, into_bag, cell.x, cell.y)
			else:
				_dusk.belt_item(slot, def)
		"store":
			if h.from == "pack":
				_dusk.put_to(grid.at.x, grid.at.y, slot, def, cell.x, cell.y)
			else:
				_dusk.arrange_store(grid.at.x, grid.at.y, slot, def, cell.x, cell.y)
		"floor":
			_dusk.drop_item(slot, def)
		"wares":
			_dusk.sell(grid.at.x, grid.at.y, slot, def)
	_let_go()


func _say_why_not(grid: Grid) -> void:
	if not say.is_valid():
		return
	var from: String = _held.from
	if grid.kind == "wares":
		say.call("They won't buy that at the counter.")
	elif grid.kind == "floor":
		say.call("Take it first: it's not yours yet.")
	elif grid.kind == "store" and from == "store":
		say.call("Take it first, then put it there.")
	elif grid.kind == "store":
		say.call("It won't fit there.")
	else:
		say.call("No room for it there.")


## What you hold put on: in hand, on the off arm (a shield, or a light held up and lit), worn, the
## bag on your back, or a potion on the belt.
func _put_on(id: String) -> void:
	var h := _held
	if not _fits(id, h.def):
		if say.is_valid():
			say.call("The %s doesn't go there." % _name(h.def))
		return
	# From a chest, a crate or the ground (the third notes: "Allow equipping directly from
	# looting"): taken as any take is (the core's checks and time), then put on as it lands in your
	# pack (_mind_equip_after). A potion taken goes on the belt by itself.
	if h.from == "store":
		var had := _stacks_of(h.def)
		if bag.take(h.at.x, h.at.y, h.slot, h.def) > 0 and _kind(h.def) != Item.POTION:
			_equip_after = {"def": h.def, "id": id, "had": had, "until": Time.get_ticks_msec() + 4000}
		_let_go()
		return
	if h.from != "pack":
		if say.is_valid():
			say.call("Take it first: it's not yours yet.")
		return
	_equip(id, h.slot, h.def, h.get("loop", 0))
	_let_go()


## How many stacks of `def` your pack holds now.
func _stacks_of(def: int) -> int:
	var items: PackedInt32Array = _dusk.character_items()
	var n := 0
	for slot in range(items.size() / STACK):
		n += int(items[slot * STACK] == def)
	return n


## A thing taken to be put on (_put_on, from a container or the ground): once a stack more of it is
## in your pack, the newest (a take adds it last) goes where it was put; given up after a few
## seconds (the take refused, or broken off).
func _mind_equip_after() -> void:
	if _equip_after.is_empty():
		return
	if Time.get_ticks_msec() > int(_equip_after.until):
		_equip_after = {}
		return
	var def: int = _equip_after.def
	if _stacks_of(def) <= int(_equip_after.had):
		return
	var items: PackedInt32Array = _dusk.character_items()
	for slot in range(items.size() / STACK - 1, -1, -1):
		if items[slot * STACK] == def:
			_equip(_equip_after.id, slot, def, 0)
			break
	_equip_after = {}


## What's in your pack at `slot` (of `def`) put on the figure's `id`: in hand, on the off arm,
## worn, the bag on your back, or on the belt.
func _equip(id: String, slot: int, def: int, loop: int) -> void:
	var h := {"slot": slot, "def": def, "loop": loop}
	match id:
		"hand":
			bag.wield(h.slot, h.def)
		"off":
			if _kind(h.def) == Item.LIGHT:
				_dusk.light(h.slot, h.def)
			else:
				_dusk.wield(h.slot, h.def)
		"back", "head", "torso", "arms", "legs":
			_dusk.wear(h.slot, h.def)
		_:
			if h.get("loop", 0) == 0 and _dusk.belt_check(h.slot, h.def) > 0:
				_dusk.belt_item(h.slot, h.def)
			elif say.is_valid():
				say.call("No room on your belt.")


## Whether a thing of `def`'s kind goes in the figure's slot `id`.
func _fits(id: String, def: int) -> bool:
	var kind := _kind(def)
	match id:
		"hand":
			return kind == Item.WEAPON or kind == Item.BOW
		"off":
			return kind == Item.SHIELD or kind == Item.LIGHT
		"back":
			return kind == Item.BAG
		"head", "torso", "arms", "legs":
			for place: Array in PLACES:
				if place[1] == id:
					return kind == Item.ARMOUR and def < _covers.size() and (_covers[def] & place[0]) != 0
			return false
	return kind == Item.POTION


## What's on you taken off: what's in hand put away (not while it clings), the shield too, the
## light put out, a piece of armour, a potion off the belt.
func _take_off(what: Dictionary) -> void:
	match what.from:
		"hand":
			if _dusk.hand_clings():
				bag.clings()
			else:
				_dusk.unwield()
		"shield":
			_dusk.unshield()
		"light":
			_dusk.douse()
		"worn":
			_dusk.take_off(what.piece, what.def)
		"belt":
			_dusk.belt_item(what.slot, what.def)
		"back":
			if say.is_valid():
				say.call("Your bag stays on your back: put another on, or drop it with what's in it.")


## Right-click: yours used as its button did; what's within reach taken; a ware bought.
func _use(what: Dictionary) -> void:
	match what.from:
		"pack":
			bag.use(what.slot, what.def)
		"belt":
			bag.drink(what.slot, what.def)
		"store":
			bag.take(what.at.x, what.at.y, what.slot, what.def)
		"ware":
			bag.buy(what.at.x, what.at.y, what.line, what.can)
		_:
			_take_off(what)


## Ctrl-click: across. Yours into the container open on the left, sold at the counter, dropped on
## the ground, or with nothing open there, between your pockets and your bag; what's within reach
## into your pack.
func _move_across(what: Dictionary) -> void:
	if what.from == "store" or what.from == "ware":
		_use(what)
		return
	if what.from == "belt":
		_dusk.belt_item(what.slot, what.def)
		return
	if what.from != "pack":
		return
	var slot: int = what.slot
	var def: int = what.def
	if _reach_panel.visible and _tab == "counter" and _counter.x >= 0:
		if _dusk.sell_price(_counter.x, _counter.y, slot, def) >= 0:
			_dusk.sell(_counter.x, _counter.y, slot, def)
		elif say.is_valid():
			say.call("They won't buy that at the counter.")
		return
	var place := _open_place()
	if place.x >= 0:
		if _dusk.store_grid(place.x, place.y).is_empty():
			_dusk.drop_item(slot, def)
		elif _dusk.put_check(place.x, place.y, slot, def, -1, -1) > 0:
			_dusk.put_item(place.x, place.y, slot, def)
		elif say.is_valid():
			say.call("No room for it in there.")
		return
	if _dusk.shift_check(slot, def) > 0:
		_dusk.shift(slot, def)
	elif say.is_valid():
		say.call(bag.no_room() if not what.bagged else "No room for it in your pockets.")


## The container or ground open on the left, or (-1, -1).
func _open_place() -> Vector2i:
	if not _reach_panel.visible or _tab == "" or _tab == "counter":
		return Vector2i(-1, -1)
	return Vector2i(_tab.get_slice(",", 0).to_int(), _tab.get_slice(",", 1).to_int())


# --- The mouse ---

func _on_grid_click(grid: Grid, event: InputEventMouseButton) -> void:
	var right := event.button_index == MOUSE_BUTTON_RIGHT
	if not _held.is_empty():
		if right:
			_let_go()
		else:
			_put_down(grid, _target(grid, event.position))
		return
	var what := _thing_at(grid, grid.cell_at(event.position))
	if what.is_empty():
		return
	if grid.kind == "wares" or right:
		_use(what)
	elif event.ctrl_pressed:
		_move_across(what)
	else:
		_pick(what)


func _on_grid_move(grid: Grid, local: Vector2) -> void:
	if _held.is_empty():
		return
	var at := _target(grid, local)
	var fits := _check(grid, at) > 0
	var c: Vector2i = icons.cells(_held.def)
	grid.set_mark(Rect2i(at, c) if grid.kind != "floor" and grid.kind != "wares" \
		else Rect2i(Vector2i.ZERO, grid.cells), 1 if fits else -1)


## A thing of yours with no place yet: a click picks it up, to put down where it goes.
func _on_loose_input(event: InputEvent, what: Dictionary) -> void:
	var button := event as InputEventMouseButton
	if button == null or not button.pressed:
		return
	if button.button_index == MOUSE_BUTTON_LEFT and _held.is_empty():
		_pick(what)
	elif button.button_index == MOUSE_BUTTON_RIGHT:
		if _held.is_empty():
			_use(what)
		else:
			_let_go()


func _thing_at(grid: Grid, cell: Vector2i) -> Dictionary:
	for child in grid.get_children():
		if not child.has_meta("what"):
			continue
		var what: Dictionary = child.get_meta("what")
		if Rect2i(what.cell, icons.cells(what.def)).has_point(cell):
			return what
	return {}


func _on_slot_input(event: InputEvent, slot: Panel) -> void:
	var button := event as InputEventMouseButton
	if button == null or not button.pressed:
		return
	if button.button_index != MOUSE_BUTTON_LEFT and button.button_index != MOUSE_BUTTON_RIGHT:
		return
	slot.accept_event()
	var id: String = slot.get_meta("slot")
	var what: Dictionary = slot.get_meta("what")
	if not _held.is_empty():
		if button.button_index == MOUSE_BUTTON_RIGHT:
			_let_go()
		else:
			_put_on(id)
		return
	if what.is_empty():
		return
	if button.button_index == MOUSE_BUTTON_RIGHT and what.from == "belt":
		bag.drink(what.slot, what.def)
	else:
		_take_off(what)


## Holding a thing over a slot: green where it goes, red where it doesn't.
func _on_slot_hover(slot: Panel, entered: bool) -> void:
	if not is_instance_valid(slot):
		return
	var state := 0
	if entered and not _held.is_empty():
		var takes: bool = _held.from == "pack" or _held.from == "store"
		state = 1 if takes and _fits(slot.get_meta("slot"), _held.def) else -1
	slot.add_theme_stylebox_override("panel", _slot_style(state))


## A click on the world (not on a panel) while you hold a thing: yours is dropped at your feet,
## what's within reach let go of. A right-click lets go.
func _on_world_input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button == null or not button.pressed or _held.is_empty():
		return
	_root.accept_event()
	if button.button_index == MOUSE_BUTTON_LEFT and _held.from == "pack":
		_dusk.drop_item(_held.slot, _held.def)
	_let_go()


func _hover_tile(tile: Control) -> void:
	if not _held.is_empty() or not card.is_valid():
		return
	var what: Dictionary = tile.get_meta("what")
	var shown := _card_for(what)
	if not shown.is_empty():
		card.call(tile, shown[0], shown[1], _from_reach(what), shown[2])


func _unhover_tile(tile: Control) -> void:
	if uncard.is_valid():
		uncard.call(tile)


func _from_reach(what: Dictionary) -> bool:
	return what.from == "store" or what.from == "ware"


# --- Cards ---

## A thing's card (VISUALS.md §11, Tooltip): the core's for a weapon, a bow, a shield, armour, a
## light or any magic thing (card.rs), else a plain one: what it weighs, its cells, how fresh or how
## burnt, a find's worth. Then what a right-click does, and at the counter what it sells for.
func _card_for(what: Dictionary) -> Array:
	var shown := []
	match what.from:
		"pack", "belt":
			shown = bag.card_of(_dusk.item_card(2 + what.slot), _dusk.card_lines(2 + what.slot))
		"hand":
			shown = bag.card_of(_dusk.item_card(0), _dusk.card_lines(0))
		"light":
			shown = bag.card_of(_dusk.item_card(1), _dusk.card_lines(1))
		"shield":
			shown = bag.card_of(_dusk.item_card(-1), _dusk.card_lines(-1))
		"worn":
			shown = bag.card_of(_dusk.item_card(-2 - what.piece), _dusk.card_lines(-2 - what.piece))
		"store":
			shown = bag.card_of(_dusk.store_card(what.at.x, what.at.y, what.slot),
				_dusk.store_card_lines(what.at.x, what.at.y, what.slot))
	if shown.is_empty():
		shown = _plain_card(what)
	elif shown[0] == "":
		shown[0] = what.get("label", _name(what.def))
	var lines: PackedStringArray = shown[1]
	var word := ""
	match what.from:
		"pack":
			word = bag.use_word(what.def)
		"belt":
			word = "Drink"
		"store":
			word = "Take"
		"ware":
			word = "Buy"
		"hand", "shield", "worn":
			word = "Take off"
		"light":
			word = "Put out"
	if word != "":
		lines.append_array(["", "Right-click: %s" % word, ""])
	if what.from == "pack" and _tab == "counter" and _counter.x >= 0 and _reach_panel.visible:
		var price: int = _dusk.sell_price(_counter.x, _counter.y, what.slot, what.def)
		if price >= 0:
			lines.append_array(["stat", "Ctrl-click to sell", "%d coin" % price])
	shown[1] = lines
	return shown


func _plain_card(what: Dictionary) -> Array:
	var def: int = what.def
	var count: int = what.get("count", 1)
	var title: String = what.get("label", _name(def))
	if count > 1:
		title += " ×%d" % count
	var lines := PackedStringArray()
	if what.from == "ware":
		lines.append_array(["stat", "Price", "%d coin" % what.price])
		lines.append_array(["stat", "Left today", str(what.left)])
	var weight := _weights[def] * count if def < _weights.size() else 0
	lines.append_array(["stat", "Weight", "%.1f kg" % (weight / 1000.0)])
	var c: Vector2i = icons.cells(def)
	lines.append_array(["stat", "Takes up", "%d × %d cells" % [c.x, c.y]])
	var cond: int = what.get("cond", 1000)
	if _kind(def) == Item.FOOD and cond < 1000:
		lines.append_array(["stat", "Freshness", bag.freshness(cond)])
	if _kind(def) == Item.LIGHT and cond < 1000:
		lines.append_array(["stat", "Left to burn", "%d%%" % roundi(cond / 10.0)])
	if what.has("in"):
		var b: PackedInt32Array = what["in"]
		lines.append_array(["stat", "Holds", "%d × %d cells" % [b[3], b[4]]])
		lines.append_array(["stat", "What's in it weighs", "%d%% less" % roundi(b[5] / 10.0)])
	if what.get("found", false) and def < _worths.size() and _worths[def] > 0:
		lines.append_array(["stat", "A find", "%d XP home" % (_worths[def] * count)])
	return [title, lines, Color(0, 0, 0, 0)]


# --- For scripted runs (main.gd's --do) ---

## --do=S:card: the inventory opened, and the card of a weapon in the pack (the pockets first, then
## the bag) shown at once, as if hovered; else a bow's, a shield's or armour's in the pack; else
## what's in hand. Its words, a part a line, or "" for none.
func card_first() -> String:
	open()
	refresh(false)
	var order := _tiles.filter(func(t: Control) -> bool:
		return is_instance_valid(t) and t.is_inside_tree())
	for kind: int in [Item.WEAPON, Item.BOW, Item.SHIELD, Item.ARMOUR, -1]:
		var best: Control = null
		var best_key := 1 << 30
		for tile: Control in order:
			var what: Dictionary = tile.get_meta("what")
			var here: bool = (what.from == "pack" and _kind(what.def) == kind) if kind >= 0 \
				else what.from == "hand"
			var key: int = int(what.get("bagged", false)) * 10000 + what.get("slot", 0)
			if here and key < best_key:
				best = tile
				best_key = key
		if best != null:
			var shown := _card_for(best.get_meta("what"))
			if card.is_valid():
				card.call(best, shown[0], shown[1], false, shown[2], true)
			return Bag.card_words(shown)
	return ""


## --do=S:arrange: the first stack in your pockets moved, at once, to the last cell of them it
## fits on, as a click would put it. Where to, in words, or "" if it fits nowhere else.
func arrange_first() -> String:
	var items: PackedInt32Array = _dusk.character_items()
	var cells: PackedInt32Array = _dusk.character_cells()
	var bagged: PackedInt32Array = _dusk.character_bagged()
	var pockets: PackedInt32Array = _dusk.character_pockets()
	for slot in range(items.size() / STACK):
		if bagged[slot] == 1 or cells[slot * 2] < 0:
			continue
		var def := items[slot * STACK]
		for y in range(pockets[2] - 1, -1, -1):
			for x in range(pockets[1] - 1, -1, -1):
				if Vector2i(x, y) == Vector2i(cells[slot * 2], cells[slot * 2 + 1]):
					continue
				if _dusk.arrange_check(slot, def, x, y) > 0 and _dusk.arrange(slot, def, x, y):
					return "Arranged the %s to (%d, %d)." % [_name(def), x, y]
		return ""
	return ""


## --do=S:takein: the first thing within reach taken onto the last free cell of your pockets it
## fits on, as a click would put it there. What and where, in words, or "".
func take_in_first() -> String:
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	var pockets: PackedInt32Array = _dusk.character_pockets()
	if reach.size() < HEADER or reach[3] == 0:
		return ""
	var x := reach[0]
	var y := reach[1]
	var def := reach[HEADER]
	for cy in range(pockets[2] - 1, -1, -1):
		for cx in range(pockets[1] - 1, -1, -1):
			if _dusk.take_to_check(x, y, 0, def, false, cx, cy) > 0:
				_dusk.take_to(x, y, 0, def, false, cx, cy)
				return "Taking the %s to (%d, %d)." % [_name(def), cx, cy]
	return ""


## --do=S:equipin: a thing within reach that can be put on, picked up and put on the figure, as a
## click would (taken, then put on as it lands): the first for your hand (a weapon or a bow), else
## for the off arm, your back, then what's worn. What, in words, or "".
func equip_in_first() -> String:
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	for id: String in ["hand", "off", "back", "head", "torso", "arms", "legs"]:
		var i := 0
		while i + HEADER <= reach.size():
			for k in reach[i + 3]:
				var def := reach[i + HEADER + k * STACK]
				if _fits(id, def):
					_held = {"from": "store", "at": Vector2i(reach[i], reach[i + 1]), "slot": k, "def": def}
					_put_on(id)
					return "Putting on the %s from within reach." % _name(def)
			i += HEADER + reach[i + 3] * STACK
	return ""


func _name(def: int) -> String:
	return _names[def] if def >= 0 and def < _names.size() else "something"


func _kind(def: int) -> int:
	return _kinds[def] if def >= 0 and def < _kinds.size() else 0


func _fixture_name(fixture: int) -> String:
	return _fixtures[fixture] if fixture < _fixtures.size() else "container"


## A grid of cells (VISUALS.md §11, Grid): your pockets, your bag, a container, or the ground or a
## counter laid out as one. It draws its cells, and over its things where one held would go (green,
## or red where it won't); a click or the mouse moving over it goes to the inventory.
class Grid extends Control:
	## "pockets", "bag", "store" (a container's), "floor" (laid out), "wares" (a counter's).
	var kind := ""
	var at := Vector2i(-1, -1)  # the tile a container, the ground or a counter is on
	var cells := Vector2i.ONE
	var px := 50
	var ground := Color()
	var edge := Color()
	var well: StyleBox  # each cell's (art.gd's Well), sunk into the panel; flat ground if none
	var good := Color()
	var bad := Color()
	var clicked: Callable  # (grid, event)
	var moved: Callable  # (grid, the mouse in the grid)
	var _mark := Rect2i()
	var _state := 0
	var _overlay: Control

	func _ready() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP
		_overlay = Control.new()
		_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
		_overlay.z_index = 1
		_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
		_overlay.draw.connect(_draw_mark)
		add_child(_overlay)

	func _draw() -> void:
		for y in cells.y:
			for x in cells.x:
				var r := Rect2(Vector2(x, y) * px, Vector2(px, px))
				if well != null:
					draw_style_box(well, r)
				else:
					draw_rect(r, ground)
					draw_rect(r, edge, false, 1.0)

	func _draw_mark() -> void:
		if _state != 0:
			_overlay.draw_rect(Rect2(Vector2(_mark.position) * px, Vector2(_mark.size) * px),
				good if _state > 0 else bad)

	## Marks `r` (in cells) as where a thing held would go: `state` 1 it fits, -1 it won't, 0 none.
	func set_mark(r: Rect2i, state: int) -> void:
		if r == _mark and state == _state:
			return
		_mark = r
		_state = state
		if _overlay != null:
			_overlay.queue_redraw()

	## A thing over its cells.
	func add_thing(tile: Control, cell: Vector2i, size_cells: Vector2i) -> void:
		tile.position = Vector2(cell * px)
		tile.size = Vector2(size_cells * px)
		add_child(tile)

	func cell_at(p: Vector2) -> Vector2i:
		return Vector2i(floori(p.x / px), floori(p.y / px))

	func _gui_input(event: InputEvent) -> void:
		var button := event as InputEventMouseButton
		if button != null and button.pressed and (button.button_index == MOUSE_BUTTON_LEFT \
				or button.button_index == MOUSE_BUTTON_RIGHT):
			clicked.call(self, button)
			accept_event()
		elif event is InputEventMouseMotion:
			moved.call(self, (event as InputEventMouseMotion).position)
