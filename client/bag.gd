extends CanvasLayer
## Zomboid's inventory windows, as lists with weights (CONCEPT.md: no grid). On the right, your
## pack: what's in your hand, and what you carry, with its weight against what you can carry
## before you can't run. What you carry is in your pockets or in the bag on your back (bag.rs),
## each with what it holds; a line moves between them. On the left, what's within your reach:
## each container beside you, and the floor, and water to drink from. Each line has its
## buttons: they ask the core for a timed action, which a blow breaks off. "Take all" and "Stash
## all finds" ask for one after another.
##
## Your stash (home.rs): a find (one that came out of the world) put into it pays its worth in
## XP, once. The pack marks finds with their worth; with your stash within reach, each line has
## a Stash button.
##
## A holdout's counter (holdout.rs), within reach, shows what it sells today and what it's short
## of; the pack's lines then have a Sell button (a find sold pays its XP too), and the weapon in
## your hand a Mend button.

const Look = preload("res://look.gd")

const WEAPON := 1
const BANDAGE := 2
const FOOD := 3
const DRINK := 4
const LIGHT := 5
const CURE := 6
const OIL := 7
const SHIELD := 8
const BOW := 9
const HOLY := 10
const ARMOUR := 11
const BAG := 12
## The places armour covers, by DuskWorld.item_covers()' bits.
const PLACES := [[1, "head"], [2, "torso"], [4, "arms"], [8, "legs"]]
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
## Consecrates the body beside you, or says why not (main.gd's O).
var hallow: Callable

var _dusk: Node
var _names := PackedStringArray()
var _kinds := PackedInt32Array()
var _weights := PackedInt32Array()
var _worths := PackedInt32Array()
var _covers := PackedInt32Array()
var _fixtures := PackedStringArray()
var _pack: VBoxContainer
var _reach: VBoxContainer
var _pack_panel: PanelContainer
var _reach_panel: PanelContainer
var _shown_pack := []  # what the pack panel shows now, to rebuild only when it changes
var _shown_reach := PackedInt32Array()
var _shown_holds := PackedInt32Array()  # what's in each full bag within reach, as last drawn
var _shown_water := false
var _queue: Array = []  # actions still to ask for: ["take" or "put", x, y, def]
var _shown_counter := []  # the counter within reach and what it sells, as last drawn
var _shown_fire := PackedInt32Array()  # the fire beside you (DuskWorld.fire_beside()), as last drawn


func _ready() -> void:
	_pack_panel = _panel(Control.PRESET_BOTTOM_RIGHT)
	_pack = _pack_panel.get_child(0)
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
	_fixtures = dusk.fixture_names()
	_shown_pack = []
	_shown_reach = PackedInt32Array()
	_shown_water = false
	_shown_counter = []
	_pack_panel.visible = true  # hidden at a death; the next survivor has a pack again
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
	var labels: PackedStringArray = _dusk.character_labels()
	var rarity: PackedInt32Array = _dusk.character_rarity()
	var shield: PackedInt32Array = _dusk.character_shield()
	var worn: PackedInt32Array = _dusk.character_worn()
	var bag: PackedInt32Array = _dusk.character_bag()
	var pockets: PackedInt32Array = _dusk.character_pockets()
	var bagged: PackedInt32Array = _dusk.character_bagged()
	var pack := [hands, items, carried, light, stash, counter, coins, labels, rarity, shield, worn,
		bag, pockets, bagged]
	if pack != _shown_pack:
		_shown_pack = pack
		_draw_pack(hands, items, carried, light, stash, counter, coins)
	var water: bool = _dusk.water_in_reach()
	var sells := PackedInt32Array()
	if counter.size() >= 3:
		sells = _dusk.counter_sells(counter[0], counter[1])
	var at_counter := [counter, sells]
	var fire: PackedInt32Array = _dusk.fire_beside()
	var holds := _holds_in(reach)
	if reach != _shown_reach or water != _shown_water or at_counter != _shown_counter \
			or fire != _shown_fire or holds != _shown_holds:
		_shown_reach = reach
		_shown_water = water
		_shown_counter = at_counter
		_shown_fire = fire
		_shown_holds = holds
		_draw_reach(reach, water, counter, sells)
	if not busy and not _queue.is_empty():
		_next_queued(reach, items)


## The fire in the hearth, stove or forge on (x, y), if it's the one beside you: "  ·  alight,
## 3h 20m of wood (J puts it out)", "  ·  laid, ...", "  ·  cold (H lays a fire)"; else "".
func _fire_line(x: int, y: int) -> String:
	var f := _shown_fire
	if f.size() < 4 or f[0] != x or f[1] != y:
		return ""
	var wood := "%dh %02dm of wood" % [f[2] / 60, f[2] % 60]
	if f[3] == 1:
		return "  ·  alight, %s (J puts it out)" % wood
	if f[2] > 0:
		return "  ·  laid, %s (H lights it)" % wood
	return "  ·  cold (H lays a fire)"


## What's in the full bags within reach, every tile's stacks in turn (DuskWorld.store_holds()):
## a partial take from one changes its weight and nothing else.
func _holds_in(reach: PackedInt32Array) -> PackedInt32Array:
	var out := PackedInt32Array()
	var i := 0
	while i + HEADER <= reach.size():
		out.append_array(_dusk.store_holds(reach[i], reach[i + 1]))
		i += HEADER + reach[i + 3] * STACK
	return out


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
		carried[1] / 1000.0, coins, "  ·  too heavy to run" if heavy else ""], Look.HEADING)
	if heavy:
		title.add_theme_color_override("font_color", Look.BAD)
	var trading := counter.size() >= 3
	var labels: PackedStringArray = _dusk.character_labels()
	var rarity: PackedInt32Array = _dusk.character_rarity()
	if hands.size() >= 2:
		var row := _row(_pack, "In hand: %s (%d%%)" % [labels[0], roundi(hands[1] / 10.0)])
		_tint(row, rarity, 0)
		if _dusk.hand_clings():
			_button(row, "It clings", func() -> void: _clings())
		else:
			_button(row, "Put away", func() -> void: _dusk.unwield())
		if trading:
			var cost: int = _dusk.mend_check(counter[0], counter[1])
			if cost > 0 or cost == -2:
				_button(row, "Mend", func() -> void: _mend(counter[0], counter[1]))
			_reading_buttons(row, counter, -1)
			if _dusk.uncurse_check(counter[0], counter[1]) != -1:
				_button(row, "Lift the curse", func() -> void: _uncurse(counter[0], counter[1]))
		_show_lines(0)
	else:
		_line(_pack, "In hand: nothing (bare fists)", Look.BODY)
	if light.size() >= 2:
		var row := _row(_pack, "Held up: %s, lit (%d%% left)" % [_name(light[0]), roundi(light[1] / 10.0)])
		_button(row, "Put out", func() -> void: _dusk.douse())
	var shield: PackedInt32Array = _dusk.character_shield()
	if shield.size() >= 2:
		var row := _row(_pack, "On your arm: %s (%d%%)  ·  hold right-click to brace" % [_name(shield[0]), roundi(shield[1] / 10.0)])
		_button(row, "Put away", func() -> void: _dusk.unshield())
	# What you wear (armour.rs): with nothing in hand, the first piece takes the Mend button.
	var worn: PackedInt32Array = _dusk.character_worn()
	var piece := -1
	for i in range(0, worn.size(), STACK):
		piece += 1
		var def := worn[i]
		var row := _row(_pack, "Worn: %s (%d%%)  ·  %s" % [_name(def), roundi(worn[i + 2] / 10.0), _places(def)])
		_button(row, "Take off", func() -> void: _dusk.take_off(piece, def))
		if trading and piece == 0 and hands.size() < 2:
			var cost: int = _dusk.mend_check(counter[0], counter[1])
			if cost > 0 or cost == -2:
				_button(row, "Mend", func() -> void: _mend(counter[0], counter[1]))
	# Your pockets, then the bag on your back (bag.rs): each stack's line under the one it's in.
	var bag: PackedInt32Array = _dusk.character_bag()
	var pockets: PackedInt32Array = _dusk.character_pockets()
	var bagged: PackedInt32Array = _dusk.character_bagged()
	var wearing := bag.size() >= 5
	if pockets.size() >= 2 and pockets[1] >= 0:
		var full := pockets[0] > pockets[1]
		var line := _line(_pack, "Pockets  ·  %.1f / %.1f kg%s" % [pockets[0] / 1000.0,
			pockets[1] / 1000.0, "  ·  over full" if full else ""], Look.SMALL)
		if full:
			line.add_theme_color_override("font_color", Look.BAD)
	for place in ([0, 1] if wearing else [0]):
		if place == 1:
			var row := _row(_pack, "On your back: %s  ·  %.1f / %.1f kg in it, %d%% lighter" % [
				_name(bag[0]), bag[2] / 1000.0, bag[3] / 1000.0, roundi(bag[4] / 10.0)])
			_button(row, "Drop", func() -> void: _dusk.drop_bag())
		for slot in range(items.size() / STACK):
			var in_bag := slot < bagged.size() and bagged[slot] == 1
			if int(in_bag) == place:
				_pack_line(slot, items, labels, rarity, stash, counter, wearing, in_bag)


## One stack of the pack: what it is, and its buttons. `wearing`: a bag's on your back, so the
## stack can move between your pockets and the bag; `in_bag`: it's in the bag now.
func _pack_line(slot: int, items: PackedInt32Array, labels: PackedStringArray,
		rarity: PackedInt32Array, stash: Vector2i, counter: PackedInt32Array, wearing: bool,
		in_bag: bool) -> void:
	var i := slot * STACK
	var def := items[i]
	var count := items[i + 1]
	var label := _describe(def, count, items[i + 3] == 1)
	if 2 + slot < labels.size() and labels[2 + slot] != _name(def):
		label = labels[2 + slot] + label.substr(_name(def).length())  # magic: what you know of it
	if _kind(def) == LIGHT and items[i + 2] < 1000:
		label += "  ·  %d%% left" % roundi(items[i + 2] / 10.0)  # a part-burnt light
	if _kind(def) == FOOD and items[i + 2] < 1000:
		label += "  ·  " + _freshness(items[i + 2])  # food that's begun to go off
	if _kind(def) == ARMOUR:
		label += "  ·  " + _places(def)  # where it goes
	var row := _row(_pack, "   " + label)
	_tint(row, rarity, 2 + slot)
	match _kind(def):
		WEAPON:
			_button(row, "Wield", func() -> void: _wield(slot, def))
		SHIELD:
			_button(row, "Take up", func() -> void: _dusk.wield(slot, def))
		BOW:
			_button(row, "Wield", func() -> void: _wield(slot, def))
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
		HOLY:
			_button(row, "Consecrate a body", func() -> void: hallow.call())
		ARMOUR, BAG:
			_button(row, "Wear", func() -> void: _dusk.wear(slot, def))
	if wearing:
		_button(row, "To pockets" if in_bag else "To bag", func() -> void: _shift(slot, def, in_bag))
	if stash.x >= 0:
		_button(row, "Stash", func() -> void: _dusk.put_item(stash.x, stash.y, slot, def))
	if counter.size() >= 3:
		_reading_buttons(row, counter, slot)
		var price: int = _dusk.sell_price(counter[0], counter[1], slot, def)
		if price >= 0:
			_button(row, "Sell · %d" % price, func() -> void: _dusk.sell(counter[0], counter[1], slot, def))
	_button(row, "Drop", func() -> void: _dusk.drop_item(slot, def))
	_show_lines(2 + slot)


## "To bag", "To pockets": as many as fit. The core says how many, and this says why none.
func _shift(slot: int, def: int, in_bag: bool) -> void:
	if _dusk.shift_check(slot, def) > 0:
		_dusk.shift(slot, def)
	elif say.is_valid():
		say.call("No room in your pockets." if in_bag else "No room in your bag.")


## Magic in colour: blue for Uncommon, gold for Rare, and dimmer while you don't know it
## (DuskWorld.character_rarity(): 10 more for unknown).
func _tint(row: HBoxContainer, rarity: PackedInt32Array, place: int) -> void:
	if place >= rarity.size() or rarity[place] == 0:
		return
	var known := rarity[place] < 10
	var colour := Look.UNCOMMON if rarity[place] % 10 == 1 else Look.RARE
	if not known:
		colour = colour.darkened(0.25)
	row.get_child(0).add_theme_color_override("font_color", colour)


## Under a magic thing you know, its lines: what each property does.
func _show_lines(place: int) -> void:
	var lines: PackedStringArray = _dusk.character_lines(place)
	for k in range(1, lines.size()):
		var line := _line(_pack, "      " + lines[k], Look.SMALL)
		line.add_theme_color_override("font_color", Look.LORE)


## At a counter: "Read" for a magic thing you don't know yet (`slot` -1: what's in hand).
func _reading_buttons(row: HBoxContainer, counter: PackedInt32Array, slot: int) -> void:
	var price: int = _dusk.identify_check(counter[0], counter[1], slot)
	if price == -1:
		return
	_button(row, "Read · %d" % maxi(price, 0), func() -> void: _read(counter[0], counter[1], slot))


func _read(x: int, y: int, slot: int) -> void:
	if _dusk.identify_check(x, y, slot) == -2:
		say.call("You haven't the coin to have it read.")
	else:
		_dusk.identify(x, y, slot)


func _uncurse(x: int, y: int) -> void:
	if _dusk.uncurse_check(x, y) == -2:
		say.call("You haven't the coin to have the curse lifted.")
	else:
		_dusk.uncurse(x, y)


func _clings() -> void:
	say.call("It clings to your hand: it's cursed. The abbey can lift a curse, for coin.")


## Taking a weapon in hand: not while a cursed one clings there.
func _wield(slot: int, def: int) -> void:
	if _dusk.hand_clings():
		_clings()
	else:
		_dusk.wield(slot, def)


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
		var header := _row(_reach, place + ("  (empty)" if n == 0 else "") + _fire_line(x, y))
		if mine:
			header.get_child(0).add_theme_color_override("font_color", Look.GOLD)
			_button(header, "Stash all finds", func() -> void: _stash_finds(x, y))
		elif fixture != 0:
			_button(header, "Make it your stash", func() -> void: _mark_stash(x, y))
		if n > 1:
			var defs := []
			for k in n:
				defs.append(reach[i + k * STACK])
			_button(header, "Take all", func() -> void: _take_all(x, y, defs))
		var known: PackedStringArray = _dusk.store_labels(x, y)
		var holds: PackedInt32Array = _dusk.store_holds(x, y)
		for slot in n:
			var def := reach[i + slot * STACK]
			var count := reach[i + slot * STACK + 1]
			var found := reach[i + slot * STACK + 3] == 1
			var text := _describe(def, count, found)
			if slot < known.size() and known[slot] != _name(def):
				text = known[slot] + text.substr(_name(def).length())  # magic, as far as you know it
			if slot < holds.size() and holds[slot] > 0:
				text += "  ·  %.1f kg in it" % (holds[slot] / 1000.0)  # a bag dropped full
			var row := _row(_reach, "   " + text)
			_button(row, "Take", func() -> void: _take(x, y, slot, def))
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
		_button(row, "Buy", func() -> void: _buy(x, y, line, can))


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
func _buy(x: int, y: int, line: int, can: int) -> void:
	if can == 1:
		_dusk.buy(x, y, line)
	elif say.is_valid():
		match can:
			-1:
				say.call("None left today: more tomorrow.")
			-3:
				say.call(_no_room())
			_:
				say.call("You haven't the coin.")


## "Take": as many as there's room for (bag.rs). The core says how many; this says why none.
## Returns the core's answer: how many, 0 for no room, -1 if it's gone.
func _take(x: int, y: int, slot: int, def: int) -> int:
	var n: int = _dusk.take_check(x, y, slot, def)
	if n > 0:
		_dusk.take(x, y, slot, def)
	elif n == 0 and say.is_valid():
		say.call(_no_room())
	return n


## Why nothing more fits: with no bag, it's the pockets.
func _no_room() -> String:
	if _dusk.character_bag().size() < 5:
		return "No room for it: your pockets are full. A bag would hold more."
	return "No room for it in your bag or your pockets."


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
					if _take(want[1], want[2], slot, want[3]) == 0:
						_queue.clear()
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


## Where a piece of armour goes: "torso, arms".
func _places(def: int) -> String:
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
	var button := Button.new()
	button.text = text
	button.add_theme_font_size_override("font_size", Look.SMALL)
	button.focus_mode = Control.FOCUS_NONE  # keys stay with the game (Space, WASD)
	button.pressed.connect(pressed)
	row.add_child(button)


## How fresh food is, from its condition (out of 1000): it's gone at none (the core's `cooking.rs`).
func _freshness(condition: int) -> String:
	if condition > 660:
		return "fresh"
	if condition > 330:
		return "stale"
	return "going off"
