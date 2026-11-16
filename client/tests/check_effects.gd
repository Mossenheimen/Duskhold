extends SceneTree
## check.cmd's: every effect in effects.gd, made once with no window, then left to play out; the
## lasting spells (zones.gd) shown, run out and gone; a readied spell's aim (aim.gd) shown and
## hidden. Any script error shows in the log, and one that never cleans up after itself is caught:
## after three seconds everything must be gone. Prints "EFFECTS OK", or "EFFECTS LEFT n".
##   Godot_..._console.exe --headless --path client -s res://tests/check_effects.gd

const Effects = preload("res://effects.gd")
const Look = preload("res://look.gd")
const Spell = preload("res://codes.gd").Spell
const Zones = preload("res://zones.gd")
const Aim = preload("res://aim.gd")

var _fx: Node3D
var _zones: Node3D
var _aim: Node3D
var _left := 40  # the zones' ticks left, counted down a frame at a time


func _initialize() -> void:
	_fx = Effects.new()
	root.add_child(_fx)
	_zones = Zones.new()
	root.add_child(_zones)
	_aim = Aim.new()
	root.add_child(_aim)
	process_frame.connect(_run, CONNECT_ONE_SHOT)  # once the node is in the tree


func _run() -> void:
	var at := Vector3(5, 0, 5)
	for kind: int in [Spell.LIGHT, Spell.SILENCE, Spell.FOG, Spell.FIREBALL]:
		_fx.spell(kind, 5, 5, 2, Vector3(1, 0, 1))
	_fx.spell_broke(at)
	_fx.turn(at, 8.0)
	_fx.rise(at, Look.HOLY)
	_fx.ward(5, 5, 0)
	_fx.ward(5, 5, 1)
	_fx.second_wind(at)
	_fx.gold(at, false)
	_fx.gold(at, true)
	_fx.hit(at, false, true)
	_fx.hit(at, true, false)
	_fx.block(at, Vector3(6, 0, 5))
	_fx.glance(at)
	_fx.fell(at)
	_fx.terror(at)
	_fx.sear(at, 3)
	_fx.thorns(at)
	_fx.sneak_attack(at)
	_fx.bone(at, true)
	_fx.got_up(at)
	_fx.risen(at)
	_fx.light_eaten(at)
	_fx.splinters(at, Look.DOOR)
	Look.reduce_motion = true  # and once with fewer flashes: no flash light to free
	_fx.spell(Spell.FIREBALL, 5, 5, 2, Vector3(1, 0, 1))
	Look.reduce_motion = false
	_aim.show_aim(at, 8.0, Vector2i(6, 5), 2.0, true, true)
	_aim.hide_aim()
	process_frame.connect(_run_out)
	create_timer(3.0).timeout.connect(_done)


## Each frame: a Light, a Silence and a Fog, their ticks running out to none, then gone.
func _run_out() -> void:
	_left = maxi(_left - 1, 0)
	if _left > 0:
		_zones.show_zones(PackedInt32Array([Spell.LIGHT, 5, 5, 4, _left, Spell.SILENCE, 9, 5, 3, _left,
			Spell.FOG, 13, 5, 2, _left]))
	else:
		_zones.show_zones(PackedInt32Array())


func _done() -> void:
	var left := _fx.get_child_count() + _zones.get_child_count() + (1 if _aim.visible else 0)
	print("EFFECTS OK" if left == 0 else "EFFECTS LEFT %d" % left)
	quit()
