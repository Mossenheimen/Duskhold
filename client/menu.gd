extends CanvasLayer
## The menu (VISUALS.md §10, Screens; §16, slice 9's): Esc, with nothing else open to close. The
## world waits under a dim while it's open, as the playtest asked ("Escape should be pause/menu"):
## Resume (or Esc again), the settings (F10's screen, opened over it), and Save and quit (a trial
## run's is Quit: it keeps nothing).

const Look = preload("res://look.gd")
const Prompts = preload("res://prompts.gd")

## A button was pressed: "resume", "settings" or "quit".
signal chose(what: String)

var _quit: Button
var _hint: Label


func _ready() -> void:
	layer = 9  # over the HUD and its panels; the settings (10) open over it
	var dim := ColorRect.new()
	dim.color = Look.DIM
	dim.mouse_filter = Control.MOUSE_FILTER_STOP  # the world behind it takes no clicks
	add_child(dim)
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	var panel := PanelContainer.new()
	# Nearly solid, as the settings' is: the world's prompts and banners don't show through.
	var style := Look.panel_style()
	style.bg_color = Look.PANEL_SOLID
	style.remove_meta("panel")
	style.set_content_margin_all(28.0)
	panel.add_theme_stylebox_override("panel", style)
	add_child(panel)
	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 12)
	panel.add_child(rows)
	var title := Look.text(Label.new(), Look.TITLE)
	title.text = "Paused"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	rows.add_child(title)
	for row: Array in [["Resume", "resume"], ["Settings", "settings"], ["Save and quit", "quit"]]:
		var what: String = row[1]
		var button := Look.button(row[0], func() -> void: chose.emit(what), Vector2(280, 48))
		button.add_theme_font_size_override("font_size", Look.BODY)
		rows.add_child(button)
		if what == "quit":
			_quit = button
	_hint = Look.text(Label.new(), Look.SMALL, Look.TEXT_DIM)
	_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	rows.add_child(_hint)
	panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER, Control.PRESET_MODE_MINSIZE)
	panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	visible = false


## Opens it. `saved`: the game is saved as it quits (a trial run's isn't).
func open(saved: bool) -> void:
	_quit.text = "Save and quit" if saved else "Quit"
	_hint.text = Prompts.keyed("{quit} goes on  ·  {settings} the settings")
	visible = true


func close() -> void:
	visible = false


func is_open() -> bool:
	return visible
