extends Node3D
## Duskhold's client. The Rust core runs the game inside the DuskWorld node (the dusk-godot
## extension). This draws what it says and turns the player's input into intents: it decides
## nothing.
##
## The core's tile (x, y) is the 1 × 1 square from (x, 0, y) in 3D: x east, y south, Y up.
##
## Development flags, after "--" on Godot's command line (play.cmd passes them on):
## --seed=N the patch's seed; --town the little demo town instead; --walk=X,Y walk there at the
## start; --zoom=N; --speed=N the speed dial; --data=PATH another data folder (a mod, a test);
## --shot=PATH save a picture after --shot-after=SECONDS (default 3) and quit;
## --do=S:ACTION,... press swing, shove, door, struggle, drop (the first thing in the pack),
## logs (take what's kept by the fire beside you), fire (tend it), smother, sleep, drink (from
## water), light, skills (the panel), board or unboard S real seconds in;
## --news print the news too (for --headless); --hour=H start at that hour of the day;
## --save=PATH carry on from that save if it's there, and save to it.
##
## With no flags at all, the game is saved (user://world.dusk) and carries on from its save. Any
## flag makes it a trial run, which neither loads nor saves unless --save says where: so pictures
## and tests never touch the player's world.

const MapView = preload("res://map_view.gd")
const ActorView = preload("res://actor_view.gd")
const NoiseRings = preload("res://noise_rings.gd")
const Hud = preload("res://hud.gd")
const Tells = preload("res://tells.gd")
const Threats = preload("res://threats.gd")
const Bag = preload("res://bag.gd")
const Litter = preload("res://litter.gd")
const Skills = preload("res://skills.gd")
const ChunkView = preload("res://chunk_view.gd")

## Zomboid's angle: 30° down, so tiles are 2:1 diamonds, and north up and to the right.
const CAMERA_PITCH := 30.0
const CAMERA_YAW := 45.0
const CAMERA_DISTANCE := 40.0
const ZOOM_MIN := 6.0
const ZOOM_MAX := 40.0
## The speed dial: keys 1, 2 and 3.
const SPEEDS: Array[int] = [1, 2, 5]

## Event kinds, from DuskWorld.take_events().
const HEARD := 1
const SAW := 2
const REACHED := 3
const NO_WAY := 4
const STRUCK := 5
const MISSED := 6
const SHOVED := 7
const DIED := 8
const DOOR := 9
const EXHAUSTED := 10
const GRABBED := 11
const FREED := 12
const STRUGGLED := 13
const THUMPED := 14
const DOOR_BROKE := 15
const DRAGGED_DOWN := 16
const TOOK := 17
const DROPPED := 18
const WIELDED := 19
const WEAPON_BROKE := 20
const BANDAGED := 21
const ATE := 22
const DRANK := 23
const DRANK_WATER := 24
const FELL_ASLEEP := 25
const WOKE := 26
const LIT := 27
const DOUSED := 28
const BURNT_OUT := 29
const NIGHTFALL := 30
const DAYBREAK := 31
const SKILL_UP := 32
const SKILL_AT_CAP := 33
const MASTERED := 34
const BOARDED := 35
const UNBOARDED := 36
const BOARD_BROKE := 37
const MADE_HOME := 38
const STASHED := 39
const PUT := 40
const BROUGHT_HOME := 41
const LEVEL_UP := 42
const SMELLED := 43
const FEVERED := 44
const FEVER_WORSE := 45
const FEVER_TOOK := 46
const CURED := 47
const GOT_UP := 48
const KINDLED := 49
const CAUGHT := 50
const BURNT := 51
const BOUGHT := 52
const SOLD := 53
const SUPPLIED := 54
const MENDED := 55
const TOLLING_TONIGHT := 56
const TOLLING_BEGAN := 57
const TOLLED := 58
const TOLLING_ENDED := 59
const RISEN := 60
const CAME := 61
const LIGHT_EATEN := 62
const BOUNTY_CLAIMED := 63
const IDENTIFIED := 64
const CURSE_CLINGS := 65
const CURSE_LIFTED := 66
const RAIN_BEGAN := 67
const RAIN_ENDED := 68
const RAINED_OUT := 69
const TALENT_CHOSEN := 70
const SECOND_WIND := 71
const LAID := 72
const FIRE_LIT := 73
const SMOTHERED := 74
const FIRE_OUT := 75
const SAW_FIRE := 76
const DREAD := 77
const BALKED := 78
## Rain streaks falling round the camera, by DuskWorld.rain() (dry, rain, heavy).
const RAIN_DROPS := [0, 700, 1800]
## What your own timed actions are called, by DuskWorld.actor_actions() kind (5 on).
const DOING := {5: "Struggling", 6: "Taking", 7: "Dropping", 8: "Bandaging", 9: "Taking it in hand",
	10: "Putting it away", 11: "Lying down", 12: "Drinking", 13: "Eating", 14: "Drinking",
	15: "Lighting it", 16: "Putting it out", 17: "Boarding it up", 18: "Prying a board off",
	19: "Making it your stash", 20: "Putting it away", 21: "Taking the cure",
	22: "Setting it alight", 23: "Buying", 24: "Selling", 25: "Having it mended",
	26: "Claiming the bounty", 27: "Having it read", 28: "Having the curse lifted",
	29: "Catching your breath", 30: "Laying the fire", 31: "Lighting the fire",
	32: "Putting the fire out"}
## A body that rose as a revenant (DuskWorld.actor_remains()): nothing's left to draw.
const RISEN_BODY := 3
## A light, by DuskWorld.item_kinds().
const LIGHT_KIND := 5
## Moodles (DuskWorld.character_moodles()) the client acts on.
const HUNGRY := 0
const THIRSTY := 1
const ASLEEP := 8
const FEVER := 9
## Asleep, the game runs this many times as fast (an hour of sleep in a few seconds).
const SLEEP_SPEED := 60.0
## Seconds played before the news tells of the same level of dread again.
const DREAD_TOLD_EVERY := 30.0
## Why you woke, by WOKE's extra.
const WOKE_BY := ["You woke, rested.", "You got up.", "You woke to pain!", "A noise woke you."]
## Numbers an event takes.
const EVENT_SIZE := 6

var dusk: Node  # the DuskWorld
var map_view: MapView
var rings: NoiseRings
var tells: Tells
var threats: Threats
var bag: Bag
var litter: Litter
var skills: Skills
var item_names := PackedStringArray()
var rares := PackedInt32Array()  # 1 for each actor that's a rare (DuskWorld.actor_rare())
var told_get_up := false  # the first body you see fall says it'll get up
var hallowed := false  # on hallowed ground, as last told
var creature_names := PackedStringArray()  # by creature number: actor kind 1 + n is creature n
var kinds := PackedInt32Array()  # each actor's kind (DuskWorld.actor_kinds())
var fever_took := false  # the fever killed you, for the death screen
var skill_names := PackedStringArray()
var hud: Hud
var camera: Camera3D
var cursor: MeshInstance3D
var sun: DirectionalLight3D
var sky: Environment
var torch: OmniLight3D  # the light you hold up, drawn as a real light with shadows
var fires := {}  # the fires alight in hearths, stoves and forges: Vector2i -> OmniLight3D
var fixture_names := PackedStringArray()  # by fixture number (DuskWorld.fixture_names())
var fire_seen_at := -100.0  # when the news last told of the dead seeing a fire, in seconds played
var dread_was := 0  # your dread's level, as the core last said
var dread_told := {}  # when the news last told each level of dread, in seconds played
var balked_at := -100.0  # when the news last said you balked, in seconds played
var seen := PackedByteArray()  # per actor, whether you see it (DuskWorld.actor_seen())
var actors: Array = []  # an ActorView per actor, in the core's order
var me := -1  # the character's index in the actor arrays
var town_seed := 1
var speed_index := 0
var zoom := 14.0
var focus := Vector3.ZERO  # where the camera looks: the character, smoothed
var rain_fall: GPUParticles3D  # the rain, drawn round where the camera looks
var wound_names := PackedStringArray()
var part_names := PackedStringArray()
var last_hurt := ""  # what last hurt you, for the death screen
var last_hurt_at := -100  # the tick it did
var moodles := PackedInt32Array()  # yours, as of the last frame
var sleeping := false  # asleep, and the clock running fast
var wake_speed := 1.0  # the speed to go back to on waking
var dead := false
var held := 0  # how many have hold of you
var dragged_by := 0  # how many dragged you down, for the death screen
var last_thump := -100.0  # when the news last told of a blow on a door, in game seconds
var low_walls := false
var last_tick := -1
var use_town := false

var shot_path := ""
var shot_after := 3.0
var start_speed := 1.0
var print_news := false  # --news
var start_hour := -1  # --hour: -1 for the data's
var data_dir := ""  # --data: another data folder (a mod, or a test)
var walk_at := Vector2i(-1, -1)
var played := 0.0
var scripted: Array = []  # --do: [real seconds, action], soonest first
var save_path := ""  # where the game is saved; "" for a trial run, which isn't
var saved_hour := -1  # the game hour of the last autosave
var restart_asked := -100.0  # when R last asked "abandon this world?", in seconds played


func _ready() -> void:
	if OS.get_cmdline_user_args().is_empty():
		save_path = ProjectSettings.globalize_path("user://world.dusk")
	_read_args()
	_add_input_actions()
	_build_stage()
	hud = Hud.new()
	hud.echo = print_news
	add_child(hud)
	if not ClassDB.class_exists("DuskWorld"):
		hud.show_problem("The game's core (the Rust extension) isn't loaded.\n"
			+ "Start the game with play.cmd in 12_Duskhold: it builds the extension first.")
		return
	dusk = ClassDB.instantiate("DuskWorld")
	dusk.process_priority = -1  # tick the world before this script draws it
	dusk.data_dir = data_dir
	dusk.start_hour = start_hour
	add_child(dusk)
	map_view = MapView.new()
	add_child(map_view)
	rings = NoiseRings.new()
	add_child(rings)
	tells = Tells.new()
	add_child(tells)
	threats = Threats.new()
	add_child(threats)
	bag = Bag.new()
	add_child(bag)
	bag.say = hud.say
	bag.burn = _burn_body
	litter = Litter.new()
	add_child(litter)
	skills = Skills.new()
	add_child(skills)
	_start(true)


## Makes the world: the saved one if `carry_on` and there is one, else afresh, a patch of the
## Hushlands from the seed (or the demo town). The same seed and the same input play the same
## game.
func _start(carry_on := false) -> void:
	var loaded := false
	var not_loaded := ""
	if carry_on and save_path != "" and FileAccess.file_exists(save_path):
		not_loaded = dusk.load_game(save_path)
		loaded = not_loaded.is_empty()
		if loaded:
			town_seed = dusk.world_seed()
	if loaded:
		pass
	elif use_town:
		dusk.load_town(town_seed)
	else:
		var problem: String = dusk.load_patch(town_seed)
		if not problem.is_empty():
			hud.show_problem("The game's data has a problem:\n" + problem)
			return
	map_view.build(dusk, low_walls)
	for view in actors:
		view.queue_free()
	actors.clear()
	kinds = dusk.actor_kinds()
	creature_names = dusk.creature_names()
	fixture_names = dusk.fixture_names()
	fever_took = false
	told_get_up = false
	hallowed = false  # so the first frame says where you are
	for kind in kinds:
		var view := ActorView.new()
		view.setup(kind)
		add_child(view)
		actors.append(view)
	rares = PackedInt32Array()
	me = dusk.character_index()
	rings.clear()
	hud.reset()
	wound_names = dusk.wound_names()
	item_names = dusk.item_names()
	bag.start(dusk)
	skill_names = dusk.skill_names()
	skills.start(dusk)
	part_names = dusk.part_names()
	last_hurt = ""
	last_hurt_at = -100
	moodles = PackedInt32Array()
	sleeping = false
	dead = false
	held = 0
	dragged_by = 0
	last_thump = -100.0
	last_tick = -1
	_draw_actors(0.0)
	if me >= 0:
		focus = actors[me].position
	if walk_at.x >= 0:
		dusk.walk_to(walk_at.x, walk_at.y, false)
	dusk.speed = start_speed
	var clock: PackedInt32Array = dusk.clock()
	saved_hour = clock[1]
	if loaded:
		dead = dusk.character_body()[3] == 1
		if dead:
			_show_recap("You died here.")
		else:
			hud.say("You carry on.  %s" % Hud.time_of_day(clock))
	elif not not_loaded.is_empty():
		# Moved aside, not saved over: a save from an older build may load again once migrations
		# come, and a broken one is worth a look.
		var kept := save_path + ".old"
		if FileAccess.file_exists(kept):
			DirAccess.remove_absolute(kept)
		var moved := DirAccess.rename_absolute(save_path, kept) == OK
		hud.say("The saved game couldn't be loaded: %s. A new world, then%s." % [not_loaded,
			" (the old save is kept beside it, as %s)" % kept.get_file() if moved else ""])
	else:
		_save("")  # a new world takes the old one's place at once


func _process(delta: float) -> void:
	if dusk == null:
		return
	_steer()
	_draw_actors(delta)
	for noise in dusk.take_noises():
		rings.add(noise.x, noise.y, noise.z)
	rings.advance(0.0 if dusk.paused else delta * dusk.speed)
	_read_events()
	var tick: int = dusk.tick()
	if tick != last_tick:
		last_tick = tick
		map_view.show_watched(dusk.watched())
		map_view.show_view(dusk.view_tiles())
	_light_the_sky(dusk.daylight())
	_hold_up_light()
	_light_the_fires()
	_let_it_rain()
	_move_camera(delta)
	map_view.update(focus)
	_move_cursor()
	var clock: PackedInt32Array = dusk.clock()
	if not dead and clock[1] != saved_hour:
		saved_hour = clock[1]
		_save("")  # on the game hour
	moodles = dusk.character_moodles()
	var asleep := _moodle(ASLEEP) > 0
	_sleep_speed(asleep)
	hud.show_state(town_seed, clock, roundi(dusk.speed), dusk.paused, dusk.pressure(), dusk.tolling(), dusk.rain(), dusk.place())
	hud.show_moodles(moodles)
	hud.show_dread(0 if dead else dusk.character_dread(), delta)
	hud.show_asleep(asleep, clock)
	var body: PackedInt32Array = dusk.character_body()
	held = body[5] if body.size() > 5 else 0
	hud.show_body(body, dusk.character_wounds(), dusk.can_shove())
	hud.show_deeds(dusk.character_deeds())
	_mind_the_ground()
	var doing := _my_action()
	bag.refresh(doing.x != 0, dead)
	skills.refresh(dead)
	litter.show_floors(dusk.floor_items())
	hud.show_action(DOING.get(doing.x, ""), doing.y / 1000.0)
	played += delta
	_run_script()
	_maybe_take_picture()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("quit"):
		_save("")
		get_tree().quit()
	if dusk == null:
		return
	var mouse := event as InputEventMouseButton
	if mouse and mouse.pressed:
		match mouse.button_index:
			MOUSE_BUTTON_LEFT:
				if dead:
					return
				var tile := _tile_under(mouse.position)
				var aim := _foe_beside(tile)
				if aim != Vector2i.ZERO:
					if held > 0:
						dusk.struggle()
					else:
						dusk.attack(aim.x, aim.y)
				else:
					dusk.walk_to(tile.x, tile.y, Input.is_action_pressed("run"))
			MOUSE_BUTTON_WHEEL_UP:
				zoom = maxf(ZOOM_MIN, zoom / 1.15)
			MOUSE_BUTTON_WHEEL_DOWN:
				zoom = minf(ZOOM_MAX, zoom * 1.15)
	elif event.is_action_pressed("swing") and not dead:
		# Held, the swing key struggles: you can't swing with one on you.
		if held > 0:
			dusk.struggle()
		else:
			dusk.attack(0, 0)
	elif event.is_action_pressed("shove") and not dead:
		if not dusk.can_shove():
			hud.say("No hand free to shove with: struggle (F).")
		dusk.shove(0, 0)
	elif event.is_action_pressed("door") and not dead:
		var pry: int = dusk.board_check(true)
		if pry > 0 or pry == -4:
			hud.say("It's boarded up. N pries a board off (with a hammer).")
		dusk.use_door()
	elif event.is_action_pressed("board") and not dead:
		_board_up()
	elif event.is_action_pressed("burn") and not dead:
		_burn_body()
	elif event.is_action_pressed("second_wind") and not dead:
		if dusk.can_second_wind():
			dusk.second_wind()
		else:
			hud.say("No second wind: it takes the talent, and comes back when you've slept.")
	elif event.is_action_pressed("unboard") and not dead:
		_pry_board()
	elif event.is_action_pressed("tend_fire") and not dead:
		_tend_fire()
	elif event.is_action_pressed("smother_fire") and not dead:
		_smother_fire()
	elif event.is_action_pressed("sleep") and not dead:
		_sleep_or_wake()
	elif event.is_action_pressed("drink") and not dead:
		if dusk.water_in_reach():
			dusk.drink_water()
		else:
			hud.say("There's no water within reach.")
	elif event.is_action_pressed("light") and not dead:
		_light_or_douse()
	elif event.is_action_pressed("pause"):
		dusk.paused = not dusk.paused
	elif event.is_action_pressed("speed_1"):
		_set_speed(0)
	elif event.is_action_pressed("speed_2"):
		_set_speed(1)
	elif event.is_action_pressed("speed_3"):
		_set_speed(2)
	elif event.is_action_pressed("pack"):
		bag.toggle()
	elif event.is_action_pressed("skills"):
		skills.toggle()
	elif event.is_action_pressed("low_walls"):
		low_walls = not low_walls
		map_view.set_low_walls(low_walls)
	elif event.is_action_pressed("save"):
		if save_path == "":
			hud.say("This is a trial run: it isn't saved.")
		else:
			_save("Saved.")
	elif event.is_action_pressed("restart") and dead and not Input.is_key_pressed(KEY_SHIFT):
		# Dead, R brings the next survivor into this same world.
		_next_survivor()
	elif event.is_action_pressed("restart"):
		# Alive in a saved world, R asks twice: the world you leave is gone for good. Dead,
		# Shift+R leaves it at once.
		if save_path != "" and not dead and played - restart_asked > 4.0:
			restart_asked = played
			hud.say("Abandon this world for a new one? Press R again.")
		else:
			town_seed += 1
			_start()


## Saves the game, if this run is saved, and says `said` if that's not "". The core makes the
## save; the extension writes the file.
func _save(said: String) -> void:
	if save_path == "" or dusk == null:
		return
	var problem: String = dusk.save_game(save_path)
	if not problem.is_empty():
		hud.say("The game couldn't be saved: " + problem)
	elif said != "":
		hud.say(said)


## Closing the window saves too.
func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		_save("")


## Which way to swing for a click on `tile`: toward it if one of the living dead stands there
## beside you, else (0, 0), meaning "not a swing". Whether the blow can land is the core's call.
func _foe_beside(tile: Vector2i) -> Vector2i:
	if me < 0:
		return Vector2i.ZERO
	var where: PackedVector2Array = dusk.actor_positions()
	var kinds: PackedInt32Array = dusk.actor_kinds()
	var states: PackedInt32Array = dusk.actor_states()
	var mine := Vector2i(roundi(where[me].x), roundi(where[me].y))
	var aim := tile - mine
	if aim == Vector2i.ZERO or absi(aim.x) > 1 or absi(aim.y) > 1:
		return Vector2i.ZERO
	for i in kinds.size():
		if kinds[i] >= 1 and states[i] != 2 and Vector2i(roundi(where[i].x), roundi(where[i].y)) == tile:
			return aim
	return Vector2i.ZERO


## WASD, as a heading on screen. The core turns it into a heading on the map.
func _steer() -> void:
	if dead:
		return
	var right := int(Input.is_action_pressed("walk_right")) - int(Input.is_action_pressed("walk_left"))
	var up := int(Input.is_action_pressed("walk_up")) - int(Input.is_action_pressed("walk_down"))
	dusk.steer(right, up, Input.is_action_pressed("run"))


func _set_speed(index: int) -> void:
	speed_index = index
	if sleeping:
		wake_speed = float(SPEEDS[index])  # for when you wake
	else:
		dusk.speed = float(SPEEDS[index])
	dusk.paused = false


## Z: lie down to sleep, or get up. The core says whether you can; this says why not.
func _sleep_or_wake() -> void:
	if _moodle(ASLEEP) > 0:
		dusk.wake()
		return
	match dusk.sleep_check():
		-1:
			hud.say("You're not tired enough to sleep.")
		-2:
			hud.say("You can't sleep with the dead after you.")
		-3:
			hud.say("Not with it holding you!")
		-4:
			hud.say("You're in too much pain to sleep. Dress your wounds first: a bandage (Tab).")
		_:
			dusk.sleep()


## B: nail a board across the door or window beside you. The core says whether you can; this
## says why not.
func _board_up() -> void:
	match dusk.board_check(false):
		-1:
			hud.say("There's no door or window beside you to board up.")
		-2:
			hud.say("Shut the door first.")
		-3:
			hud.say("It won't take another board.")
		-4:
			hud.say("You need a hammer.")
		-5:
			hud.say("You need a plank.")
		-6:
			hud.say("You need nails: two a board.")
		_:
			dusk.board()


## Stepping onto hallowed ground, or off it: the news says so, once each way.
func _mind_the_ground() -> void:
	if dead:
		return
	var now: bool = dusk.on_hallowed()
	if now == hallowed:
		return
	hallowed = now
	if now:
		hud.say("Hallowed ground. The dead won't come in, nor heed you here.")
	else:
		hud.say("You leave hallowed ground.")


## G: pour oil on the body beside you and set it alight from the light you hold up.
func _burn_body() -> void:
	match dusk.burn_check():
		-1:
			hud.say("There's no body beside you to burn.")
		-2:
			hud.say("You need oil to burn it.")
		-3:
			hud.say("You need a flame: hold up a lit torch or candle (L).")
		_:
			dusk.burn()


## H: tend the hearth, stove or forge beside you: lay a piece of fuel on, or once it's laid,
## light it from the light you hold up. The core says which, or why not.
func _tend_fire() -> void:
	match dusk.fire_check():
		-1:
			hud.say("There's no hearth, stove or forge beside you.")
		-2:
			hud.say("It won't take more just now: let it burn down a while.")
		-3:
			hud.say("You have nothing that burns: split logs (a hearth or stove keeps a few by it) or planks.")
		-4:
			hud.say("The fire's laid. Hold up a lit torch or candle (L) to light it.")
		_:
			dusk.tend_fire()


## J: put out the fire beside you. What's left of the wood stays in it for the next time.
func _smother_fire() -> void:
	var f: PackedInt32Array = dusk.fire_beside()
	if f.size() >= 4 and f[3] == 1:
		dusk.smother_fire()
	else:
		hud.say("There's no fire alight beside you.")


## --do's "logs": takes what's kept by the hearth, stove or forge beside you (its first stack).
func _take_logs() -> void:
	var f: PackedInt32Array = dusk.fire_beside()
	if f.size() < 2:
		return
	var reach: PackedInt32Array = dusk.stores_in_reach()
	var i := 0
	while i + 4 < reach.size():
		var count := reach[i + 3]
		if reach[i] == f[0] and reach[i + 1] == f[1] and count > 0:
			dusk.take(f[0], f[1], 0, reach[i + 5])
			return
		i += 5 + count * 4


## Whether the tile (x, y) is within `tiles` of you.
func _near(x: int, y: int, tiles: float) -> bool:
	if me < 0:
		return false
	var where: PackedVector2Array = dusk.actor_positions()
	return me < where.size() and (Vector2(x, y) - where[me]).length() <= tiles


## N: pry a board off the boarded door or window beside you.
func _pry_board() -> void:
	match dusk.board_check(true):
		-1:
			hud.say("There's nothing boarded up beside you.")
		-4:
			hud.say("You need a hammer to pry it off.")
		_:
			dusk.unboard()


## Asleep, time runs at SLEEP_SPEED; awake again, at the speed it was.
func _sleep_speed(asleep: bool) -> void:
	if asleep and not sleeping:
		sleeping = true
		wake_speed = dusk.speed
		dusk.speed = SLEEP_SPEED
		dusk.paused = false
	elif not asleep and sleeping:
		sleeping = false
		dusk.speed = wake_speed


## A moodle's level now (0 if you haven't got it).
func _moodle(code: int) -> int:
	for i in range(0, moodles.size() - 1, 2):
		if moodles[i] == code:
			return moodles[i + 1]
	return 0


## Rares get their name over their heads (DuskWorld.actor_rare(), actor_name()).
func _mark_rares() -> void:
	rares = dusk.actor_rare()
	for i in mini(rares.size(), actors.size()):
		if rares[i] == 1:
			actors[i].set_title(dusk.actor_name(i))


## The dead who walked into the patch since the last frame (pressure, in the core): a view each.
## They come in at the end of the core's order, so the others keep their places.
func _add_new_actors() -> void:
	var now: PackedInt32Array = dusk.actor_kinds()
	for i in range(actors.size(), now.size()):
		var view := ActorView.new()
		view.setup(now[i])
		add_child(view)
		actors.append(view)
	kinds = now


func _draw_actors(delta: float) -> void:
	var where: PackedVector2Array = dusk.actor_positions()
	var facing: PackedInt32Array = dusk.actor_facing()
	var alert: PackedInt32Array = dusk.actor_alert()
	_add_new_actors()
	var states: PackedInt32Array = dusk.actor_states()
	if rares.size() != actors.size():
		_mark_rares()
	var remains: PackedInt32Array = dusk.actor_remains()
	var actions: PackedInt32Array = dusk.actor_actions()
	seen = dusk.actor_seen()
	for i in actors.size():
		# What you don't see isn't drawn (CONCEPT.md: never a live monster you can't see).
		actors[i].visible = _seen(i) and remains[i] != RISEN_BODY
		actors[i].position = Vector3(where[i].x + 0.5, 0.0, where[i].y + 0.5)
		actors[i].face(facing[i], delta)
		actors[i].show_alert(alert[i], delta)
		# Leaning in while a swing winds up: the tell, on the body as well as the ground.
		var swinging := actions[i * 4] == 1 and actions[i * 4 + 1] < 1000
		actors[i].show_pose(states[i], actions[i * 4 + 1] / 1000.0 if swinging else -1.0, delta)
		actors[i].show_remains(remains[i], delta)
	tells.show_actions(actions, me, seen)
	var drawn := []
	for view in actors:
		drawn.append(view.position)
	threats.show_threats(dusk.character_threats(), drawn, me)
	if me >= 0:
		map_view.set_focus(actors[me].position + Vector3(0.0, 0.9, 0.0))


func _read_events() -> void:
	var events: PackedInt32Array = dusk.take_events()
	for i in range(0, events.size(), EVENT_SIZE):
		var who := events[i + 1]
		var target := events[i + 2]
		var extra := events[i + 5]
		match events[i]:
			HEARD:
				if _seen(who):
					hud.say("%s heard something." % _a(who))
			SAW:
				hud.say("%s saw you!" % _a(who) if _seen(who) else "Something has seen you.")
			SMELLED:
				if target == me:
					if _seen(who):
						hud.say("%s has the scent of your blood! Bandage it, or it won't lose you." % _a(who))
					else:
						hud.say("Something has the scent of your blood. Bandage it: Tab.")
			NO_WAY:
				if who == me:
					hud.say("There's no way there.")
			STRUCK:
				if target == me and extra > 0:
					var wound := wound_names[floori(extra / 100.0) - 1]
					var part := part_names[extra % 100]
					last_hurt = "%s gave you a %s on the %s." % [_a(who), wound, part]
					last_hurt_at = dusk.tick()
					hud.say(last_hurt)
					hud.flash()
				elif who == me:
					hud.say("You hit it.")
			MISSED:
				if who == me:
					hud.say("You missed.")
				elif target == me or _beside(who):
					hud.say("%s reached for you and missed." % _a(who))
			SHOVED:
				if who == me:
					hud.say("You shoved it down." if extra == 1 else "You shoved it back.")
			DIED:
				if who == me:
					_die()
				else:
					hud.say("It's down for good.")
			DOOR:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You opened the door." if extra == 1 else "You shut the door.")
			EXHAUSTED:
				if who == me:
					hud.say("You're out of breath.")
			GRABBED:
				if target == me:
					hud.say("It has hold of you! F to struggle, Q to shove it off.")
					hud.flash()
			FREED:
				if who == me:
					hud.say("You got free.")
			STRUGGLED:
				if who == me:
					hud.say("You couldn't get free.")
			THUMPED:
				# Doors are loud: tell of it now and then, not at every blow.
				var now: float = dusk.tick() / 10.0
				if now - last_thump > 6.0:
					last_thump = now
					hud.say("Something is beating on a door.")
			DOOR_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("A door gave way!")
			DRAGGED_DOWN:
				if who == me:
					dragged_by = extra
			TOOK:
				if who == me:
					hud.say("You took %s." % _items(extra))
			DROPPED:
				if who == me:
					hud.say("You dropped %s." % _items(extra))
			WIELDED:
				if who == me:
					hud.say("You took the %s in hand." % _item_name(extra))
			WEAPON_BROKE:
				if who == me:
					hud.say("Your %s broke!" % _item_name(extra))
			BANDAGED:
				if who == me:
					hud.say("You bandaged your %s." % part_names[extra])
			ATE:
				if who == me:
					hud.say("You ate the %s." % _item_name(extra))
			DRANK:
				if who == me:
					hud.say("You drank the %s." % _item_name(extra))
			DRANK_WATER:
				if who == me:
					hud.say("You drank your fill.")
			FELL_ASLEEP:
				if who == me:
					hud.say("You fell asleep in the bed." if extra == 1 else "You fell asleep on the ground.")
			WOKE:
				if who == me and extra >= 0 and extra < WOKE_BY.size():
					hud.say(WOKE_BY[extra])
			LIT:
				if who == me:
					hud.say("You lit the %s and held it up." % _item_name(extra))
			DOUSED:
				if who == me:
					hud.say("You put out the %s." % _item_name(extra))
			BURNT_OUT:
				if who == me:
					hud.say("Your %s burnt out." % _item_name(extra))
			NIGHTFALL:
				hud.say("The sun is going down. Night is coming.")
			DAYBREAK:
				hud.say("The sky is lightening. Dawn is coming.")
			SKILL_UP:
				if who == me:
					hud.say("%s rose to level %d.  (K: skills)" % [_skill_name(extra), extra % 100])
			SKILL_AT_CAP:
				if who == me:
					if dusk.masteries_left() > 0:
						hud.say("%s will rise no further unless you master it (K)." % _skill_name(extra))
					else:
						hud.say("%s will rise no further: your masteries are taken." % _skill_name(extra))
			MASTERED:
				if who == me:
					hud.say("You mastered %s: level %d, and on to 10." % [_skill_name(extra), extra % 100])
			BOARDED:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You nailed a board across it: %d on it now." % extra)
			UNBOARDED:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You pried a board off: %d left." % extra)
			MADE_HOME:
				if who == me:
					hud.say("You slept the night through. This is home now: a container under its roof can be your stash.")
			STASHED:
				if who == me:
					hud.say("That's your stash. Finds put into it pay their worth.")
			PUT:
				if who == me:
					hud.say("You put away %s." % _items(extra))
			BROUGHT_HOME:
				# (x, y) carry the item and how many; extra is the XP.
				if who == me:
					hud.say("Brought home: %s, %d XP." % [_items(events[i + 3] * 1000 + events[i + 4]), extra])
			LEVEL_UP:
				if who == me:
					hud.say("You reached level %d. A little more health, and a talent to choose: K." % extra)
					hud.flash_good()
			FEVERED:
				if who == me:
					hud.say("The wound burns. You've caught ghoul fever: three days to find a cure.")
					hud.flash()
			FEVER_WORSE:
				if who == me:
					if extra >= 3:
						hud.say("You're dying of the fever. A day left to find the cure.")
					else:
						hud.say("The fever has you weak: your breath comes back slowly.")
					hud.flash()
			FEVER_TOOK:
				if who == me:
					fever_took = true
			CURED:
				if who == me:
					hud.say("You took the cure. The fever breaks.")
					hud.flash_good()
			DIED:
				# The first of the dead you see go down: what happens next, and what stops it.
				if who != me and _seen(who) and not told_get_up:
					told_get_up = true
					hud.say("It'll get up again in a day or so, unless you burn it: oil and a flame, G.")
			GOT_UP:
				if _seen(who):
					hud.say("%s gets up again!" % _a(who))
			KINDLED:
				if who == me:
					hud.say("You set it alight. It won't get up again.")
			CAUGHT:
				if _seen(who):
					hud.say("The fire spreads to another body.")
			BOUGHT:
				if who == me:
					hud.say("Bought %s for %d coin." % [_item_name(events[i + 3]), extra])
			SOLD:
				if who == me:
					hud.say("Sold %s for %d coin." % [_items(events[i + 3] * 1000 + events[i + 4]), extra])
			SUPPLIED:
				if who == me:
					hud.say("The sisters were short of that. The dusk falls back a little (%.1f%%)." % (extra / 10.0))
					hud.flash_good()
			MENDED:
				if who == me:
					hud.say("Your %s is mended, good as new." % _item_name(events[i + 3]))
			TOLLING_TONIGHT:
				hud.say("Tonight the old chapel's bell will toll, at 22:00. The dead will come to it, and they won't give up till dawn.")
			TOLLING_BEGAN:
				hud.say("The bell is tolling! Every dead thing for miles is walking to the chapel.")
				hud.flash()
			TOLLING_ENDED:
				hud.say("Dawn. The bell is quiet, and the dead give up as they did.")
				hud.flash_good()
			LIGHT_EATEN:
				if target == me:
					hud.say("%s puts out your light! It wants you in the dark." % dusk.actor_name(who))
					hud.flash()
			TALENT_CHOSEN:
				if who == me:
					var texts: PackedStringArray = dusk.talent_texts()
					if extra * 2 + 1 < texts.size():
						hud.say("Talent: %s. %s." % [texts[extra * 2], texts[extra * 2 + 1].left(1).to_upper() + texts[extra * 2 + 1].substr(1)])
			SECOND_WIND:
				if who == me:
					hud.say("You catch your second wind.")
					hud.flash_good()
			RAIN_BEGAN:
				hud.say("It's raining hard. Lights gutter outdoors; sound and blood don't carry far." if extra == 1 else "It starts to rain: lights gutter outdoors, and the dead hear and smell less.")
			RAIN_ENDED:
				hud.say("The rain stops.")
			RAINED_OUT:
				if who == me:
					hud.say("The rain puts out your %s. Light it under a roof." % _item_name(extra))
			IDENTIFIED:
				if who == me:
					hud.say("You know what it is now: %s." % _known_as(extra))
					hud.flash_good()
			CURSE_CLINGS:
				if who == me:
					hud.say("The %s clings to your hand: it's cursed! The abbey can lift a curse." % _item_name(extra))
					hud.flash()
			CURSE_LIFTED:
				if who == me:
					hud.say("The sisters lift the curse. The %s lets go of your hand." % _item_name(extra))
					hud.flash_good()
			BOUNTY_CLAIMED:
				if who == me:
					hud.say("The sisters pay the bounty on %s: %d coin, and the dusk falls back." % [dusk.actor_name(target), extra])
					hud.flash_good()
			RISEN:
				# Your old body: you'd want to know, seen or not.
				hud.say("Midnight. %s's body rises as a revenant, in the gear it fell in." % dusk.actor_name(who))
			BOARD_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("The last board gave way!" if extra == 0 else "A board gave way! %d left." % extra)
			LAID:
				if who == me:
					var f: PackedInt32Array = dusk.fire_beside()
					if f.size() >= 4 and f[3] == 1:
						hud.say("You put another %s on the fire." % _item_name(extra))
					else:
						hud.say("You lay the fire. Hold up a light (L) and press H to light it.")
			FIRE_LIT:
				if who == me:
					hud.say("The fire catches. The room is lit, and so is anything seen through its windows.")
			SMOTHERED:
				if who == me:
					hud.say("You put the fire out. What's left of the wood stays in it.")
				else:
					hud.say("%s puts out the fire! It wants you in the dark." % dusk.actor_name(who))
					hud.flash()
			FIRE_OUT:
				if _near(events[i + 3], events[i + 4], 16.0):
					hud.say("The fire has burnt out.")
			SAW_FIRE:
				# Once in a while: a crowd seeing it would fill the news.
				if played - fire_seen_at > 20.0:
					fire_seen_at = played
					if _seen(who):
						hud.say("%s has seen the firelight, and comes to it." % _a(who))
					else:
						hud.say("Something out in the dark has seen the firelight.")
			DREAD:
				if who == me:
					_tell_dread(extra)
			BALKED:
				if who == me and played - balked_at > 4.0:
					balked_at = played
					hud.say("You can't make yourself go nearer the dead.")


## Your dread reached a new level (the core's DREAD news): told as it rises, and when it lets go.
## It hovers at a level's edge as the dead come and go from sight, so each level is told at most
## once in DREAD_TOLD_EVERY seconds.
func _tell_dread(level: int) -> void:
	var told: float = dread_told.get(level, -100.0)
	var again := played - told > DREAD_TOLD_EVERY
	if level != dread_was and again:
		dread_told[level] = played
	if level > dread_was and again:
		match level:
			1:
				hud.say("Your nerve slips, and your aim with it. Light, food and rest ease dread.")
			2:
				hud.say("You're afraid, and your hands shake. Get into the light.")
			3:
				hud.say("You're frightened! You can't make yourself go nearer the dead.")
				hud.flash()
	elif level == 0 and again:
		hud.say("Your nerve steadies.")
	elif level < dread_was and dread_was == 3 and again:
		hud.say("The worst of the fear passes.")
	dread_was = level


## "A shambler", "A ghoul", or a rare's own name: what actor `i` is, for the news.
func _a(i: int) -> String:
	if i >= 0 and i < rares.size() and rares[i] == 1:
		return dusk.actor_name(i)
	var n := kinds[i] - 1 if i >= 0 and i < kinds.size() else -1
	if n < 0 or n >= creature_names.size():
		return "Something"
	var name := creature_names[n]
	return ("An " if "aeiou".contains(name.left(1)) else "A ") + name


## Whether actor `i` stands beside you.
func _beside(i: int) -> bool:
	if me < 0 or i < 0:
		return false
	var where: PackedVector2Array = dusk.actor_positions()
	return (where[i] - where[me]).length() < 1.6


## How you died, from what the client last saw: a hold, a blow just now, bleeding, or hunger and
## thirst (the moodles of the frame before).
func _die() -> void:
	dead = true
	dread_was = 0  # the next survivor starts calm
	var how := last_hurt
	var starving := _moodle(HUNGRY) >= 3
	var parched := _moodle(THIRSTY) >= 3
	if fever_took:
		how = "The ghoul fever took you."
	elif dragged_by >= 3:
		how = "%d of them got hold of you, and dragged you down." % dragged_by
	elif dragged_by > 0:
		how = "It had hold of you, and you had no breath left to throw it off."
	elif dusk.tick() - last_hurt_at <= 1:
		pass  # the blow itself
	elif dusk.character_body()[4] > 0:
		how = "You bled to death."
	elif _moodle(FEVER) >= 3:
		how = "The ghoul fever wore you down."
	elif starving and parched:
		how = "Hunger and thirst wore you down."
	elif parched:
		how = "You died of thirst."
	elif starving:
		how = "You starved."
	elif how.is_empty():
		how = "You died."
	_show_recap(how)
	_save("")  # the world keeps your death: there's no going back to before it


## The death screen's recap: who, how, how long, and what they carried (the body still has it).
func _show_recap(how: String) -> void:
	var carried := []
	var hands: PackedInt32Array = dusk.character_hands()
	if hands.size() >= 2:
		carried.append(_item_name(hands[0]))
	var items: PackedInt32Array = dusk.character_items()
	for i in range(0, items.size() - 3, 4):
		var count := items[i + 1]
		carried.append(_item_name(items[i]) + ("" if count == 1 else " ×%d" % count))
	hud.show_death(dusk.actor_name(me), how, dusk.character_lasted(), ", ".join(carried), true)


## After a death: the next survivor comes to the abbey, in this world. The old body stays where
## it fell, with its gear; the home and stash carry over.
func _next_survivor() -> void:
	if not dusk.next_character():
		return
	_add_new_actors()
	me = dusk.character_index()
	dead = false
	fever_took = false
	hallowed = false
	last_hurt = ""
	last_hurt_at = -100
	held = 0
	dragged_by = 0
	moodles = PackedInt32Array()
	sleeping = false
	hud.hide_death()
	bag.start(dusk)
	skills.start(dusk)
	hud.say("%s comes to the abbey. Somewhere out there, the last one's body lies in its gear." % dusk.actor_name(me))
	_save("")


func _move_camera(delta: float) -> void:
	if me >= 0:
		focus = focus.lerp(actors[me].position, 1.0 - exp(-6.0 * delta))
	camera.size = lerpf(camera.size, zoom, 1.0 - exp(-10.0 * delta))
	var pitch := deg_to_rad(CAMERA_PITCH)
	var yaw := deg_to_rad(CAMERA_YAW)
	var back := Vector3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch))
	camera.look_at_from_position(focus + back * CAMERA_DISTANCE, focus, Vector3.UP)


func _move_cursor() -> void:
	var tile := _tile_under(get_viewport().get_mouse_position())
	var size: Vector2i = dusk.map_size()
	cursor.visible = tile.x >= 0 and tile.y >= 0 and tile.x < size.x and tile.y < size.y
	cursor.position = Vector3(tile.x + 0.5, 0.025, tile.y + 0.5)


## The tile under a point on the screen, where the mouse ray meets the ground.
func _tile_under(screen: Vector2) -> Vector2i:
	var from := camera.project_ray_origin(screen)
	var hit: Variant = Plane(Vector3.UP, 0.0).intersects_ray(from, camera.project_ray_normal(screen))
	if hit == null:
		return Vector2i(-1, -1)
	return Vector2i(floori(hit.x), floori(hit.z))


## The camera, the sun, the sky's light and the tile cursor.
func _build_stage() -> void:
	camera = Camera3D.new()
	camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	camera.size = zoom
	camera.near = 0.1
	camera.far = 200.0
	add_child(camera)

	sun = DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55.0, -35.0, 0.0)  # from the south-west: shadows fall up-right
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_ORTHOGONAL
	sun.directional_shadow_max_distance = 90.0
	add_child(sun)

	sky = Environment.new()
	sky.background_mode = Environment.BG_COLOR
	sky.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	sky.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	var world_environment := WorldEnvironment.new()
	world_environment.environment = sky
	add_child(world_environment)
	_light_the_sky(100)

	torch = OmniLight3D.new()
	torch.light_color = Color(1.0, 0.72, 0.42)
	torch.shadow_enabled = true
	torch.omni_attenuation = 1.2
	torch.visible = false
	add_child(torch)

	var square := PlaneMesh.new()
	square.size = Vector2.ONE
	var outline := ShaderMaterial.new()
	outline.shader = preload("res://shaders/cursor.gdshader")
	cursor = MeshInstance3D.new()
	cursor.mesh = square
	cursor.material_override = outline
	cursor.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(cursor)


## The sun, the sky's light and the background follow the core's daylight (DuskWorld.daylight(),
## out of 100): a low blue moon at night, warm near dusk and dawn, white by day. Only the look:
## what you can see is the core's view.
func _light_the_sky(daylight: int) -> void:
	var t := clampf(daylight / 100.0, 0.0, 1.0)
	var warm := clampf(1.0 - absf(t - 0.6) / 0.3, 0.0, 1.0)  # dusk and dawn
	sun.light_energy = lerpf(0.06, 1.1, t * t)
	sun.light_color = Color(0.55, 0.62, 0.95).lerp(Color(1.0, 0.94, 0.84), t).lerp(
		Color(1.0, 0.62, 0.36), warm * 0.55)
	sky.ambient_light_energy = lerpf(0.16, 0.6, t)
	sky.ambient_light_color = Color(0.28, 0.33, 0.55).lerp(Color(0.58, 0.63, 0.72), t)
	sky.background_color = Color(0.01, 0.012, 0.025).lerp(Color(0.07, 0.08, 0.1), t)


## The light you hold up (DuskWorld.character_light()): a real light over your head, reaching
## as far as the core's does, with a little flicker.
func _hold_up_light() -> void:
	var held: PackedInt32Array = dusk.character_light()
	torch.visible = held.size() >= 3 and me >= 0 and not dead
	if not torch.visible:
		return
	torch.omni_range = held[2] + 1.0
	var now := Time.get_ticks_msec() / 1000.0
	torch.light_energy = 3.0 * (1.0 + 0.06 * sin(now * 13.0) + 0.04 * sin(now * 31.0))
	# In the hand on the camera's side, so it lights the body you see, not from inside it.
	torch.position = actors[me].position + Vector3(0.35, 1.8, 0.35)


## The fires alight in hearths, stoves and forges (DuskWorld.hearth_fires()): over each, embers
## glowing on its top and a real light with shadows, reaching as far as the core's, flickering.
## Only the look: what a fire lights, and who sees it, is the core's.
func _light_the_fires() -> void:
	var lit: PackedInt32Array = dusk.hearth_fires()
	var now := Time.get_ticks_msec() / 1000.0
	var alight := {}
	for i in range(0, lit.size() - 4, 5):
		var at := Vector2i(lit[i], lit[i + 1])
		alight[at] = true
		if not fires.has(at):
			fires[at] = _make_fire(at, lit[i + 4])
		var fire: OmniLight3D = fires[at]
		fire.omni_range = lit[i + 2] + 1.0
		var phase := float(at.x * 7 + at.y * 13)
		fire.light_energy = 2.6 * (1.0 + 0.08 * sin(now * 11.0 + phase) + 0.05 * sin(now * 27.0 + phase))
	for at: Vector2i in fires.keys():
		if not alight.has(at):
			fires[at].queue_free()
			fires.erase(at)


## A fire's light over the fixture on `at` (its number `fixture`), and its embers on top.
func _make_fire(at: Vector2i, fixture: int) -> OmniLight3D:
	var id: String = fixture_names[fixture] if fixture < fixture_names.size() else ""
	var height: float = ChunkView.FURNITURE[id][0].y if ChunkView.FURNITURE.has(id) else 1.0
	var fire := OmniLight3D.new()
	fire.light_color = Color(1.0, 0.56, 0.24)
	fire.shadow_enabled = true
	fire.omni_attenuation = 1.3
	# Above the fixture, so its own box doesn't shut the light in.
	fire.position = Vector3(at.x + 0.5, height + 0.45, at.y + 0.5)
	var embers := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.5, 0.12, 0.45)
	embers.mesh = box
	var glow := StandardMaterial3D.new()
	glow.albedo_color = Color(1.0, 0.42, 0.1)
	glow.emission_enabled = true
	glow.emission = Color(1.0, 0.42, 0.1)
	glow.emission_energy_multiplier = 4.0
	embers.material_override = glow
	embers.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	embers.position = Vector3(0.0, -0.39, 0.0)
	fire.add_child(embers)
	add_child(fire)
	return fire


## The rain: thin streaks falling in a box round where the camera looks, as many as the core's
## rain says (DuskWorld.rain()). Only the look: what it does is the core's.
func _let_it_rain() -> void:
	var drops: int = RAIN_DROPS[clampi(dusk.rain(), 0, 2)]
	if rain_fall == null:
		rain_fall = GPUParticles3D.new()
		var fall := ParticleProcessMaterial.new()
		fall.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
		fall.emission_box_extents = Vector3(22.0, 0.5, 22.0)
		fall.direction = Vector3(0.12, -1.0, 0.05)
		fall.spread = 3.0
		fall.initial_velocity_min = 16.0
		fall.initial_velocity_max = 20.0
		fall.gravity = Vector3(0.0, -9.8, 0.0)
		rain_fall.process_material = fall
		var streak := BoxMesh.new()
		streak.size = Vector3(0.02, 0.55, 0.02)
		var wet := StandardMaterial3D.new()
		wet.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		wet.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		wet.albedo_color = Color(0.7, 0.78, 0.9, 0.35)
		streak.material = wet
		rain_fall.draw_pass_1 = streak
		rain_fall.lifetime = 0.9
		rain_fall.visibility_aabb = AABB(Vector3(-25, -20, -25), Vector3(50, 25, 50))
		rain_fall.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(rain_fall)
	rain_fall.emitting = drops > 0
	if drops > 0 and rain_fall.amount != drops:
		rain_fall.amount = drops
	rain_fall.position = focus + Vector3(0.0, 14.0, 0.0)


## L: light the first torch or candle in your pack, or put out the one you hold up.
func _light_or_douse() -> void:
	if dusk.character_light().size() > 0:
		dusk.douse()
		return
	var items: PackedInt32Array = dusk.character_items()
	var kinds: PackedInt32Array = dusk.item_kinds()
	for i in range(0, items.size() - 3, 4):
		if items[i] < kinds.size() and kinds[items[i]] == LIGHT_KIND:
			dusk.light(floori(i / 4.0), items[i])
			return
	hud.say("You have nothing to light.")


## Whether you see actor `i` now.
func _seen(i: int) -> bool:
	return i >= 0 and i < seen.size() and seen[i] == 1


## The controls, as input actions on physical keys (so WASD is WASD on any keyboard layout).
func _add_input_actions() -> void:
	var keys := {
		"walk_up": [KEY_W, KEY_UP],
		"walk_down": [KEY_S, KEY_DOWN],
		"walk_left": [KEY_A, KEY_LEFT],
		"walk_right": [KEY_D, KEY_RIGHT],
		"run": [KEY_SHIFT],
		"pause": [KEY_SPACE],
		"speed_1": [KEY_1],
		"speed_2": [KEY_2],
		"speed_3": [KEY_3],
		"swing": [KEY_F],
		"shove": [KEY_Q],
		"door": [KEY_E],
		"sleep": [KEY_Z],
		"drink": [KEY_X],
		"light": [KEY_L],
		"low_walls": [KEY_V],
		"pack": [KEY_TAB],
		"skills": [KEY_K],
		"board": [KEY_B],
		"burn": [KEY_G],
		"second_wind": [KEY_C],
		"unboard": [KEY_N],
		"tend_fire": [KEY_H],
		"smother_fire": [KEY_J],
		"restart": [KEY_R],
		"save": [KEY_F5],
		"quit": [KEY_ESCAPE],
	}
	for action: String in keys:
		if InputMap.has_action(action):
			continue
		InputMap.add_action(action)
		for key in keys[action]:
			var press := InputEventKey.new()
			press.physical_keycode = key
			InputMap.action_add_event(action, press)


func _read_args() -> void:
	for arg in OS.get_cmdline_user_args():
		var value := arg.get_slice("=", 1)
		if arg.begins_with("--seed="):
			town_seed = value.to_int()
		elif arg == "--town":
			use_town = true
		elif arg.begins_with("--walk="):
			walk_at = Vector2i(value.get_slice(",", 0).to_int(), value.get_slice(",", 1).to_int())
		elif arg.begins_with("--zoom="):
			zoom = value.to_float()
		elif arg.begins_with("--data="):
			data_dir = value
		elif arg.begins_with("--speed="):
			start_speed = value.to_float()
		elif arg.begins_with("--do="):
			for step in value.split(","):
				scripted.append([step.get_slice(":", 0).to_float(), step.get_slice(":", 1)])
		elif arg.begins_with("--shot="):
			shot_path = value
		elif arg.begins_with("--shot-after="):
			shot_after = value.to_float()
		elif arg == "--news":
			print_news = true
		elif arg.begins_with("--save="):
			save_path = value
		elif arg.begins_with("--hour="):
			start_hour = value.to_int()


## --shot: saves what's on the screen once the time has come, then quits.
func _maybe_take_picture() -> void:
	if shot_path.is_empty():
		return
	if played < shot_after:
		return
	var path := shot_path
	shot_path = ""
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(path)
	print("Saved ", path)
	get_tree().quit()


## --do: presses the scripted actions as their time comes (for pictures of a fight or a door).
func _run_script() -> void:
	while not scripted.is_empty() and played >= scripted[0][0]:
		match scripted.pop_front()[1]:
			"swing":
				dusk.attack(0, 0)
			"shove":
				dusk.shove(0, 0)
			"door":
				dusk.use_door()
			"struggle":
				dusk.struggle()
			"drop":  # the first thing in the pack
				var items: PackedInt32Array = dusk.character_items()
				if items.size() >= 3:
					dusk.drop_item(0, items[0])
			"sleep":
				_sleep_or_wake()
			"drink":
				dusk.drink_water()
			"light":
				_light_or_douse()
			"skills":
				skills.toggle()
			"save":
				_save("Saved.")
			"board":
				_board_up()
			"burn":
				_burn_body()
			"unboard":
				_pry_board()
			"logs":
				_take_logs()
			"fire":
				_tend_fire()
			"smother":
				_smother_fire()
			"next":  # after a death, the next survivor
				_next_survivor()


## Your own timed action: (its kind, how far through it is out of 1000), or (0, 0).
func _my_action() -> Vector2i:
	if me < 0:
		return Vector2i.ZERO
	var actions: PackedInt32Array = dusk.actor_actions()
	return Vector2i(actions[me * 4], actions[me * 4 + 1])


## What an item of yours is called now, by its kind: the first of yours that's of it.
func _known_as(def: int) -> String:
	var labels: PackedStringArray = dusk.character_labels()
	var hands: PackedInt32Array = dusk.character_hands()
	if hands.size() >= 2 and hands[0] == def:
		return labels[0]
	var light: PackedInt32Array = dusk.character_light()
	if light.size() >= 2 and light[0] == def:
		return labels[1]
	var items: PackedInt32Array = dusk.character_items()
	for i in range(0, items.size(), 4):
		if items[i] == def:
			return labels[2 + floori(i / 4.0)]
	return _item_name(def)


func _item_name(def: int) -> String:
	return item_names[def] if def >= 0 and def < item_names.size() else "something"


## A skill's name, from an event's 100 × skill + level.
func _skill_name(packed: int) -> String:
	var skill := floori(packed / 100.0)
	return skill_names[skill] if skill >= 0 and skill < skill_names.size() else "A skill"


## "3 apples" or "the axe", from an event's 1000 × item + count.
func _items(packed: int) -> String:
	var count := packed % 1000
	var def := floori(packed / 1000.0)
	return ("the " + _item_name(def)) if count == 1 else "%d × %s" % [count, _item_name(def)]
