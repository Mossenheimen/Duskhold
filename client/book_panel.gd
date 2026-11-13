extends CanvasLayer
## A book panel (VISUALS.md §10, zone J: skills, making, spells): a panel at the left middle, one
## open at a time (main.gd's `_book`), drawn only while it's open and only when what it shows has
## changed. A panel extends this. Its `start` reads what it needs from the core once a world is
## made (after `super`), and its `refresh` asks `_awake`, then `_changed` with what it would show,
## and draws its rows into `_list` if so.

const Look = preload("res://look.gd")

var _dusk: Node
var _panel: PanelContainer
var _list: VBoxContainer  # its rows, top to bottom
var _shown: Variant = null  # what it showed as last drawn; null draws it afresh


func _ready() -> void:
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", Look.panel_style())
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 3)
	_panel.add_child(_list)
	add_child(_panel)
	_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_LEFT, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	_panel.visible = false


## A new world: drawn afresh when it next opens.
func start(dusk: Node) -> void:
	_dusk = dusk
	_shown = null


## Closed as another book panel opens: one at a time.
func close() -> void:
	_panel.visible = false


func toggle() -> void:
	_panel.visible = not _panel.visible


func is_open() -> bool:
	return _panel.visible


## Whether to look at all this frame: a world, you alive (dead, the panel closes), and it open.
func _awake(dead: bool) -> bool:
	if _dusk == null:
		return false
	if dead:
		_panel.visible = false
		return false
	return _panel.visible


## Whether `now` (what the panel would show) differs from what it last drew. If so, it's what's
## shown from now on, and the rows are cleared for drawing it.
func _changed(now: Variant) -> bool:
	if _shown != null and now == _shown:
		return false
	_shown = now
	for child in _list.get_children():
		child.queue_free()
	return true
