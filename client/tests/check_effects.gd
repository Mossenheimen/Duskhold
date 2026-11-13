extends SceneTree
## check.cmd's: every effect in effects.gd, made once with no window, then left to play out. Any
## script error shows in the log, and one that never cleans up after itself is caught: after three
## seconds every effect must be gone. Prints "EFFECTS OK", or "EFFECTS LEFT n".
##   Godot_..._console.exe --headless --path client -s res://tests/check_effects.gd

const Effects = preload("res://effects.gd")
const Look = preload("res://look.gd")
const Spell = preload("res://codes.gd").Spell

var _fx: Node3D


func _initialize() -> void:
	_fx = Effects.new()
	root.add_child(_fx)
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
	_fx.sneak_attack(at)
	_fx.bone(at, true)
	_fx.got_up(at)
	_fx.risen(at)
	_fx.light_eaten(at)
	_fx.splinters(at, Look.DOOR)
	Look.reduce_motion = true  # and once with fewer flashes: no flash light to free
	_fx.spell(Spell.FIREBALL, 5, 5, 2, Vector3(1, 0, 1))
	Look.reduce_motion = false
	create_timer(3.0).timeout.connect(_done)


func _done() -> void:
	var left := _fx.get_child_count()
	print("EFFECTS OK" if left == 0 else "EFFECTS LEFT %d" % left)
	quit()
