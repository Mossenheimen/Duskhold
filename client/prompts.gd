extends RefCounted
## Zone F's prompts (VISUALS.md §10, §11): what you can do beside you, worded from the core's own
## checks (DuskWorld's *_check, the ones the keys' handlers in main.gd ask), never a rule of its
## own. Each is a key cap, a verb and a thing: "[G] Butcher the deer". One that can't be done
## says why; the ones that can come first, and at most MOST show.
##
## Only what's easily missed is prompted: a carcass or a body, a plant, an altar, a fire, and
## water when you're thirsty. Doors and boards, everywhere in a town, are left to the keys (F1).

const MOST := 2


## The prompts beside you now, likeliest first, each [its line, whether it can be done now].
## `creatures`, `fixtures`, `kinds`: DuskWorld.creature_names(), fixture_names() and
## actor_kinds(); `thirsty`: whether you'd want a drink (the moodle).
static func here(dusk: Node, creatures: PackedStringArray, fixtures: PackedStringArray,
		kinds: PackedInt32Array, thirsty: bool) -> Array:
	var all := []
	# G cuts up a beast's carcass before it burns any body (main.gd's _burn_body), so its prompt
	# does too.
	var carcass: int = dusk.butcher_check()
	if carcass >= 0:
		all.append(_line("burn", "Butcher the " + _creature(carcass, creatures, kinds)))
	elif carcass == -2:
		all.append(_line("burn", "Butcher the carcass", "you need a knife or an axe"))
	else:
		match dusk.burn_check():
			-1:
				pass
			-2:
				all.append(_line("burn", "Burn the body", "you need oil"))
			-3:
				all.append(_line("burn", "Burn the body", "hold up a light to set it alight (L)"))
			_:
				all.append(_line("burn", "Burn the body"))
	# O consecrates the body beside you; with none, a Cleric with Warding wards the shut door.
	var body: int = dusk.consecrate_check()
	if body >= 0:
		all.append(_line("hallow", "Consecrate the body"))
	elif body == -2:
		all.append(_line("hallow", "Consecrate the body", "you need holy water"))
	else:
		match dusk.ward_check():
			1:
				all.append(_line("hallow", "Ward the door"))
			-3:
				all.append(_line("hallow", "Ward the door", "not enough faith: pray at an altar"))
	var plant: int = dusk.pick_check()
	if plant > 0:
		all.append(_line("pick", "Pick the " + _fixture(plant, fixtures)))
	if dusk.pray_check() == 1:
		all.append(_line("pray", "Pray at the altar"))
	# H: a hearth, stove or forge beside you. Alight with all it can hold, there's nothing to do.
	var fire: PackedInt32Array = dusk.fire_beside()
	if fire.size() >= 5:
		var it := "the " + _fixture(fire[4], fixtures)
		var alight := fire[3] == 1
		match dusk.fire_check():
			1:
				all.append(_line("tend_fire", ("Feed " if alight else "Lay a fire in ") + it))
			2:
				all.append(_line("tend_fire", "Light " + it))
			-3:
				if not alight:
					all.append(_line("tend_fire", "Lay a fire in " + it, "you have nothing that burns"))
			-4:
				all.append(_line("tend_fire", "Light " + it, "hold up a lit torch or candle (L)"))
	if thirsty and dusk.water_in_reach():
		all.append(_line("drink", "Drink your fill"))
	# The ones you can do first; within each, the order above.
	var shown := all.filter(func(p: Array) -> bool: return p[1])
	shown.append_array(all.filter(func(p: Array) -> bool: return not p[1]))
	return shown.slice(0, MOST)


## The key `action` is on, as the keyboard shows it ("G"): read from the InputMap, so a remapped
## key is said wherever it's shown (VISUALS.md §11, key caps), and on another layout its label.
static func key(action: String) -> String:
	for event in InputMap.action_get_events(action):
		var press := event as InputEventKey
		if press != null:
			# A run with no window (--headless) has no keyboard layout to ask: the key's own name.
			var label := KEY_NONE
			if DisplayServer.get_name() != "headless":
				label = DisplayServer.keyboard_get_label_from_physical(press.physical_keycode)
			return OS.get_keycode_string(label if label != KEY_NONE else press.physical_keycode)
	return "?"


## A prompt: "[G] Burn the body", or with `why` it can't be done, "[G] Burn the body: you need oil".
static func _line(action: String, what: String, why := "") -> Array:
	var line := "[%s] %s" % [key(action), what]
	return [line if why == "" else "%s: %s" % [line, why], why == ""]


## The kind of the actor at `index` (in the actor arrays), as a name: "deer".
static func _creature(index: int, creatures: PackedStringArray, kinds: PackedInt32Array) -> String:
	var n := kinds[index] - 1 if index < kinds.size() else -1
	return creatures[n] if n >= 0 and n < creatures.size() else "carcass"


## Fixture number `f` as words: "shop_shelf" is "shop shelf".
static func _fixture(f: int, fixtures: PackedStringArray) -> String:
	return fixtures[f].replace("_", " ") if f >= 0 and f < fixtures.size() else "thing"
