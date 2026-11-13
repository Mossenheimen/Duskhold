extends RefCounted
## What you can do here (VISUALS.md §10 zone F, §11 prompts): everything beside you, worded from
## the core's own checks (DuskWorld's *_check, the ones main.gd's handlers ask before they act),
## never a rule of its own. One key does it all, as in Zomboid, Valheim or Don't Starve: E (`use`)
## does the likeliest; held, it lists them all, to pick one by its number or a click.
##
## Each entry is [its verb (main.gd's `_do`), its words ("Butcher the deer"), why it can't be done
## now ("" if it can), its place]. The place says how it's offered:
## - LIKELY: E may do it, and its prompt shows: a body or a carcass, a plant, an altar, a fire, the
##   door or window you face, and water when you're thirsty;
## - QUIET: E may do it, with no prompt: a door or window beside you that you don't face (they're
##   everywhere in a town);
## - MORE: never E's, since it's done on purpose: boarding up, prying a board off, a ward, putting
##   a fire out. The line under the prompt names it when it can be done;
## - LISTED: only in the held list: water when you aren't thirsty, and what can't be done yet.

const LIKELY := 0
const QUIET := 1
const MORE := 2
const LISTED := 3
const MOST := 9  # the held list's longest: a number key each
## Keys' names as a key cap says them, where Godot's are long.
const SHORT := {"Escape": "Esc", "Minus": "-", "Equal": "="}

## "{use}" in a line of text: an action's key cap; "{pack|all}" every key it's on.
static var _braces := RegEx.create_from_string("\\{([a-z_0-9]+)(\\|all)?\\}")


## Everything you can do here now, in the order it's offered. `creatures`, `fixtures`, `kinds`:
## DuskWorld.creature_names(), fixture_names() and actor_kinds(); `thirsty`: whether you'd want a
## drink (the moodle).
static func here(dusk: Node, creatures: PackedStringArray, fixtures: PackedStringArray,
		kinds: PackedInt32Array, thirsty: bool) -> Array:
	var all := []
	# The door or window you face comes first: it's what you walked up to.
	var door: PackedInt32Array = dusk.door_check()
	var faced := door.size() >= 4 and door[3] == 1
	if faced:
		_door(all, door, LIKELY)
	# A beast's carcass is cut up before any body is burnt (main.gd's _burn_body).
	var carcass: int = dusk.butcher_check()
	if carcass >= 0:
		all.append(_entry("burn", "Butcher the " + _creature(carcass, creatures, kinds)))
	elif carcass == -2:
		all.append(_entry("burn", "Butcher the carcass", "you need a knife or an axe"))
	else:
		match dusk.burn_check():
			-1:
				pass
			-2:
				all.append(_entry("burn", "Burn the body", "you need oil"))
			-3:
				all.append(_entry("burn", "Burn the body", "hold up a light to set it alight {light}"))
			_:
				all.append(_entry("burn", "Burn the body"))
	var body: int = dusk.consecrate_check()
	if body >= 0:
		all.append(_entry("consecrate", "Consecrate the body"))
	elif body == -2:
		all.append(_entry("consecrate", "Consecrate the body", "you need holy water"))
	var plant: int = dusk.pick_check()
	if plant > 0:
		all.append(_entry("pick", "Pick the " + _fixture(plant, fixtures)))
	if dusk.pray_check() == 1:
		all.append(_entry("pray", "Pray at the altar"))
	# A hearth, stove or forge: alight with all it can hold, there's nothing to feed it.
	var fire: PackedInt32Array = dusk.fire_beside()
	if fire.size() >= 5:
		var it := "the " + _fixture(fire[4], fixtures)
		var alight := fire[3] == 1
		match dusk.fire_check():
			1:
				all.append(_entry("tend_fire", ("Feed " if alight else "Lay a fire in ") + it))
			2:
				all.append(_entry("tend_fire", "Light " + it))
			-3:
				if not alight:
					all.append(_entry("tend_fire", "Lay a fire in " + it, "you have nothing that burns"))
			-4:
				all.append(_entry("tend_fire", "Light " + it, "hold up a lit torch or candle {light}"))
		if alight:
			all.append(_entry("smother_fire", "Put out " + it, "", MORE))
	if dusk.water_in_reach():
		all.append(_entry("drink", "Drink your fill", "", LIKELY if thirsty else LISTED))
	if door.size() >= 4 and not faced:
		_door(all, door, QUIET)
	# The boards, on the door or window beside you (the core picks which: the one you face first).
	match dusk.board_check(false):
		-1, -3:
			pass  # none beside you, or it takes no more
		-2:
			all.append(_entry("board", "Board it up", "shut it first", LISTED))
		-4:
			all.append(_entry("board", "Board it up", "you need a hammer", LISTED))
		-5:
			all.append(_entry("board", "Board it up", "you need a plank", LISTED))
		-6:
			all.append(_entry("board", "Board it up", "you need nails: two a board", LISTED))
		_:
			all.append(_entry("board", "Board it up", "", MORE))
	match dusk.board_check(true):
		-1:
			pass
		-4:
			all.append(_entry("unboard", "Pry a board off", "you need a hammer", LISTED))
		_:
			all.append(_entry("unboard", "Pry a board off", "", MORE))
	match dusk.ward_check():
		1:
			all.append(_entry("ward", "Ward the door", "", MORE))
		-3:
			all.append(_entry("ward", "Ward the door", "not enough faith: pray at an altar", LISTED))
	return all.slice(0, MOST)


## The entry E does (an index in `entries`): the first that can be done and is E's to do (LIKELY
## or QUIET); with none, the first likely one, so its handler says why not; else -1, and E lists
## what there is instead.
static func first(entries: Array) -> int:
	for i in entries.size():
		if entries[i][2] == "" and entries[i][3] <= QUIET:
			return i
	for i in entries.size():
		if entries[i][3] == LIKELY:
			return i
	return -1


## What zone F shows of `entries`, each [a line, whether it's bright]. Listed (E held), all of
## them, numbered, then how to pick. Else E's prompt ("[E] Butcher the deer", dimmed with its
## reason if it can't be done); or, when E would only open a door you don't face, what's easily
## missed beside you, dimmed and with no key; then, dimmed, what holding E offers besides.
static func lines(entries: Array, listing: bool) -> Array:
	var shown := []
	if listing:
		for i in entries.size():
			shown.append(["[%d] %s" % [i + 1, _words(entries[i])], entries[i][2] == ""])
		shown.append([keyed("A number or a click  ·  {quit} closes"), false])
		return shown
	var e := first(entries)
	if e >= 0 and entries[e][3] == LIKELY:
		shown.append([keyed("{use} ") + _words(entries[e]), entries[e][2] == ""])
	else:
		for entry: Array in entries:
			if entry[3] == LIKELY:
				shown.append([_words(entry), false])
				break
	var more := PackedStringArray()
	for i in entries.size():
		if i != e and entries[i][2] == "" and entries[i][3] in [LIKELY, MORE]:
			var words: String = entries[i][1]
			more.append(words.left(1).to_lower() + words.substr(1))
	if not more.is_empty():
		var named := ", ".join(more.slice(0, 2)) + (", …" if more.size() > 2 else "")
		shown.append([keyed("Hold {use}: ") + named, false])
	return shown


## The keys `action` is on, as the keyboard shows them ("E", "Ctrl+N"): read from the InputMap,
## so a remapped key is said wherever it's shown (VISUALS.md §11, key caps), and on another
## layout, its own label.
static func keys(action: String) -> PackedStringArray:
	var all := PackedStringArray()
	if not InputMap.has_action(action):
		return all
	for event in InputMap.action_get_events(action):
		var press := event as InputEventKey
		if press == null:
			continue
		# A run with no window (--headless) has no keyboard layout to ask: the key's own name.
		var label := KEY_NONE
		if DisplayServer.get_name() != "headless":
			label = DisplayServer.keyboard_get_label_from_physical(press.physical_keycode)
		var called := OS.get_keycode_string(label if label != KEY_NONE else press.physical_keycode)
		called = SHORT.get(called, called)
		if press.shift_pressed:
			called = "Shift+" + called
		if press.ctrl_pressed:
			called = "Ctrl+" + called
		all.append(called)
	return all


## The first key `action` is on ("E"), or "?".
static func key(action: String) -> String:
	var all := keys(action)
	return all[0] if not all.is_empty() else "?"


## `text` with each "{action}" in it as that action's key cap ("[E]"), and "{action|all}" as every
## key it's on ("[Tab] or [I]"). Without `caps`, the keys alone ("E"), for the keys panel's own
## column. Braces round anything that isn't an action stay as they are.
static func keyed(text: String, caps := true) -> String:
	var out := text
	for found in _braces.search_all(text):
		var all := keys(found.get_string(1))
		if all.is_empty():
			continue
		var shown := PackedStringArray()
		for k in all:
			shown.append("[%s]" % k if caps else k)
		var every := found.get_string(2) != ""
		out = out.replace(found.get_string(), " or ".join(shown) if every else shown[0])
	return out


## The door or window from DuskWorld.door_check(), opened or shut. A boarded one can't be; a
## broken window offers nothing (you climb through it).
static func _door(all: Array, door: PackedInt32Array, place: int) -> void:
	var what := ("Shut " if door[1] == 1 else "Open ") + ("the window" if door[0] == 1 else "the door")
	match door[2]:
		1:
			all.append(_entry("door", what, "", place))
		-2:
			all.append(_entry("door", what, "it's boarded up", place))


static func _entry(verb: String, what: String, why := "", place := LIKELY) -> Array:
	return [verb, what, keyed(why), place]


## An entry in words: "Burn the body", or with why it can't be done, "Burn the body: you need oil".
static func _words(entry: Array) -> String:
	return entry[1] if entry[2] == "" else "%s: %s" % [entry[1], entry[2]]


## The kind of the actor at `index` (in the actor arrays), as a name: "deer".
static func _creature(index: int, creatures: PackedStringArray, kinds: PackedInt32Array) -> String:
	var n := kinds[index] - 1 if index < kinds.size() else -1
	return creatures[n] if n >= 0 and n < creatures.size() else "carcass"


## Fixture number `f` as words: "shop_shelf" is "shop shelf".
static func _fixture(f: int, fixtures: PackedStringArray) -> String:
	return fixtures[f].replace("_", " ") if f >= 0 and f < fixtures.size() else "thing"
