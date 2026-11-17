extends RefCounted
## The GUI's dress, as Diablo's and Path of Exile's (the user, 2026-11-17: "a more artistic
## rendering ... similar to Diablo/poe"; VISUALS.md §4, The GUI's dress, and §11): what panels,
## cells, slots, buttons, cards, bars and rules are drawn with, in place of flat colour.
##
## - **Frame**, a panel's: a warm near-black ground with a grain, darker toward its edges, in a bronze
##   band with a gold hairline inside it and a plate set with a garnet at each corner.
## - **Well**, a cell's or a slot's (the inventory's, the bar's): sunk into the ground, shadowed at
##   its top and left and lit at its foot; an edge lit in a colour when asked (where a thing held
##   would go, a spell readied).
## - **Plate**, a button's (bronze) or a card's header (a grey made the colour asked for).
## - **Glow**, a thing's tile: its rarity's colour welling up behind its icon.
## - **Fill**, a bar's: its colour, lit along its top and shaded along its foot.
## - **Rule**: a line fading out either side of a gold ornament, between a panel's parts.
##
## The art is SVG in icons/ui/, drawn by hand (imported at twice its canvas, like every icon); the
## grain and the glows are made as the game starts. Nothing is downloaded.


## What the pieces are drawn from, and how.
class Paint:
	## Icons are imported at twice their canvas (project.godot's importer_defaults).
	const SCALE := 2.0
	const FRAME := preload("res://icons/ui/frame.svg")
	const WELL := preload("res://icons/ui/well.svg")
	const BUTTON := preload("res://icons/ui/button.svg")
	const BANNER := preload("res://icons/ui/banner.svg")
	const ORNAMENT := preload("res://icons/ui/divider.svg")

	static var _grain: NoiseTexture2D
	static var _vignette: GradientTexture2D
	static var _glow: GradientTexture2D

	## A panel's grain: a soft, warm mottle, seamless as it tiles (made once).
	static func grain() -> Texture2D:
		if _grain == null:
			var noise := FastNoiseLite.new()
			noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
			noise.frequency = 0.03
			noise.fractal_octaves = 4
			var ramp := Gradient.new()
			ramp.offsets = PackedFloat32Array([0.0, 0.55, 1.0])
			ramp.colors = PackedColorArray([Color(0, 0, 0, 0.35), Color(0, 0, 0, 0),
				Color("#6b5236", 0.35)])
			_grain = NoiseTexture2D.new()
			_grain.width = 256
			_grain.height = 256
			_grain.seamless = true
			_grain.noise = noise
			_grain.color_ramp = ramp
		return _grain

	## A panel's ground darkening toward its edges (made once, stretched over the panel).
	static func vignette() -> Texture2D:
		if _vignette == null:
			var g := Gradient.new()
			g.offsets = PackedFloat32Array([0.0, 0.62, 1.0])
			g.colors = PackedColorArray([Color(0, 0, 0, 0), Color(0, 0, 0, 0), Color(0, 0, 0, 0.6)])
			_vignette = GradientTexture2D.new()
			_vignette.fill = GradientTexture2D.FILL_SQUARE
			_vignette.fill_from = Vector2(0.5, 0.5)
			_vignette.fill_to = Vector2(1.0, 0.5)
			_vignette.width = 64
			_vignette.height = 64
			_vignette.gradient = g
		return _vignette

	## A tile's glow: white at its middle fading to nothing, tinted as it's drawn (made once).
	static func glow() -> Texture2D:
		if _glow == null:
			var g := Gradient.new()
			g.offsets = PackedFloat32Array([0.0, 1.0])
			g.colors = PackedColorArray([Color(1, 1, 1, 1), Color(1, 1, 1, 0)])
			_glow = GradientTexture2D.new()
			_glow.fill = GradientTexture2D.FILL_RADIAL
			_glow.fill_from = Vector2(0.5, 0.55)
			_glow.fill_to = Vector2(0.5, 0.0)
			_glow.width = 64
			_glow.height = 64
			_glow.gradient = g
		return _glow

	## `texture` over `rect` in nine pieces: its corners (`corner` units of its canvas) as they
	## are, its edges stretched along themselves, its middle stretched, or left out.
	static func nine(item: RID, rect: Rect2, texture: Texture2D, corner: float, middle := true,
			modulate := Color.WHITE) -> void:
		var t := texture.get_size()
		var c := corner * SCALE
		var d := minf(corner, minf(rect.size.x, rect.size.y) / 2.0)
		var xs := [0.0, c, t.x - c, t.x]
		var ys := [0.0, c, t.y - c, t.y]
		var xd := [rect.position.x, rect.position.x + d, rect.end.x - d, rect.end.x]
		var yd := [rect.position.y, rect.position.y + d, rect.end.y - d, rect.end.y]
		for j in 3:
			for i in 3:
				if i == 1 and j == 1 and not middle:
					continue
				var dst := Rect2(xd[i], yd[j], xd[i + 1] - xd[i], yd[j + 1] - yd[j])
				if dst.size.x <= 0.0 or dst.size.y <= 0.0:
					continue
				var src := Rect2(xs[i], ys[j], xs[i + 1] - xs[i], ys[j + 1] - ys[j])
				RenderingServer.canvas_item_add_texture_rect_region(item, dst, texture.get_rid(),
					src, modulate)

	## `texture` over `rect` in three pieces across: its ends (`end` units of its canvas, scaled
	## to the height) as they are, its middle stretched.
	static func three(item: RID, rect: Rect2, texture: Texture2D, end: float,
			modulate := Color.WHITE) -> void:
		var t := texture.get_size()
		var c := end * SCALE
		var d := minf(c * rect.size.y / t.y, rect.size.x / 2.0)
		var xs := [0.0, c, t.x - c, t.x]
		var xd := [rect.position.x, rect.position.x + d, rect.end.x - d, rect.end.x]
		for i in 3:
			var dst := Rect2(xd[i], rect.position.y, xd[i + 1] - xd[i], rect.size.y)
			if dst.size.x > 0.0:
				RenderingServer.canvas_item_add_texture_rect_region(item, dst, texture.get_rid(),
					Rect2(xs[i], 0.0, xs[i + 1] - xs[i], t.y), modulate)

	## A `width` px line round the inside of `rect`, in `colour`.
	static func outline(item: RID, rect: Rect2, colour: Color, width: float) -> void:
		var w := width
		var rs := RenderingServer
		rs.canvas_item_add_rect(item, Rect2(rect.position, Vector2(rect.size.x, w)), colour)
		rs.canvas_item_add_rect(item, Rect2(Vector2(rect.position.x, rect.end.y - w),
			Vector2(rect.size.x, w)), colour)
		rs.canvas_item_add_rect(item, Rect2(rect.position + Vector2(0, w),
			Vector2(w, rect.size.y - 2.0 * w)), colour)
		rs.canvas_item_add_rect(item, Rect2(Vector2(rect.end.x - w, rect.position.y + w),
			Vector2(w, rect.size.y - 2.0 * w)), colour)


## A panel's: the ground (`bg_color`: its colour and how solid, `Look.PANEL` or its solid kind),
## its grain and its darker edges, then the frame over them, its corners `corner` px.
class Frame extends StyleBox:
	var bg_color := Color("#140f0be6")
	var corner := 24.0

	func _init() -> void:
		content_margin_left = 18.0
		content_margin_right = 18.0
		content_margin_top = 16.0
		content_margin_bottom = 16.0

	func _draw(item: RID, rect: Rect2) -> void:
		var inside := rect.grow(-3.0)
		var rs := RenderingServer
		rs.canvas_item_add_rect(item, inside, bg_color)
		rs.canvas_item_add_texture_rect(item, inside, Paint.grain().get_rid(), true,
			Color(1, 1, 1, bg_color.a))
		rs.canvas_item_add_texture_rect(item, inside, Paint.vignette().get_rid(), false,
			Color(1, 1, 1, bg_color.a))
		Paint.nine(item, rect, Paint.FRAME, corner, false)


## A cell's or a slot's: sunk into the ground; `edge` lit round it when it isn't clear.
class Well extends StyleBox:
	var edge := Color(0, 0, 0, 0)
	var width := 2.0
	var corner := 12.0

	func _init() -> void:
		content_margin_left = 4.0
		content_margin_right = 4.0
		content_margin_top = 4.0
		content_margin_bottom = 4.0

	func _draw(item: RID, rect: Rect2) -> void:
		Paint.nine(item, rect, Paint.WELL, corner, true)
		if edge.a > 0.0:
			Paint.outline(item, rect, edge, width)


## A plate: a button's bronze (nine pieces, `corner` px), or a card's header (`ends`: three
## pieces, pointed), made `modulate`'s colour.
class Plate extends StyleBox:
	var texture: Texture2D = Paint.BUTTON
	var corner := 10.0
	var ends := false
	var modulate := Color.WHITE

	func _init() -> void:
		content_margin_left = 12.0
		content_margin_right = 12.0
		content_margin_top = 4.0
		content_margin_bottom = 5.0

	func _draw(item: RID, rect: Rect2) -> void:
		if ends:
			Paint.three(item, rect, texture, corner, modulate)
		else:
			Paint.nine(item, rect, texture, corner, true, modulate)


## A thing's tile: `colour` welling up behind its icon (its rarity's, or a faint warm one for a
## plain thing), and `edge` round it when it isn't clear.
class Glow extends StyleBox:
	var colour := Color(1, 1, 1, 0.1)
	var edge := Color(0, 0, 0, 0)

	func _draw(item: RID, rect: Rect2) -> void:
		RenderingServer.canvas_item_add_texture_rect(item, rect, Paint.glow().get_rid(), false,
			colour)
		if edge.a > 0.0:
			Paint.outline(item, rect, edge, 1.0)


## A bar's fill: its colour, lit along its top and shaded along its foot.
class Fill extends StyleBox:
	var colour := Color.WHITE

	func _draw(item: RID, rect: Rect2) -> void:
		var rs := RenderingServer
		rs.canvas_item_add_rect(item, rect, colour)
		var lit := Rect2(rect.position, Vector2(rect.size.x, rect.size.y * 0.4))
		rs.canvas_item_add_rect(item, lit, Color(1, 1, 1, 0.18))
		var foot := Rect2(Vector2(rect.position.x, rect.end.y - rect.size.y * 0.25),
			Vector2(rect.size.x, rect.size.y * 0.25))
		rs.canvas_item_add_rect(item, foot, Color(0, 0, 0, 0.25))


## A rule between a panel's parts: a gold line fading out either side of an ornament.
class Rule extends Control:
	var colour := Color("#c9a466")

	func _init() -> void:
		custom_minimum_size = Vector2(0, 14)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		resized.connect(queue_redraw)

	func _draw() -> void:
		var o := Paint.ORNAMENT.get_size() / Paint.SCALE
		var y := roundf(size.y / 2.0)
		var mid := size.x / 2.0
		var gap := o.x / 2.0 + 2.0
		draw_polyline_colors(PackedVector2Array([Vector2(0, y), Vector2(mid - gap, y)]),
			PackedColorArray([Color(colour, 0.0), colour]), 1.0)
		draw_polyline_colors(PackedVector2Array([Vector2(mid + gap, y), Vector2(size.x, y)]),
			PackedColorArray([colour, Color(colour, 0.0)]), 1.0)
		draw_texture_rect(Paint.ORNAMENT, Rect2(Vector2(mid, y) - o / 2.0, o), false)
