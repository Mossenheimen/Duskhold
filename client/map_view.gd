extends Node3D
## The map, a chunk at a time round the camera (chunk_view.gd draws each). Chunks are built
## nearest first, a few a frame, and dropped once they're well out of view. Over it all, the fog
## of war: what you don't see now is dim, and what you've never seen is black. Over the ground,
## what the dead you see can see: their sight cones, tile by tile.

const ChunkView = preload("res://chunk_view.gd")

## Chunks within this many of the one the camera looks at are drawn; past UNLOAD_RADIUS they go.
const LOAD_RADIUS := 3
const UNLOAD_RADIUS := 4
## The most chunks built in one frame, so walking on doesn't stutter.
const BUILDS_PER_FRAME := 3

var map_size := Vector2i.ZERO
var _dusk: Node
var _chunk_size := 32
var _names := PackedStringArray()
var _low_walls := false
var _chunks := {}  # Vector2i: ChunkView
var _materials := {}
var _watched_material := ShaderMaterial.new()
var _watched_image: Image
var _watched_texture: ImageTexture
var _overlay: MeshInstance3D
var _fog_material := ShaderMaterial.new()
var _view_image: Image
var _view_texture: ImageTexture
var _fog: MeshInstance3D


func _init() -> void:
	var cut_away := ShaderMaterial.new()
	cut_away.shader = preload("res://shaders/wall.gdshader")
	var glass := ShaderMaterial.new()
	glass.shader = preload("res://shaders/glass.gdshader")
	var plain := StandardMaterial3D.new()
	plain.vertex_color_use_as_albedo = true
	plain.roughness = 0.95
	_materials = {"cut_away": cut_away, "glass": glass, "plain": plain}
	_watched_material.shader = preload("res://shaders/watched.gdshader")
	_fog_material.shader = preload("res://shaders/fog_of_war.gdshader")
	_fog_material.render_priority = Material.RENDER_PRIORITY_MIN  # before the cones and rings


## Starts drawing a new world: no chunks yet (update() brings them in), the fog and the overlay.
func build(dusk: Node, low_walls: bool) -> void:
	for view in _chunks.values():
		view.queue_free()
	_chunks.clear()
	_dusk = dusk
	_low_walls = low_walls
	_chunk_size = dusk.chunk_size()
	_names = dusk.fixture_names()
	map_size = dusk.map_size()
	_build_watched()
	_build_fog()


## Brings in the chunks round `focus` (a point in the world), nearest first, and lets the far
## ones go.
func update(focus: Vector3) -> void:
	if _dusk == null:
		return
	var middle := Vector2i(floori(focus.x / _chunk_size), floori(focus.z / _chunk_size))
	var chunks := Vector2i(ceili(map_size.x / float(_chunk_size)), ceili(map_size.y / float(_chunk_size)))
	var wanted: Array[Vector2i] = []
	for cy in range(maxi(0, middle.y - LOAD_RADIUS), mini(chunks.y, middle.y + LOAD_RADIUS + 1)):
		for cx in range(maxi(0, middle.x - LOAD_RADIUS), mini(chunks.x, middle.x + LOAD_RADIUS + 1)):
			var at := Vector2i(cx, cy)
			if not _chunks.has(at):
				wanted.append(at)
	var nearer := func(a: Vector2i, b: Vector2i) -> bool: return (a - middle).length_squared() < (b - middle).length_squared()
	wanted.sort_custom(nearer)
	for at in wanted.slice(0, BUILDS_PER_FRAME):
		var view := ChunkView.new()
		view.build(_dusk, at.x, at.y, _names, _materials, _low_walls)
		add_child(view)
		_chunks[at] = view
	for at: Vector2i in _chunks.keys():
		if maxi(absi(at.x - middle.x), absi(at.y - middle.y)) > UNLOAD_RADIUS:
			_chunks[at].queue_free()
			_chunks.erase(at)


## How many chunks are drawn now.
func chunk_count() -> int:
	return _chunks.size()


## Low walls are only a way of looking: the core's walls don't change.
func set_low_walls(low: bool) -> void:
	_low_walls = low
	for view in _chunks.values():
		view.build_walls(low)


## An edge owned by `tile` changed in the core (a door opened, shut or gave way): its chunk
## draws its walls again, if it's drawn.
func refresh_edge(tile: Vector2i) -> void:
	var at := Vector2i(floori(tile.x / float(_chunk_size)), floori(tile.y / float(_chunk_size)))
	if _chunks.has(at):
		_chunks[at].refresh_walls()


## A tile changed in the core (furniture broken up): its chunk draws its furniture again.
func refresh_tile(tile: Vector2i) -> void:
	var at := Vector2i(floori(tile.x / float(_chunk_size)), floori(tile.y / float(_chunk_size)))
	if _chunks.has(at):
		_chunks[at].refresh_fixtures()


## Where the character is: walls, furniture and trees between it and the camera are cut away
## round it.
func set_focus(at: Vector3) -> void:
	_materials["cut_away"].set_shader_parameter("focus", at)
	_materials["glass"].set_shader_parameter("focus", at)


## What the dead can see, from DuskWorld.watched().
func show_watched(levels: PackedByteArray) -> void:
	if levels.size() != map_size.x * map_size.y:
		return
	_watched_image.set_data(map_size.x, map_size.y, false, Image.FORMAT_R8, levels)
	_watched_texture.update(_watched_image)


## What you make out of the map, from DuskWorld.view_tiles().
func show_view(tiles: PackedByteArray) -> void:
	if tiles.size() != map_size.x * map_size.y:
		return
	_view_image.set_data(map_size.x, map_size.y, false, Image.FORMAT_R8, tiles)
	_view_texture.update(_view_image)


func _build_fog() -> void:
	if _fog:
		_fog.queue_free()
	_view_image = Image.create_empty(map_size.x, map_size.y, false, Image.FORMAT_R8)
	_view_texture = ImageTexture.create_from_image(_view_image)
	_fog_material.set_shader_parameter("view", _view_texture)
	_fog_material.set_shader_parameter("map_size", Vector2(map_size))
	var quad := QuadMesh.new()
	quad.size = Vector2(2.0, 2.0)
	_fog = MeshInstance3D.new()
	_fog.mesh = quad
	_fog.material_override = _fog_material
	_fog.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_fog.extra_cull_margin = 16384.0  # it covers the screen, wherever the camera looks
	add_child(_fog)


func _build_watched() -> void:
	if _overlay:
		_overlay.queue_free()
	_watched_image = Image.create_empty(map_size.x, map_size.y, false, Image.FORMAT_R8)
	_watched_texture = ImageTexture.create_from_image(_watched_image)
	_watched_material.set_shader_parameter("watched", _watched_texture)
	_watched_material.set_shader_parameter("map_size", Vector2(map_size))
	var plane := PlaneMesh.new()
	plane.size = Vector2(map_size)
	_overlay = MeshInstance3D.new()
	_overlay.mesh = plane
	_overlay.material_override = _watched_material
	_overlay.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_overlay.position = Vector3(map_size.x / 2.0, 0.012, map_size.y / 2.0)
	add_child(_overlay)
