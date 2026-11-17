extends Node3D
## Damage numbers (the first playtest: "I'd also like to see damage numbers and health bars during
## combat"; VISUALS.md §12, Effects): each blow that lands, its harm rising from where it landed and
## fading out, as Diablo's and Path of Exile's: white on the dead, red on you. With less motion they
## fade where they are. They're drawn over everything, at the size of the marks over heads.

const Look = preload("res://look.gd")
const FONT := preload("res://fonts/AlegreyaSC-Bold.ttf")

const LIFE := 0.9  # seconds each lives
const RISE := 0.8  # metres it rises over its life
const HEIGHT := 1.5  # metres over the feet it starts at: about the chest

var _numbers: Array = []  # [Label3D, its age, where it started]


## A blow's harm (`tenths` of a point) on whoever stands at `feet`, in `colour`.
func add(feet: Vector3, tenths: int, colour: Color) -> void:
	if tenths <= 0:
		return
	var label := Label3D.new()
	# Whole from ten up, as Diablo's; a tenth under ten (a bruise's "0.5").
	if tenths >= 100 or tenths % 10 == 0:
		label.text = str(roundi(tenths / 10.0))
	else:
		label.text = "%.1f" % (tenths / 10.0)
	label.font = FONT
	label.font_size = 72
	label.pixel_size = 0.007
	label.outline_size = 18
	label.outline_modulate = Look.OUTLINE
	label.modulate = colour
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.no_depth_test = true
	label.render_priority = 10
	# A little aside, so blows close together don't sit on one another.
	var start := feet + Vector3(randf_range(-0.25, 0.25), HEIGHT, randf_range(-0.25, 0.25))
	label.position = start
	add_child(label)
	_numbers.append([label, 0.0, start])


func _process(delta: float) -> void:
	for i in range(_numbers.size() - 1, -1, -1):
		var n: Array = _numbers[i]
		n[1] += delta
		var t: float = n[1] / LIFE
		var label: Label3D = n[0]
		if t >= 1.0:
			label.queue_free()
			_numbers.remove_at(i)
			continue
		var rise := 0.0 if Look.reduce_motion else RISE * (1.0 - pow(1.0 - t, 2.0))
		label.position = (n[2] as Vector3) + Vector3(0, rise, 0)
		# Whole for most of its life, fading over the last third.
		label.modulate.a = clampf((1.0 - t) * 3.0, 0.0, 1.0)
		label.outline_modulate.a = label.modulate.a * Look.OUTLINE.a
