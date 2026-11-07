extends Node3D
## Duskhold's client. The Rust core runs the game inside the DuskWorld node (the dusk-godot
## extension). This draws what it says and turns the player's input into intents: it decides
## nothing.
##
## The core's tile (x, y) is the 1 × 1 square from (x, 0, y) in 3D: x east, y south, Y up.
##
## Development flags, after "--" on Godot's command line (play.cmd passes them on):
## --seed=N the patch's seed; --town the little demo town instead; --walk=X,Y walk there at the
## start; --zoom=N; --speed=N the speed dial; --shot=PATH save a picture after
## --shot-after=SECONDS (default 3) and quit.

const MapView = preload("res://map_view.gd")
const ActorView = preload("res://actor_view.gd")
const NoiseRings = preload("res://noise_rings.gd")
const Hud = preload("res://hud.gd")
const Tells = preload("res://tells.gd")

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
## Numbers an event takes.
const EVENT_SIZE := 6

var dusk: Node  # the DuskWorld
var map_view: MapView
var rings: NoiseRings
var tells: Tells
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
var dead := false
var low_walls := false
var last_tick := -1
var use_town := false

var shot_path := ""
var shot_after := 3.0
var start_speed := 1.0
var walk_at := Vector2i(-1, -1)
var played := 0.0


func _ready() -> void:
	_read_args()
	_add_input_actions()
	_build_stage()
	hud = Hud.new()
	add_child(hud)
	if not ClassDB.class_exists("DuskWorld"):
		hud.show_problem("The game's core (the Rust extension) isn't loaded.\n"
			+ "Start the game with play.cmd in 12_Duskhold: it builds the extension first.")
		return
	dusk = ClassDB.instantiate("DuskWorld")
	dusk.process_priority = -1  # tick the world before this script draws it
	add_child(dusk)
	map_view = MapView.new()
	add_child(map_view)
	rings = NoiseRings.new()
	add_child(rings)
	tells = Tells.new()
	add_child(tells)
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
	part_names = dusk.part_names()
	last_hurt = ""
	dead = false
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
	hud.show_state(town_seed, tick, roundi(dusk.speed), dusk.paused)
	hud.show_body(dusk.character_body(), dusk.character_wounds())
	_maybe_take_picture(delta)


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
					dusk.attack(aim.x, aim.y)
				else:
					dusk.walk_to(tile.x, tile.y, Input.is_action_pressed("run"))
			MOUSE_BUTTON_WHEEL_UP:
				zoom = maxf(ZOOM_MIN, zoom / 1.15)
			MOUSE_BUTTON_WHEEL_DOWN:
				zoom = minf(ZOOM_MAX, zoom * 1.15)
	elif event.is_action_pressed("swing") and not dead:
		dusk.attack(0, 0)
	elif event.is_action_pressed("shove") and not dead:
		dusk.shove(0, 0)
	elif event.is_action_pressed("door") and not dead:
		dusk.use_door()
	elif event.is_action_pressed("pause"):
		dusk.paused = not dusk.paused
	elif event.is_action_pressed("speed_1"):
		_set_speed(0)
	elif event.is_action_pressed("speed_2"):
		_set_speed(1)
	elif event.is_action_pressed("speed_3"):
		_set_speed(2)
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
	dusk.speed = float(SPEEDS[index])
	dusk.paused = false


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
				if who == me:
					hud.say("You opened the door." if extra == 1 else "You shut the door.")
			EXHAUSTED:
				if who == me:
					hud.say("You're out of breath.")


## Whether actor `i` stands beside you.
func _beside(i: int) -> bool:
	if me < 0 or i < 0:
		return false
	var where: PackedVector2Array = dusk.actor_positions()
	return (where[i] - where[me]).length() < 1.6


func _die() -> void:
	dead = true
	var how := last_hurt
	if dusk.character_body()[4] > 0 or how.is_empty():
		how = "You bled to death." if not how.is_empty() else "You died."
	hud.show_death(how, dusk.tick() / 10.0)


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
		"low_walls": [KEY_V],
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
		elif arg.begins_with("--speed="):
			start_speed = value.to_float()
		elif arg.begins_with("--shot="):
			shot_path = value
		elif arg.begins_with("--shot-after="):
			shot_after = value.to_float()


## --shot: saves what's on the screen once the time has come, then quits.
func _maybe_take_picture(delta: float) -> void:
	if shot_path.is_empty():
		return
	played += delta
	if played < shot_after:
		return
	var path := shot_path
	shot_path = ""
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(path)
	print("Saved ", path)
	get_tree().quit()
