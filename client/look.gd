extends RefCounted
## The look in one place (VISUALS.md: §4 colour, §8 type, §10 layout, §11 the GUI's parts): every
## colour the client draws with, the text sizes, and the GUI's panel, bar and outlined text.
## Code names a token here, never a colour of its own: a new colour is a new token, here and in
## VISUALS.md, in the same commit. Colours are sRGB as picked (a MultiMesh instance's goes
## through srgb_to_linear()). Each script that draws preloads it, as the client's scripts are:
## const Look = preload("res://look.gd").
##
## Not here yet: the looks of things (furniture, plants: chunk_view.gd's FURNITURE and PLANTS),
## which move to data/looks.toml in a later slice (VISUALS.md §16).
##
## The settings that change the look (VISUALS.md §13; settings.gd sets them) live here too. A token
## a setting changes is a static var, not a constant: the alert states (colour-blind mode) and the
## panel's ground (high contrast). Whatever copies one (a shader's parameter, a material, a label's
## colour) copies it again when `version` moves on; `restyle` brings the panels already built up
## to date, and high contrast's outline on every text is a theme on the window (`contrast`).

# The settings (§13).

## Moves on whenever a setting changes a token, so what copied one copies it again.
static var version := 0
## The alert states in the Okabe–Ito set (§4, Colour rules).
static var colour_blind := false
## Panels at 95%, and every text outlined.
static var high_contrast := false
## No flash at the screen's edge, no breathing dread, nothing popping.
static var reduce_motion := false
## The bars say their numbers, not only on hover.
static var show_numbers := false

# The world: the Hushlands (§4).

## The ground, by DuskWorld's ground code: nothing, grass, dirt, road, water, a plank floor,
## shallows, a bridge.
const GROUND: Array[Color] = [Color("#12141A"), Color("#5E6B4F"), Color("#6E5940"),
	Color("#857A66"), Color("#334D61"), Color("#78593D"), Color("#4D6B78"), Color("#664A2E")]
const WALL := Color("#A8A191")
const DOOR := Color("#614026")
const BOARD := Color("#9E7A4D")
const FENCE := Color("#73614D")
const TRUNK := Color("#4D3B2B")
const LEAVES := Color("#3D5436")
const LITTER := Color("#9E855C")  # what lies on a floor

# Actors: the second cue after their shape (§4, §5).

const YOU := Color("#BF5233")
const SHAMBLER := Color("#8F9980")
const GHOUL := Color("#D1C7B3")
const REVENANT := Color("#733D33")
const SKELETON := Color("#E0DBC7")
const WIGHT := Color("#384057")
const DEER := Color("#8C613D")
const SCUT := Color("#EBE6D9")  # a deer's white tail, flashing as it bolts
const WOLF := Color("#66666B")
const ASH := Color("#211F1C")  # a body burnt to ash

# Light, and what's drawn in it (§3).

const SUN_NIGHT := Color("#8C9EF2")
const SUN_DAY := Color("#FFF0D6")
const SUN_DUSK := Color("#FF9E5C")
const AMBIENT_NIGHT := Color("#47548C")
const AMBIENT_DAY := Color("#94A1B8")
const SKY_NIGHT := Color("#030306")
const SKY_DAY := Color("#12141A")
const TORCH := Color("#FFB86B")
const FIRE := Color("#FF8F3D")
const EMBER := Color("#FF6B1A")
const RAIN := Color("#B3C7E659")  # a streak, 35%
const SHOT := Color("#D9CCA6")  # an arrow's flight

# Signals: one meaning each, in the world and the GUI alike (§4, §7).

## The alert states, as set (colour-blind mode swaps them: `set_look`). Calm: one of the dead
## that isn't after you. Alert: it heard something, "?". Danger: it sees you, it's after you, "!";
## its tell; the lines on you. Danger written as text keeps 4.5:1 on a panel.
static var CALM := Color("#F5E699")
static var ALERT := Color("#FFA733")
static var DANGER := Color("#FF3B2B")
static var DANGER_TEXT := Color("#FF6A5C")
## Calm, alert, danger and danger's text: as picked, and in the Okabe–Ito set, which stays
## distinct across the common colour-vision deficiencies (sky blue, yellow, vermillion; the text a
## lighter vermillion, 7.1:1 on a panel, where #D55E00 itself is 5:1).
const SIGNALS: Array[Color] = [Color("#F5E699"), Color("#FFA733"), Color("#FF3B2B"), Color("#FF6A5C")]
const SIGNALS_SEEN: Array[Color] = [Color("#56B4E9"), Color("#F0E442"), Color("#D55E00"), Color("#F07F45")]
const GRIP := Color("#BF0D0D")  # a hold on you
const SOUND := Color("#E6F2FF")  # a noise ring
const HOLY := Color("#FFF2B3")  # turned "+", consecrated, hallowed
const REST := Color("#99B8FF")  # asleep "z", a restful moodle
const YOURS := Color("#F2EBCC")  # your own tell and progress
const GOOD := Color("#BFE699")  # can be made now, healed
const GOLD := Color("#FFC74D")  # XP, a level, a mastery, a find's worth, your stash
const UNCOMMON := Color("#8CBFFF")  # an uncommon magic item's name
const RARE := Color("#FFCC59")  # a rare magic item's name
const LORE := Color("#BFCCE6")  # what a known magic item's properties do
const ARCANE := Color("#B3A6FF")  # a Wizard's: its slots, a Light spell's light, a Silence's ring
const FOG := Color("#B8BEC780")  # a Fog spell's cloud, 50%
const DREAD := Color("#05000A")  # the screen's edges darkening with dread

# Bars and moodles (§4).

const HEALTH := Color("#BF2924")
const ENDURANCE := Color("#D9AD38")
const FAITH := Color("#D1D6FF")
const SKILL := Color("#8CB8E6")
const CAPPED := Color("#F2D973")  # a skill waiting at the free cap
## A bad moodle's tint by how bad: its first level, the second, the third. A good one is REST.
const MILD := Color("#F2D973")
const BAD := Color("#FF9940")
const WORST := Color("#FF4D40")
const SEVERITY: Array[Color] = [MILD, BAD, WORST]
## A moodle's glyph on its tile: the panel's own dark, opaque. White would fail the 3:1 a part
## that carries meaning needs on three of the four (1.4:1 on MILD); this is 5.8:1 on WORST and 9:1
## to 13.6:1 on the rest.
const INK := Color("#0D0F14")
const HURT_TEXT := Color("#FFD1C7")  # your wounds, listed
## The body diagram (§11): a part with no wound, and the bands round a bandaged one. A wounded
## part takes its worst wound's SEVERITY: a scratch MILD, a laceration BAD, a bite WORST.
const WHOLE := Color("#5E6573")
const DRESSING := Color("#F2EEE6")

# The GUI (§4).

static var PANEL := Color("#0D0F14B8")  # every panel's ground, 72%; 95% in high contrast
const PANEL_SOLID := Color("#0D0F14F2")  # high contrast's
const DIM := Color("#00000080")  # the world behind a choice or a menu, 50%
const EDGE := Color("#FFFFFF1A")  # a panel's 1 px edge, 10%
const TEXT := Color("#FFFFFF")
const TEXT_DIM := Color("#FFFFFFB3")  # hints, secondary lines, what can't be done yet: 70%
const OUTLINE := Color("#000000D9")  # round text drawn straight on the world: 85%
const TRACK := Color("#00000080")  # a bar's empty part, 50%
const VEIL := Color("#05081499")  # sleep, 60%
const HURT := Color("#B30000")  # the flash when you're hurt (up to 35%)

# Type, in pixels at the base screen of 1920 x 1080 (§8). The smallest is the 18 px Microsoft's
# guidelines ask of a PC game at 1080p.

const SMALL := 18  # hints, legends, item weights
const BODY := 20  # news, panel rows, moodle names
const HEADING := 24  # panel titles, the status line
const BANNER := 32  # held, paused, asleep
const TITLE := 48  # screen titles, in the serif
const OUTLINE_SIZE := 6

# Faces (§8): Alegreya Sans for everything in the GUI, as the project's default font (project.godot,
# gui/theme/custom_font: Regular); its Medium for headings and banners; and the serif, Alegreya,
# only for titles. All under the SIL Open Font License, in fonts/ with their licences.

const SANS_MEDIUM := preload("res://fonts/AlegreyaSans-Medium.ttf")
const SERIF := preload("res://fonts/Alegreya-Variable.ttf")

# Layout (§10).

const MARGIN := 24  # every panel's distance from the screen's edges

# Icons (§11): white SVG silhouettes on a 64 x 64 canvas (icons/), imported at twice that with
# mipmaps (project.godot's importer_defaults), so they stay sharp at any GUI size.

const TILE := 44  # a moodle's tile
const GLYPH := 32  # the glyph on it
const POP := 1.25  # a new or worsened moodle's tile starts this big...
const POP_TIME := 0.3  # ...and shrinks back over this many seconds
const HOVER := 0.4  # seconds of hovering before a detail shows


## The settings that change tokens (§13): `colour_blind` swaps the alert states for the
## Okabe–Ito set, `high_contrast` the panels' ground for 95%. The panels already built are brought
## up to date by `restyle`; what copies a token sees `version` move on.
static func set_look(blind: bool, contrast: bool) -> void:
	colour_blind = blind
	high_contrast = contrast
	var signals := SIGNALS_SEEN if blind else SIGNALS
	CALM = signals[0]
	ALERT = signals[1]
	DANGER = signals[2]
	DANGER_TEXT = signals[3]
	PANEL = PANEL_SOLID if contrast else Color("#0D0F14B8")
	version += 1


## Every panel under `node` built before a setting changed (`panel_style`), with the ground it
## would be built with now.
static func restyle(node: Node) -> void:
	var control := node as Control
	if control != null and control.has_theme_stylebox_override("panel"):
		var style := control.get_theme_stylebox("panel") as StyleBoxFlat
		if style != null and style.has_meta("panel"):
			style.bg_color = PANEL
	for child in node.get_children():
		restyle(child)


## A moodle's tile (§11): its severity's `colour`, 4 px corners, a 1 px OUTLINE to hold it apart
## from bright ground, and the glyph centred in it.
static func tile_style(colour: Color) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = colour
	style.border_color = OUTLINE
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.set_content_margin_all((TILE - GLYPH) / 2.0)
	return style


## A panel's ground (§11): PANEL with a 1 px EDGE, 4 px corners, 12 px across and 10 down.
static func panel_style() -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = PANEL
	style.set_meta("panel", true)  # for restyle: high contrast changes its ground
	style.border_color = EDGE
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 12.0
	style.content_margin_right = 12.0
	style.content_margin_top = 10.0
	style.content_margin_bottom = 10.0
	return style


## Text at `size`, in the face that size calls for (§8): the serif for a title, Medium for a
## heading or a banner, else the project's own (Alegreya Sans).
static func sized(control: Control, size: int) -> Control:
	control.add_theme_font_size_override("font_size", size)
	if size >= TITLE:
		control.add_theme_font_override("font", SERIF)
	elif size >= HEADING:
		control.add_theme_font_override("font", SANS_MEDIUM)
	return control


## A label at `size`, in `colour`. Drawn straight on the world (`outlined`), it gets the outline
## that keeps it readable on anything (§8). In high contrast every text has it (`contrast`).
static func text(label: Label, size: int, colour := TEXT, outlined := false) -> Label:
	sized(label, size)
	if colour != TEXT:
		label.add_theme_color_override("font_color", colour)
	if outlined:
		label.add_theme_constant_override("outline_size", OUTLINE_SIZE)
		label.add_theme_color_override("font_outline_color", OUTLINE)
	return label


## High contrast's theme, for the window (settings.gd sets it): every label's and button's text
## outlined, as text on the world is, whatever it's on and whenever it's built. A text's own
## outline, where it has one, stays as it is.
static func contrast() -> Theme:
	var theme := Theme.new()
	for type: String in ["Label", "Button"]:
		theme.set_constant("outline_size", type, OUTLINE_SIZE)
		theme.set_color("font_outline_color", type, OUTLINE)
	return theme


## A bar's fill in `colour`, on the TRACK (§11).
static func bar(progress: ProgressBar, colour: Color) -> ProgressBar:
	var fill := StyleBoxFlat.new()
	fill.bg_color = colour
	var back := StyleBoxFlat.new()
	back.bg_color = TRACK
	progress.add_theme_stylebox_override("fill", fill)
	progress.add_theme_stylebox_override("background", back)
	progress.show_percentage = false
	progress.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return progress
