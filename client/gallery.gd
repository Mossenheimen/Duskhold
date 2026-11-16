extends Node3D
## The looks gallery, a picture route (VISUALS.md §14): every look in data/looks.toml laid out in
## rows on plain ground, under the game's camera, sun and sky, each with its name, and the
## character beside each row for scale. So the kit can be seen whole, by day or at night, without
## walking the world to find each thing. Nothing here is the game: no world is made.
##
##   .\play.cmd gallery.tscn -- [--daylight=0..100] [--zoom=Z] [--only=id,id] [--torch]
##                              [--cols=N] [--data=DIR] --shot=PNG --shot-after=S
##
## Each look stands as a fixture placed "fixed" does, its front toward the camera's side; each of
## its variants has its own place ("grave 2").

const Look = preload("res://look.gd")
const Looks = preload("res://looks.gd")
const Main = preload("res://main.gd")
const ActorView = preload("res://actor_view.gd")

## Metres between one look and the next.
const GAP := 2.2
## Fixtures that stand out of doors: on grass, not a plank floor.
const OUTDOORS := ["tree", "yarrow", "chamomile", "feverfew", "bramble", "picked_plant", "wheat",
	"well", "hay", "grave", "tomb", "sack"]

var _daylight := 100
var _zoom := 14.0
var _only := PackedStringArray()
var _torch := false
var _cols := 6
var _data_dir := ""
var _shot := ""
var _shot_after := 2.0
var _played := 0.0


func _ready() -> void:
	_read_args()
	get_window().size = Vector2i(1600, 900)
	if not ClassDB.class_exists("DuskWorld"):
		push_error("The game's core (the Rust extension) isn't loaded: start it with play.cmd.")
		get_tree().quit()
		return
	var dusk: Node = ClassDB.instantiate("DuskWorld")
	dusk.data_dir = _data_dir
	add_child(dusk)
	Looks.read(dusk)
	var shown: Array = []  # [name, mesh, out of doors]
	var ids: Array = Looks.fixtures()
	ids.sort()
	for id: String in ids:
		if not _only.is_empty() and not _only.has(id):
			continue
		var count := (Looks.fixture(id).variants as Array).size()
		for n in count:
			shown.append([id if count == 1 else "%s %d" % [id, n + 1], Looks.fixture_mesh(id, n),
				OUTDOORS.has(id)])
	var strewn := Looks.scatter()
	for k in strewn.size():
		var id: String = strewn[k].id
		if not _only.is_empty() and not _only.has(id):
			continue
		var count := (strewn[k].variants as Array).size()
		for n in count:
			shown.append([id if count == 1 else "%s %d" % [id, n + 1], Looks.scatter_mesh(k, n),
				true])
	_lay_out(shown)


func _process(delta: float) -> void:
	_played += delta
	if not _shot.is_empty() and _played >= _shot_after:
		var path := _shot
		_shot = ""
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png(path)
		print("Saved ", path)
		get_tree().quit()


func _read_args() -> void:
	for arg in OS.get_cmdline_user_args():
		var value := arg.get_slice("=", 1)
		if arg.begins_with("--daylight="):
			_daylight = value.to_int()
		elif arg.begins_with("--zoom="):
			_zoom = value.to_float()
		elif arg.begins_with("--only="):
			_only = value.split(",")
		elif arg == "--torch":
			_torch = true
		elif arg.begins_with("--cols="):
			_cols = maxi(1, value.to_int())
		elif arg.begins_with("--data="):
			_data_dir = value
		elif arg.begins_with("--shot="):
			_shot = value
		elif arg.begins_with("--shot-after="):
			_shot_after = value.to_float()


## The looks in rows of `_cols`, west to east, each row a step south; the ground under them; the
## camera on the middle of it all; the sun and sky at `_daylight`, as main.gd's _light_the_sky.
func _lay_out(shown: Array) -> void:
	var cols := mini(_cols, maxi(1, shown.size()))
	var rows := ceili(shown.size() / float(cols))
	var material := ShaderMaterial.new()
	material.shader = preload("res://shaders/looks.gdshader")
	material.set_shader_parameter("cuts", false)
	# The ground, square (ground.gdshader maps it by one size): a texel is half a GAP, two by two
	# under each look, grass for what stands out of doors and planks for the rest.
	var texels := (maxi(cols, rows) + 1) * 2
	var ground := Image.create_empty(texels, texels, false, Image.FORMAT_RGBA8)
	ground.fill(_ground_texel(5))
	for i in shown.size():
		var cell := Vector2i(i % cols, floori(i / float(cols)))
		var at := Vector3((cell.x + 1) * GAP, 0.0, (cell.y + 1) * GAP)
		var thing := MeshInstance3D.new()
		thing.mesh = shown[i][1]
		thing.material_override = material
		thing.position = at
		thing.rotation.y = PI  # its front to the camera's side, as "fixed" stands
		add_child(thing)
		_label(shown[i][0], at + Vector3(0.0, 0.0, 0.85))
		if shown[i][2]:
			for dy in 2:
				for dx in 2:
					ground.set_pixel(cell.x * 2 + 1 + dx, cell.y * 2 + 1 + dy, _ground_texel(1))
		if cell.x == 0:
			var you := ActorView.new()
			you.setup(0)
			you.position = at + Vector3(-GAP * 0.55, 0.0, 0.0)
			add_child(you)
	var side := texels * GAP / 2.0
	var square := MeshInstance3D.new()
	var plane := PlaneMesh.new()
	plane.size = Vector2(side, side)
	square.mesh = plane
	var surface := ShaderMaterial.new()
	surface.shader = preload("res://shaders/ground.gdshader")
	surface.set_shader_parameter("ground", ImageTexture.create_from_image(ground))
	surface.set_shader_parameter("origin", Vector2.ZERO)
	surface.set_shader_parameter("size", side)
	square.material_override = surface
	square.position = Vector3(side / 2.0, 0.0, side / 2.0)
	add_child(square)
	_stage(Vector3((cols + 1) * GAP / 2.0, 0.0, (rows + 1) * GAP / 2.0))


## A texel of ground for ground.gdshader: its colour, and its kind in alpha.
func _ground_texel(kind: int) -> Color:
	var colour: Color = Look.GROUND[kind]
	colour.a = (kind + 0.5) / 16.0
	return colour


func _label(text: String, at: Vector3) -> void:
	var label := Label3D.new()
	label.text = text
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.no_depth_test = true
	label.font_size = 30
	label.pixel_size = 0.0075
	label.outline_size = 8
	label.outline_modulate = Look.OUTLINE
	label.position = at
	add_child(label)


## The camera, at the game's angle, on `focus`; the sun from the south-west; the sky at the
## daylight asked for; and a torch held up over the middle, if asked.
func _stage(focus: Vector3) -> void:
	var camera := Camera3D.new()
	camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	camera.size = _zoom
	camera.near = 0.1
	camera.far = 200.0
	add_child(camera)
	var pitch := deg_to_rad(Main.CAMERA_PITCH)
	var yaw := deg_to_rad(Main.CAMERA_YAW)
	var back := Vector3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch))
	camera.look_at_from_position(focus + back * Main.CAMERA_DISTANCE, focus, Vector3.UP)
	var t := clampf(_daylight / 100.0, 0.0, 1.0)
	var warm := clampf(1.0 - absf(t - 0.6) / 0.3, 0.0, 1.0)
	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55.0, -35.0, 0.0)
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_ORTHOGONAL
	sun.directional_shadow_max_distance = 90.0
	sun.light_energy = lerpf(0.06, 1.1, t * t)
	sun.light_color = Look.SUN_NIGHT.lerp(Look.SUN_DAY, t).lerp(Look.SUN_DUSK, warm * 0.55)
	add_child(sun)
	var sky := Environment.new()
	sky.background_mode = Environment.BG_COLOR
	sky.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	sky.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	sky.ambient_light_energy = lerpf(0.16, 0.6, t)
	sky.ambient_light_color = Look.AMBIENT_NIGHT.lerp(Look.AMBIENT_DAY, t)
	sky.background_color = Look.SKY_NIGHT.lerp(Look.SKY_DAY, t)
	var world := WorldEnvironment.new()
	world.environment = sky
	add_child(world)
	if _torch:
		var torch := OmniLight3D.new()
		torch.light_color = Look.TORCH
		torch.shadow_enabled = true
		torch.omni_attenuation = 1.2
		torch.omni_range = 8.0
		torch.light_energy = 3.0
		torch.position = focus + Vector3(0.35, 1.8, 0.35)
		add_child(torch)
