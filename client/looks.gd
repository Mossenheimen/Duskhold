extends RefCounted
## The kit (VISUALS.md §5): everything made of data/looks.toml's parts, as DuskWorld.looks() hands
## them over. A look's variant is baked once into one mesh, flat-shaded, each part's colour and
## surface in its vertices (looks.gdshader draws them), and everything drawn of it shares that
## mesh: a chunk's tables are one MultiMesh of it. dusk-data's looks.rs makes the same corners, so
## what it works out from a look (a silhouette, an icon) is what's on screen.

const Codes = preload("res://codes.gd")
const Shape = Codes.Shape
const Look = preload("res://look.gd")

## Numbers a part is in a look's variant (dusk-data's PART_FLOATS): shape, sides, taper, size (3),
## at (3), turn (3, degrees), colour (3, sRGB), surface, cut.
const PART := 17
## A fixture with no look (no looks.toml): a crate-sized box in wood, as before there were looks.
static var NO_LOOK := {"variants": [PackedFloat32Array([0, 4, 1, 0.75, 0.7, 0.75, 0, 0.35, 0, 0, 0,
	0, 0.5, 0.39, 0.25, 2, 1])], "placed": 1, "faces": "", "backs": "", "reach": 1,
	"grow": Vector2.ONE, "vary": 0.045, "height": 0.7}

static var _looks := {}
static var _meshes := {}


## Reads the looks (DuskWorld.looks(), once its data is loaded), and the world's colours with them.
static func read(dusk: Node) -> void:
	_looks = dusk.looks()
	_meshes.clear()
	Look.take_materials(_looks.get("materials", {}))


## A material's colour by name ("wood.oak"), sRGB, or `otherwise`.
static func material(name: String, otherwise := Color.MAGENTA) -> Color:
	return _looks.get("materials", {}).get(name, otherwise)


## A fixture's look, by its id (NO_LOOK if it has none).
static func fixture(id: String) -> Dictionary:
	return _looks.get("fixture", {}).get(id, NO_LOOK)


## The fixtures with looks, by id.
static func fixtures() -> Array:
	return _looks.get("fixture", {}).keys()


## Variant `n` of fixture `id`'s look as a mesh, baked the first time it's asked for.
static func fixture_mesh(id: String, n: int) -> ArrayMesh:
	var key := "fixture:%s:%d" % [id, n]
	if not _meshes.has(key):
		var variants: Array = fixture(id).variants
		_meshes[key] = bake(variants[n % variants.size()])
	return _meshes[key]


## Parts (PART numbers each) as one mesh: every face flat, wound as Godot draws a front, its part's
## colour (linear) in the vertices' colour, and in UV2 its surface and whether it may be cut away.
static func bake(parts: PackedFloat32Array) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in range(0, parts.size() - PART + 1, PART):
		_add_part(st, parts.slice(i, i + PART))
	return st.commit()


static func _add_part(st: SurfaceTool, p: PackedFloat32Array) -> void:
	var size := Vector3(p[3], p[4], p[5])
	var at := Vector3(p[6], p[7], p[8])
	var turn := Basis.from_euler(Vector3(deg_to_rad(p[9]), deg_to_rad(p[10]), deg_to_rad(p[11])))
	var place := Transform3D(turn * Basis.from_scale(size), at)
	st.set_color(Color(p[12], p[13], p[14]).srgb_to_linear())
	st.set_uv2(Vector2(p[15], p[16]))
	for face: Array in faces(int(p[0]), int(p[1]), p[2]):
		var points: Array[Vector3] = []
		for corner: Vector3 in face:
			points.append(place * corner)
		_add_face(st, points, at)


## A face, a convex polygon given round its edge, as a fan of triangles with the face's own normal,
## each wound as Godot draws a front: clockwise seen from outside, away from `inside`.
static func _add_face(st: SurfaceTool, points: Array[Vector3], inside: Vector3) -> void:
	# Newell's normal: sound however the polygon's first corners lie.
	var normal := Vector3.ZERO
	var middle := Vector3.ZERO
	for i in points.size():
		var a := points[i]
		var b := points[(i + 1) % points.size()]
		normal += Vector3((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y))
		middle += a
	middle /= points.size()
	if normal.length_squared() < 1e-14:
		return
	if normal.dot(middle - inside) < 0.0:
		normal = -normal
	normal = normal.normalized()
	st.set_normal(normal)
	for k in range(1, points.size() - 1):
		var a := points[0]
		var b := points[k]
		var c := points[k + 1]
		if (b - a).cross(c - a).dot(normal) > 0.0:
			var swap := b
			b = c
			c = swap
		st.add_vertex(a)
		st.add_vertex(b)
		st.add_vertex(c)


## A unit shape's faces (−½ to ½ each way), each a polygon of its corners in order round its edge:
## the corners dusk-data's `unit_points` gives. Round shapes start half a side round from +x.
static func faces(shape: int, sides: int, taper: float) -> Array:
	match shape:
		Shape.BOX:
			var c: Array[Vector3] = []
			for x in [-0.5, 0.5]:
				for y in [-0.5, 0.5]:
					for z in [-0.5, 0.5]:
						c.append(Vector3(x, y, z))
			# c[i] has x from bit 2, y from bit 1, z from bit 0.
			return [[c[0], c[1], c[3], c[2]], [c[4], c[5], c[7], c[6]], [c[0], c[1], c[5], c[4]],
				[c[2], c[3], c[7], c[6]], [c[0], c[2], c[6], c[4]], [c[1], c[3], c[7], c[5]]]
		Shape.PRISM:
			var v := [Vector3(-0.5, -0.5, -0.5), Vector3(0.5, -0.5, -0.5), Vector3(0, 0.5, -0.5),
				Vector3(-0.5, -0.5, 0.5), Vector3(0.5, -0.5, 0.5), Vector3(0, 0.5, 0.5)]
			return [[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[0], v[1], v[4], v[3]],
				[v[1], v[2], v[5], v[4]], [v[2], v[0], v[3], v[5]]]
		Shape.CYLINDER:
			var bottom: Array[Vector3] = []
			var top: Array[Vector3] = []
			for k in sides:
				var a := PI / sides + TAU * k / sides
				bottom.append(Vector3(0.5 * cos(a), -0.5, 0.5 * sin(a)))
				top.append(Vector3(0.5 * cos(a) * taper, 0.5, 0.5 * sin(a) * taper))
			var out := [bottom]
			for k in sides:
				var j := (k + 1) % sides
				if taper == 0.0:
					out.append([bottom[k], bottom[j], Vector3(0, 0.5, 0)])
				else:
					out.append([bottom[k], bottom[j], top[j], top[k]])
			if taper > 0.0:
				out.append(top)
			return out
		Shape.BALL:
			var rings := maxi(floori(sides / 2.0), 2)
			var north := Vector3(0, 0.5, 0)
			var south := Vector3(0, -0.5, 0)
			var lines := []
			for i in range(1, rings):
				var polar := PI * i / rings
				var line: Array[Vector3] = []
				for k in sides:
					var a := PI / sides + TAU * k / sides
					line.append(Vector3(0.5 * cos(a) * sin(polar), 0.5 * cos(polar), 0.5 * sin(a) * sin(polar)))
				lines.append(line)
			var out := []
			for k in sides:
				var j := (k + 1) % sides
				out.append([north, lines[0][k], lines[0][j]])
				for i in range(lines.size() - 1):
					out.append([lines[i][k], lines[i][j], lines[i + 1][j], lines[i + 1][k]])
				out.append([lines[-1][k], lines[-1][j], south])
			return out
	return []
