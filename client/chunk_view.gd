extends Node3D
## One chunk of the map, as the core gives it: its ground as one textured square, its fixtures
## (trees, furniture), and the walls, doors, windows and fences on its tiles' edges.

const WALL_HEIGHT := 2.2
const LOW_WALL_HEIGHT := 0.35
const WALL_THICKNESS := 0.12
const DOOR_TOP := 1.85
const SILL := 0.9
const WINDOW_TOP := 1.65
const FENCE_HEIGHT := 0.9

## Ground colours by the core's ground code (DuskWorld.chunk_tiles()), picked by eye in sRGB.
const GROUND := [
	Color(0.07, 0.08, 0.1),  # nothing
	Color(0.37, 0.42, 0.31),  # grass: the Hushlands' grey-green
	Color(0.43, 0.35, 0.25),  # dirt
	Color(0.52, 0.48, 0.4),  # road
	Color(0.2, 0.3, 0.38),  # water
	Color(0.47, 0.35, 0.24),  # a floor: planks
	Color(0.3, 0.42, 0.47),  # shallows: water you can wade
	Color(0.4, 0.29, 0.18),  # a bridge: darker planks over the water
]
const WALL := Color(0.66, 0.63, 0.57)
const DOOR := Color(0.38, 0.25, 0.15)
const BOARD := Color(0.62, 0.48, 0.3)
## Room across an opening for this many boards; each one tilts a little, nailed by hand.
const BOARD_SLOTS := 4
const BOARD_TILT := 9.0
const FENCE := Color(0.45, 0.38, 0.3)
const TRUNK := Color(0.3, 0.23, 0.17)
const LEAVES := Color(0.24, 0.33, 0.21)

## How furniture looks, by fixture id: its size (x, height, z) and colour. Anything not listed is
## a plain crate-sized box; "tree" is drawn as a tree.
const FURNITURE := {
	"bed": [Vector3(0.9, 0.45, 0.9), Color(0.58, 0.47, 0.38)],
	"wardrobe": [Vector3(0.85, 1.9, 0.6), Color(0.33, 0.22, 0.14)],
	"cupboard": [Vector3(0.85, 0.95, 0.6), Color(0.42, 0.3, 0.2)],
	"table": [Vector3(0.9, 0.75, 0.9), Color(0.45, 0.32, 0.2)],
	"chair": [Vector3(0.45, 0.9, 0.45), Color(0.4, 0.28, 0.18)],
	"stove": [Vector3(0.8, 0.9, 0.7), Color(0.22, 0.22, 0.24)],
	"crate": [Vector3(0.75, 0.7, 0.75), Color(0.5, 0.39, 0.25)],
	"barrel": [Vector3(0.6, 0.85, 0.6), Color(0.42, 0.3, 0.18)],
	"shelf": [Vector3(0.9, 1.8, 0.4), Color(0.36, 0.25, 0.16)],
	"pew": [Vector3(0.95, 0.5, 0.5), Color(0.4, 0.28, 0.17)],
	"altar": [Vector3(0.95, 1.0, 0.7), Color(0.62, 0.6, 0.56)],
	"counter": [Vector3(0.95, 1.0, 0.8), Color(0.5, 0.36, 0.22)],
	# The sister who keeps the counter: a figure in grey, until a holdout's people are actors.
	"sister": [Vector3(0.42, 1.6, 0.42), Color(0.36, 0.38, 0.46)],
	# The market town's shops.
	"forge": [Vector3(0.95, 1.0, 0.9), Color(0.3, 0.27, 0.26)],
	"anvil": [Vector3(0.6, 0.7, 0.35), Color(0.17, 0.17, 0.19)],
	"rack": [Vector3(0.9, 1.6, 0.3), Color(0.38, 0.27, 0.17)],
	"till": [Vector3(0.95, 1.0, 0.8), Color(0.46, 0.33, 0.2)],
	"shop_shelf": [Vector3(0.9, 1.7, 0.45), Color(0.47, 0.35, 0.22)],
	"bar": [Vector3(0.95, 1.05, 0.8), Color(0.32, 0.2, 0.12)],
	"cask": [Vector3(0.75, 0.8, 0.9), Color(0.46, 0.32, 0.17)],
	"hearth": [Vector3(0.95, 1.2, 0.7), Color(0.42, 0.4, 0.38)],
}

const SIDE_WEST := 1
const EDGE_WALL := 1
const EDGE_DOOR_SHUT := 2
const EDGE_DOOR_OPEN := 3
const EDGE_WINDOW_SHUT := 4
const EDGE_WINDOW_OPEN := 5
const EDGE_FENCE := 6

var _edges := PackedInt32Array()
var _materials: Dictionary
var _walls: Node3D
var _dusk: Node
var _chunk := Vector2i.ZERO
var _low_walls := false


## Builds the chunk (cx, cy) from the core. `names` are the fixtures' ids by number;
## `materials` the map's shared ones.
func build(dusk: Node, cx: int, cy: int, names: PackedStringArray, materials: Dictionary,
		low_walls: bool) -> void:
	_materials = materials
	_dusk = dusk
	_chunk = Vector2i(cx, cy)
	var size: int = dusk.chunk_size()
	var origin := Vector2i(cx * size, cy * size)
	var tiles: PackedInt32Array = dusk.chunk_tiles(cx, cy)
	_edges = dusk.chunk_edges(cx, cy)
	_build_ground(tiles, origin, size)
	_build_fixtures(tiles, origin, size, names)
	_walls = Node3D.new()
	add_child(_walls)
	build_walls(low_walls)


## The chunk's edges changed in the core (a door opened, shut or gave way): fetch them again.
func refresh_walls() -> void:
	_edges = _dusk.chunk_edges(_chunk.x, _chunk.y)
	build_walls(_low_walls)


## The walls, doors, windows and fences, full height or low. Low walls are only a way of
## looking: the core's walls don't change.
func build_walls(low_walls: bool) -> void:
	_low_walls = low_walls
	for child in _walls.get_children():
		child.queue_free()
	var height := LOW_WALL_HEIGHT if low_walls else WALL_HEIGHT
	var solid: Array[Transform3D] = []
	var colours := PackedColorArray()
	var glass: Array[Transform3D] = []
	var planks: Array[Transform3D] = []
	var plank_colours := PackedColorArray()
	for i in range(0, _edges.size(), 4):
		var x := _edges[i]
		var y := _edges[i + 1]
		var west := _edges[i + 2] == SIDE_WEST
		var kind := _edges[i + 3] & 15
		var boards := _edges[i + 3] >> 4
		# Each piece of the edge: from, to (heights), and what it is.
		var pieces := []
		match kind:
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
			EDGE_FENCE:
				pieces = [[0.0, FENCE_HEIGHT, "fence"]]
		for piece in pieces:
			var bottom: float = piece[0]
			var top := minf(piece[1], height)
			if top - bottom < 0.01:
				continue
			var what: String = piece[2]
			# Walls overlap at the corners; doors and panes sit inside the frame, thinner.
			var length := 1.0 + WALL_THICKNESS if what == "wall" else 1.0 - WALL_THICKNESS * 0.5
			var thick: float = {"wall": WALL_THICKNESS, "fence": 0.07}.get(what, WALL_THICKNESS * 0.4)
			var mid := (bottom + top) / 2.0
			var size := Vector3(thick, top - bottom, length) if west else Vector3(length, top - bottom, thick)
			var centre := Vector3(x, mid, y + 0.5) if west else Vector3(x + 0.5, mid, y)
			var place := Transform3D(Basis.from_scale(size), centre)
			if what == "glass":
				glass.append(place)
			else:
				solid.append(place)
				colours.append({"wall": WALL, "door": DOOR, "fence": FENCE}[what].srgb_to_linear())
		# Boards nailed across a door or window, sticking out a little on both sides.
		var opening := Vector2(0.2, DOOR_TOP - 0.15) if kind in [EDGE_DOOR_SHUT, EDGE_DOOR_OPEN] \
			else Vector2(SILL, WINDOW_TOP)
		for b in boards:
			var mid := opening.x + (b + 0.5) * (opening.y - opening.x) / BOARD_SLOTS
			if mid > height:
				continue
			var size := Vector3(WALL_THICKNESS * 1.8, 0.15, 1.08) if west else Vector3(1.08, 0.15, WALL_THICKNESS * 1.8)
			var tilt := deg_to_rad(BOARD_TILT if b % 2 == 0 else -BOARD_TILT)
			var turn := Basis(Vector3.RIGHT, tilt) if west else Basis(Vector3.BACK, tilt)
			var centre := Vector3(x, mid, y + 0.5) if west else Vector3(x + 0.5, mid, y)
			planks.append(Transform3D(turn * Basis.from_scale(size), centre))
			plank_colours.append(BOARD.darkened(_shade(x + b, y)).srgb_to_linear())
	_add_cut_away(_walls, BoxMesh.new(), solid, colours)
	# Boards aren't cut away, as glass isn't: the ones you've just nailed are right beside you,
	# where the cut-away is, and you want to see them.
	if not planks.is_empty():
		_walls.add_child(_instances(BoxMesh.new(), planks, plank_colours, _materials["plain"]))
	var white := PackedColorArray()
	white.resize(glass.size())
	white.fill(Color.WHITE)
	var panes := _instances(BoxMesh.new(), glass, white, _materials["glass"])
	panes.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_walls.add_child(panes)


func _build_ground(tiles: PackedInt32Array, origin: Vector2i, size: int) -> void:
	var image := Image.create_empty(size, size, false, Image.FORMAT_RGBA8)
	for y in size:
		for x in size:
			var ground := tiles[y * size + x] & 15
			var colour: Color = GROUND[ground].darkened(_shade(origin.x + x, origin.y + y))
			colour.a = 0.5 if ground == 4 or ground == 6 else 1.0  # water shines, shallow or deep
			image.set_pixel(x, y, colour)
	var material := ShaderMaterial.new()
	material.shader = preload("res://shaders/ground.gdshader")
	material.set_shader_parameter("ground", ImageTexture.create_from_image(image))
	material.set_shader_parameter("origin", Vector2(origin))
	material.set_shader_parameter("size", float(size))
	var plane := PlaneMesh.new()
	plane.size = Vector2(size, size)
	var square := MeshInstance3D.new()
	square.mesh = plane
	square.material_override = material
	square.position = Vector3(origin.x + size / 2.0, 0.0, origin.y + size / 2.0)
	square.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(square)


func _build_fixtures(tiles: PackedInt32Array, origin: Vector2i, size: int,
		names: PackedStringArray) -> void:
	var trunks: Array[Transform3D] = []
	var trunk_colours := PackedColorArray()
	var crowns: Array[Transform3D] = []
	var crown_colours := PackedColorArray()
	var boxes: Array[Transform3D] = []
	var box_colours := PackedColorArray()
	for y in size:
		for x in size:
			var fixture := tiles[y * size + x] >> 4
			if fixture <= 0 or fixture >= names.size():
				continue
			var tx := origin.x + x
			var ty := origin.y + y
			var centre := Vector3(tx + 0.5, 0.0, ty + 0.5)
			var shade := _shade(tx, ty)
			if names[fixture] == "tree":
				# A little variety from tile to tile: height, girth, the green.
				var grow := 0.85 + 2.0 * shade * 3.0
				trunks.append(Transform3D(Basis.from_scale(Vector3(1, grow, 1)), centre + Vector3(0, 0.7 * grow, 0)))
				trunk_colours.append(TRUNK.srgb_to_linear())
				var crown := Basis.from_euler(Vector3(0, shade * 40.0, 0)).scaled(Vector3(grow, grow, grow))
				crowns.append(Transform3D(crown, centre + Vector3(0, 1.9 * grow, 0)))
				crown_colours.append(LEAVES.lightened(shade * 2.5).srgb_to_linear())
			else:
				var look: Array = FURNITURE.get(names[fixture], [Vector3(0.75, 0.7, 0.75), Color(0.5, 0.39, 0.25)])
				var box: Vector3 = look[0]
				boxes.append(Transform3D(Basis.from_scale(box), centre + Vector3(0, box.y / 2.0, 0)))
				box_colours.append((look[1] as Color).darkened(shade).srgb_to_linear())
	if not trunks.is_empty():
		var trunk := CylinderMesh.new()
		trunk.top_radius = 0.09
		trunk.bottom_radius = 0.14
		trunk.height = 1.4
		trunk.radial_segments = 6
		trunk.rings = 1
		add_child(_instances(trunk, trunks, trunk_colours, _materials["plain"]))
		var leaves := SphereMesh.new()
		leaves.radius = 0.85
		leaves.height = 1.5
		leaves.radial_segments = 7
		leaves.rings = 4
		_add_cut_away(self, leaves, crowns, crown_colours)
	if not boxes.is_empty():
		_add_cut_away(self, BoxMesh.new(), boxes, box_colours)


## Instances drawn with the cut-away material, plus a shadow-only copy: a hole cut for the
## camera mustn't let the sun in.
func _add_cut_away(parent: Node3D, mesh: Mesh, places: Array[Transform3D],
		colours: PackedColorArray) -> void:
	var seen := _instances(mesh, places, colours, _materials["cut_away"])
	seen.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(seen)
	var shadows := MultiMeshInstance3D.new()
	shadows.multimesh = seen.multimesh
	shadows.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_SHADOWS_ONLY
	parent.add_child(shadows)


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


## A little variation from tile to tile, the same every time: 0 to 0.045.
func _shade(x: int, y: int) -> float:
	return float(posmod(x * 73 + y * 151 + posmod(x * y, 7) * 31, 17)) / 17.0 * 0.045
