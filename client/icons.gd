extends Node
## Items' icons (VISUALS.md §11, Item tile), each drawn from the item's own look (data/looks.toml's
## [[item]], looks.gd), so every thing you carry is drawn in the kit, as the world is: its mesh on a
## small stage of its own, seen from the front, a little above and to its right, lit from the upper
## left, fitted to the item's cells and drawn once into a texture, kept by item. One a frame, as
## they're asked for: `drawn` says when each is ready. With no window (--headless) none is drawn,
## and `icon` gives null: the tiles show the item's name instead.

const Looks = preload("res://looks.gd")

## Emitted when item `def`'s icon is drawn.
signal drawn(def: int)

## Pixels a cell's icon is drawn at: sharp at any GUI size up to twice the base.
const CELL := 96
## How the camera looks at a thing: degrees down from level, and round to its right.
const PITCH := 24.0
const YAW := 28.0
## Room left round a thing, as a share of its cells.
const MARGIN := 0.08

var _ids := PackedStringArray()
var _sizes := PackedInt32Array()
var _icons := {}  # definition -> ImageTexture, or null for one with no look
var _queue: Array[int] = []
var _busy := false
var _stage: SubViewport
var _camera: Camera3D
var _thing: MeshInstance3D


## A new world: the items' ids and sizes from the core, and the icons drawn again as they're asked
## for (a new world's data may look different).
func start(dusk: Node) -> void:
	_ids = dusk.item_ids()
	_sizes = dusk.item_sizes()
	_icons.clear()
	_queue.clear()
	if _stage == null and DisplayServer.get_name() != "headless":
		_build_stage()


## Item `def`'s icon, or null: not drawn yet (it's on its way: `drawn`), no look, or no window.
func icon(def: int) -> Texture2D:
	if _icons.has(def):
		return _icons[def]
	if _stage != null and not _queue.has(def):
		_queue.append(def)
	return null


## Item `def`'s cells, across and down (DuskWorld.item_sizes()).
func cells(def: int) -> Vector2i:
	if def < 0 or def * 2 + 1 >= _sizes.size():
		return Vector2i.ONE
	return Vector2i(maxi(1, _sizes[def * 2]), maxi(1, _sizes[def * 2 + 1]))


## Every item's icon asked for at once (the gallery's, a picture of them all).
func ask_all() -> void:
	for def in _ids.size():
		icon(def)


## Whether every icon asked for is drawn.
func done() -> bool:
	return _queue.is_empty() and not _busy


func _process(_delta: float) -> void:
	if _stage != null and not _busy and not _queue.is_empty():
		_draw_next()


func _draw_next() -> void:
	_busy = true
	var def: int = _queue.pop_front()
	var mesh: ArrayMesh = Looks.item_mesh(_ids[def]) if def < _ids.size() else null
	if mesh == null:
		_icons[def] = null
		_busy = false
		return
	var size := cells(def)
	_stage.size = size * CELL
	_thing.mesh = mesh
	_frame(mesh, size)
	_stage.render_target_update_mode = SubViewport.UPDATE_ONCE
	await RenderingServer.frame_post_draw
	if _thing.mesh == mesh:
		_icons[def] = ImageTexture.create_from_image(_stage.get_texture().get_image())
		drawn.emit(def)
	_busy = false


## The camera on `mesh`, fitted to `size` cells: every corner of it in view, with a margin, and its
## middle in the middle.
func _frame(mesh: ArrayMesh, size: Vector2i) -> void:
	var pitch := deg_to_rad(PITCH)
	var yaw := deg_to_rad(YAW)
	var back := Vector3(sin(yaw) * cos(pitch), sin(pitch), -cos(yaw) * cos(pitch))
	var view := Basis.looking_at(-back, Vector3.UP)
	var points: PackedVector3Array = mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	var depth := 0.0
	for p in points:
		var v := view.inverse() * p
		lo = Vector2(minf(lo.x, v.x), minf(lo.y, v.y))
		hi = Vector2(maxf(hi.x, v.x), maxf(hi.y, v.y))
		depth = maxf(depth, absf(v.z))
	var middle := view * Vector3((lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0, 0.0)
	var aspect := float(size.x) / float(size.y)
	var span := maxf(hi.y - lo.y, (hi.x - lo.x) / aspect)
	_camera.size = maxf(span, 0.01) * (1.0 + 2.0 * MARGIN)
	_camera.near = 0.01
	_camera.far = depth * 2.0 + 4.0
	_camera.transform = Transform3D(view, middle + back * (depth + 2.0))


## The stage: a world of its own (nothing of the game's shows in it), clear behind, a key light
## from the upper left, a fill from the right, and the kit's own shader, its patterns in the thing's
## own space.
func _build_stage() -> void:
	_stage = SubViewport.new()
	_stage.own_world_3d = true
	_stage.transparent_bg = true
	_stage.msaa_3d = Viewport.MSAA_4X
	_stage.render_target_update_mode = SubViewport.UPDATE_DISABLED
	add_child(_stage)
	_camera = Camera3D.new()
	_camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	_stage.add_child(_camera)
	var key := DirectionalLight3D.new()
	key.light_energy = 1.25
	_stage.add_child(key)
	key.look_at_from_position(Vector3(-1.0, 1.6, -1.3), Vector3.ZERO)
	var fill := DirectionalLight3D.new()
	fill.light_energy = 0.4
	_stage.add_child(fill)
	fill.look_at_from_position(Vector3(1.4, 0.4, -0.6), Vector3.ZERO)
	var sky := Environment.new()
	sky.background_mode = Environment.BG_CLEAR_COLOR
	sky.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	sky.ambient_light_color = Color.WHITE
	sky.ambient_light_energy = 0.45
	sky.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	var world := WorldEnvironment.new()
	world.environment = sky
	_stage.add_child(world)
	var kit := ShaderMaterial.new()
	kit.shader = preload("res://shaders/looks.gdshader")
	kit.set_shader_parameter("cuts", false)
	kit.set_shader_parameter("local", true)
	_thing = MeshInstance3D.new()
	_thing.material_override = kit
	_stage.add_child(_thing)
