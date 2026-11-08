extends CanvasLayer
## Zomboid's inventory windows, as lists with weights (CONCEPT.md: no grid). On the right, your
## pack: what's in your hand, and what you carry, with its weight against what you can carry
## before you can't run. On the left, what's within your reach: each container beside you, and
## the floor. Each line has its buttons: they ask the core for a timed action, which a blow breaks
## off. "Take all" asks for one take after another.

const WEAPON := 1
const BANDAGE := 2

var _dusk: Node
var _names := PackedStringArray()
var _kinds := PackedInt32Array()
var _weights := PackedInt32Array()
var _fixtures := PackedStringArray()
var _pack: VBoxContainer
var _reach: VBoxContainer
var _pack_panel: PanelContainer
var _reach_panel: PanelContainer
var _shown_pack := []  # what the pack panel shows now, to rebuild only when it changes
var _shown_reach := PackedInt32Array()
var _queue: Array = []  # takes still to ask for: [x, y, def]


func _ready() -> void:
	_pack_panel = _panel(Control.PRESET_BOTTOM_RIGHT)
	_pack = _pack_panel.get_child(0)
	_reach_panel = _panel(Control.PRESET_BOTTOM_LEFT)
	_reach_panel.position.y -= 100.0  # above the legend
	_reach = _reach_panel.get_child(0)


## A new world: the item names and the fixtures' come from the core.
func start(dusk: Node) -> void:
	_dusk = dusk
	_names = dusk.item_names()
	_kinds = dusk.item_kinds()
	_weights = dusk.item_weights()
	_fixtures = dusk.fixture_names()
	_shown_pack = []
	_shown_reach = PackedInt32Array()
	_queue.clear()


func toggle() -> void:
	_pack_panel.visible = not _pack_panel.visible


## Each frame: redraw what changed, and ask for the next queued take once you're free.
## `busy`: you're in the middle of an action; `dead`: nothing more to do.
func refresh(busy: bool, dead: bool) -> void:
	if _dusk == null:
		return
	if dead:
		_pack_panel.visible = false
		_reach_panel.visible = false
		return
	var hands: PackedInt32Array = _dusk.character_hands()
	var items: PackedInt32Array = _dusk.character_items()
	var carried: PackedInt32Array = _dusk.character_load()
	var pack := [hands, items, carried]
	if pack != _shown_pack:
		_shown_pack = pack
		_draw_pack(hands, items, carried)
	var reach: PackedInt32Array = _dusk.stores_in_reach()
	if reach != _shown_reach:
		_shown_reach = reach
		_draw_reach(reach)
	if not busy and not _queue.is_empty():
		_take_next(reach)


func _draw_pack(hands: PackedInt32Array, items: PackedInt32Array, carried: PackedInt32Array) -> void:
	for child in _pack.get_children():
		child.queue_free()
	if carried.size() < 2:
		return
	var heavy := carried[0] > carried[1]
	var title := _line(_pack, "Pack  ·  %.1f / %.1f kg%s" % [carried[0] / 1000.0, carried[1] / 1000.0,
		"  ·  too heavy to run" if heavy else ""], 17)
	if heavy:
		title.add_theme_color_override("font_color", Color(1.0, 0.55, 0.4))
	if hands.size() >= 2:
		var row := _row(_pack, "In hand: %s (%d%%)" % [_name(hands[0]), roundi(hands[1] / 10.0)])
		_button(row, "Put away", func() -> void: _dusk.unwield())
	else:
		_line(_pack, "In hand: nothing (bare fists)", 15)
	var slot := -1
	for i in range(0, items.size(), 3):
		slot += 1
		var def := items[i]
		var count := items[i + 1]
		var row := _row(_pack, _describe(def, count))
		match _kind(def):
			WEAPON:
				_button(row, "Wield", func() -> void: _dusk.wield(slot, def))
			BANDAGE:
				_button(row, "Use", func() -> void: _dusk.use_item(slot, def))
		_button(row, "Drop", func() -> void: _dusk.drop_item(slot, def))


func _draw_reach(reach: PackedInt32Array) -> void:
	for child in _reach.get_children():
		child.queue_free()
	_reach_panel.visible = reach.size() > 0
	var i := 0
	while i + 4 <= reach.size():
		var x := reach[i]
		var y := reach[i + 1]
		var fixture := reach[i + 2]
		var n := reach[i + 3]
		i += 4
		var place := "On the ground" if fixture == 0 else _fixture_name(fixture).capitalize()
		var header := _row(_reach, place + ("  (empty)" if n == 0 else ""))
		if n > 1:
			var defs := []
			for k in n:
				defs.append(reach[i + k * 3])
			_button(header, "Take all", func() -> void: _take_all(x, y, defs))
		for slot in n:
			var def := reach[i + slot * 3]
			var count := reach[i + slot * 3 + 1]
			var row := _row(_reach, "   " + _describe(def, count))
			_button(row, "Take", func() -> void: _dusk.take(x, y, slot, def))
		i += n * 3


## "Take all": one take after another, each asked for as the last one finishes.
func _take_all(x: int, y: int, defs: Array) -> void:
	for def in defs:
		_queue.append([x, y, def])


## The next queued take: the first stack on that tile that's still the item it was.
func _take_next(reach: PackedInt32Array) -> void:
	var want: Array = _queue.pop_front()
	var i := 0
	while i + 4 <= reach.size():
		var n := reach[i + 3]
		if reach[i] == want[0] and reach[i + 1] == want[1]:
			for slot in n:
				if reach[i + 4 + slot * 3] == want[2]:
					_dusk.take(want[0], want[1], slot, want[2])
					return
		i += 4 + n * 3


func _describe(def: int, count: int) -> String:
	var grams := _weights[def] * count if def < _weights.size() else 0
	var label := _name(def) if count == 1 else "%s ×%d" % [_name(def), count]
	return "%s  ·  %.1f kg" % [label, grams / 1000.0]


func _name(def: int) -> String:
	return _names[def] if def < _names.size() else "something"


func _kind(def: int) -> int:
	return _kinds[def] if def < _kinds.size() else 0


func _fixture_name(fixture: int) -> String:
	return _fixtures[fixture] if fixture < _fixtures.size() else "container"


func _panel(preset: Control.LayoutPreset) -> PanelContainer:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.06, 0.08, 0.72)
	style.set_content_margin_all(10)
	style.set_corner_radius_all(4)
	panel.add_theme_stylebox_override("panel", style)
	var list := VBoxContainer.new()
	list.add_theme_constant_override("separation", 2)
	panel.add_child(list)
	add_child(panel)
	panel.set_anchors_and_offsets_preset(preset, Control.PRESET_MODE_MINSIZE, 16)
	panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN if preset == Control.PRESET_BOTTOM_RIGHT \
		else Control.GROW_DIRECTION_END
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	return panel


func _line(parent: Control, text: String, size: int) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	parent.add_child(label)
	return label


func _row(parent: Control, text: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	parent.add_child(row)
	var label := _line(row, text, 15)
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.custom_minimum_size = Vector2(250, 0)
	return row


func _button(row: HBoxContainer, text: String, pressed: Callable) -> void:
	var button := Button.new()
	button.text = text
	button.add_theme_font_size_override("font_size", 13)
	button.focus_mode = Control.FOCUS_NONE  # keys stay with the game (Space, WASD)
	button.pressed.connect(pressed)
	row.add_child(button)
