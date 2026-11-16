extends Node3D
## One chunk of the map, as the core gives it: its ground as one textured square, its fixtures
## (trees, plants, furniture: each its look from data/looks.toml, through looks.gd), and the
## walls, doors, windows and fences on its tiles' edges.

const WALL_HEIGHT := 2.2
const LOW_WALL_HEIGHT := 0.35
const WALL_THICKNESS := 0.12
const DOOR_TOP := 1.85
const SILL := 0.9
const WINDOW_TOP := 1.65
const FENCE_HEIGHT := 0.9

const Look = preload("res://look.gd")
const Looks = preload("res://looks.gd")
## Room across an opening for this many boards; each one tilts a little, nailed by hand.
const BOARD_SLOTS := 4
const BOARD_TILT := 9.0
## What things are made of, for their look (shaders/wall.gdshader draws each, as a code over 16 in
## an instance's custom data): codes.gd's Surface.
const Surface = preload("res://codes.gd").Surface
const Placed = preload("res://codes.gd").Placed
const Side = preload("res://codes.gd").Side
## Where a fixture's back goes, by its tile's walls (DuskWorld.chunk_walls()), the far walls first:
## north, west, east, then south. Each with the way out through it, in tiles.
const BACKS := [[Side.NORTH, Vector2(0, -1)], [Side.WEST, Vector2(-1, 0)],
	[Side.EAST, Vector2(1, 0)], [Side.SOUTH, Vector2(0, 1)]]

const SIDE_WEST := 1
## An edge's kind, by name (codes.gd).
const Edge = preload("res://codes.gd").Edge
## A broken window keeps a rim of glass in its frame, top and bottom: what cuts you climbing.
const SHARDS := 0.1

var _edges := PackedInt32Array()
var _materials: Dictionary
var _walls: Node3D
var _dusk: Node
var _chunk := Vector2i.ZERO
var _low_walls := false
var _fixtures: Node3D  # the furniture and trees, rebuilt when a piece is broken up
var _names := PackedStringArray()
var _fires := {}  # Vector2i: where the embers of a fire laid in the fixture there sit, in the world


## Where a fire laid in the fixture on `tile` burns (its look's fire, turned and placed as it
## stands), or null if it takes none.
func fire_spot(tile: Vector2i) -> Variant:
	return _fires.get(tile)


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
	_build_ground(origin, size)
	_names = names
	_fixtures = Node3D.new()
	add_child(_fixtures)
	_build_fixtures(tiles, origin, size, names)
	_build_scatter(tiles, origin, size)
	_walls = Node3D.new()
	add_child(_walls)
	build_walls(low_walls)


## A tile changed in the core (furniture broken up): its furniture is drawn again.
func refresh_fixtures() -> void:
	for child in _fixtures.get_children():
		child.queue_free()
	var size: int = _dusk.chunk_size()
	var tiles: PackedInt32Array = _dusk.chunk_tiles(_chunk.x, _chunk.y)
	_build_fixtures(tiles, Vector2i(_chunk.x * size, _chunk.y * size), size, _names)


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
	var surfaces := PackedFloat32Array()
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
			Edge.WALL:
				pieces = [[0.0, WALL_HEIGHT, "wall"]]
			Edge.DOOR_SHUT:
				pieces = [[DOOR_TOP, WALL_HEIGHT, "wall"], [0.0, DOOR_TOP, "door"]]
			Edge.DOOR_OPEN:
				pieces = [[DOOR_TOP, WALL_HEIGHT, "wall"]]
			Edge.WINDOW_SHUT:
				pieces = [[0.0, SILL, "wall"], [WINDOW_TOP, WALL_HEIGHT, "wall"], [SILL, WINDOW_TOP, "glass"]]
			Edge.WINDOW_OPEN:
				pieces = [[0.0, SILL, "wall"], [WINDOW_TOP, WALL_HEIGHT, "wall"]]
			Edge.WINDOW_BROKEN:
				pieces = [[0.0, SILL, "wall"], [WINDOW_TOP, WALL_HEIGHT, "wall"],
					[SILL, SILL + SHARDS, "glass"], [WINDOW_TOP - SHARDS, WINDOW_TOP, "glass"]]
			Edge.FENCE:
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
				colours.append({"wall": Look.WALL, "door": Look.DOOR, "fence": Look.FENCE}[what].srgb_to_linear())
				surfaces.append(Surface.PLASTER if what == "wall" else Surface.WOOD)
		# Boards nailed across a door or window, sticking out a little on both sides.
		var opening := Vector2(0.2, DOOR_TOP - 0.15) if kind in [Edge.DOOR_SHUT, Edge.DOOR_OPEN] \
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
			plank_colours.append(Look.BOARD.darkened(_shade(x + b, y)).srgb_to_linear())
	_add_cut_away(_walls, BoxMesh.new(), solid, colours, surfaces)
	# Boards aren't cut away, as glass isn't: the ones you've just nailed are right beside you,
	# where the cut-away is, and you want to see them.
	if not planks.is_empty():
		var wood := PackedFloat32Array()
		wood.resize(planks.size())
		wood.fill(Surface.WOOD)
		_walls.add_child(_instances(BoxMesh.new(), planks, plank_colours, _materials["solid"], wood))
	var white := PackedColorArray()
	white.resize(glass.size())
	white.fill(Color.WHITE)
	var panes := _instances(BoxMesh.new(), glass, white, _materials["glass"])
	panes.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_walls.add_child(panes)


## The ground: a texel a tile, its colour and its kind (ground.gdshader draws its surface), with a
## tile of the next chunks' ground all round (DuskWorld.chunk_grounds()), so the borders between
## grounds can wander across the chunk's edge without a seam.
func _build_ground(origin: Vector2i, size: int) -> void:
	var grounds: PackedByteArray = _dusk.chunk_grounds(_chunk.x, _chunk.y)
	var side := size + 2
	var image := Image.create_empty(side, side, false, Image.FORMAT_RGBA8)
	for y in side:
		for x in side:
			var ground: int = grounds[y * side + x] if y * side + x < grounds.size() else 0
			var colour: Color = Look.GROUND[ground].darkened(_shade(origin.x + x - 1, origin.y + y - 1))
			colour.a = (ground + 0.5) / 16.0  # its kind, for its surface (ground.gdshader)
			image.set_pixel(x, y, colour)
	var material := ShaderMaterial.new()
	material.shader = preload("res://shaders/ground.gdshader")
	material.set_shader_parameter("ground", ImageTexture.create_from_image(image))
	material.set_shader_parameter("origin", Vector2(origin))
	material.set_shader_parameter("size", float(size))
	material.set_shader_parameter("margin", 1.0)
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
	_fires.clear()
	var walls: PackedByteArray = _dusk.chunk_walls(_chunk.x, _chunk.y)
	# Each look's variant drawn here, as one MultiMesh: "id:variant" -> [id, variant, places, tints].
	var groups := {}
	for y in size:
		for x in size:
			var fixture := tiles[y * size + x] >> 4
			if fixture <= 0 or fixture >= names.size():
				continue
			var id := names[fixture]
			var look := Looks.fixture(id)
			var tx := origin.x + x
			var ty := origin.y + y
			var variant := _hash(tx, ty, 1) % (look.variants as Array).size()
			var grow: Vector2 = look.grow
			var turn := Basis(Vector3.UP, _yaw(look, tiles, walls, Vector2i(x, y), size, names))
			var place := Transform3D(turn.scaled(Vector3.ONE * lerpf(grow.x, grow.y, _rand(tx, ty, 2))),
				Vector3(tx + 0.5, 0.0, ty + 0.5))
			var key := "%s:%d" % [id, variant]
			if not groups.has(key):
				groups[key] = [id, variant, [] as Array[Transform3D], []]
			groups[key][2].append(place)
			var tint := 1.0 + (_rand(tx, ty, 3) - 0.5) * 2.0 * float(look.vary)
			groups[key][3].append(Color(tint, tint, tint))
			if look.has("fire"):
				_fires[Vector2i(tx, ty)] = place * (look.fire as Vector3)
	for group: Array in groups.values():
		var seen := _instances(Looks.fixture_mesh(group[0], group[1]), group[2],
			PackedColorArray(group[3]), _materials["kit"])
		seen.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		_fixtures.add_child(seen)
		var shadows := MultiMeshInstance3D.new()
		shadows.multimesh = seen.multimesh
		shadows.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_SHADOWS_ONLY
		_fixtures.add_child(shadows)


## How a fixture's turned on its tile (looks.toml): its front toward what it `faces`, or its back
## toward what it `backs`, if one's within its reach in this chunk; else as it's `placed`: its back
## to a wall of its tile (the far walls first), its front south, or a quarter turn by its place and
## a little more. A turn about y, in radians: its model's front is -z.
func _yaw(look: Dictionary, tiles: PackedInt32Array, walls: PackedByteArray, at: Vector2i,
		size: int, names: PackedStringArray) -> float:
	for which in ["faces", "backs"]:
		var target: String = look.get(which, "")
		if target.is_empty():
			continue
		var way := _nearest(target, tiles, at, size, int(look.reach), names)
		if way != Vector2.ZERO:
			return _front_to(way if which == "faces" else -way)
	var tx := _chunk.x * size + at.x
	var ty := _chunk.y * size + at.y
	match int(look.placed):
		Placed.LOOSE:
			return (_hash(tx, ty, 4) % 4) * PI / 2.0 + (_rand(tx, ty, 5) - 0.5) * deg_to_rad(20.0)
		Placed.WALL:
			var bits := walls[at.y * size + at.x] if at.y * size + at.x < walls.size() else 0
			for back: Array in BACKS:
				if bits & int(back[0]):
					return _front_to(-(back[1] as Vector2))
	return _front_to(Vector2(0, 1))


## The turn about y that sets a model's front (-z) toward `way` (x east, y south, in tiles).
static func _front_to(way: Vector2) -> float:
	return atan2(-way.x, -way.y)


## What's strewn on the ground (looks.toml's [[scatter]]: tufts, stones, reeds): only a look, on
## tiles with no fixture, each where its tile's own numbers put it, a little in from the edges.
## Each scatter's variant is one MultiMesh, casting no shadow (there are thousands).
func _build_scatter(tiles: PackedInt32Array, origin: Vector2i, size: int) -> void:
	var strewn: Array = Looks.scatter()
	if strewn.is_empty():
		return
	var groups := {}  # "k:variant" -> [k, variant, places, tints]
	for y in size:
		for x in size:
			var tile := tiles[y * size + x]
			if tile >> 4 != 0:
				continue
			var tx := origin.x + x
			var ty := origin.y + y
			for k in strewn.size():
				var scatter: Dictionary = strewn[k]
				if not (scatter.on as PackedInt32Array).has(tile & 15) \
						or _rand(tx, ty, 100 + k) >= float(scatter.chance):
					continue
				var count: Vector2i = scatter.count
				var grow: Vector2 = scatter.grow
				var variants: int = (scatter.variants as Array).size()
				for j in count.x + _hash(tx, ty, 200 + k) % (count.y - count.x + 1):
					var salt := 1000 + 64 * k + 8 * j
					var at := Vector3(tx + 0.18 + 0.64 * _rand(tx, ty, salt),
						0.0, ty + 0.18 + 0.64 * _rand(tx, ty, salt + 1))
					var turn := Basis(Vector3.UP, TAU * _rand(tx, ty, salt + 2))
					var place := Transform3D(turn.scaled(Vector3.ONE * lerpf(grow.x, grow.y,
						_rand(tx, ty, salt + 3))), at)
					var variant := _hash(tx, ty, salt + 4) % variants
					var key := "%d:%d" % [k, variant]
					if not groups.has(key):
						groups[key] = [k, variant, [] as Array[Transform3D], []]
					groups[key][2].append(place)
					var tint := 1.0 + (_rand(tx, ty, salt + 5) - 0.5) * 2.0 * float(scatter.vary)
					groups[key][3].append(Color(tint, tint, tint))
	for group: Array in groups.values():
		var strew := _instances(Looks.scatter_mesh(group[0], group[1]), group[2],
			PackedColorArray(group[3]), _materials["kit"])
		strew.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(strew)


## The way to the nearest tile of fixture `target` within `reach` of `at` in this chunk, as one of
## the four (zero if there's none). One in a row of its own kind (pews, chairs at a long table)
## faces across its row, toward it; one alone, along whichever axis it's further, north or south
## when it's as far both ways.
func _nearest(target: String, tiles: PackedInt32Array, at: Vector2i, size: int, reach: int,
		names: PackedStringArray) -> Vector2:
	for ring in range(1, reach + 1):
		for dy in range(-ring, ring + 1):
			for dx in range(-ring, ring + 1):
				if maxi(absi(dx), absi(dy)) != ring:
					continue
				var x := at.x + dx
				var y := at.y + dy
				if x < 0 or y < 0 or x >= size or y >= size:
					continue
				var fixture := tiles[y * size + x] >> 4
				if fixture > 0 and fixture < names.size() and names[fixture] == target:
					var own := tiles[at.y * size + at.x] >> 4
					var along_x := _is(tiles, at + Vector2i(-1, 0), size, own) \
						or _is(tiles, at + Vector2i(1, 0), size, own)
					var along_y := _is(tiles, at + Vector2i(0, -1), size, own) \
						or _is(tiles, at + Vector2i(0, 1), size, own)
					var across_y := absi(dx) <= absi(dy)
					if along_x != along_y:
						across_y = along_x
					if across_y and dy != 0:
						return Vector2(0, signi(dy))
					return Vector2(signi(dx), 0) if dx != 0 else Vector2(0, signi(dy))
	return Vector2.ZERO


## Whether the tile `at` of this chunk holds fixture number `fixture`.
static func _is(tiles: PackedInt32Array, at: Vector2i, size: int, fixture: int) -> bool:
	return at.x >= 0 and at.y >= 0 and at.x < size and at.y < size \
		and tiles[at.y * size + at.x] >> 4 == fixture


## A tile's own number for `salt`, the same every time: what it takes of a look's variants, turns
## and sizes.
static func _hash(x: int, y: int, salt: int) -> int:
	var h := (x * 73856093) ^ (y * 19349663) ^ (salt * 83492791)
	h = (h ^ (h >> 13)) * 1274126177
	return absi(h ^ (h >> 16))


## The same as a number from 0 to 1.
static func _rand(x: int, y: int, salt: int) -> float:
	return float(_hash(x, y, salt) % 10007) / 10007.0

## Instances drawn with the cut-away material, plus a shadow-only copy: a hole cut for the
## camera mustn't let the sun in.
func _add_cut_away(parent: Node3D, mesh: Mesh, places: Array[Transform3D],
		colours: PackedColorArray, surfaces := PackedFloat32Array()) -> void:
	var seen := _instances(mesh, places, colours, _materials["cut_away"], surfaces)
	seen.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(seen)
	var shadows := MultiMeshInstance3D.new()
	shadows.multimesh = seen.multimesh
	shadows.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_SHADOWS_ONLY
	parent.add_child(shadows)


## Many copies of one mesh, each placed and coloured (colours in linear space), and each of a
## surface (PLASTER, WOOD...: plain if none are given).
func _instances(mesh: Mesh, places: Array[Transform3D], colours: PackedColorArray,
		material: Material, surfaces := PackedFloat32Array()) -> MultiMeshInstance3D:
	var multimesh := MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.use_colors = true
	multimesh.use_custom_data = true
	multimesh.mesh = mesh
	multimesh.instance_count = places.size()
	for i in places.size():
		multimesh.set_instance_transform(i, places[i])
		multimesh.set_instance_color(i, colours[i])
		var surface: float = surfaces[i] if i < surfaces.size() else float(Surface.PLAIN)
		multimesh.set_instance_custom_data(i, Color(surface / 16.0, 0.0, 0.0, 0.0))
	var node := MultiMeshInstance3D.new()
	node.multimesh = multimesh
	node.material_override = material
	return node


## A little variation from tile to tile, the same every time: 0 to 0.045.
func _shade(x: int, y: int) -> float:
	return float(posmod(x * 73 + y * 151 + posmod(x * y, 7) * 31, 17)) / 17.0 * 0.045
