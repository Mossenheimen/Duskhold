extends Node3D
## The map as the core gives it: the ground, the furniture, and the walls, doors and windows on the
## tiles' edges. Over the ground, what the dead can see: their sight cones, tile by tile.

const WALL_HEIGHT := 2.2
const LOW_WALL_HEIGHT := 0.35
const WALL_THICKNESS := 0.12
const DOOR_TOP := 1.85
const SILL := 0.9
const WINDOW_TOP := 1.65

const EARTH := Color(0.37, 0.42, 0.31)  # the Hushlands' grey-green fields
const FLOOR := Color(0.47, 0.35, 0.24)  # planks
const FURNITURE := Color(0.32, 0.22, 0.15)
const WALL := Color(0.66, 0.63, 0.57)
const DOOR := Color(0.38, 0.25, 0.15)

## Tile and edge kinds, as DuskWorld.tiles() and edges() give them.
const GROUND_EARTH := 1
const SIDE_WEST := 1
const EDGE_WALL := 1
const EDGE_DOOR_SHUT := 2
const EDGE_DOOR_OPEN := 3
const EDGE_WINDOW_SHUT := 4
const EDGE_WINDOW_OPEN := 5
const GROUND_SOLID := 3

var map_size := Vector2i.ZERO
var _edges := PackedInt32Array()
var _walls: Node3D
var _wall_material := ShaderMaterial.new()
var _glass_material := ShaderMaterial.new()
var _watched_material := ShaderMaterial.new()
var _watched_image: Image
var _watched_texture: ImageTexture


func _init() -> void:
	_wall_material.shader = preload("res://shaders/wall.gdshader")
	_glass_material.shader = preload("res://shaders/glass.gdshader")
	_watched_material.shader = preload("res://shaders/watched.gdshader")


## Builds it all from the core's map, the walls full height or low.
func build(dusk: Node, low_walls: bool) -> void:
	for child in get_children():
		child.queue_free()
	map_size = dusk.map_size()
	_edges = dusk.edges()
	_build_ground(dusk.tiles())
	_walls = Node3D.new()
	add_child(_walls)
	_build_walls(LOW_WALL_HEIGHT if low_walls else WALL_HEIGHT)
	_build_watched()


## Low walls are only a way of looking: the core's walls don't change.
func set_low_walls(low: bool) -> void:
	for child in _walls.get_children():
		child.queue_free()
	_build_walls(LOW_WALL_HEIGHT if low else WALL_HEIGHT)


## Where the character is: walls between it and the camera are cut away round it.
func set_focus(at: Vector3) -> void:
	_wall_material.set_shader_parameter("focus", at)
	_glass_material.set_shader_parameter("focus", at)


## What the dead can see, from DuskWorld.watched().
func show_watched(levels: PackedByteArray) -> void:
	if levels.size() != map_size.x * map_size.y:
		return
	_watched_image.set_data(map_size.x, map_size.y, false, Image.FORMAT_R8, levels)
	_watched_texture.update(_watched_image)


func _build_ground(tiles: PackedInt32Array) -> void:
	var ground: Array[Transform3D] = []
	var ground_colours := PackedColorArray()
	var solid: Array[Transform3D] = []
	var solid_colours := PackedColorArray()
	for i in range(0, tiles.size(), 3):
		var x := tiles[i]
		var y := tiles[i + 1]
		var kind := tiles[i + 2]
		var shade := _shade(x, y)
		var centre := Vector3(x + 0.5, 0.0, y + 0.5)
		ground.append(Transform3D(Basis(), centre))
		ground_colours.append(_linear((EARTH if kind == GROUND_EARTH else FLOOR).darkened(shade)))
		if kind == GROUND_SOLID:
			var box := Basis.from_scale(Vector3(0.8, 0.75, 0.8))
			solid.append(Transform3D(box, centre + Vector3(0.0, 0.375, 0.0)))
			solid_colours.append(_linear(FURNITURE.darkened(shade)))
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE
	add_child(_instances(plane, ground, ground_colours, _plain_material()))
	add_child(_instances(BoxMesh.new(), solid, solid_colours, _plain_material()))


func _build_walls(height: float) -> void:
	var solid: Array[Transform3D] = []
	var solid_colours := PackedColorArray()
	var glass: Array[Transform3D] = []
	for i in range(0, _edges.size(), 4):
		var x := _edges[i]
		var y := _edges[i + 1]
		var west := _edges[i + 2] == SIDE_WEST
		# Each piece of the edge: from, to (heights), and what it is.
		var pieces := []
		match _edges[i + 3]:
			EDGE_WALL:
				pieces = [[0.0, WALL_HEIGHT, "wall"]]
			EDGE_DOOR_SHUT:
				pieces = [[DOOR_TOP, WALL_HEIGHT, "wall"], [0.0, DOOR_TOP, "door"]]
			EDGE_DOOR_OPEN:
				pieces = [[DOOR_TOP, WALL_HEIGHT, "wall"]]
			EDGE_WINDOW_SHUT:
				pieces = [[0.0, SILL, "wall"], [WINDOW_TOP, WALL_HEIGHT, "wall"], [SILL, WINDOW_TOP, "glass"]]
			EDGE_WINDOW_OPEN:
				pieces = [[0.0, SILL, "wall"], [WINDOW_TOP, WALL_HEIGHT, "wall"]]
		for piece in pieces:
			var bottom: float = piece[0]
			var top := minf(piece[1], height)
			if top - bottom < 0.01:
				continue
			var what: String = piece[2]
			# Walls overlap at the corners; doors and panes sit inside the frame, thinner.
			var length := 1.0 + WALL_THICKNESS if what == "wall" else 1.0 - WALL_THICKNESS
			var thick := WALL_THICKNESS if what == "wall" else WALL_THICKNESS * 0.4
			var mid := (bottom + top) / 2.0
			var size := Vector3(thick, top - bottom, length) if west else Vector3(length, top - bottom, thick)
			var centre := Vector3(x, mid, y + 0.5) if west else Vector3(x + 0.5, mid, y)
			var place := Transform3D(Basis.from_scale(size), centre)
			if what == "glass":
				glass.append(place)
			else:
				solid.append(place)
				solid_colours.append(_linear(WALL if what == "wall" else DOOR))
	var box := BoxMesh.new()
	var walls := _instances(box, solid, solid_colours, _wall_material)
	walls.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_walls.add_child(walls)
	# Shadows come from the whole walls: a hole cut for the camera mustn't let the sun in.
	var shadows := MultiMeshInstance3D.new()
	shadows.multimesh = walls.multimesh
	shadows.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_SHADOWS_ONLY
	_walls.add_child(shadows)
	var white := PackedColorArray()
	white.resize(glass.size())
	white.fill(Color.WHITE)
	var panes := _instances(box, glass, white, _glass_material)
	panes.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_walls.add_child(panes)


func _build_watched() -> void:
	_watched_image = Image.create_empty(map_size.x, map_size.y, false, Image.FORMAT_R8)
	_watched_texture = ImageTexture.create_from_image(_watched_image)
	_watched_material.set_shader_parameter("watched", _watched_texture)
	_watched_material.set_shader_parameter("map_size", Vector2(map_size))
	var plane := PlaneMesh.new()
	plane.size = Vector2(map_size)
	var overlay := MeshInstance3D.new()
	overlay.mesh = plane
	overlay.material_override = _watched_material
	overlay.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	overlay.position = Vector3(map_size.x / 2.0, 0.012, map_size.y / 2.0)
	add_child(overlay)


## Many copies of one mesh, each placed and coloured (colours in linear space).
func _instances(mesh: Mesh, places: Array[Transform3D], colours: PackedColorArray,
		material: Material) -> MultiMeshInstance3D:
	var multimesh := MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.use_colors = true
	multimesh.mesh = mesh
	multimesh.instance_count = places.size()
	for i in places.size():
		multimesh.set_instance_transform(i, places[i])
		multimesh.set_instance_color(i, colours[i])
	var node := MultiMeshInstance3D.new()
	node.multimesh = multimesh
	node.material_override = material
	return node


func _plain_material() -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.vertex_color_use_as_albedo = true
	material.roughness = 0.95
	return material


## A little variation from tile to tile, the same every time.
func _shade(x: int, y: int) -> float:
	return float((x * 73 + y * 151 + (x * y) % 7 * 31) % 17) / 17.0 * 0.045


## Colours here are picked by eye (sRGB); instance colours go to the GPU as linear.
func _linear(colour: Color) -> Color:
	return colour.srgb_to_linear()
