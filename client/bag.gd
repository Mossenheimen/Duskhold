extends CanvasLayer
## What's within your reach, as Zomboid's list with weights (VISUALS.md §10, zone G), while the
## inventory is shut: each container beside you, and the floor, and water to drink from, a fire, a
## holdout's counter. Each line has its buttons: they ask the core for a timed action, which a blow
## breaks off. "Take all" and "Stash all finds" ask for one after another. Your pack is the
## inventory's (inventory.gd, Path of Exile 2's grids since 2026-11-16), and while it's open, so is
## all of this: it hides, and the inventory's left side shows the same, as grids.
##
## The rules for what a thing's buttons do live here, for both: using a thing as its first button
## did (`use`), taking, stashing, buying, mending, reading, lifting a curse, and what to say when
## the core says no.
##
## Your stash (home.rs): a find (one that came out of the world) put into it pays its worth in
## XP, once.
##
## A holdout's counter (holdout.rs), within reach, shows what it sells today and what it's short
## of, and the bounties posted there.
##
## What's within reach shows its card on hover, as your things do (VISUALS.md §11, Tooltip; the
## core's card.rs, as Path of Exile's and Diablo's are): its name in its magic's colour, its
## numbers as it was made and as far as you know its magic, its properties as numbers, a unique's
## effect in words, and how it stands against what you hold or wear there.

const Look = preload("res://look.gd")
const Prompts = preload("res://prompts.gd")

## An item's kind, by name (codes.gd), and a potion's.
const Item = preload("res://codes.gd").Item
const Potion = preload("res://codes.gd").Potion
## The places armour covers, by DuskWorld.item_covers()' bits.
const PLACES := [[1, "head"], [2, "torso"], [4, "arms"], [8, "legs"]]
## Numbers a stack takes, and a container's header, in DuskWorld's arrays.
const STACK := 4
const HEADER := 5
## A thing's numbers (DuskWorld.item_card(): its rarity, quality, condition, worth, find).
const CARD := 5
## Why a container can't be your stash, by DuskWorld.stash_check().
const NO_STASH := {-1: "That's not a container within your reach.",
	-2: "A stash wants a roof over it.",
	-3: "You have no home yet: sleep a night through somewhere first.",
	-4: "That's too far from where you sleep to be your stash."}

## Says a line of news (the HUD's).
var say: Callable
## Burns the body beside you, or says why not (main.gd's G).
var burn: Callable
## Consecrates the body beside you, or says why not (main.gd's O).
var hallow: Callable
## Shows a thing's card beside its line, and takes it away (the HUD's hover_card, unhover_card).
var card: Callable
var uncard: Callable

var _dusk: Node
var _names := PackedStringArray()
var _kinds := PackedInt32Array()
var _weights := PackedInt32Array()
var _worths := PackedInt32Array()
var _covers := PackedInt32Array()
var _potions := PackedInt32Array()  # what each item does as a potion (DuskWorld.potion_info())
var _fixtures := PackedStringArray()
var _reach: VBoxContainer
var _reach_panel: PanelContainer
var _hidden := false  # the inventory is open: its left side shows what's within reach
var _shown_reach := PackedInt32Array()
var _shown_holds := PackedInt32Array()  # what's in each full bag within reach, as last drawn
var _shown_water := false
var _queue: Array = []  # actions still to ask for: ["take" or "put", x, y, def]
var _shown_counter := []  # the counter within reach and what it sells, as last drawn
var _shown_fire := PackedInt32Array()  # the fire beside you (DuskWorld.fire_beside()), as last drawn


func _ready() -> void:
	_reach_panel = _panel(Control.PRESET_BOTTOM_LEFT)
	_reach_panel.visible = false  # until something's within reach
	_reach = _reach_panel.get_child(0)


## A new world: the item names and the fixtures' come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.item_names()
	_kinds = dusk.item_kinds()
	_weights = dusk.item_weights()
	_worths = dusk.item_worths()
	_covers = dusk.item_covers()
	_potions = dusk.potion_info()
	_fixtures = dusk.fixture_names()
	_shown_reach = PackedInt32Array()
	_shown_water = false
	_shown_counter = []
	_draw_reach(_shown_reach, false, PackedInt32Array(), PackedInt32Array())
	_queue.clear()


## Each frame: redraw what changed, and ask for the next queued action once you're free.
## `busy`: you're in the middle of an action; `dead`: nothing more to do; `hidden`: the inventory
## is open, and shows what's within reach itself (the queue goes on).
func refresh(busy: bool, dead: bool, hidden := false) -> void:
	if _dusk == null:
		return
	if dead:
		_reach_panel.visible = false
		return
	var items: PackedInt32Array = _dusk.character_items()
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	var counter: PackedInt32Array = _dusk.counter_in_reach()
	var water: bool = _dusk.water_in_reach()
	var sells := PackedInt32Array()
	if counter.size() >= 3:
		sells = _dusk.counter_sells(counter[0], counter[1])
	var at_counter := [counter, sells]
	var fire: PackedInt32Array = _dusk.fire_beside()
	var holds := _holds_in(reach)
	if reach != _shown_reach or water != _shown_water or at_counter != _shown_counter \
			or fire != _shown_fire or holds != _shown_holds or hidden != _hidden:
		_shown_reach = reach
		_shown_water = water
		_shown_counter = at_counter
		_shown_fire = fire
		_shown_holds = holds
		_hidden = hidden
		_draw_reach(reach, water, counter, sells)
	if not busy and not _queue.is_empty():
		_next_queued(reach, items)


## The fire in the hearth, stove or forge on (x, y), if it's the one beside you: "  ·  alight,
## 3h 20m of wood (hold [E]: put it out)", "  ·  laid, ...", "  ·  cold ([E] lays a fire)"; else
## "". The keys are read from the InputMap.
func fire_line(x: int, y: int) -> String:
	var f := _shown_fire
	if f.size() < 4 or f[0] != x or f[1] != y:
		return ""
	var wood := "%dh %02dm of wood" % [f[2] / 60, f[2] % 60]
	if f[3] == 1:
		return Prompts.keyed("  ·  alight, %s (hold {use}: put it out)") % wood
	if f[2] > 0:
		return Prompts.keyed("  ·  laid, %s ({use} lights it)") % wood
	return Prompts.keyed("  ·  cold ({use} lays a fire)")


## What's in the full bags within reach, every tile's stacks in turn (DuskWorld.store_holds()):
## a partial take from one changes its weight and nothing else.
func _holds_in(reach: PackedInt32Array) -> PackedInt32Array:
	var out := PackedInt32Array()
	var i := 0
	while i + HEADER <= reach.size():
		out.append_array(_dusk.store_holds(reach[i], reach[i + 1]))
		i += HEADER + reach[i + 3] * STACK
	return out


## A potion drunk from the pack (potion.rs), or why not: quickest from the belt.
func drink(slot: int, def: int) -> void:
	if _dusk.use_ticks(slot, def) >= 0:
		_dusk.use_item(slot, def)
	elif say.is_valid():
		var mana := def * 4 < _potions.size() and _potions[def * 4] == Potion.MANA
		say.call("You've no mana for it to fill: only a Wizard has." if mana else "You can't drink that now.")


## The stack at `slot` of your pack used as its first button did, the one its kind has (the
## inventory's right-click): a weapon or a bow taken in hand, a shield taken up, armour or a bag put
## on, a light lit, food eaten, drink or a potion drunk, a bandage tied, the cure taken, a decoy
## set, a piece put up, a body burnt or consecrated; or why not.
func use(slot: int, def: int) -> void:
	match _kind(def):
		Item.WEAPON, Item.BOW:
			wield(slot, def)
		Item.SHIELD:
			_dusk.wield(slot, def)
		Item.ARMOUR, Item.BAG:
			_dusk.wear(slot, def)
		Item.LIGHT:
			_dusk.light(slot, def)
		Item.BANDAGE, Item.FOOD, Item.DRINK, Item.DECOY:
			_dusk.use_item(slot, def)
		Item.CURE:
			take_cure(slot, def)
		Item.POTION:
			drink(slot, def)
		Item.PIECE:
			put_up(slot, def)
		Item.OIL:
			burn.call()
		Item.HOLY:
			hallow.call()
		_:
			if say.is_valid():
				say.call("There's nothing to do with the %s but carry it." % _name(def))


## What `use` does to a thing of kind `def`'s, in a word or two, for its card ("Wield", "Eat"),
## or "" for none.
func use_word(def: int) -> String:
	match _kind(def):
		Item.WEAPON, Item.BOW:
			return "Wield"
		Item.SHIELD:
			return "Take up"
		Item.ARMOUR, Item.BAG:
			return "Wear"
		Item.LIGHT:
			return "Light"
		Item.BANDAGE, Item.CURE:
			return "Use"
		Item.FOOD:
			return "Eat"
		Item.DRINK, Item.POTION:
			return "Drink"
		Item.DECOY:
			return "Set"
		Item.PIECE:
			return "Put up"
		Item.OIL:
			return "Burn a body"
		Item.HOLY:
			return "Consecrate a body"
	return ""


## Magic in colour: blue for Uncommon, gold for Rare, and dimmer while you don't know it
## (DuskWorld.character_rarity(), store_rarity(): 10 more for unknown).
func _tint(row: HBoxContainer, rarity: PackedInt32Array, place: int) -> void:
	if place < rarity.size() and rarity[place] != 0:
		row.get_child(0).add_theme_color_override("font_color", rarity_colour(rarity[place]))


## A rarity code's colour; none (alpha 0) for a mundane thing.
func rarity_colour(code: int) -> Color:
	if code == 0:
		return Color(0, 0, 0, 0)
	var colour: Color = [Look.UNCOMMON, Look.RARE, Look.UNIQUE][clampi(code % 10, 1, 3) - 1]
	return colour if code < 10 else colour.darkened(0.25)


## Hovering `row`'s words shows the card `what` gives ([title, lines, colour], or [] for none)
## beside the row, to its right.
func _hoverable(row: HBoxContainer, what: Callable) -> void:
	var words := row.get_child(0) as Control
	words.mouse_filter = Control.MOUSE_FILTER_STOP
	words.mouse_entered.connect(func() -> void: _hover(row, what))
	words.mouse_exited.connect(_unhover.bind(row))


func _unhover(row: HBoxContainer) -> void:
	if uncard.is_valid():
		uncard.call(row)


func _hover(row: HBoxContainer, what: Callable) -> void:
	var shown: Array = what.call()
	if not shown.is_empty() and card.is_valid():
		card.call(row, shown[0], shown[1], true, shown[2])


## A card's words for a trial run's print, a part a line ("Card: Redhand | Fine hand axe | ... |
## Damage 21.4–31.3 | ..."): `shown` as `card_of` gives it.
static func card_words(shown: Array) -> String:
	var words := PackedStringArray([shown[0]])
	var lines: PackedStringArray = shown[1]
	for i in range(0, lines.size() - 2, 3):
		if lines[i] != "rule":
			words.append((lines[i + 1] + " " + lines[i + 2]).strip_edges())
	return " | ".join(words)


## A thing's card (VISUALS.md §11, Tooltip; DuskWorld.card_lines()): [its title, its lines, its
## title's colour], or [] for a thing with none. The title is what it's called as far as you know
## it, in its magic's colour (`numbers`: DuskWorld.item_card()); the lines are the rest of the
## card, three words each, as the HUD draws them (hud.gd's _card_line).
func card_of(numbers: PackedInt32Array, lines: PackedStringArray) -> Array:
	if numbers.size() < CARD or lines.size() < 3:
		return []
	if lines[0] != "title":
		return ["", lines, rarity_colour(numbers[0])]
	return [lines[1], lines.slice(3), rarity_colour(numbers[0])]


## What one of a stack is worth in XP, by its card (DuskWorld.item_card()); -1 for no card.
func worth_of(numbers: PackedInt32Array) -> int:
	return numbers[3] if numbers.size() >= CARD else -1


func read(x: int, y: int, slot: int) -> void:
	if _dusk.identify_check(x, y, slot) == -2:
		say.call("You haven't the coin to have it read.")
	else:
		_dusk.identify(x, y, slot)


func uncurse(x: int, y: int) -> void:
	if _dusk.uncurse_check(x, y) == -2:
		say.call("You haven't the coin to have the curse lifted.")
	else:
		_dusk.uncurse(x, y)


func clings() -> void:
	say.call("It clings to your hand: it's cursed. The abbey can lift a curse, for coin.")


## Taking a weapon in hand: not while a cursed one clings there.
func wield(slot: int, def: int) -> void:
	if _dusk.hand_clings():
		clings()
	else:
		_dusk.wield(slot, def)


## The cure, taken only with a fever to cure: the core won't start it otherwise, so say why.
func take_cure(slot: int, def: int) -> void:
	if _dusk.character_fever()[0] == 0:
		say.call("You've no fever to cure. Keep it: a ghoul's bite brings one.")
	else:
		_dusk.use_item(slot, def)


## A door or a wall of planks put up on the side you face (build.rs), or why not.
func put_up(slot: int, def: int) -> void:
	match _dusk.put_up_check(def):
		-1:
			say.call("Face an open side of your tile to put it up on: there's a wall, a door or a window where you face.")
		-2:
			say.call("A door hangs only in a gap in a wall: face one framed at both ends, as where a door gave way.")
		-3:
			say.call("Not while someone's stepping through.")
		-4:
			say.call("You need a hammer to put it up.")
		_:
			_dusk.use_item(slot, def)


func _draw_reach(reach: PackedInt32Array, water: bool, counter: PackedInt32Array,
		sells: PackedInt32Array) -> void:
	for child in _reach.get_children():
		child.queue_free()
	_reach_panel.visible = (reach.size() > 0 or water or counter.size() >= 3) and not _hidden
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
		var header := _row(_reach, place + ("  (empty)" if n == 0 else "") + fire_line(x, y))
		if mine:
			header.get_child(0).add_theme_color_override("font_color", Look.GOLD)
			_button(header, "Stash all finds", func() -> void: stash_finds(x, y))
		elif fixture != 0 and _dusk.stash_check(x, y) == 0:
			# Only where it can be (the second playtest: "obnoxious when you see it everywhere").
			_button(header, "Make it your stash", func() -> void: mark_stash(x, y))
		if n > 1:
			var defs := []
			for k in n:
				defs.append(reach[i + k * STACK])
			_button(header, "Take all", func() -> void: take_all(x, y, defs))
		var known: PackedStringArray = _dusk.store_labels(x, y)
		var magic: PackedInt32Array = _dusk.store_rarity(x, y)
		var holds: PackedInt32Array = _dusk.store_holds(x, y)
		for slot in n:
			var def := reach[i + slot * STACK]
			var count := reach[i + slot * STACK + 1]
			var found := reach[i + slot * STACK + 3] == 1
			var text := describe(def, count, found, worth_of(_dusk.store_card(x, y, slot)))
			if slot < known.size() and known[slot] != _name(def):
				text = known[slot] + text.substr(_name(def).length())  # magic, as far as you know it
			if slot < holds.size() and holds[slot] > 0:
				text += "  ·  %.1f kg in it" % (holds[slot] / 1000.0)  # a bag dropped full
			var row := _row(_reach, "   " + text)
			_tint(row, magic, slot)
			_button(row, "Take", func() -> void: take(x, y, slot, def))
			var what := func() -> Array:
				return card_of(_dusk.store_card(x, y, slot), _dusk.store_card_lines(x, y, slot))
			_hoverable(row, what)
		i += n * STACK


## A holdout's counter: what it sells today, at what price and how many are left, and what it's
## short of (selling it those pushes the dusk back).
func _draw_counter(counter: PackedInt32Array, sells: PackedInt32Array) -> void:
	var x := counter[0]
	var y := counter[1]
	var name: String = _dusk.holdout_name(counter[2])
	# "The abbey on the hill": only the first letter raised (capitalize() raises every word).
	var title := name.left(1).to_upper() + name.substr(1)
	var header := _line(_reach, title + "  ·  the counter", Look.HEADING)
	header.add_theme_color_override("font_color", Look.HOLY)
	var wants: PackedInt32Array = _dusk.counter_wants(x, y)
	if wants.size() > 0:
		var short := []
		for w in wants:
			short.append(_name(w))
		_line(_reach, "Short of: %s.  Sell it these to push the dusk back." % ", ".join(short), Look.SMALL)
	_draw_bounties(x, y)
	for line in range(0, sells.size() / 4):
		var def := sells[line * 4]
		var price := sells[line * 4 + 1]
		var left := sells[line * 4 + 2]
		var can := sells[line * 4 + 3]
		var text := "   %s  ·  %d coin  ·  %s" % [_name(def), price,
			"none left today" if left == 0 else "%d left today" % left]
		var row := _row(_reach, text)
		_button(row, "Buy", func() -> void: buy(x, y, line, can))


## The bounties posted at the counter: each rare, where it was last heard of, and a Claim button
## once it's been put down.
func _draw_bounties(x: int, y: int) -> void:
	var bounties: PackedInt32Array = _dusk.counter_bounties(x, y)
	if bounties.size() == 0:
		return
	_line(_reach, "Bounties:", Look.SMALL)
	for k in range(0, bounties.size() / 5):
		var who := bounties[k * 5]
		var state := bounties[k * 5 + 3]
		var coin := bounties[k * 5 + 4]
		var name: String = _dusk.actor_name(who)
		var more: PackedStringArray = _dusk.rare_affixes(who)
		var also := ""
		if more.size() > 1:
			also = " (" + ", ".join(more.slice(1)) + " too)"
		var text := "   %s%s  ·  %d coin  ·  " % [name, also, coin]
		match state:
			0:
				text += "last heard of about (%d, %d)" % [bounties[k * 5 + 1], bounties[k * 5 + 2]]
			1:
				text += "put down"
			_:
				text += "paid"
		var row := _row(_reach, text)
		if state == 1:
			_button(row, "Claim", func() -> void: _dusk.claim(x, y, who))


## "Buy": the core says whether you can, and this says why not.
func buy(x: int, y: int, line: int, can: int) -> void:
	if can == 1:
		_dusk.buy(x, y, line)
	elif say.is_valid():
		match can:
			-1:
				say.call("None left today: more tomorrow.")
			-3:
				say.call(no_room())
			_:
				say.call("You haven't the coin.")


## "Take": as many as there's room for (bag.rs). The core says how many; this says why none.
## Returns the core's answer: how many, 0 for no room, -1 if it's gone.
func take(x: int, y: int, slot: int, def: int) -> int:
	var n: int = _dusk.take_check(x, y, slot, def)
	if n > 0:
		_dusk.take(x, y, slot, def)
	elif n == 0 and say.is_valid():
		say.call(no_room())
	return n


## Why nothing more fits: with no bag, it's the pockets.
func no_room() -> String:
	if _dusk.character_bag().size() < 6:
		return "No room for it: your pockets are full. A bag would hold more."
	return "No room for it in your bag or your pockets."


## "Mend": the weapon in your hand, made good as new for its price.
func mend(x: int, y: int) -> void:
	var cost: int = _dusk.mend_check(x, y)
	if cost > 0:
		_dusk.mend(x, y)
		if say.is_valid():
			say.call("Mending it costs %d coin." % cost)
	elif say.is_valid():
		say.call("You haven't the coin to have it mended." if cost == -2 else "There's nothing to mend.")


## "Make it your stash": the core says whether it can be, and this says why not.
func mark_stash(x: int, y: int) -> void:
	var why: int = _dusk.stash_check(x, y)
	if why == 0:
		_dusk.mark_stash(x, y)
	elif say.is_valid():
		say.call(NO_STASH.get(why, "It can't be your stash."))


## "Take all": one take after another, each asked for as the last one finishes.
func take_all(x: int, y: int, defs: Array) -> void:
	for def in defs:
		_queue.append(["take", x, y, def])


## "Stash all finds": every find in the pack, put into the stash one after another.
func stash_finds(x: int, y: int) -> void:
	var items: PackedInt32Array = _dusk.character_items()
	for i in range(0, items.size(), STACK):
		if items[i + 3] == 1:
			_queue.append(["put", x, y, items[i]])
	if _queue.is_empty() and say.is_valid():
		say.call("You're carrying no finds.")


## The next queued action: a take of the first stack on that tile that's still the item it was,
## or a put of the first find of that item in the pack. Once there's no room, the takes stop.
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
					if take(want[1], want[2], slot, want[3]) == 0:
						_queue.clear()
					return
		i += HEADER + n * STACK


## "3 apples · 0.5 kg", and for a find, what it brings home: `each` XP apiece (as it was made and
## its magic: DuskWorld.item_card()), or -1 for its kind's.
func describe(def: int, count: int, found := false, each := -1) -> String:
	var grams := _weights[def] * count if def < _weights.size() else 0
	var label := _name(def) if count == 1 else "%s ×%d" % [_name(def), count]
	label = "%s  ·  %.1f kg" % [label, grams / 1000.0]
	if each < 0:
		each = _worths[def] if def < _worths.size() else 0
	var worth := each * count
	if found and worth > 0:
		label += "  ·  a find: %d XP home" % worth
	return label


func _name(def: int) -> String:
	return _names[def] if def < _names.size() else "something"


func _kind(def: int) -> int:
	return _kinds[def] if def < _kinds.size() else 0


## Where a piece of armour goes: "torso, arms".
func places(def: int) -> String:
	var bits := _covers[def] if def < _covers.size() else 0
	var names := PackedStringArray()
	for place in PLACES:
		if bits & place[0]:
			names.append(place[1])
	return ", ".join(names)


func _fixture_name(fixture: int) -> String:
	return _fixtures[fixture] if fixture < _fixtures.size() else "container"


func _panel(preset: Control.LayoutPreset) -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", Look.panel_style())
	var list := VBoxContainer.new()
	list.add_theme_constant_override("separation", 2)
	panel.add_child(list)
	add_child(panel)
	panel.set_anchors_and_offsets_preset(preset, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN if preset == Control.PRESET_BOTTOM_RIGHT \
		else Control.GROW_DIRECTION_END
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	return panel


func _line(parent: Control, text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	Look.sized(label, size)
	parent.add_child(label)
	return label


func _row(parent: Control, text: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	parent.add_child(row)
	var label := _line(row, text, Look.BODY)
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.custom_minimum_size = Vector2(300, 0)
	return row


func _button(row: HBoxContainer, text: String, pressed: Callable) -> void:
	row.add_child(Look.button(text, pressed))


## How fresh food is, from its condition (out of 1000): it's gone at none (the core's `cooking.rs`).
func freshness(condition: int) -> String:
	if condition > 660:
		return "fresh"
	if condition > 330:
		return "stale"
	return "going off"
