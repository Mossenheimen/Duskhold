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

const CALM := Color("#F5E699")  # one of the dead that isn't after you
const ALERT := Color("#FFA733")  # it heard something: "?"
const DANGER := Color("#FF3B2B")  # it sees you, it's after you: "!"; its tell; the lines on you
const DANGER_TEXT := Color("#FF6A5C")  # danger written on a panel: keeps 4.5:1
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
const HURT_TEXT := Color("#FFD1C7")  # your wounds, listed

# The GUI (§4).

const PANEL := Color("#0D0F14B8")  # every panel's ground, 72%
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
const TITLE := 48  # screen titles (the serif, once there are fonts)
const OUTLINE_SIZE := 6

# Layout (§10).

const MARGIN := 24  # every panel's distance from the screen's edges


## A panel's ground (§11): PANEL with a 1 px EDGE, 4 px corners, 12 px across and 10 down.
static func panel_style() -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = PANEL
	style.border_color = EDGE
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 12.0
	style.content_margin_right = 12.0
	style.content_margin_top = 10.0
	style.content_margin_bottom = 10.0
	return style


## A label at `size`, in `colour`. Drawn straight on the world (`outlined`), it gets the outline
## that keeps it readable on anything (§8).
static func text(label: Label, size: int, colour := TEXT, outlined := false) -> Label:
	label.add_theme_font_size_override("font_size", size)
	if colour != TEXT:
		label.add_theme_color_override("font_color", colour)
	if outlined:
		label.add_theme_constant_override("outline_size", OUTLINE_SIZE)
		label.add_theme_color_override("font_outline_color", OUTLINE)
	return label


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
