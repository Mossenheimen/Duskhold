extends CanvasLayer
## Making things (T): each recipe with what it takes, and a "Make" button when you have it all;
## else what's missing. And the furniture beside you, if it breaks down for planks. The core
## decides what can be made and does the making; this only shows it and asks.

const Look = preload("res://look.gd")

const READY := Look.GOOD
const SHORT := Look.TEXT_DIM

var _dusk: Node
var _panel: PanelContainer
var _list: VBoxContainer
var _texts := PackedStringArray()  # two lines a recipe (DuskWorld.recipe_texts())
var _items := PackedStringArray()  # item names, for what's missing
var _shown := []  # the checks and the furniture beside you, as last drawn


func _ready() -> void:
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", Look.panel_style())
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 3)
	_panel.add_child(_list)
	add_child(_panel)
	# Zone J, the left middle, where skills open too: one book panel at a time.
	_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER_LEFT, Control.PRESET_MODE_MINSIZE, Look.MARGIN)
	_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	_panel.visible = false


## A new world: the recipes and item names come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_texts = dusk.recipe_texts()
	_items = dusk.item_names()
	_shown = []


## Closed as the other book panel opens: one at a time (VISUALS.md §10, zone J).
func close() -> void:
	_panel.visible = false


func toggle() -> void:
	_panel.visible = not _panel.visible


func is_open() -> bool:
	return _panel.visible


## Each frame: redraw if anything changed (only while it's open).
func refresh(dead: bool) -> void:
	if _dusk == null:
		return
	if dead:
		_panel.visible = false
		return
	if not _panel.visible:
		return
	var now := [_dusk.craft_checks(), _dusk.break_check()]
	if now == _shown:
		return
	_shown = now
	_draw(now[0], now[1])


func _draw(checks: PackedInt32Array, furniture: PackedInt32Array) -> void:
	for child in _list.get_children():
		child.queue_free()
	_list.add_child(_label("Make  ·  from what you carry", Look.HEADING))
	for r in range(_texts.size() / 2):
		var code := checks[r * 2] if r * 2 < checks.size() else -3
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var lines := VBoxContainer.new()
		lines.add_theme_constant_override("separation", 0)
		lines.custom_minimum_size = Vector2(300, 0)
		row.add_child(lines)
		# Its name, and beside it why it can't be made yet: the line under it can run long.
		var top := HBoxContainer.new()
		top.add_theme_constant_override("separation", 10)
		lines.add_child(top)
		var called := _label(_texts[r * 2], Look.BODY)
		called.add_theme_color_override("font_color", READY if code == 0 else Color.WHITE)
		top.add_child(called)
		var takes := _label(_texts[r * 2 + 1], Look.SMALL)
		takes.modulate = SHORT
		lines.add_child(takes)
		if code == 0:
			row.add_child(_button("Make", func() -> void: _dusk.craft(r)))
		else:
			var why := _label(_why(code, checks[r * 2 + 1]), Look.SMALL)
			why.modulate = SHORT
			why.size_flags_vertical = Control.SIZE_SHRINK_END
			top.add_child(why)
	_list.add_child(_label(" ", 6))
	if furniture.size() >= 4:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_list.add_child(row)
		var line := _label("Break it down: %d plank%s (loud)" % [furniture[2], "" if furniture[2] == 1 else "s"], Look.SMALL)
		line.custom_minimum_size = Vector2(300, 0)
		row.add_child(line)
		row.add_child(_button("Break", func() -> void: _dusk.break_down()))
	elif furniture.size() >= 1 and furniture[0] == -2:
		_list.add_child(_label("A hammer or an axe would break the furniture here down for planks.", Look.SMALL))
	else:
		var hint := _label("Stand by a table, a chair, a bed or a pew to break it down for planks.", Look.SMALL)
		hint.modulate = SHORT
		_list.add_child(hint)


## Why a recipe can't be made: none of its tools, short of an item, or no fire to cook on.
func _why(code: int, item: int) -> String:
	match code:
		-1:
			return "no tool for it"
		-2:
			return "needs more %s" % (_items[item] if item < _items.size() else "of something")
		-4:
			return "needs a lit fire beside you"
	return ""


func _button(text: String, pressed: Callable) -> Button:
	var button := Button.new()
	button.text = text
	button.add_theme_font_size_override("font_size", Look.SMALL)
	button.focus_mode = Control.FOCUS_NONE  # keys stay with the game
	button.pressed.connect(pressed)
	return button


func _label(text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	Look.sized(label, size)
	return label
