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
## sleep or drink (from water) S real seconds in; --news print the news too (for --headless).

const MapView = preload("res://map_view.gd")
const ActorView = preload("res://actor_view.gd")
const NoiseRings = preload("res://noise_rings.gd")
const Hud = preload("res://hud.gd")
const Tells = preload("res://tells.gd")
const Threats = preload("res://threats.gd")
const Bag = preload("res://bag.gd")
const Litter = preload("res://litter.gd")

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
## What your own timed actions are called, by DuskWorld.actor_actions() kind (5 on).
const DOING := {5: "Struggling", 6: "Taking", 7: "Dropping", 8: "Bandaging", 9: "Taking it in hand",
	10: "Putting it away", 11: "Lying down", 12: "Drinking", 13: "Eating", 14: "Drinking"}
## Moodles (DuskWorld.character_moodles()) the client acts on.
const HUNGRY := 0
const THIRSTY := 1
const ASLEEP := 8
## Asleep, the game runs this many times as fast (an hour of sleep in a few seconds).
const SLEEP_SPEED := 60.0
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
var item_names := PackedStringArray()
var hud: Hud
var camera: Camera3D
var cursor: MeshInstance3D
var actors: Array = []  # an ActorView per actor, in the core's order
var me := -1  # the character's index in the actor arrays
var town_seed := 1
var speed_index := 0
var zoom := 14.0
var focus := Vector3.ZERO  # where the camera looks: the character, smoothed
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
var data_dir := ""  # --data: another data folder (a mod, or a test)
var walk_at := Vector2i(-1, -1)
var played := 0.0
var scripted: Array = []  # --do: [real seconds, action], soonest first


func _ready() -> void:
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
	litter = Litter.new()
	add_child(litter)
	_start()


## Makes the world afresh: a patch of the Hushlands from the seed (or the demo town). The same
## seed and the same input play the same game.
func _start() -> void:
	if use_town:
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
	for kind in dusk.actor_kinds():
		var view := ActorView.new()
		view.setup(kind)
		add_child(view)
		actors.append(view)
	me = dusk.character_index()
	rings.clear()
	hud.reset()
	wound_names = dusk.wound_names()
	item_names = dusk.item_names()
	bag.start(dusk)
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
	_move_camera(delta)
	map_view.update(focus)
	_move_cursor()
	var clock: PackedInt32Array = dusk.clock()
	moodles = dusk.character_moodles()
	var asleep := _moodle(ASLEEP) > 0
	_sleep_speed(asleep)
	hud.show_state(town_seed, clock, roundi(dusk.speed), dusk.paused)
	hud.show_moodles(moodles)
	hud.show_asleep(asleep, clock)
	var body: PackedInt32Array = dusk.character_body()
	held = body[5] if body.size() > 5 else 0
	hud.show_body(body, dusk.character_wounds())
	var doing := _my_action()
	bag.refresh(doing.x != 0, dead)
	litter.show_floors(dusk.floor_items())
	hud.show_action(DOING.get(doing.x, ""), doing.y / 1000.0)
	played += delta
	_run_script()
	_maybe_take_picture()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("quit"):
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
		dusk.shove(0, 0)
	elif event.is_action_pressed("door") and not dead:
		dusk.use_door()
	elif event.is_action_pressed("sleep") and not dead:
		_sleep_or_wake()
	elif event.is_action_pressed("drink") and not dead:
		if dusk.water_in_reach():
			dusk.drink_water()
		else:
			hud.say("There's no water within reach.")
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
	elif event.is_action_pressed("low_walls"):
		low_walls = not low_walls
		map_view.set_low_walls(low_walls)
	elif event.is_action_pressed("restart"):
		town_seed += 1
		_start()


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
		if kinds[i] == 1 and states[i] != 2 and Vector2i(roundi(where[i].x), roundi(where[i].y)) == tile:
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
		_:
			dusk.sleep()


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


func _draw_actors(delta: float) -> void:
	var where: PackedVector2Array = dusk.actor_positions()
	var facing: PackedInt32Array = dusk.actor_facing()
	var alert: PackedInt32Array = dusk.actor_alert()
	var states: PackedInt32Array = dusk.actor_states()
	var actions: PackedInt32Array = dusk.actor_actions()
	for i in actors.size():
		actors[i].position = Vector3(where[i].x + 0.5, 0.0, where[i].y + 0.5)
		actors[i].face(facing[i], delta)
		actors[i].show_alert(alert[i], delta)
		# Leaning in while a swing winds up: the tell, on the body as well as the ground.
		var swinging := actions[i * 4] == 1 and actions[i * 4 + 1] < 1000
		actors[i].show_pose(states[i], actions[i * 4 + 1] / 1000.0 if swinging else -1.0, delta)
	tells.show_actions(actions, me)
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
				hud.say("A shambler heard something.")
			SAW:
				hud.say("A shambler saw you!")
			NO_WAY:
				if who == me:
					hud.say("There's no way there.")
			STRUCK:
				if target == me and extra > 0:
					var wound := wound_names[floori(extra / 100.0) - 1]
					var part := part_names[extra % 100]
					last_hurt = "A shambler gave you a %s on the %s." % [wound, part]
					last_hurt_at = dusk.tick()
					hud.say(last_hurt)
					hud.flash()
				elif who == me:
					hud.say("You hit it.")
			MISSED:
				if who == me:
					hud.say("You missed.")
				elif target == me or _beside(who):
					hud.say("A shambler reached for you and missed.")
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
	var how := last_hurt
	var starving := _moodle(HUNGRY) >= 3
	var parched := _moodle(THIRSTY) >= 3
	if dragged_by >= 3:
		how = "%d of them got hold of you, and dragged you down." % dragged_by
	elif dragged_by > 0:
		how = "It had hold of you, and you had no breath left to throw it off."
	elif dusk.tick() - last_hurt_at <= 1:
		pass  # the blow itself
	elif dusk.character_body()[4] > 0:
		how = "You bled to death."
	elif starving and parched:
		how = "Hunger and thirst wore you down."
	elif parched:
		how = "You died of thirst."
	elif starving:
		how = "You starved."
	elif how.is_empty():
		how = "You died."
	hud.show_death(how, dusk.clock()[3])


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

	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55.0, -35.0, 0.0)  # from the south-west: shadows fall up-right
	sun.light_color = Color(1.0, 0.94, 0.84)
	sun.light_energy = 1.1
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_ORTHOGONAL
	sun.directional_shadow_max_distance = 90.0
	add_child(sun)

	var sky := Environment.new()
	sky.background_mode = Environment.BG_COLOR
	sky.background_color = Color(0.07, 0.08, 0.1)
	sky.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	sky.ambient_light_color = Color(0.58, 0.63, 0.72)
	sky.ambient_light_energy = 0.6
	sky.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	var world_environment := WorldEnvironment.new()
	world_environment.environment = sky
	add_child(world_environment)

	var square := PlaneMesh.new()
	square.size = Vector2.ONE
	var outline := ShaderMaterial.new()
	outline.shader = preload("res://shaders/cursor.gdshader")
	cursor = MeshInstance3D.new()
	cursor.mesh = square
	cursor.material_override = outline
	cursor.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(cursor)


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
		"low_walls": [KEY_V],
		"pack": [KEY_TAB],
		"restart": [KEY_R],
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


## Your own timed action: (its kind, how far through it is out of 1000), or (0, 0).
func _my_action() -> Vector2i:
	if me < 0:
		return Vector2i.ZERO
	var actions: PackedInt32Array = dusk.actor_actions()
	return Vector2i(actions[me * 4], actions[me * 4 + 1])


func _item_name(def: int) -> String:
	return item_names[def] if def >= 0 and def < item_names.size() else "something"


## "3 apples" or "the axe", from an event's 1000 × item + count.
func _items(packed: int) -> String:
	var count := packed % 1000
	var def := floori(packed / 1000.0)
	return ("the " + _item_name(def)) if count == 1 else "%d × %s" % [count, _item_name(def)]
