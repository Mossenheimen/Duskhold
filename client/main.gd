extends Node3D
## Duskhold's client. The Rust core runs the game inside the DuskWorld node (the dusk-godot
## extension). This draws what it says and turns the player's input into intents: it decides
## nothing.
##
## The core's tile (x, y) is the 1 × 1 square from (x, 0, y) in 3D: x east, y south, Y up.
##
## Development flags, after "--" on Godot's command line (play.cmd passes them on):
## --seed=N the patch's seed; --town the little demo town instead; --walk=X,Y walk there at the
## start; --zoom=N; --speed=N the speed dial; --data=PATH another data folder (a mod, a test);
## --shot=PATH save a picture after --shot-after=SECONDS (default 3) and quit;
## --do=S:ACTION,... press swing (swing_n, _e, _s, _w: one way), heavy, shove, door, struggle, drop (the
## first thing in the pack),
## logs (take what's kept by the fire beside you), fire (tend it), smother, sleep, drink (from
## water), light, skills (the panel), board or unboard, sneak, shield (take one up), brace, bow (take one up), shoot (the nearest of the dead you see), cook (the last cooked recipe
## that can be made), butcher, hunt (shoot the nearest
## beast within a bow's reach), turn (the dead), pray, consecrate, ward, mend,
## pick (the wild plant beside you), use (E tapped: the likeliest thing here), here (E held: all
## of it, listed), power1 to power4 (the number keys), belt1 to belt4 (the belt's keys), tobelt (the
## first potion off the belt, put on it), bar (the bar in words), swap (the bar's first two powers),
## keys (F1), moodle (the worst one's detail, as if hovered),
## esc (closes what Esc would, never quits), settings (the screen, F10), zoom_in and zoom_out (three
## notches of the wheel), body (the worst-wounded part's detail), bandage (the
## first in the pack), inventory (I: opens or shuts it), card (the inventory opened, a weapon's card
## in it shown), arrange (the first stack in your pockets moved within them, at once), takein (the
## first thing within reach taken onto a cell of your pockets)
## S real seconds in;
## --class=NAME take up that class at the start (a trial run asks for none: the first, else);
## --choose ask who comes, as the player's own game does (for a picture of the choice);
## --news print the news too, and zone F's prompts as they change (for --headless); --hour=H start at that hour of the day;
## --save=PATH carry on from that save if it's there, and save to it;
## --look=colour_blind,high_contrast,reduce_motion,show_numbers turn settings on (not kept: for pictures);
## --aim=DX,DY aim a readied spell, and a script's spells, at your tile plus (DX, DY) (for pictures);
## --give=ITEM:GRADE:PROP+PROP,... put finds in the pack at the start, made as GRADE says (crude,
## plain, fine, masterwork) with those properties, known (by their ids: --give=sword:masterwork:keen).
##
## With no flags at all, the game is saved (user://world.dusk) and carries on from its save. Any
## flag makes it a trial run, which neither loads nor saves unless --save says where: so pictures
## and tests never touch the player's world.

const MapView = preload("res://map_view.gd")
const ActorView = preload("res://actor_view.gd")
const NoiseRings = preload("res://noise_rings.gd")
const Hud = preload("res://hud.gd")
const Tells = preload("res://tells.gd")
const Effects = preload("res://effects.gd")
const Aim = preload("res://aim.gd")
const Threats = preload("res://threats.gd")
const Bag = preload("res://bag.gd")
const Inventory = preload("res://inventory.gd")
const Icons = preload("res://icons.gd")
const Litter = preload("res://litter.gd")
const Skills = preload("res://skills.gd")
const Craft = preload("res://craft.gd")
const Spells = preload("res://spells.gd")
const Zones = preload("res://zones.gd")
const Look = preload("res://look.gd")
const Looks = preload("res://looks.gd")
const Prompts = preload("res://prompts.gd")
const Settings = preload("res://settings.gd")
const Hotbar = preload("res://hotbar.gd")
const Menu = preload("res://menu.gd")
const Numbers = preload("res://numbers.gd")
const Bars = preload("res://bars.gd")
## The numbers DuskWorld hands over, by name (codes.gd, made from the bridge's codes.rs).
const Codes = preload("res://codes.gd")
const Event = Codes.Event
const Item = Codes.Item
const Action = Codes.Action
const Moodle = Codes.Moodle
const State = Codes.State
const Remains = Codes.Remains
const Power = Codes.Power
const Potion = Codes.Potion
const Spell = Codes.Spell

## Zomboid's angle: 30° down, so tiles are 2:1 diamonds, and north up and to the right.
const CAMERA_PITCH := 30.0
const CAMERA_YAW := 45.0
const CAMERA_DISTANCE := 40.0
## The zoom, as Zomboid's: how much of the world the camera shows, in metres top to bottom, from
## a room's worth to a few streets'. A notch of the wheel (or - and =) is a step of 12%, eased.
const ZOOM_MIN := 6.0
const ZOOM_MAX := 40.0
const ZOOM_STEP := 1.12
## The speed dial (F2, F3 and F4 by default).
const SPEEDS: Array[int] = [1, 2, 5]

## Rain streaks falling round the camera, by DuskWorld.rain() (dry, rain, heavy).
const RAIN_DROPS := [0, 700, 1800]
## What your own timed actions are called, by their kind (DuskWorld.actor_actions()).
const DOING := {
	Action.STRUGGLE: "Struggling",
	Action.TAKE: "Taking",
	Action.DROP: "Dropping",
	Action.BANDAGE: "Bandaging",
	Action.WIELD: "Taking it in hand",
	Action.PUT_AWAY: "Putting it away",
	Action.SLEEP: "Lying down",
	Action.DRINK_WATER: "Drinking",
	Action.EAT: "Eating",
	Action.DRINK: "Drinking",
	Action.LIGHT: "Lighting it",
	Action.DOUSE: "Putting it out",
	Action.BOARD: "Boarding it up",
	Action.UNBOARD: "Prying a board off",
	Action.REPAIR: "Mending it",
	Action.BUILD: "Putting it up",
	Action.STASH: "Making it your stash",
	Action.PUT: "Putting it away",
	Action.CURE: "Taking the cure",
	Action.BURN: "Setting it alight",
	Action.BUY: "Buying",
	Action.SELL: "Selling",
	Action.MEND: "Having it mended",
	Action.CLAIM: "Claiming the bounty",
	Action.IDENTIFY: "Having it read",
	Action.UNCURSE: "Having the curse lifted",
	Action.SECOND_WIND: "Catching your breath",
	Action.LAY_FIRE: "Laying the fire",
	Action.LIGHT_FIRE: "Lighting the fire",
	Action.SMOTHER: "Putting the fire out",
	Action.CRAFT: "Making it",
	Action.BREAK_DOWN: "Breaking it down",
	Action.SHOOT: "Drawing the bow",
	Action.BUTCHER: "Butchering it",
	Action.TURN: "Holding up the holy symbol",
	Action.PRAY: "Praying",
	Action.CONSECRATE: "Consecrating it",
	Action.HEAL: "Mending the wound",
	Action.WARD: "Warding the door",
	Action.PICK: "Picking it",
	Action.WEAR: "Putting it on",
	Action.TAKE_OFF: "Taking it off",
	Action.CAST: "Casting",
	Action.WIND: "Winding it",
	Action.BELT: "Moving it",
}
## What the news says as a potion's lasting effect wears off (potion.rs), by DuskWorld's potion code.
const WANED_TOLD := {Potion.HASTE: "The haste wears off: your legs are your own again.",
	Potion.IRON_SKIN: "The iron skin wears off: blows bite as they did.",
	Potion.VIGOUR: "The vigour ebbs: your breath costs what it did.",
	Potion.COURAGE: "The courage ebbs. The dark is the dark again."}
## What the news says of a wild plant (fixtures.toml's id) the first time you're beside one.
const PLANT_TOLD := {"yarrow": "Yarrow grows here: made into a poultice, it staunches a wound",
	"chamomile": "Chamomile grows here: steeped at a fire, its tea calms",
	"bramble": "A bramble, black with berries",
	"feverfew": "Feverfew grows here, where the dead lie: brewed at a fire, it cures the fever"}
## How high one climbing is lifted halfway over, in metres (a window's sill is 0.9).
const CLIMB_LIFT := 0.55
## A fire's light sits this far over its embers (looks.toml's fire), so its fixture doesn't shut
## it in.
const FIRE_OVER := 0.45
## Asleep, the game runs this many times as fast (an hour of sleep in a few seconds).
const SLEEP_SPEED := 60.0
## E held this long (ms) lists everything you can do here; a shorter press does the likeliest.
const HOLD_MS := 350
## Why a readied spell can't be cast on the spot aimed at, by DuskWorld.cast_check()'s code.
const AIM_WHY := {-2: "not with it holding you", -3: "not mana enough", -4: "you've no component for it",
	-5: "too far", -6: "you can't see that spot"}
## The kinds with no blood (by DuskWorld.creature_names()): a blow chips bone off them instead.
const BLOODLESS := ["skeleton"]
## Seconds played before the news tells of the same level of dread again.
const DREAD_TOLD_EVERY := 30.0
## Why you woke, by Event.WOKE's extra.
const WOKE_BY := ["You woke, rested.", "You got up.", "You woke to pain!", "A noise woke you.",
	"The cold woke you, shivering. Get warm: a roof, a fire."]
## Numbers an event takes.
const EVENT_SIZE := 6

var dusk: Node  # the DuskWorld
var map_view: MapView
var rings: NoiseRings
var tells: Tells
var effects: Effects  # a power or a spell as it happens: bursts, waves, a fireball's flight
var aim: Aim  # a readied spell's reach, round you and at the mouse
var threats: Threats
var bag: Bag
var inventory: Inventory  # I or Tab: your pack as Path of Exile 2's, and what's within reach
var icons: Icons  # each item's icon, drawn from its look
var litter: Litter
var skills: Skills
var item_names := PackedStringArray()
var rares := PackedInt32Array()  # 1 for each actor that's a rare (DuskWorld.actor_rare())
var told_get_up := false  # the first body you see fall says it'll get up
var hallowed := false  # on hallowed ground, as last told
var creature_names := PackedStringArray()  # by creature number: actor kind 1 + n is creature n
var region_name := ""  # the region the world is a patch of (DuskWorld.region_name()), for zone A
var kinds := PackedInt32Array()  # each actor's kind (DuskWorld.actor_kinds())
var fever_took := false  # the fever killed you, for the death screen
var skill_names := PackedStringArray()
var hud: Hud
var camera: Camera3D
var cursor: MeshInstance3D
var sun: DirectionalLight3D
var sky: Environment
var torch: OmniLight3D  # the light you hold up, drawn as a real light with shadows
var fires := {}  # the fires alight in hearths, stoves and forges: Vector2i -> OmniLight3D
var fixture_names := PackedStringArray()  # by fixture number (DuskWorld.fixture_names())
var fire_seen_at := -100.0  # when the news last told of the dead seeing a fire, in seconds played
var dread_was := 0  # your dread's level, as the core last said
var cold_was := 0  # how cold you are, its level, as the core last said (cold.rs)
var craft: Node  # the crafting panel (B)
var spells: Node  # the spells panel (P)
var hotbar: Hotbar  # the bar: your powers on the number keys, then the belt's loops
var potion_lines := PackedStringArray()  # what each item does as a potion (DuskWorld.potion_lines())
var zones: Node3D  # the spells lasting on their spots, drawn
var readied := -1  # the spell readied, cast on the next spot clicked; -1 none
var spell_names := PackedStringArray()
var recipe_texts := PackedStringArray()  # two lines a recipe (DuskWorld.recipe_texts())
var dread_told := {}  # when the news last told each level of dread, in seconds played
var balked_at := -100.0  # when the news last said you balked, in seconds played
var resisted_at := -100.0  # when the news last told of a blow on bone, in seconds played
var blocked_at := -100.0  # when the news last told of a block, in seconds played
var thorned_at := -100.0  # and of thorns biting back
var driven_at := -100.0  # and of a spear's blow driving one back
var glanced_at := -100.0  # when the news last told of a blow glancing off your armour
var tumbled_at := -100.0  # when it last told of one of the dead tumbling over a sill or a fence
var skidded_at := -100.0  # ...and of your blow glancing off theirs
var seen := PackedByteArray()  # per actor, whether you see it (DuskWorld.actor_seen())
var actors: Array = []  # an ActorView per actor, in the core's order
var me := -1  # the character's index in the actor arrays
var town_seed := 1
var speed_index := 0
var zoom := 14.0
var focus := Vector3.ZERO  # where the camera looks: the character, smoothed
var rain_fall: GPUParticles3D  # the rain, drawn round where the camera looks
var wound_names := PackedStringArray()
var part_names := PackedStringArray()
var last_hurt := ""  # what last hurt you, for the death screen
var last_hurt_at := -100  # the tick it did
var moodles := PackedInt32Array()  # yours, as of the last frame
var sleeping := false  # asleep, and the clock running fast
var wake_speed := 1.0  # the speed to go back to on waking
var dead := false
var held := 0  # how many have hold of you
var dragged_by := 0  # how many dragged you down, for the death screen
var last_thump := -100.0  # when the news last told of a blow on a door, in game seconds
var low_walls := false
var last_tick := -1
var use_town := false

var shot_path := ""
var shot_after := 3.0
var start_speed := 1.0
var print_news := false  # --news
var start_hour := -1  # --hour: -1 for the data's
var data_dir := ""  # --data: another data folder (a mod, or a test)
var walk_at := Vector2i(-1, -1)
var played := 0.0
var scripted: Array = []  # --do: [real seconds, action], soonest first
var save_path := ""  # where the game is saved; "" for a trial run, which isn't
var saved_hour := -1  # the game hour of the last autosave
var restart_asked := -100.0  # when a new world was last asked for ("abandon this one?"), in seconds played
var class_wanted := ""  # --class: the class a trial run's survivors take up
var choose_asked := false  # --choose: a trial run asks who comes, as the player's own game does
var keys_mode := 0  # the keys panel (F1): 0 for a world's first game hour, 1 shown, 2 hidden
var settings: Settings  # the settings screen (F10), and the keys' defaults and remapping
var paused_before := false  # whether the game was paused when the settings opened
var menu: Menu  # Esc's menu, which pauses (menu.gd)
var menu_paused_before := false  # whether the game was paused when the menu opened
## Running, or walking: Shift switches (the playtest: "start with running on and make run/walk a
## toggle"). Held with a swing or a click, Shift is a heavy swing's, and switches nothing.
var running := true
var run_tapped := false  # Shift went down, and nothing else has used it yet
var look_flags := ""  # --look: settings a trial run turns on
var gives := PackedStringArray()  # --give: finds a trial run starts with
## --aim=DX,DY: for pictures, the tile aimed at is yours plus this, not the mouse's, and a script's
## spells land there. (0, 0) is the mouse, as in play.
var aim_offset := Vector2i.ZERO
var aiming := false  # a readied spell's aim has zone F (_aim)
var choosing_class := false  # the game waits, paused, for a class to be chosen
var class_texts := PackedStringArray()  # every class's name and line (DuskWorld.class_texts())
var plants_told := {}  # the wild plants the news has named, by fixture number
var here: Array = []  # what you can do here (Prompts.here), as last worked out
var listing := false  # E held: all of `here` listed, for a number or a click to pick one
var use_down := -1  # when E went down (Time.get_ticks_msec()), -1 while it's up
var bracing := false  # the right mouse button, held behind a shield
## On guard (guard.rs): the right mouse button held. You face the cursor, a click swings that way
## (or shoots the bow in your hand), and a shield on your arm braces.
var guarding := false
var pursuing := -1  # the one of the dead clicked and held on: swung at again while the button's down
var pursuit_heavy := false  # and whether with Shift: heavy swings
var numbers: Numbers  # damage numbers, where blows land
var bars: Bars  # health bars over the dead you fight


func _ready() -> void:
	if OS.get_cmdline_user_args().is_empty():
		save_path = ProjectSettings.globalize_path("user://world.dusk")
	_read_args()
	Settings.add_keys()
	_build_stage()
	settings = Settings.new()
	settings.keep = OS.get_cmdline_user_args().is_empty()
	add_child(settings)
	settings.shown.connect(_on_settings)
	menu = Menu.new()
	add_child(menu)
	menu.chose.connect(_on_menu)
	_settle_window()
	# The health bars over the dead under everything else on the screen.
	var bar_layer := CanvasLayer.new()
	add_child(bar_layer)
	bars = Bars.new()
	bar_layer.add_child(bars)
	# The bar before the HUD: the HUD's details and cards draw over it.
	hotbar = Hotbar.new()
	add_child(hotbar)
	hotbar.settings = settings
	hotbar.use_power = _power
	hotbar.use_loop = _drink_loop
	hud = Hud.new()
	hud.echo = print_news
	add_child(hud)
	settings.keys_changed.connect(hud.rebuild_keys)
	hud.chose.connect(_do_here)
	hud.death_chose.connect(_on_death_chose)
	hotbar.card = hud.hover_card
	hotbar.uncard = hud.unhover_card
	if not ClassDB.class_exists("DuskWorld"):
		hud.show_problem("The game's core (the Rust extension) isn't loaded.\n"
			+ "Start the game with play.cmd in 12_Duskhold: it builds the extension first.")
		return
	dusk = ClassDB.instantiate("DuskWorld")
	dusk.process_priority = -1  # tick the world before this script draws it
	dusk.data_dir = data_dir
	dusk.start_hour = start_hour
	add_child(dusk)
	map_view = MapView.new()
	add_child(map_view)
	rings = NoiseRings.new()
	add_child(rings)
	tells = Tells.new()
	add_child(tells)
	effects = Effects.new()
	add_child(effects)
	numbers = Numbers.new()
	add_child(numbers)
	aim = Aim.new()
	add_child(aim)
	threats = Threats.new()
	add_child(threats)
	bag = Bag.new()
	add_child(bag)
	bag.say = hud.say
	bag.burn = _burn_body
	bag.hallow = _consecrate
	bag.card = hud.hover_card
	bag.uncard = hud.unhover_card
	icons = Icons.new()
	add_child(icons)
	inventory = Inventory.new()
	add_child(inventory)
	inventory.say = hud.say
	inventory.card = hud.hover_card
	inventory.uncard = hud.unhover_card
	inventory.bag = bag
	inventory.icons = icons
	hotbar.icons = icons
	litter = Litter.new()
	add_child(litter)
	skills = Skills.new()
	add_child(skills)
	craft = Craft.new()
	add_child(craft)
	spells = Spells.new()
	add_child(spells)
	spells.ready_spell = _ready_spell
	spells.slot_of = hotbar.slot_of_spell
	zones = Zones.new()
	add_child(zones)
	_start(true)


## Makes the world: the saved one if `carry_on` and there is one, else afresh, a patch of the
## Hushlands from the seed (or the demo town). The same seed and the same input play the same
## game.
func _start(carry_on := false) -> void:
	var loaded := false
	var not_loaded := ""
	if carry_on and save_path != "" and FileAccess.file_exists(save_path):
		not_loaded = dusk.load_game(save_path)
		loaded = not_loaded.is_empty()
		if loaded:
			town_seed = dusk.world_seed()
	if loaded:
		pass
	elif use_town:
		dusk.load_town(town_seed)
	else:
		var problem: String = dusk.load_patch(town_seed)
		if not problem.is_empty():
			hud.show_problem("The game's data has a problem:\n" + problem)
			return
	# The looks of things, and the world's colours with them, before anything's drawn.
	Looks.read(dusk)
	map_view.build(dusk, low_walls)
	for view in actors:
		view.queue_free()
	actors.clear()
	kinds = dusk.actor_kinds()
	creature_names = dusk.creature_names()
	region_name = dusk.region_name()
	fixture_names = dusk.fixture_names()
	hud.set_body_names(dusk.part_names(), dusk.wound_names())
	fever_took = false
	told_get_up = false
	hallowed = false  # so the first frame says where you are
	for kind in kinds:
		var view := ActorView.new()
		view.setup(kind)
		add_child(view)
		actors.append(view)
	rares = PackedInt32Array()
	me = dusk.character_index()
	rings.clear()
	hud.reset()
	wound_names = dusk.wound_names()
	item_names = dusk.item_names()
	bag.start(dusk)
	icons.start(dusk)
	inventory.start(dusk)
	skill_names = dusk.skill_names()
	skills.start(dusk)
	craft.start(dusk)
	spells.start(dusk)
	hotbar.start(dusk)
	potion_lines = dusk.potion_lines()
	zones.clear()
	readied = -1
	spell_names = dusk.spell_names()
	recipe_texts = dusk.recipe_texts()
	part_names = dusk.part_names()
	last_hurt = ""
	last_hurt_at = -100
	moodles = PackedInt32Array()
	sleeping = false
	dead = false
	held = 0
	dragged_by = 0
	last_thump = -100.0
	last_tick = -1
	_draw_actors(0.0)
	if me >= 0:
		focus = actors[me].position
	if walk_at.x >= 0:
		dusk.walk_to(walk_at.x, walk_at.y, false)
	dusk.speed = start_speed
	var clock: PackedInt32Array = dusk.clock()
	saved_hour = clock[1]
	choosing_class = false
	if not loaded:
		_offer_classes()
	if loaded:
		dead = dusk.character_body()[3] == 1
		if dead:
			_show_recap("You died here.")
		else:
			hud.say("You carry on.  %s" % Hud.time_of_day(clock))
	elif not not_loaded.is_empty():
		# Moved aside, not saved over: a broken one is worth a look, and one the game can't read
		# (from before saves outlasted updates, format 36 or older; or one naming something the
		# data has lost) loads in the build that made it.
		var kept := save_path + ".old"
		if FileAccess.file_exists(kept):
			DirAccess.remove_absolute(kept)
		var moved := DirAccess.rename_absolute(save_path, kept) == OK
		hud.say("The saved game couldn't be loaded: %s. A new world, then%s." % [not_loaded,
			" (the old save is kept beside it, as %s)" % kept.get_file() if moved else ""])
	else:
		_save("")  # a new world takes the old one's place at once


func _process(delta: float) -> void:
	if dusk == null:
		return
	_steer()
	_mind_held_buttons()
	_draw_actors(delta)
	_mind_the_guard()
	# The sound rings and what the dead see, only with their settings on (the playtest's).
	for noise in dusk.take_noises():
		if Look.show_rings:
			rings.add(noise.x, noise.y, noise.z)
	rings.visible = Look.show_rings
	rings.advance(0.0 if dusk.paused else delta * dusk.speed)
	_read_events()
	var tick: int = dusk.tick()
	var ticked := tick != last_tick
	if ticked:
		last_tick = tick
		if Look.show_sight:
			map_view.show_watched(dusk.watched())
		map_view.show_view(dusk.view_tiles())
	_light_the_sky(dusk.daylight())
	_hold_up_light()
	_light_the_fires()
	# Sneaking, you crouch.
	if me >= 0 and me < actors.size():
		actors[me].scale.y = 0.72 if dusk.is_sneaking() and not dead else 1.0
	_let_it_rain()
	_move_camera(delta)
	map_view.update(focus)
	_move_cursor()
	_aim()
	var clock: PackedInt32Array = dusk.clock()
	if not dead and clock[1] != saved_hour:
		saved_hour = clock[1]
		_save("")  # on the game hour
	moodles = dusk.character_moodles()
	var asleep := _moodle(Moodle.ASLEEP) > 0
	_sleep_speed(asleep)
	# Paused under the menu or the settings: their own screens say so.
	var paused: bool = dusk.paused and not menu.is_open() and not settings.is_open()
	hud.show_state(region_name, clock, roundi(dusk.speed), paused, dusk.pressure(),
		dusk.tolling(), dusk.rain(), dusk.place(), -1 if save_path != "" else town_seed)
	_mind_the_keys(clock)
	hud.show_moodles(moodles)
	hud.show_dread(0 if dead else dusk.character_dread(), delta)
	hud.show_asleep(asleep, clock)
	var body: PackedInt32Array = dusk.character_body()
	held = body[5] if body.size() > 5 else 0
	hud.show_body(body, dusk.character_wounds(), dusk.can_shove())
	hud.show_wound_parts(dusk.character_wound_parts())
	hud.show_faith(dusk.faith())
	hud.show_mana(dusk.character_mana())
	hud.show_pace(not running and not dead)
	hud.show_deeds(dusk.character_deeds())
	if ticked:
		_here_now(asleep)
	_mind_the_ground()
	_mind_the_plants()
	var doing := _my_action()
	bag.refresh(doing.x != 0, dead, inventory.is_open())
	inventory.refresh(dead)
	hud.make_room(inventory.room_on_right())
	skills.refresh(dead)
	craft.refresh(dead)
	spells.refresh(dead, readied)
	hotbar.refresh(dead or choosing_class, readied, _my_tile())
	zones.show_zones(dusk.spell_zones())
	litter.show_floors(dusk.floor_items())
	hud.show_action(DOING.get(doing.x, ""), doing.y / 1000.0)
	played += delta
	_run_script()
	_maybe_take_picture()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("quit"):
		if not _close_top():
			_open_menu()
		return
	if dusk == null:
		return
	if event.is_action_pressed("settings"):
		settings.toggle()
		return
	if settings.is_open():
		return  # the settings screen has the keys (and Esc closes it)
	if menu.is_open():
		return  # the menu has the world waiting: Esc goes on
	# Shift on its own switches between walking and running; held with a swing or a click, it's
	# a heavy swing's, and switches nothing.
	if event.is_action_pressed("run"):
		run_tapped = true
	elif event.is_action_released("run") and run_tapped:
		run_tapped = false
		_toggle_run()
	var key := event as InputEventKey
	var number := int(key.physical_keycode) - int(KEY_1) if key and key.pressed and not key.echo else -1
	# Who comes into the world: a number takes up that class, and nothing else is heeded.
	if choosing_class:
		if number >= 0 and number < class_texts.size() / 2:
			_take_up_class(number)
		return
	# E's list is up: a number picks from it, in place of the powers.
	if listing and number >= 0 and number < Prompts.MOST:
		_do_here(number)
		return
	# A trackpad's pinch zooms too: fingers apart, in.
	var pinch := event as InputEventMagnifyGesture
	if pinch and pinch.factor > 0.0:
		_zoom(-log(pinch.factor) / log(ZOOM_STEP))
		return
	var mouse := event as InputEventMouseButton
	if mouse and mouse.pressed:
		match mouse.button_index:
			MOUSE_BUTTON_LEFT:
				if dead:
					return
				var tile := _tile_under(mouse.position)
				# A readied spell is cast on the spot clicked.
				if readied >= 0:
					_cast_on(tile)
					return
				# On guard, a click swings where you face (the cursor's way), or shoots the bow in
				# your hand at the spot: Zomboid's, never a walk.
				if guarding:
					_guarded_click(tile)
					return
				# One of the dead clicked (Diablo's; the second playtest): go after it wherever it
				# walks and swing once it's in reach; held, swing again till the button's let go
				# (_mind_held_buttons). Shift-click: heavy swings, your weight behind them.
				var foe := _foe_under(mouse.position)
				if foe >= 0:
					if held > 0:
						dusk.struggle()
					else:
						pursuit_heavy = Input.is_action_pressed("run")
						if pursuit_heavy:
							run_tapped = false
						pursuing = foe
						dusk.pursue(foe, pursuit_heavy, running)
				else:
					run_tapped = false
					dusk.walk_to(tile.x, tile.y, running)
			MOUSE_BUTTON_RIGHT:
				# A readied spell let go; else on guard while the button's held (_mind_held_buttons):
				# facing the cursor, and with a shield on your arm, braced behind it.
				if not dead:
					if readied >= 0:
						_let_go()
					else:
						_guard(true)
			# The wheel zooms, a step a notch (a trackpad's finer scroll, a part of one).
			MOUSE_BUTTON_WHEEL_UP:
				_zoom(-(mouse.factor if mouse.factor > 0.0 else 1.0))
			MOUSE_BUTTON_WHEEL_DOWN:
				_zoom(mouse.factor if mouse.factor > 0.0 else 1.0)
	elif event.is_action_pressed("swing") and not dead:
		# Held, the swing key struggles: you can't swing with one on you. With Shift, it's a
		# heavy swing.
		if held > 0:
			dusk.struggle()
		elif Input.is_action_pressed("run"):
			run_tapped = false
			dusk.heavy_attack(0, 0)
		else:
			dusk.attack(0, 0)
	elif event.is_action_pressed("shove") and not dead:
		if not dusk.can_shove():
			hud.say(Prompts.keyed("No hand free to shove with: struggle {swing}."))
		dusk.shove(0, 0)
	elif event.is_action_pressed("use") and not dead:
		# Tapped, E does the likeliest thing here; held, it lists them all (_mind_held_buttons).
		# Pressed over the list, it closes it.
		if listing:
			_close_here()
		else:
			use_down = Time.get_ticks_msec()
	elif event.is_action_released("use"):
		if use_down >= 0:
			use_down = -1
			_use_here()
	elif event.is_action_pressed("new_world"):
		# Alive in a saved world, it asks twice: the world you leave is gone for good.
		if save_path != "" and not dead and played - restart_asked > 4.0:
			restart_asked = played
			hud.say(Prompts.keyed("Abandon this world for a new one? Press {new_world} again."))
		else:
			town_seed += 1
			_start()
	elif event.is_action_pressed("light"):
		# Dead, the light's key brings the next survivor into this same world.
		if dead:
			_next_survivor()
		else:
			_light_or_douse()
	elif event.is_action_pressed("sleep") and not dead:
		_sleep_or_wake()
	elif event.is_action_pressed("power_1") and not dead:
		_power(0)
	elif event.is_action_pressed("power_2") and not dead:
		_power(1)
	elif event.is_action_pressed("power_3") and not dead:
		_power(2)
	elif event.is_action_pressed("power_4") and not dead:
		_power(3)
	elif event.is_action_pressed("belt_1") and not dead:
		_drink_loop(0)
	elif event.is_action_pressed("belt_2") and not dead:
		_drink_loop(1)
	elif event.is_action_pressed("belt_3") and not dead:
		_drink_loop(2)
	elif event.is_action_pressed("belt_4") and not dead:
		_drink_loop(3)
	elif event.is_action_pressed("speed_1"):
		_set_speed(0)
	elif event.is_action_pressed("speed_2"):
		_set_speed(1)
	elif event.is_action_pressed("speed_3"):
		_set_speed(2)
	elif event.is_action_pressed("pack"):
		inventory.toggle()
	elif event.is_action_pressed("skills"):
		_book(skills)
	elif event.is_action_pressed("craft") and not dead:
		_book(craft)
	elif event.is_action_pressed("spells") and not dead:
		_book(spells)
	elif event.is_action_pressed("keys"):
		_toggle_keys()
	# The GUI's size is Ctrl with - or =, so before the zoom, which is - or = alone.
	elif event.is_action_pressed("gui_smaller"):
		hud.say("GUI size %d%%." % settings.scale_gui(-1))
	elif event.is_action_pressed("gui_larger"):
		hud.say("GUI size %d%%." % settings.scale_gui(1))
	elif event.is_action_pressed("zoom_in", true):
		_zoom(-1.0)
	elif event.is_action_pressed("zoom_out", true):
		_zoom(1.0)
	elif event.is_action_pressed("sneak") and not dead:
		dusk.set_sneaking(not dusk.is_sneaking())
	elif event.is_action_pressed("low_walls"):
		low_walls = not low_walls
		map_view.set_low_walls(low_walls)
	elif event.is_action_pressed("save"):
		if save_path == "":
			hud.say("This is a trial run: it isn't saved.")
		else:
			_save("Saved.")

## Saves the game, if this run is saved, and says `said` if that's not "". The core makes the
## save; the extension writes the file.
func _save(said: String) -> void:
	if save_path == "" or dusk == null:
		return
	var problem: String = dusk.save_game(save_path)
	if not problem.is_empty():
		hud.say("The game couldn't be saved: " + problem)
	elif said != "":
		hud.say(said)


## Closing the window saves too.
func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		_save("")


## The one of the dead (or a beast) you see under the mouse at `screen`, alive: its index, or -1.
## Picked on the screen, by its figure from its feet to its head, since a click on its shoulders
## falls on the ground behind it; the nearest, if two stand close. Whether it can be reached is the
## core's call (Intent::Pursue).
func _foe_under(screen: Vector2) -> int:
	if me < 0:
		return -1
	var states: PackedInt32Array = dusk.actor_states()
	var best := -1
	var nearest := INF
	for i in mini(actors.size(), states.size()):
		if i == me or i >= kinds.size() or kinds[i] == 0 or states[i] == State.DEAD or not _seen(i):
			continue
		var feet: Vector3 = actors[i].position
		if camera.is_position_behind(feet):
			continue
		var low := camera.unproject_position(feet)
		var high := camera.unproject_position(feet + Vector3(0, 1.7, 0))
		# Within a third of its height of the line up its middle (at least 18 px, however far out).
		var reach := maxf(18.0, low.distance_to(high) / 3.0)
		var off := screen.distance_to(Geometry2D.get_closest_point_to_segment(screen, low, high))
		if off <= reach and off < nearest:
			nearest = off
			best = i
	return best


## WASD, as a heading on screen. The core turns it into a heading on the map.
func _steer() -> void:
	if dead or choosing_class or settings.is_open():
		return
	var right := int(Input.is_action_pressed("walk_right")) - int(Input.is_action_pressed("walk_left"))
	var up := int(Input.is_action_pressed("walk_up")) - int(Input.is_action_pressed("walk_down"))
	dusk.steer(right, up, running)


func _set_speed(index: int) -> void:
	speed_index = index
	if sleeping:
		wake_speed = float(SPEEDS[index])  # for when you wake
	else:
		dusk.speed = float(SPEEDS[index])
	dusk.paused = false


## Sleep (Z): lie down to sleep, or get up. The core says whether you can; this says why not.
func _sleep_or_wake() -> void:
	if _moodle(Moodle.ASLEEP) > 0:
		dusk.wake()
		return
	match dusk.sleep_check():
		-1:
			hud.say("You're not tired enough to sleep.")
		-2:
			hud.say("You can't sleep with the dead after you.")
		-3:
			hud.say("Not with it holding you!")
		-4:
			hud.say(Prompts.keyed("You're in too much pain to sleep. Dress your wounds first: a bandage {pack}."))
		-5:
			hud.say("You're too cold to sleep. Get warm first: under a roof, by a fire.")
		_:
			dusk.sleep()


## Board it up (E's list): nail a board across the door or window beside you. The core says whether you can; this
## says why not.
func _board_up() -> void:
	var code: int = dusk.board_check(false)
	if code < 0:
		hud.say(Prompts.refusal("board", code))
	else:
		dusk.board()


## Stepping onto hallowed ground, or off it: the news says so, once each way.
func _mind_the_ground() -> void:
	if dead:
		return
	var now: bool = dusk.on_hallowed()
	if now == hallowed:
		return
	hallowed = now
	if now:
		hud.say("Hallowed ground. The dead won't come in, nor heed you here.")
	else:
		hud.say("You leave hallowed ground.")


## Zone F: what you can do here (prompts.gd), worked out each tick and whenever E is used;
## nothing while you're asleep, held, choosing who comes, or dead.
func _here_now(asleep: bool) -> void:
	if aiming:
		return  # a readied spell's aim has zone F (_aim)
	if dead or asleep or held > 0 or choosing_class:
		here = []
	else:
		here = Prompts.here(dusk, creature_names, fixture_names, kinds, _moodle(Moodle.THIRSTY) > 0)
	if here.is_empty():
		listing = false
	hud.show_prompts(Prompts.lines(here, listing), listing)


## E, tapped: the likeliest thing here (Prompts.first). With nothing that's E's to do, the list
## of what there is.
func _use_here() -> void:
	_here_now(_moodle(Moodle.ASLEEP) > 0)
	var e := Prompts.first(here)
	if e >= 0:
		_do(here[e][0])
	else:
		_open_here()


## E, held: everything you can do here, numbered, till a number or a click picks one, or E or Esc
## closes it. It's worked out afresh as you move, like the prompts.
func _open_here() -> void:
	listing = true
	_here_now(_moodle(Moodle.ASLEEP) > 0)
	if not listing:
		hud.say("There's nothing to do here.")


func _close_here() -> void:
	listing = false
	_here_now(_moodle(Moodle.ASLEEP) > 0)


## The `n`th thing in E's list (from 0), done, and the list closed. A number past its end does
## nothing.
func _do_here(n: int) -> void:
	if not listing or n < 0 or n >= here.size():
		return
	var verb: String = here[n][0]
	listing = false
	_do(verb)
	_here_now(_moodle(Moodle.ASLEEP) > 0)


## What each of Prompts.here's verbs does: its handler, which asks the core and says why not.
func _do(verb: String) -> void:
	match verb:
		"door":
			_use_door()
		"burn":
			_burn_body()
		"consecrate":
			_consecrate()
		"pick":
			_pick()
		"pray":
			_pray()
		"tend_fire":
			_tend_fire()
		"smother_fire":
			_smother_fire()
		"drink":
			_drink()
		"board":
			_board_up()
		"unboard":
			_pry_board()
		"repair":
			_repair()
		"ward":
			_ward()


## Held buttons: E long enough lists everything here (a press whose letting go went astray counts
## as a tap); the right mouse button let go ends a brace.
func _mind_held_buttons() -> void:
	if use_down >= 0 and Time.get_ticks_msec() - use_down >= HOLD_MS:
		use_down = -1
		if Input.is_action_pressed("use"):
			_open_here()
		else:
			_use_here()
	if guarding and not Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT):
		_guard(false)
	# Held on one of the dead: each time you're free again, after it once more, until the button's
	# let go or it's down (the second playtest: "if I click and hold, it should continue to attack
	# till I release").
	if pursuing >= 0:
		var states: PackedInt32Array = dusk.actor_states()
		if dead or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) \
				or pursuing >= states.size() or states[pursuing] == State.DEAD:
			pursuing = -1
		elif dusk.pursuing() < 0 and _my_action().x == 0:
			dusk.pursue(pursuing, pursuit_heavy, running)


## E at a door or window: open or shut it. The core says whether it can be; this says why not.
func _use_door() -> void:
	var door: PackedInt32Array = dusk.door_check()
	var code: int = door[2] if door.size() >= 4 else -1
	if code < 0:
		hud.say(Prompts.refusal("door", code))
	else:
		dusk.use_door()


## E at water: drink your fill.
func _drink() -> void:
	if dusk.water_in_reach():
		dusk.drink_water()
	else:
		hud.say("There's no water within reach.")


## The number keys, and a click on the bar: the power on key `k + 1`, in the bar's order (hotbar.gd:
## yours as you put them; else turning the dead, the Second Wind, Mending, then the spells you
## know). Its handler asks the core, and says why not.
func _power(k: int) -> void:
	var power: Array = hotbar.power_at(k)
	if power.is_empty():
		if dusk.character_powers().is_empty():
			hud.say(Prompts.keyed("{power_1} to {power_4} hold your powers: a Cleric's turning and Mending, a Fighter's Second Wind, a Wizard's spells."))
		else:
			hud.say(Prompts.keyed("Nothing on {power_%d} yet." % (k + 1)))
		return
	match power[0]:
		Power.TURN:
			_turn_the_dead()
		Power.SECOND_WIND:
			_second_wind()
		Power.MEND:
			_mend()
		Power.SPELL:
			_ready_spell(power[1])


## The belt's keys, and a click on its loop: drink the potion in loop `k` (DuskWorld.character_belt(),
## potion.rs), or say why not. Quickest from the belt.
func _drink_loop(k: int) -> void:
	var belt: PackedInt32Array = dusk.character_belt()
	var i := k * 3
	if i + 2 >= belt.size() or belt[i] < 0:
		hud.say(Prompts.keyed("Nothing on {belt_%d}: a potion you take goes on your belt." % (k + 1)))
		return
	if held > 0:
		hud.say(Prompts.keyed("Not with it holding you! Struggle {swing} or shove {shove}."))
		return
	if dusk.use_ticks(belt[i], belt[i + 1]) < 0:
		var potions: PackedInt32Array = dusk.potion_info()
		var mana := belt[i + 1] * 4 < potions.size() and potions[belt[i + 1] * 4] == Potion.MANA
		hud.say("You've no mana for it to fill: only a Wizard has." if mana else "You can't drink that now.")
		return
	dusk.use_item(belt[i], belt[i + 1])


## A power: catch your second wind (the Fighter's talent), once between sleeps.
func _second_wind() -> void:
	if dusk.can_second_wind():
		dusk.second_wind()
	else:
		hud.say("Your second wind is spent: it comes back when you've slept.")


## Beside a wild plant: the first time of each kind, what it's good for (its prompt says the key).
func _mind_the_plants() -> void:
	if dead:
		return
	var f: int = dusk.pick_check()
	if f <= 0 or plants_told.has(f):
		return
	plants_told[f] = true
	var names: PackedStringArray = dusk.fixture_names()
	var plant: String = names[f] if f < names.size() else ""
	hud.say("%s." % PLANT_TOLD.get(plant, "Something grows here"))


## E at a carcass or a body: cut up the beast's carcass beside you, if there's one; else pour oil on the body beside
## you and set it alight from the light you hold up.
func _burn_body() -> void:
	var carcass: int = dusk.butcher_check()
	if carcass >= 0:
		dusk.butcher()
		return
	if carcass != -1:  # -1: no carcass, so a body to burn
		hud.say(Prompts.refusal("butcher", carcass))
		return
	var code: int = dusk.burn_check()
	if code < 0:
		hud.say(Prompts.refusal("burn", code))
	else:
		dusk.burn()


## Who comes into the world: a new survivor takes up a class before it's under way. The player's
## own game asks, paused, each class with its line; a trial run asks nothing (--class says which,
## else it stays the first), so scripts and pictures run unattended.
func _offer_classes() -> void:
	class_texts = dusk.class_texts()
	var count := class_texts.size() / 2
	if count < 2 or not dusk.can_choose_class():
		_give()
		return
	if not OS.get_cmdline_user_args().is_empty() and not choose_asked:
		for c in count:
			if class_texts[c * 2].to_lower() == class_wanted.to_lower():
				_take_up_class(c)
		_give()
		return
	choosing_class = true
	dusk.paused = true
	var lines := []
	for c in count:
		lines.append("%d   %s: %s" % [c + 1, class_texts[c * 2], class_texts[c * 2 + 1]])
	lines.append("")
	lines.append("Press a number.")
	hud.show_choice("Who comes to the abbey?", "\n".join(lines))


## --give: the finds a trial run starts with, after its class's kit (DuskWorld.give()).
func _give() -> void:
	for g in gives:
		var parts := g.split(":")
		var props := parts[2].split("+", false) if parts.size() > 2 else PackedStringArray()
		if not dusk.give(parts[0], parts[1] if parts.size() > 1 else "", props):
			push_warning("--give: no such thing as %s" % g)
	gives = PackedStringArray()


## Takes up class `c` (by DuskWorld.class_texts()): its kit in place of the one you came with.
func _take_up_class(c: int) -> void:
	if not dusk.choose_class(c):
		return
	if choosing_class:
		choosing_class = false
		hud.show_choice("", "")
		dusk.paused = false
	bag.start(dusk)
	inventory.start(dusk)
	skills.start(dusk)
	_save("")


## A power (the Cleric's first): hold up the holy symbol and turn the dead (a Cleric's). The core says whether you can.
func _turn_the_dead() -> void:
	match dusk.turn_check():
		-1:
			hud.say("Only a Cleric can turn the dead.")
		-2:
			hud.say("Your faith is spent. Pray at an altar: the abbey's, or the old chapel's in town.")
		_:
			dusk.turn()


## E at an altar: kneel and pray at it beside you. Anyone may, for comfort; a Cleric's faith comes back.
func _pray() -> void:
	if dusk.pray_check() == 1:
		dusk.pray()
	else:
		hud.say("There's no altar beside you to pray at: the abbey's chapel has one, and so does the old chapel in town.")


## E at a body: consecrate it with holy water (or a Cleric's faith), and it never gets up.
func _consecrate() -> void:
	var code: int = dusk.consecrate_check()
	if code >= 0:
		dusk.consecrate()
	else:
		hud.say(Prompts.refusal("consecrate", code if code == -2 else -1))


## Ward the door (E's list): a Cleric with Warding wards the shut door beside you.
func _ward() -> void:
	var code: int = dusk.ward_check()
	if code == 1:
		dusk.ward()
	else:
		hud.say(Prompts.refusal("ward", code))


## A power: mend your worst bleeding wound with faith (Mending, a Cleric's talent).
func _mend() -> void:
	match dusk.heal_check():
		1:
			dusk.heal()
		-2:
			hud.say("There's nothing to mend.")
		-3:
			hud.say("Not enough faith to mend it. Pray at an altar.")
		_:
			hud.say(Prompts.keyed("No mending: it takes the talent (a Cleric's: {skills})."))


## E at a wild plant: pick the one beside you: the one you face, else the one you stand on, else the
## first round you.
func _pick() -> void:
	if dusk.pick_check() > 0:
		dusk.pick()
	else:
		hud.say("Nothing to pick beside you. Wild plants grow in the open, in the woods and by the water.")


## E at a fire: tend the hearth, stove or forge beside you: lay a piece of fuel on, or once it's laid,
## light it from the light you hold up. The core says which, or why not.
func _tend_fire() -> void:
	var code: int = dusk.fire_check()
	if code < 0:
		hud.say(Prompts.refusal("fire", code))
	else:
		dusk.tend_fire()


## Put it out (E's list): the fire beside you. What's left of the wood stays in it for the next time.
func _smother_fire() -> void:
	var f: PackedInt32Array = dusk.fire_beside()
	if f.size() >= 4 and f[3] == 1:
		dusk.smother_fire()
	else:
		hud.say("There's no fire alight beside you.")


## --do's "logs": takes what's kept by the hearth, stove or forge beside you (its first stack).
func _take_logs() -> void:
	var f: PackedInt32Array = dusk.fire_beside()
	if f.size() < 2:
		return
	var reach: PackedInt32Array = dusk.stores_in_reach()
	var i := 0
	while i + 4 < reach.size():
		var count := reach[i + 3]
		if reach[i] == f[0] and reach[i + 1] == f[1] and count > 0:
			dusk.take(f[0], f[1], 0, reach[i + 5])
			return
		i += 5 + count * 4


## Whether the tile (x, y) is within `tiles` of you.
func _near(x: int, y: int, tiles: float) -> bool:
	if me < 0:
		return false
	var where: PackedVector2Array = dusk.actor_positions()
	return me < where.size() and (Vector2(x, y) - where[me]).length() <= tiles


## Pry a board off (E's list): the boarded door or window beside you.
func _pry_board() -> void:
	var code: int = dusk.board_check(true)
	if code < 0:
		hud.say(Prompts.refusal("unboard", code))
	else:
		dusk.unboard()


## Mends the door or window beside you (barricade.rs), or says why not.
func _repair() -> void:
	var code: int = dusk.repair_check()
	if code < 0:
		hud.say(Prompts.refusal("repair", code))
	else:
		dusk.repair()


## Asleep, time runs at SLEEP_SPEED; awake again, at the speed it was.
func _sleep_speed(asleep: bool) -> void:
	if asleep and not sleeping:
		sleeping = true
		wake_speed = dusk.speed
		dusk.speed = SLEEP_SPEED
		dusk.paused = false
	elif not asleep and sleeping:
		sleeping = false
		dusk.speed = wake_speed


## A moodle's level now (0 if you haven't got it).
func _moodle(code: int) -> int:
	for i in range(0, moodles.size() - 1, 2):
		if moodles[i] == code:
			return moodles[i + 1]
	return 0


## Rares get their name over their heads (DuskWorld.actor_rare(), actor_name()).
func _mark_rares() -> void:
	rares = dusk.actor_rare()
	for i in mini(rares.size(), actors.size()):
		if rares[i] == 1:
			actors[i].set_title(dusk.actor_name(i))


## The dead who walked into the patch since the last frame (pressure, in the core): a view each.
## They come in at the end of the core's order, so the others keep their places.
func _add_new_actors() -> void:
	var now: PackedInt32Array = dusk.actor_kinds()
	for i in range(actors.size(), now.size()):
		var view := ActorView.new()
		view.setup(now[i])
		add_child(view)
		actors.append(view)
	kinds = now


func _draw_actors(delta: float) -> void:
	var where: PackedVector2Array = dusk.actor_positions()
	var facing: PackedInt32Array = dusk.actor_facing()
	var alert: PackedInt32Array = dusk.actor_alert()
	_add_new_actors()
	var states: PackedInt32Array = dusk.actor_states()
	if rares.size() != actors.size():
		_mark_rares()
	var remains: PackedInt32Array = dusk.actor_remains()
	var actions: PackedInt32Array = dusk.actor_actions()
	var climbs: PackedFloat32Array = dusk.actor_climbs()
	seen = dusk.actor_seen()
	for i in actors.size():
		# What you don't see isn't drawn (CONCEPT.md: never a live monster you can't see).
		actors[i].visible = _seen(i) and remains[i] != Remains.GONE
		# Climbing, it's lifted over the sill or the fence and down again.
		var lift := sin(climbs[i] * PI) * CLIMB_LIFT if climbs[i] >= 0.0 else 0.0
		actors[i].position = Vector3(where[i].x + 0.5, lift, where[i].y + 0.5)
		actors[i].face(facing[i], delta)
		actors[i].show_alert(alert[i], delta)
		# How it holds itself through what it's doing: a swing's lean (the tell, on the body as well
		# as the ground), a heavy swing rearing back, kneeling to pray, a power's glow.
		actors[i].show_pose(states[i], actions[i * 4], actions[i * 4 + 1] / 1000.0, delta)
		actors[i].show_remains(remains[i], delta)
	tells.show_actions(actions, me, seen)
	var drawn := []
	for view in actors:
		drawn.append(view.position)
	threats.show_threats(dusk.character_threats(), drawn, me)
	if me >= 0:
		map_view.set_focus(actors[me].position + Vector3(0.0, 0.9, 0.0))


func _read_events() -> void:
	var events: PackedInt32Array = dusk.take_events()
	for i in range(0, events.size(), EVENT_SIZE):
		var who := events[i + 1]
		var target := events[i + 2]
		var extra := events[i + 5]
		_fight_effect(events[i], who, target, events[i + 3], events[i + 4], extra)
		match events[i]:
			Event.HEARD:
				if _seen(who):
					hud.say("%s heard something." % _a(who))
			Event.SAW:
				# A wolf howls as it sees you, for the pack (DuskWorld.wild_kinds() 2).
				var wild: PackedInt32Array = dusk.wild_kinds()
				var n := kinds[who] - 1 if who < kinds.size() else -1
				if n >= 0 and n < wild.size() and wild[n] == 2:
					hud.say("%s sees you, and howls for the pack!" % _a(who) if _seen(who) else "A howl: something out there has seen you.")
				else:
					hud.say("%s saw you!" % _a(who) if _seen(who) else "Something has seen you.")
			Event.SMELLED:
				if target == me:
					if _seen(who):
						hud.say("%s has the scent of your blood! Bandage it, or it won't lose you." % _a(who))
					else:
						hud.say(Prompts.keyed("Something has the scent of your blood. Bandage it: {pack}."))
			Event.NO_WAY:
				if who == me:
					hud.say("There's no way there.")
					# Held on one that can't be reached: don't ask again each frame.
					pursuing = -1
			Event.STRUCK:
				# Its harm times 1000, and its wound (the bridge's take_events): the harm rises where
				# it landed, on you or one you see, red on you.
				var harm := floori(extra / 1000.0)
				if target == me or who == me or _seen(target):
					numbers.add(_drawn(target, events[i + 3], events[i + 4]), harm,
						Look.DANGER_TEXT if target == me else Look.TEXT)
				var hurt := extra % 1000
				if target == me and hurt > 0:
					var wound := wound_names[floori(hurt / 100.0) - 1]
					var part := part_names[hurt % 100]
					last_hurt = "%s gave you a %s on the %s." % [_a(who), wound, part]
					last_hurt_at = dusk.tick()
					hud.say(last_hurt)
					hud.flash()
				# Your own blows are numbers now, not news.
			Event.MISSED:
				if who == me:
					hud.say("You missed.")
				elif target == me or _beside(who):
					hud.say("%s reached for you and missed." % _a(who))
			Event.SHOVED:
				if who == me:
					hud.say("You shoved it down." if extra == 1 else "You shoved it back.")
			Event.DIED:
				if who == me:
					_die()
				else:
					hud.say("It's down for good.")
			Event.DOOR:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					var what := "window" if extra & 2 else "door"
					hud.say(("You opened the %s." if extra & 1 else "You shut the %s.") % what)
			Event.EXHAUSTED:
				if who == me:
					hud.say("You're out of breath.")
			Event.GRABBED:
				if target == me:
					hud.say(Prompts.keyed("It has hold of you! {swing} to struggle, {shove} to shove it off."))
					hud.flash()
			Event.FREED:
				if who == me:
					hud.say("You got free.")
			Event.STRUGGLED:
				if who == me:
					hud.say("You couldn't get free.")
			Event.THUMPED:
				# Doors are loud: tell of it now and then, not at every blow (nor of your own).
				var now: float = dusk.tick() / 10.0
				if who != me and now - last_thump > 6.0:
					last_thump = now
					hud.say("Something is beating on a %s." % ("window" if extra & 2 else "door"))
			Event.DOOR_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("A door gave way!")
			Event.WINDOW_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("Glass shatters!")
			Event.FELLED:
				if who == me:
					hud.say("You knock it down. Finish it!")
			Event.TERRIFIED:
				if who == me:
					hud.say("%s breaks and flees from you!" % _a(target))
			Event.WET:
				# A new level of wet (cold.rs): only soaked through is news; the moodle says the rest.
				if who == me and extra == 3:
					hud.say("You're soaked through. Wet, you'll get cold: get under a roof, or by a fire.")
			Event.COLD:
				if who == me:
					_tell_cold(extra)
			Event.CAST:
				if who == me:
					hud.say("You cast %s." % _spell_name(extra))
					_spell_effect(extra, events[i + 3], events[i + 4])
			Event.SCORCHED:
				if target == me:
					hud.say("Your own fireball burns you!" if who == me else "Fire burns you!")
			Event.SPELL_BROKE:
				if who == me:
					hud.say("The blow breaks your words: the %s is lost. Nothing's spent." % _spell_name(extra))
					effects.spell_broke(actors[me].position)
					effects.rise(actors[me].position, Look.ARCANE)
			Event.GREW:
				# At midnight: the day's exertion made you stronger or fitter (extra: the stat × 100
				# plus the level).
				if who == me:
					var strength := extra < 100
					hud.say(("You feel stronger: Strength %d." if strength else "You feel fitter: Fitness %d.") % (extra % 100))
			Event.DRAGGED_DOWN:
				if who == me:
					dragged_by = extra
			Event.TOOK:
				if who == me:
					hud.say("You took %s." % _items(extra))
			Event.DROPPED:
				if who == me:
					hud.say("You dropped %s." % _items(extra))
			Event.NO_ROOM:
				if who == me:
					hud.say("No room for %s: it's at your feet." % _items(extra))
			Event.WIELDED:
				if who == me:
					var kinds: PackedInt32Array = dusk.item_kinds()
					var shield := extra < kinds.size() and kinds[extra] == Item.SHIELD
					hud.say(("You took up the %s on your arm. Hold the right mouse button to brace behind it." if shield
						else "You took the %s in hand.") % _item_name(extra))
			Event.WEAPON_BROKE:
				if who == me:
					var kinds: PackedInt32Array = dusk.item_kinds()
					var worn := extra < kinds.size() and kinds[extra] == Item.ARMOUR
					hud.say(("Your %s falls apart!" if worn else "Your %s broke!") % _item_name(extra))
			Event.GLANCED:
				# Every blow that glances would fill the news: now and then.
				if who == me and played - glanced_at > 3.0:
					glanced_at = played
					hud.say("The blow glances off your %s." % _item_name(extra % 65536))
				elif target == me and played - skidded_at > 15.0:
					skidded_at = played
					hud.say("Your blow glances off %s's %s. A point would go through." % [_a(who), _item_name(extra % 65536)])
			Event.WORE:
				if who == me:
					hud.say("You put on the %s." % _item_name(extra))
			Event.TOOK_OFF:
				if who == me:
					hud.say("You take off the %s." % _item_name(extra))
			Event.CLIMBED:
				# One of the dead that tumbles over is a free blow while it's down: say so when
				# you see it, now and then.
				if who != me and extra & 2 and _seen(who) and played - tumbled_at > 10.0:
					tumbled_at = played
					hud.say("%s tumbles in over the %s. Finish it while it's down!" % [_a(who), "sill" if extra & 1 else "fence"])
				elif who == me and extra & 4:
					hud.say("The broken glass cut you as you climbed through. Bandage it!")
					hud.flash()
			Event.BANDAGED:
				if who == me:
					hud.say("You bandaged your %s." % part_names[extra])
			Event.ATE:
				if who == me:
					hud.say("You ate the %s." % _item_name(extra))
			Event.DRANK:
				if who == me:
					hud.say("You drank the %s." % _item_name(extra))
			Event.DRANK_WATER:
				if who == me:
					hud.say("You drank your fill.")
			Event.QUAFFED:
				# A potion (potion.rs): what it does, in the core's words.
				if who == me:
					var does: String = potion_lines[extra] if extra >= 0 and extra < potion_lines.size() else ""
					hud.say("You drink the %s%s." % [_item_name(extra), ": " + does if does != "" else ""])
					hud.flash_good()
			Event.WANED:
				if who == me:
					hud.say(WANED_TOLD.get(extra, "A potion wears off."))
			Event.FELL_ASLEEP:
				if who == me:
					hud.say("You fell asleep in the bed." if extra == 1 else "You fell asleep on the ground.")
			Event.WOKE:
				if who == me and extra >= 0 and extra < WOKE_BY.size():
					hud.say(WOKE_BY[extra])
			Event.LIT:
				if who == me:
					hud.say("You lit the %s and held it up." % _item_name(extra))
			Event.DOUSED:
				if who == me:
					hud.say("You put out the %s." % _item_name(extra))
			Event.BURNT_OUT:
				if who == me:
					hud.say("Your %s burnt out." % _item_name(extra))
			Event.NIGHTFALL:
				hud.say("The sun is going down. Night is coming.")
			Event.DAYBREAK:
				hud.say("The sky is lightening. Dawn is coming.")
			Event.SKILL_UP:
				if who == me:
					hud.say(Prompts.keyed("%s rose to level %d.  ({skills}: skills)") % [_skill_name(extra), extra % 100])
					effects.gold(actors[me].position, false)
			Event.SKILL_AT_CAP:
				if who == me:
					if dusk.masteries_left() > 0:
						hud.say(Prompts.keyed("%s will rise no further unless you master it: {skills}.") % _skill_name(extra))
					else:
						hud.say("%s will rise no further: your masteries are taken." % _skill_name(extra))
			Event.MASTERED:
				if who == me:
					hud.say("You mastered %s: level %d, and on to 10." % [_skill_name(extra), extra % 100])
					effects.gold(actors[me].position, false)
			Event.BOARDED:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You nailed a board across it: %d on it now." % extra)
			Event.UNBOARDED:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You pried a board off: %d left." % extra)
			Event.REPAIRED:
				if who == me:
					hud.say("You mended it: it's whole again." if extra == 1 else "You re-nailed the board: it's as strong as you nail them.")
			Event.PUT_UP:
				# A door or a wall of your own (build.rs): the edge is drawn again.
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You hang the door: it's shut." if extra == 1 else "You put up the wall of planks.")
			Event.WALL_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("A wall of planks gave way!")
			Event.MADE_HOME:
				if who == me:
					hud.say("You slept the night through. This is home now: a container under its roof can be your stash.")
			Event.STASHED:
				if who == me:
					hud.say("That's your stash. Finds put into it pay their worth.")
			Event.PUT:
				if who == me:
					hud.say("You put away %s." % _items(extra))
			Event.BROUGHT_HOME:
				# (x, y) carry the item and how many; extra is the XP.
				if who == me:
					hud.say("Brought home: %s, %d XP." % [_items(events[i + 3] * 1000 + events[i + 4]), extra])
			Event.LEVEL_UP:
				if who == me:
					hud.say(Prompts.keyed("You reached level %d. A little more health, and a talent to choose: {skills}.") % extra)
					hud.flash_good()
					effects.gold(actors[me].position, true)
			Event.FEVERED:
				if who == me:
					hud.say("The wound burns. You've caught ghoul fever: three days to find a cure.")
					hud.flash()
			Event.FEVER_WORSE:
				if who == me:
					if extra >= 3:
						hud.say("You're dying of the fever. A day left to find the cure.")
					else:
						hud.say("The fever has you weak: your breath comes back slowly.")
					hud.flash()
			Event.FEVER_TOOK:
				if who == me:
					fever_took = true
			Event.CURED:
				if who == me:
					hud.say("You took the cure. The fever breaks.")
					hud.flash_good()
			Event.DIED:
				# The first of the dead you see go down: what happens next, and what stops it.
				if who != me and _seen(who) and not told_get_up:
					told_get_up = true
					hud.say(Prompts.keyed("It'll get up again in a day or so, unless you burn it (oil and a flame) or consecrate it (holy water): {use} beside it."))
			Event.GOT_UP:
				if _seen(who):
					hud.say("%s gets up again!" % _a(who))
			Event.KINDLED:
				if who == me:
					hud.say("You set it alight. It won't get up again.")
			Event.CAUGHT:
				if _seen(who):
					hud.say("The fire spreads to another body.")
			Event.BOUGHT:
				if who == me:
					hud.say("Bought %s for %d coin." % [_item_name(events[i + 3]), extra])
			Event.SOLD:
				if who == me:
					hud.say("Sold %s for %d coin." % [_items(events[i + 3] * 1000 + events[i + 4]), extra])
			Event.SUPPLIED:
				if who == me:
					hud.say("The sisters were short of that. The dusk falls back a little (%.1f%%)." % (extra / 10.0))
					hud.flash_good()
			Event.MENDED:
				if who == me:
					hud.say("Your %s is mended, good as new." % _item_name(events[i + 3]))
			Event.TOLLING_TONIGHT:
				hud.say("Tonight the old chapel's bell will toll, at 22:00. The dead will come to it, and they won't give up till dawn.")
			Event.TOLLING_BEGAN:
				hud.say("The bell is tolling! Every dead thing for miles is walking to the chapel.")
				hud.flash()
			Event.TOLLING_ENDED:
				hud.say("Dawn. The bell is quiet, and the dead give up as they did.")
				hud.flash_good()
			Event.LIGHT_EATEN:
				if target == me:
					hud.say("%s puts out your light! It wants you in the dark." % dusk.actor_name(who))
					hud.flash()
			Event.TALENT_CHOSEN:
				if who == me:
					var texts: PackedStringArray = dusk.talent_texts()
					if extra * 2 + 1 < texts.size():
						hud.say("Talent: %s. %s." % [texts[extra * 2], texts[extra * 2 + 1].left(1).to_upper() + texts[extra * 2 + 1].substr(1)])
			Event.SECOND_WIND:
				if who == me:
					hud.say("You catch your second wind.")
					hud.flash_good()
					effects.second_wind(actors[me].position)
			Event.CLASS_CHOSEN:
				if who == me and extra * 2 < class_texts.size():
					hud.say("You take up the %s's way." % class_texts[extra * 2])
			Event.TURNED:
				if who == me:
					effects.turn(actors[me].position, dusk.turn_reach())
					var turned := extra % 1000
					var fell := floori(extra / 1000.0)
					if turned + fell == 0:
						hud.say("You hold up the holy symbol, but none of the dead near you see it.")
					else:
						# One or many; with Sunder, the weakest fall instead.
						var flee := "One of the dead turns and flees" if turned == 1 else "%d of the dead turn and flee" % turned
						var fall := "one falls" if fell == 1 else "%d fall" % fell
						if turned == 0:
							hud.say("You hold up the holy symbol! %s." % ("One of the dead falls" if fell == 1 else "%d of the dead fall" % fell))
						else:
							hud.say("You hold up the holy symbol! %s%s." % [flee, ", and %s" % fall if fell > 0 else ""])
						hud.flash_good()
			Event.PRAYED:
				if who == me:
					hud.say("You pray. Your faith comes back, and your dread eases." if extra == 1 else "You pray, and your dread eases.")
					hud.flash_good()
					effects.rise(actors[me].position, Look.HOLY)
			Event.CONSECRATED:
				if who == me:
					hud.say("You pour the holy water over it. It won't get up again." if extra == 1 else "You say the rites over it. It won't get up again.")
					effects.rise(Vector3(events[i + 3] + 0.5, 0.0, events[i + 4] + 0.5), Look.HOLY)
			Event.HEALED:
				if who == me:
					hud.say("Your faith closes the wound.")
					hud.flash_good()
					effects.rise(actors[me].position, Look.GOOD)
			Event.WARDED:
				if who == me:
					hud.say("You ward the door. The dead won't beat on it tonight.")
					effects.ward(events[i + 3], events[i + 4], extra)
			Event.PICKED:
				map_view.refresh_tile(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					var picked := extra / 65536
					if picked > 0:
						hud.say("You pick it: %s ×%d." % [_item_name(extra % 65536), picked])
					else:
						hud.say("You pick it over, and find nothing worth taking.")
			Event.REGREW:
				map_view.refresh_tile(Vector2i(events[i + 3], events[i + 4]))
			Event.WARD_FADED:
				if _near(events[i + 3], events[i + 4], 16.0):
					hud.say("The ward on the door has faded.")
			Event.RAIN_BEGAN:
				hud.say("It's raining hard. Lights gutter outdoors; sound and blood don't carry far." if extra == 1 else "It starts to rain: lights gutter outdoors, and the dead hear and smell less.")
			Event.RAIN_ENDED:
				hud.say("The rain stops.")
			Event.RAINED_OUT:
				if who == me:
					hud.say("The rain puts out your %s. Light it under a roof." % _item_name(extra))
			Event.IDENTIFIED:
				if who == me:
					hud.say("You know what it is now: %s." % _known_as(extra))
					hud.flash_good()
			Event.CURSE_CLINGS:
				if who == me:
					hud.say("The %s clings to your hand: it's cursed! The abbey can lift a curse." % _item_name(extra))
					hud.flash()
			Event.CURSE_LIFTED:
				if who == me:
					hud.say("The sisters lift the curse. The %s lets go of your hand." % _item_name(extra))
					hud.flash_good()
			Event.BOUNTY_CLAIMED:
				if who == me:
					hud.say("The sisters pay the bounty on %s: %d coin, and the dusk falls back." % [dusk.actor_name(target), extra])
					hud.flash_good()
			Event.RISEN:
				# Your old body: you'd want to know, seen or not.
				hud.say("Midnight. %s's body rises as a revenant, in the gear it fell in." % dusk.actor_name(who))
			Event.SHOT:
				_streak(Vector2i(extra / 4096, extra % 4096), Vector2i(events[i + 3], events[i + 4]))
			Event.BLOCKED:
				if who == me and played - blocked_at > 3.0:
					blocked_at = played
					hud.say("You take the blow on your shield.")
			Event.THORNED:
				# A unique's thorns (magic.rs): what you wear, or the shield you blocked with.
				if who == me and played - thorned_at > 3.0:
					thorned_at = played
					hud.say("%s is hurt striking you." % _a(target))
			Event.DRIVEN_BACK:
				# A spear's edge (edge.rs): a blow drove it back a tile, out of reach.
				if who == me and played - driven_at > 3.0:
					driven_at = played
					hud.say("Your blow drives it back.")
			Event.DECOY_SET:
				# Wound and set down (decoy.rs): it rattles once its time's up, and the dead come.
				if who == me:
					hud.say("You wind the decoy and set it down: it starts rattling in %d s. Be elsewhere." % extra)
			Event.SNEAK_ATTACK:
				if who == me:
					hud.say("A sneak attack! %s never knew you were there." % _a(target))
			Event.STARTLED:
				if who < seen.size() and seen[who] == 1:
					hud.say("%s takes fright and bolts!" % _a(who))
			Event.BUTCHERED:
				if who == me:
					var count := extra / 65536
					hud.say("You cut it up: %d %s." % [count, _item_name(extra % 65536)])
			Event.SPOILED:
				if who == me:
					hud.say("Your %s has gone off. You throw it away." % _item_name(extra))
			Event.CRAFTED:
				if who == me and extra * 2 < recipe_texts.size():
					hud.say("You made a %s." % recipe_texts[extra * 2])
			Event.BROKE_DOWN:
				map_view.refresh_tile(Vector2i(events[i + 3], events[i + 4]))
				if who == me:
					hud.say("You break it down: %d plank%s." % [extra, "" if extra == 1 else "s"])
			Event.BOARD_BROKE:
				map_view.refresh_edge(Vector2i(events[i + 3], events[i + 4]))
				hud.say("The last board gave way!" if extra == 0 else "A board gave way! %d left." % extra)
			Event.LAID:
				if who == me:
					var f: PackedInt32Array = dusk.fire_beside()
					if f.size() >= 4 and f[3] == 1:
						hud.say("You put another %s on the fire." % _item_name(extra))
					else:
						hud.say(Prompts.keyed("You lay the fire. Hold up a light {light}, and {use} lights it."))
			Event.FIRE_LIT:
				if who == me:
					hud.say("The fire catches. The room is lit, and so is anything seen through its windows.")
			Event.SMOTHERED:
				if who == me:
					hud.say("You put the fire out. What's left of the wood stays in it.")
				else:
					hud.say("%s puts out the fire! It wants you in the dark." % dusk.actor_name(who))
					hud.flash()
			Event.FIRE_OUT:
				if _near(events[i + 3], events[i + 4], 16.0):
					hud.say("The fire has burnt out.")
			Event.SAW_FIRE:
				# Once in a while: a crowd seeing it would fill the news.
				if played - fire_seen_at > 20.0:
					fire_seen_at = played
					if _seen(who):
						hud.say("%s has seen the firelight, and comes to it." % _a(who))
					else:
						hud.say("Something out in the dark has seen the firelight.")
			Event.DREAD:
				if who == me:
					_tell_dread(extra)
			Event.BALKED:
				if who == me and played - balked_at > 4.0:
					balked_at = played
					hud.say("You can't make yourself go nearer the dead.")
			Event.RESISTED:
				# Once in a while: every blow on bone would fill the news.
				if who == me and played - resisted_at > 15.0:
					resisted_at = played
					if extra == 1:
						hud.say("The blow shatters bone!")
					else:
						hud.say("Your blow glances off the bones. Something blunt would shatter them.")


## An arrow's flight (the core's Event.SHOT news): a thin streak from the bow to where it landed,
## fading in a moment. Only the look: where it went is the core's.
func _streak(from: Vector2i, to: Vector2i) -> void:
	var a := Vector3(from.x + 0.5, 1.2, from.y + 0.5)
	var b := Vector3(to.x + 0.5, 0.9, to.y + 0.5)
	var length := a.distance_to(b)
	if length < 0.1:
		return
	var box := BoxMesh.new()
	box.size = Vector3(0.04, 0.04, length)
	var streak := MeshInstance3D.new()
	streak.mesh = box
	var paint := StandardMaterial3D.new()
	paint.albedo_color = Look.SHOT
	paint.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	paint.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	streak.material_override = paint
	streak.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(streak)
	streak.look_at_from_position((a + b) / 2.0, b, Vector3.UP)
	var fade := create_tween()
	fade.tween_property(paint, "albedo_color:a", 0.0, 0.35)
	fade.tween_callback(streak.queue_free)


## Right-click: shoot the bow in your hand at the tile under the cursor. The core says whether
## you can; this says why not.
func _shoot(tile: Vector2i) -> void:
	match dusk.shoot_check(tile.x, tile.y):
		-1:
			hud.say("There's no bow in your hand.")
		-2:
			hud.say("You have no arrows.")
		-3:
			hud.say("A bow takes both hands: put out the light, or put away the shield.")
		-4:
			hud.say("Too far for the bow.")
		_:
			dusk.shoot(tile.x, tile.y)


## The right mouse button held, or let go: on guard (guard.rs), and braced behind a shield if one's
## on your arm.
func _guard(on: bool) -> void:
	guarding = on
	dusk.set_guard(on)
	var shield: bool = dusk.character_shield().size() >= 2
	if shield or bracing:
		bracing = on and shield
		dusk.set_bracing(bracing)


## Each frame on guard: you face the cursor on the ground (or --aim's spot, for pictures). And the
## health bars over the dead you see.
func _mind_the_guard() -> void:
	if guarding and not dead and me >= 0:
		var p := _ground_under(get_viewport().get_mouse_position())
		if aim_offset != Vector2i.ZERO:
			p = Vector2(_my_tile() + aim_offset) + Vector2(0.5, 0.5)
		dusk.face_toward(p.x, p.y)
	var at := []
	for view in actors:
		at.append(view.position)
	bars.show_bars(camera, at, kinds, seen, rares, dusk.actor_health())


## A click on guard: the bow in your hand shot at the spot, else a swing where you face (Shift: a
## heavy one); held, a struggle.
func _guarded_click(tile: Vector2i) -> void:
	run_tapped = false
	var hands: PackedInt32Array = dusk.character_hands()
	var item_kinds: PackedInt32Array = dusk.item_kinds()
	if hands.size() >= 2 and hands[0] < item_kinds.size() and item_kinds[hands[0]] == Item.BOW:
		_shoot(tile)
	elif held > 0:
		dusk.struggle()
	elif Input.is_action_pressed("run"):
		dusk.heavy_attack(0, 0)
	else:
		dusk.attack(0, 0)


## The nearest of the dead you see, by its index in the actor arrays; -1 for none.
func _nearest_dead() -> int:
	var where: PackedVector2Array = dusk.actor_positions()
	var best := -1
	for k in where.size():
		if k != me and _seen(k) and k < kinds.size() and kinds[k] != 0 \
				and (best < 0 or where[k].distance_to(where[me]) < where[best].distance_to(where[me])):
			best = k
	return best


## The right mouse button, held: brace behind your shield (none on your arm, and the news says so).
func _brace(on: bool) -> void:
	if on and dusk.character_shield().size() < 2:
		hud.say(Prompts.keyed("You've no shield on your arm to brace behind. Take one up: {pack}."))
		return
	dusk.set_bracing(on)


## How cold you are reached a new level (the core's Event.COLD news, cold.rs): told as it rises, and
## when you're warm again. The cold drifts slowly, so it doesn't hover at a level's edge.
func _tell_cold(level: int) -> void:
	if level > cold_was:
		match level:
			1:
				hud.say("You're getting chilly.")
			2:
				hud.say("You're cold, and shivering: your aim is worse, and everything costs more breath.")
			3:
				hud.say("You're freezing! Nothing heals, and the cold is taking your health. Get to a fire.")
				hud.flash()
	elif level == 0:
		hud.say("You're warm again.")
	cold_was = level


## Your dread reached a new level (the core's Event.DREAD news): told as it rises, and when it lets go.
## It hovers at a level's edge as the dead come and go from sight, so each level is told at most
## once in DREAD_TOLD_EVERY seconds.
func _tell_dread(level: int) -> void:
	var told: float = dread_told.get(level, -100.0)
	var again := played - told > DREAD_TOLD_EVERY
	if level != dread_was and again:
		dread_told[level] = played
	if level > dread_was and again:
		match level:
			1:
				hud.say("Your nerve slips, and your aim with it. Light, food and rest ease dread.")
			2:
				hud.say("You're afraid, and your hands shake. Get into the light.")
			3:
				hud.say("You're frightened! You can't make yourself go nearer the dead.")
				hud.flash()
	elif level == 0 and again:
		hud.say("Your nerve steadies.")
	elif level < dread_was and dread_was == 3 and again:
		hud.say("The worst of the fear passes.")
	dread_was = level


## "A shambler", "A ghoul", or a rare's own name: what actor `i` is, for the news.
func _a(i: int) -> String:
	if i >= 0 and i < rares.size() and rares[i] == 1:
		return dusk.actor_name(i)
	var n := kinds[i] - 1 if i >= 0 and i < kinds.size() else -1
	if n < 0 or n >= creature_names.size():
		return "Something"
	var name := creature_names[n]
	return ("An " if "aeiou".contains(name.left(1)) else "A ") + name


## Whether actor `i` stands beside you.
func _beside(i: int) -> bool:
	if me < 0 or i < 0:
		return false
	var where: PackedVector2Array = dusk.actor_positions()
	return (where[i] - where[me]).length() < 1.6


## How you died, from what the client last saw: a hold, a blow just now, bleeding, or hunger and
## thirst (the moodles of the frame before).
func _die() -> void:
	dead = true
	dread_was = 0  # the next survivor starts calm
	var how := last_hurt
	var starving := _moodle(Moodle.HUNGRY) >= 3
	var parched := _moodle(Moodle.THIRSTY) >= 3
	if fever_took:
		how = "The ghoul fever took you."
	elif dragged_by >= 3:
		how = "%d of them got hold of you, and dragged you down." % dragged_by
	elif dragged_by > 0:
		how = "It had hold of you, and you had no breath left to throw it off."
	elif dusk.tick() - last_hurt_at <= 1:
		pass  # the blow itself
	elif dusk.character_body()[4] > 0:
		how = "You bled to death."
	elif _moodle(Moodle.FEVER) >= 3:
		how = "The ghoul fever wore you down."
	elif starving and parched:
		how = "Hunger and thirst wore you down."
	elif parched:
		how = "You died of thirst."
	elif starving:
		how = "You starved."
	elif how.is_empty():
		how = "You died."
	_show_recap(how)
	_save("")  # the world keeps your death: there's no going back to before it


## The death screen's recap: who, how, and how long.
func _show_recap(how: String) -> void:
	hud.show_death(dusk.actor_name(me), how, dusk.character_lasted(), true)


## A button of the death screen: Restart (the next survivor, in this world), or New World.
func _on_death_chose(what: String) -> void:
	if what == "restart":
		_next_survivor()
	else:
		town_seed += 1
		_start()


## After a death: the next survivor comes to the abbey, in this world. The old body stays where
## it fell, with its gear; the home and stash carry over.
func _next_survivor() -> void:
	if not dusk.next_character():
		return
	_add_new_actors()
	me = dusk.character_index()
	dead = false
	fever_took = false
	hallowed = false
	last_hurt = ""
	last_hurt_at = -100
	held = 0
	dragged_by = 0
	moodles = PackedInt32Array()
	sleeping = false
	hud.hide_death()
	bag.start(dusk)
	inventory.start(dusk)
	skills.start(dusk)
	craft.start(dusk)
	spells.start(dusk)
	hotbar.start(dusk)
	readied = -1
	hud.say("%s comes to the abbey. Somewhere out there, the last one's body lies in its gear." % dusk.actor_name(me))
	_save("")
	_offer_classes()


func _move_camera(delta: float) -> void:
	if me >= 0:
		focus = focus.lerp(actors[me].position, 1.0 - exp(-6.0 * delta))
	camera.size = lerpf(camera.size, zoom, 1.0 - exp(-10.0 * delta))
	var pitch := deg_to_rad(CAMERA_PITCH)
	var yaw := deg_to_rad(CAMERA_YAW)
	var back := Vector3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch))
	camera.look_at_from_position(focus + back * CAMERA_DISTANCE, focus, Vector3.UP)


func _move_cursor() -> void:
	var tile := _tile_under(get_viewport().get_mouse_position())
	var size: Vector2i = dusk.map_size()
	cursor.visible = tile.x >= 0 and tile.y >= 0 and tile.x < size.x and tile.y < size.y
	cursor.position = Vector3(tile.x + 0.5, 0.025, tile.y + 0.5)


## The tile under a point on the screen, where the mouse ray meets the ground.
func _tile_under(screen: Vector2) -> Vector2i:
	var p := _ground_under(screen)
	return Vector2i(floori(p.x), floori(p.y)) if p.x > -1e8 else Vector2i(-1, -1)


## The point on the ground under `screen`, in tiles (x, y); far off (-1e9) if there's none.
func _ground_under(screen: Vector2) -> Vector2:
	var from := camera.project_ray_origin(screen)
	var hit: Variant = Plane(Vector3.UP, 0.0).intersects_ray(from, camera.project_ray_normal(screen))
	if hit == null:
		return Vector2(-1e9, -1e9)
	return Vector2(hit.x, hit.z)


## The camera, the sun, the sky's light and the tile cursor.
func _build_stage() -> void:
	camera = Camera3D.new()
	camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	camera.size = zoom
	camera.near = 0.1
	camera.far = 200.0
	add_child(camera)

	sun = DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55.0, -35.0, 0.0)  # from the south-west: shadows fall up-right
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_ORTHOGONAL
	sun.directional_shadow_max_distance = 90.0
	add_child(sun)

	sky = Environment.new()
	sky.background_mode = Environment.BG_COLOR
	sky.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	sky.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	var world_environment := WorldEnvironment.new()
	world_environment.environment = sky
	add_child(world_environment)
	_light_the_sky(100)

	torch = OmniLight3D.new()
	torch.light_color = Look.TORCH
	torch.shadow_enabled = true
	torch.omni_attenuation = 1.2
	torch.visible = false
	add_child(torch)

	var square := PlaneMesh.new()
	square.size = Vector2.ONE
	var outline := ShaderMaterial.new()
	outline.shader = preload("res://shaders/cursor.gdshader")
	cursor = MeshInstance3D.new()
	cursor.mesh = square
	cursor.material_override = outline
	cursor.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(cursor)


## The sun, the sky's light and the background follow the core's daylight (DuskWorld.daylight(),
## out of 100): a low blue moon at night, warm near dusk and dawn, white by day. Only the look:
## what you can see is the core's view.
func _light_the_sky(daylight: int) -> void:
	var t := clampf(daylight / 100.0, 0.0, 1.0)
	var warm := clampf(1.0 - absf(t - 0.6) / 0.3, 0.0, 1.0)  # dusk and dawn
	sun.light_energy = lerpf(0.06, 1.1, t * t)
	sun.light_color = Look.SUN_NIGHT.lerp(Look.SUN_DAY, t).lerp(Look.SUN_DUSK, warm * 0.55)
	sky.ambient_light_energy = lerpf(0.16, 0.6, t)
	sky.ambient_light_color = Look.AMBIENT_NIGHT.lerp(Look.AMBIENT_DAY, t)
	sky.background_color = Look.SKY_NIGHT.lerp(Look.SKY_DAY, t)


## The light you hold up (DuskWorld.character_light()): a real light over your head, reaching
## as far as the core's does, with a little flicker.
func _hold_up_light() -> void:
	var held: PackedInt32Array = dusk.character_light()
	torch.visible = held.size() >= 3 and me >= 0 and not dead
	if not torch.visible:
		return
	torch.omni_range = held[2] + 1.0
	var now := Time.get_ticks_msec() / 1000.0
	torch.light_energy = 3.0 * (1.0 + 0.06 * sin(now * 13.0) + 0.04 * sin(now * 31.0))
	# In the hand on the camera's side, so it lights the body you see, not from inside it.
	torch.position = actors[me].position + Vector3(0.35, 1.8, 0.35)


## The fires alight in hearths, stoves and forges (DuskWorld.hearth_fires()): over each, embers
## glowing on its top and a real light with shadows, reaching as far as the core's, flickering.
## Only the look: what a fire lights, and who sees it, is the core's.
func _light_the_fires() -> void:
	var lit: PackedInt32Array = dusk.hearth_fires()
	var now := Time.get_ticks_msec() / 1000.0
	var alight := {}
	for i in range(0, lit.size() - 4, 5):
		var at := Vector2i(lit[i], lit[i + 1])
		alight[at] = true
		if not fires.has(at):
			fires[at] = _make_fire()
		var fire: OmniLight3D = fires[at]
		# Over its embers, where its look lays a fire (looks.toml's fire), as it stands; over the
		# tile till its chunk is drawn.
		var spot: Variant = map_view.fire_spot(at)
		fire.position = (spot if spot != null else Vector3(at.x + 0.5, 1.0, at.y + 0.5)) \
			+ Vector3(0.0, FIRE_OVER, 0.0)
		fire.omni_range = lit[i + 2] + 1.0
		var phase := float(at.x * 7 + at.y * 13)
		fire.light_energy = 2.6 * (1.0 + 0.08 * sin(now * 11.0 + phase) + 0.05 * sin(now * 27.0 + phase))
	for at: Vector2i in fires.keys():
		if not alight.has(at):
			fires[at].queue_free()
			fires.erase(at)


## A fire's light, and its embers under it (`_light_the_fires` sets it over its fixture's fire).
func _make_fire() -> OmniLight3D:
	var fire := OmniLight3D.new()
	fire.light_color = Look.FIRE
	fire.shadow_enabled = true
	fire.omni_attenuation = 1.3
	var embers := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.5, 0.12, 0.45)
	embers.mesh = box
	var glow := StandardMaterial3D.new()
	glow.albedo_color = Look.EMBER
	glow.emission_enabled = true
	glow.emission = Look.EMBER
	glow.emission_energy_multiplier = 4.0
	embers.material_override = glow
	embers.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	embers.position = Vector3(0.0, -FIRE_OVER, 0.0)
	fire.add_child(embers)
	add_child(fire)
	return fire


## The rain: thin streaks falling in a box round where the camera looks, as many as the core's
## rain says (DuskWorld.rain()). Only the look: what it does is the core's.
func _let_it_rain() -> void:
	var drops: int = RAIN_DROPS[clampi(dusk.rain(), 0, 2)]
	if rain_fall == null:
		rain_fall = GPUParticles3D.new()
		var fall := ParticleProcessMaterial.new()
		fall.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
		fall.emission_box_extents = Vector3(22.0, 0.5, 22.0)
		fall.direction = Vector3(0.12, -1.0, 0.05)
		fall.spread = 3.0
		fall.initial_velocity_min = 16.0
		fall.initial_velocity_max = 20.0
		fall.gravity = Vector3(0.0, -9.8, 0.0)
		rain_fall.process_material = fall
		var streak := BoxMesh.new()
		streak.size = Vector3(0.02, 0.55, 0.02)
		var wet := StandardMaterial3D.new()
		wet.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		wet.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		wet.albedo_color = Look.RAIN
		streak.material = wet
		rain_fall.draw_pass_1 = streak
		rain_fall.lifetime = 0.9
		rain_fall.visibility_aabb = AABB(Vector3(-25, -20, -25), Vector3(50, 25, 50))
		rain_fall.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(rain_fall)
	rain_fall.emitting = drops > 0
	if drops > 0 and rain_fall.amount != drops:
		rain_fall.amount = drops
	rain_fall.position = focus + Vector3(0.0, 14.0, 0.0)


## Light (R): light the first torch or candle in your pack, or put out the one you hold up.
func _light_or_douse() -> void:
	if dusk.character_light().size() > 0:
		dusk.douse()
		return
	var items: PackedInt32Array = dusk.character_items()
	var kinds: PackedInt32Array = dusk.item_kinds()
	for i in range(0, items.size() - 3, 4):
		if items[i] < kinds.size() and kinds[items[i]] == Item.LIGHT:
			dusk.light(floori(i / 4.0), items[i])
			return
	hud.say("You have nothing to light.")


## Zooms by `notches` (out for more, in for fewer; a part of one from a trackpad), within
## ZOOM_MIN and ZOOM_MAX. The camera eases to it (`_move_camera`); the player's own game keeps it.
func _zoom(notches: float) -> void:
	zoom = clampf(zoom * pow(ZOOM_STEP, notches), ZOOM_MIN, ZOOM_MAX)
	settings.keep_zoom(zoom)
	if print_news:
		print("zoom: ", snappedf(zoom, 0.01))


## Whether you see actor `i` now.
func _seen(i: int) -> bool:
	return i >= 0 and i < seen.size() and seen[i] == 1


## The player's own game fills the screen, with the settings it was left with (VISUALS.md §13):
## the GUI's size, what's on or off, any keys remapped. A trial run keeps the 1600 x 900 window
## and the defaults, so its pictures stay alike, with any --look settings on.
func _settle_window() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		keys_mode = 2
		settings.turn_on(look_flags)
		return
	get_window().mode = Window.MODE_MAXIMIZED
	settings.load_kept()
	zoom = clampf(settings.zoom, ZOOM_MIN, ZOOM_MAX)


## The settings screen opened or closed: the world waits while it's open, and goes on as it was.
func _on_settings(open: bool) -> void:
	if dusk == null:
		return
	if open:
		paused_before = dusk.paused
		dusk.paused = true
	else:
		dusk.paused = paused_before

## Esc with nothing open: the menu, the world waiting under it (menu.gd).
func _open_menu() -> void:
	if dusk != null:
		menu_paused_before = dusk.paused
		dusk.paused = true
	menu.open(save_path != "")


## The menu closed: the world goes on as it was.
func _close_menu() -> void:
	menu.close()
	if dusk != null:
		dusk.paused = menu_paused_before


## A button of the menu: go on, the settings over it, or save and quit.
func _on_menu(what: String) -> void:
	match what:
		"resume":
			_close_menu()
		"settings":
			settings.open()
		"quit":
			_save("")
			get_tree().quit()


## Shift tapped: running, or walking (quieter, and your breath comes back).
func _toggle_run() -> void:
	running = not running


## F1's keys (zone I): open while F1 has them; for a world's first game hour in the player's own
## game, until F1 is first pressed, a line saying they're there.
func _mind_the_keys(clock: PackedInt32Array) -> void:
	var first_hour := clock.size() > 3 and clock[3] < 60
	hud.show_keys(keys_mode == 1, keys_mode == 0 and first_hour)


## F1: the keys (zone I), over the middle of the screen. Panels don't stack (VISUALS.md §10):
## opening the keys closes the book panel, which they'd lie over.
func _toggle_keys() -> void:
	keys_mode = 2 if hud.keys_shown() else 1
	if keys_mode == 1:
		for panel: Node in [skills, craft, spells]:
			panel.close()


## Opens or closes a book panel (zone J: skills, making, spells), one at a time: opening one
## closes the others, and the keys.
func _book(panel: Node) -> void:
	for other: Node in [skills, craft, spells]:
		if other != panel:
			other.close()
	panel.toggle()
	if panel.is_open() and keys_mode == 1:
		keys_mode = 2


## Esc backs out of what's open before it opens the menu (VISUALS.md §10): the settings close, the
## menu closes (the game goes on), what you hold in the inventory goes back, the inventory shuts,
## E's list closes, a readied spell is let go, then the keys close, then the book panel. True if it
## closed something.
func _close_top() -> bool:
	if settings != null and settings.is_open():
		settings.close()
		return true
	if menu != null and menu.is_open():
		_close_menu()
		return true
	if dusk == null:
		return false
	if inventory.back():
		return true
	if listing:
		_close_here()
		return true
	if readied >= 0:
		_let_go()
		return true
	if hud.keys_shown():
		keys_mode = 2
		return true
	for panel: Node in [skills, craft, spells]:
		if panel.is_open():
			panel.close()
			return true
	return false


func _read_args() -> void:
	for arg in OS.get_cmdline_user_args():
		var value := arg.get_slice("=", 1)
		if arg.begins_with("--seed="):
			town_seed = value.to_int()
		elif arg == "--town":
			use_town = true
		elif arg.begins_with("--walk="):
			walk_at = Vector2i(value.get_slice(",", 0).to_int(), value.get_slice(",", 1).to_int())
		elif arg.begins_with("--zoom="):
			zoom = value.to_float()
		elif arg.begins_with("--data="):
			data_dir = value
		elif arg.begins_with("--speed="):
			start_speed = value.to_float()
		elif arg.begins_with("--do="):
			for step in value.split(","):
				scripted.append([step.get_slice(":", 0).to_float(), step.get_slice(":", 1)])
		elif arg.begins_with("--shot="):
			shot_path = value
		elif arg.begins_with("--shot-after="):
			shot_after = value.to_float()
		elif arg == "--news":
			print_news = true
		elif arg.begins_with("--save="):
			save_path = value
		elif arg.begins_with("--hour="):
			start_hour = value.to_int()
		elif arg.begins_with("--class="):
			class_wanted = value
		elif arg == "--choose":
			choose_asked = true
		elif arg.begins_with("--look="):
			look_flags = value
		elif arg.begins_with("--give="):
			gives = value.split(",")
		elif arg.begins_with("--aim="):
			aim_offset = Vector2i(value.get_slice(",", 0).to_int(), value.get_slice(",", 1).to_int())


## --shot: saves what's on the screen once the time has come, then quits.
func _maybe_take_picture() -> void:
	if shot_path.is_empty():
		return
	if played < shot_after:
		return
	var path := shot_path
	shot_path = ""
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(path)
	print("Saved ", path)
	get_tree().quit()


## --do: presses the scripted actions as their time comes (for pictures of a fight or a door).
func _run_script() -> void:
	while not scripted.is_empty() and played >= scripted[0][0]:
		var action: String = scripted.pop_front()[1]
		match action:
			"swing":
				dusk.attack(0, 0)
			# A swing one way (at a door or a window, say): north, east, south or west.
			"swing_n":
				dusk.attack(0, -1)
			"swing_e":
				dusk.attack(1, 0)
			"swing_s":
				dusk.attack(0, 1)
			"swing_w":
				dusk.attack(-1, 0)
			"heavy":
				dusk.heavy_attack(0, 0)
			"shove":
				dusk.shove(0, 0)
			"door":
				dusk.use_door()
			"struggle":
				dusk.struggle()
			"drop":  # the first thing in the pack
				var items: PackedInt32Array = dusk.character_items()
				if items.size() >= 3:
					dusk.drop_item(0, items[0])
			"sleep":
				_sleep_or_wake()
			"drink":
				dusk.drink_water()
			"light":
				_light_or_douse()
			"skills":
				_book(skills)
			"craft":
				_book(craft)
			"keys":  # F1: the keys panel
				_toggle_keys()
			"moodle":  # the worst moodle's detail, as if the mouse rested on it
				hud.hover_first_moodle()
			"card":  # the inventory opened, and a weapon's card in it (else the one in hand), as if hovered
				var shown: String = inventory.card_first()
				if print_news and shown != "":
					print("Card: ", shown)
			"inventory":  # I: the inventory opened, or shut
				inventory.toggle()
				if print_news:
					print("Inventory opened." if inventory.is_open() else "Inventory shut.")
			"arrange":  # the first stack in your pockets moved within them, at once, as a click would
				var moved: String = inventory.arrange_first()
				if print_news and moved != "":
					print(moved)
			"takein":  # the first thing within reach taken onto a cell of your pockets, as a click would
				var taking: String = inventory.take_in_first()
				if print_news and taking != "":
					print(taking)
			"esc":  # what Esc closes first, else the menu (a script never quits by it)
				if not _close_top():
					_open_menu()
					if print_news:
						print("Menu opened.")
			"guard":  # on guard, facing the nearest of the dead you see (else as you are)
				_guard(true)
				if print_news:
					print("On guard." if dusk.is_guarding() else "Not on guard.")
				var who := _nearest_dead()
				if who >= 0:
					var p: Vector3 = actors[who].position
					dusk.face_toward(p.x, p.z)
			"unguard":
				_guard(false)
			"pursue":  # after the nearest of the dead still standing, as a click on it does (seen or not: a script can't turn to look)
				var states: PackedInt32Array = dusk.actor_states()
				var where: PackedVector2Array = dusk.actor_positions()
				var who := -1
				for k in mini(where.size(), states.size()):
					if k != me and k < kinds.size() and kinds[k] != 0 and states[k] != State.DEAD \
							and (who < 0 or where[k].distance_to(where[me]) < where[who].distance_to(where[me])):
						who = k
				if who >= 0:
					dusk.pursue(who, false, running)
				if print_news:
					print("Going after it." if dusk.pursuing() >= 0 else "Nothing to go after.")
			"run":  # Shift tapped: walking or running
				_toggle_run()
				if print_news:
					print("Running." if running else "Walking.")
			"settings":  # F10: the settings screen
				settings.toggle()
			"zoom_in", "zoom_out":  # three notches of the wheel, sent as a mouse sends them
				for n in 3:
					var wheel := InputEventMouseButton.new()
					wheel.button_index = MOUSE_BUTTON_WHEEL_UP if action == "zoom_in" else MOUSE_BUTTON_WHEEL_DOWN
					wheel.position = get_viewport().get_visible_rect().size / 2.0
					wheel.pressed = true
					Input.parse_input_event(wheel)
			"body":  # the worst-wounded part's detail, as if the mouse rested on it
				hud.hover_worst_part()
			"bandage", "decoy", "putup":  # the first bandage in the pack put on (the core picks the wound), decoy set down, or door or wall put up
				var want: int = {"bandage": Item.BANDAGE, "decoy": Item.DECOY}.get(action, Item.PIECE)
				var items: PackedInt32Array = dusk.character_items()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(0, items.size() - 3, 4):
					if items[k] < kinds.size() and kinds[items[k]] == want:
						if want == Item.PIECE:
							bag.put_up(k / 4, items[k])  # as its button does: says why not
						else:
							dusk.use_item(k / 4, items[k])
						break
			"belt1", "belt2", "belt3", "belt4":  # the belt's keys: the potion in that loop drunk
				_drink_loop(action.right(1).to_int() - 1)
			"tobelt":  # the first potion off the belt put on it, as its button does
				var stacks: PackedInt32Array = dusk.character_items()
				var belted: PackedInt32Array = dusk.character_belted()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(0, stacks.size() - 3, 4):
					if kinds[stacks[k]] == Item.POTION and belted[k / 4] == 0:
						dusk.belt_item(k / 4, stacks[k])
						break
			"bar":  # the bar's slots in words, for a script's news
				if print_news:
					for row in hotbar.words():
						print("Bar: ", row)
			"swap":  # the bar's first two powers swapped, as a drag does (a trial run keeps it only while it runs)
				hotbar.swap(0, 1)
			"cook":  # the last cooked recipe that can be made now (a stew before a roast)
				var checks: PackedInt32Array = dusk.craft_checks()
				var texts: PackedStringArray = dusk.recipe_texts()
				for r in range(checks.size() / 2 - 1, -1, -1):
					if checks[r * 2] == 0 and texts[r * 2 + 1].ends_with("at a fire"):
						dusk.craft(r)
						break
			"breakdown":
				dusk.break_down()
			"sneak":
				dusk.set_sneaking(not dusk.is_sneaking())
			"shield":  # take up the first shield in the pack
				var items: PackedInt32Array = dusk.character_items()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(0, items.size() - 3, 4):
					if items[k] < kinds.size() and kinds[items[k]] == Item.SHIELD:
						dusk.wield(k / 4, items[k])
						break
			"wield":  # take the last weapon or bow in the pack in hand (the last one --give gave)
				var items: PackedInt32Array = dusk.character_items()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(items.size() - 4, -1, -4):
					if items[k] < kinds.size() and kinds[items[k]] in [Item.WEAPON, Item.BOW]:
						dusk.wield(k / 4, items[k])
						break
			"wear":  # put on the first armour or bag in the pack
				var items: PackedInt32Array = dusk.character_items()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(0, items.size() - 3, 4):
					if items[k] < kinds.size() and kinds[items[k]] in [Item.ARMOUR, Item.BAG]:
						dusk.wear(k / 4, items[k])
						break
			"dropbag":  # the bag on your back, with what's in it
				dusk.drop_bag()
			"tobag":  # the first stack in your pockets that fits, into the bag
				var items: PackedInt32Array = dusk.character_items()
				var bagged: PackedInt32Array = dusk.character_bagged()
				for k in bagged.size():
					if bagged[k] == 0 and dusk.shift_check(k, items[k * 4]) > 0:
						dusk.shift(k, items[k * 4])
						break
			"take":  # the first thing within reach that there's room for
				var reach: PackedInt32Array = dusk.stores_in_reach()
				var i := 0
				var taken := false
				while i + 5 <= reach.size() and not taken:
					for slot in reach[i + 3]:
						var def := reach[i + 5 + slot * 4]
						if dusk.take_check(reach[i], reach[i + 1], slot, def) > 0:
							dusk.take(reach[i], reach[i + 1], slot, def)
							taken = true
							break
					i += 5 + reach[i + 3] * 4
			"brace":
				_brace(true)
			"spells":
				_book(spells)
				if print_news:
					for row in spells.rows():
						print("Spell: ", row)
			"cast_light", "cast_silence", "cast_fog", "cast_fireball":
				# The spell of that name, on the nearest of the dead you see, else 3 tiles east; or, with
				# --aim, where it says.
				var spell := spell_names.find(action.substr(5).capitalize())
				var where: PackedVector2Array = dusk.actor_positions()
				var spot := Vector2i(roundi(where[me].x) + 3, roundi(where[me].y))
				var best := -1
				for k in where.size():
					if k == me or k >= kinds.size() or kinds[k] == 0 or k >= seen.size() or seen[k] == 0:
						continue
					if best < 0 or where[k].distance_to(where[me]) < where[best].distance_to(where[me]):
						best = k
				if aim_offset != Vector2i.ZERO:
					spot = _my_tile() + aim_offset
				elif best >= 0:
					spot = Vector2i(roundi(where[best].x), roundi(where[best].y))
				if spell >= 0:
					readied = spell
					_cast_on(spot)
			"shoot":  # at the nearest of the dead you see
				var where: PackedVector2Array = dusk.actor_positions()
				var best := -1
				for k in where.size():
					if k == me or k >= kinds.size() or kinds[k] == 0 or k >= seen.size() or seen[k] == 0:
						continue
					if best < 0 or where[k].distance_to(where[me]) < where[best].distance_to(where[me]):
						best = k
				if best >= 0:
					_shoot(Vector2i(roundi(where[best].x), roundi(where[best].y)))
			"hunt":  # shoot at the nearest beast within a bow's reach (seen or not: a script can't turn to look)
				var beasts: PackedInt32Array = dusk.wild_kinds()
				var where: PackedVector2Array = dusk.actor_positions()
				var states: PackedInt32Array = dusk.actor_states()
				var best := -1
				for k in where.size():
					if k == me or k >= kinds.size() or kinds[k] == 0 or where[k].distance_to(where[me]) > 12.0:
						continue
					if kinds[k] - 1 >= beasts.size() or beasts[kinds[k] - 1] != 1 or states[k] == State.DEAD:
						continue
					if best < 0 or where[k].distance_to(where[me]) < where[best].distance_to(where[me]):
						best = k
				if best >= 0:
					_shoot(Vector2i(roundi(where[best].x), roundi(where[best].y)))
			"bow":  # take up the first bow in the pack
				var items: PackedInt32Array = dusk.character_items()
				var kinds: PackedInt32Array = dusk.item_kinds()
				for k in range(0, items.size() - 3, 4):
					if items[k] < kinds.size() and kinds[items[k]] == Item.BOW:
						dusk.wield(k / 4, items[k])
						break
			"save":
				_save("Saved.")
			"board":
				_board_up()
			"burn", "butcher":  # a beast's carcass is cut up, else a body burned
				_burn_body()
			"unboard":
				_pry_board()
			"repair":
				_repair()
			"logs":
				_take_logs()
			"fire":
				_tend_fire()
			"smother":
				_smother_fire()
			"turn":
				_turn_the_dead()
			"pray":
				_pray()
			"consecrate":
				_consecrate()
			"ward":
				_ward()
			"mend":
				_mend()
			"pick":  # the wild plant beside you
				_pick()
			"use":  # E, tapped: the likeliest thing here
				_use_here()
			"here":  # E, held: everything here, listed
				_open_here()
			"power1", "power2", "power3", "power4":  # the number keys
				_power(action.right(1).to_int() - 1)
			"next":  # after a death, the next survivor
				_next_survivor()


## Your own timed action: (its kind, how far through it is out of 1000), or (0, 0).
func _my_action() -> Vector2i:
	if me < 0:
		return Vector2i.ZERO
	var actions: PackedInt32Array = dusk.actor_actions()
	return Vector2i(actions[me * 4], actions[me * 4 + 1])


## What an item of yours is called now, by its kind: the first of yours that's of it.
func _known_as(def: int) -> String:
	var labels: PackedStringArray = dusk.character_labels()
	var hands: PackedInt32Array = dusk.character_hands()
	if hands.size() >= 2 and hands[0] == def:
		return labels[0]
	var light: PackedInt32Array = dusk.character_light()
	if light.size() >= 2 and light[0] == def:
		return labels[1]
	var shield: PackedInt32Array = dusk.character_shield()
	var shield_label: PackedStringArray = dusk.shield_label()
	if shield.size() >= 2 and shield[0] == def and shield_label.size() >= 2:
		return shield_label[0]
	var worn: PackedInt32Array = dusk.character_worn()
	var worn_labels: PackedStringArray = dusk.worn_labels()
	for i in range(0, worn.size(), 4):
		var called := floori(i / 2.0)  # two labels to a piece's four numbers
		if worn[i] == def and called < worn_labels.size():
			return worn_labels[called]
	var items: PackedInt32Array = dusk.character_items()
	for i in range(0, items.size(), 4):
		if items[i] == def:
			return labels[2 + floori(i / 4.0)]
	return _item_name(def)


func _spell_name(spell: int) -> String:
	return spell_names[spell] if spell >= 0 and spell < spell_names.size() else "spell"


## Readies a spell (its number): the next spot clicked is the one it's cast on. Readying the one
## readied lets it go.
func _ready_spell(spell: int) -> void:
	if readied == spell:
		_let_go()
		return
	var info: PackedInt32Array = dusk.spell_info()
	var reach: int = info[spell * Spells.INFO + 3] if spell * Spells.INFO + 3 < info.size() else 0
	readied = spell
	hud.say("%s ready: click a spot within %d tiles. Right-click lets it go." % [_spell_name(spell), reach])


func _let_go() -> void:
	readied = -1
	hud.say("You let the spell go.")


## The fight's effects (effects.gd), from its events, only where you'd see them: on you, by you,
## or on one you see; a door giving way on a tile you see. `x`, `y`: the event's tile.
func _fight_effect(kind: int, who: int, target: int, x: int, y: int, extra: int) -> void:
	match kind:
		Event.STRUCK:
			if target == me or who == me or _seen(target):
				effects.hit(_drawn(target, x, y), _bloodless(target), extra % 1000 > 0)
		Event.BLOCKED:
			if who == me:
				effects.block(_drawn(me, x, y), _drawn(target, x, y))
		Event.GLANCED:
			if who == me or target == me:
				effects.glance(_drawn(who, x, y))
		Event.FELLED:
			if who == me:
				effects.fell(_drawn(target, x, y))
		Event.TERRIFIED:
			if who == me:
				effects.terror(_drawn(target, x, y))
		Event.SEARED:  # a weapon's fire, or its holy harm (extra: 1 fire, 2 holy, 3 both)
			if who == me or _seen(target):
				effects.sear(_drawn(target, x, y), extra)
		Event.THORNED:
			if who == me:
				effects.thorns(_drawn(target, x, y))
		Event.SNEAK_ATTACK:
			if who == me:
				effects.sneak_attack(_drawn(target, x, y))
		Event.RESISTED:
			if who == me:
				effects.bone(_drawn(target, x, y), extra == 1)
		Event.GOT_UP:
			if _seen(who):
				effects.got_up(_drawn(who, x, y))
		Event.RISEN:
			if _seen(target):
				effects.risen(_drawn(target, x, y))
		Event.LIGHT_EATEN:
			if target == me:
				effects.light_eaten(_drawn(me, x, y))
		Event.DOOR_BROKE, Event.BOARD_BROKE, Event.WALL_BROKE:
			# The edge is the tile's north or west one: between it and the tile beyond.
			if map_view.sees(x, y) or map_view.sees(x - 1, y) or map_view.sees(x, y - 1):
				var wood := Look.BOARD if kind == Event.BOARD_BROKE else Look.DOOR
				effects.splinters(Vector3(x + 0.25, 1.0, y + 0.25), wood)


## Where actor `i` is drawn, or tile (x, y)'s middle with no such actor.
func _drawn(i: int, x: int, y: int) -> Vector3:
	return actors[i].position if i >= 0 and i < actors.size() else Vector3(x + 0.5, 0.0, y + 0.5)


## Whether actor `i` has no blood to shed: blows on it chip bone instead (BLOODLESS).
func _bloodless(i: int) -> bool:
	var n: int = kinds[i] - 1 if i >= 0 and i < kinds.size() else -1
	return n >= 0 and n < creature_names.size() and creature_names[n] in BLOODLESS


## The tile you stand on.
func _my_tile() -> Vector2i:
	var where: PackedVector2Array = dusk.actor_positions()
	return Vector2i(roundi(where[me].x), roundi(where[me].y)) if me >= 0 and me < where.size() else Vector2i.ZERO


## A readied spell, aimed at the tile under the mouse (aim.gd): how far you can cast it, and how
## far it would reach there, coloured by the core's checks (DuskWorld.cast_check(): can it be
## cast; spell_catches(): would it burn you too). Gone with none readied.
func _aim() -> void:
	var info: PackedInt32Array = dusk.spell_info()
	var i := readied * Spells.INFO
	if readied < 0 or dead or me < 0 or i + 5 >= info.size():
		aim.hide_aim()
		if aiming:
			aiming = false
			_here_now(_moodle(Moodle.ASLEEP) > 0)  # zone F back to what's here
		return
	var tile := _tile_under(get_viewport().get_mouse_position())
	if aim_offset != Vector2i.ZERO:
		tile = _my_tile() + aim_offset
	# A Light reaches as far as it lights; the rest, their own reach (spell_info: fifth and sixth).
	var reach: int = info[i + 5] if info[i] == Spell.LIGHT else info[i + 4]
	var can: int = dusk.cast_check(readied, tile.x, tile.y)
	var burns: bool = dusk.spell_catches(readied, tile.x, tile.y)
	# Too far or out of sight is a click all the same: you'd go closer first.
	var closer := can in [-5, -6]
	aim.show_aim(actors[me].position, info[i + 3], tile, reach, can == 0 or closer, burns)
	# And in words, under you, so the colours aren't alone (VISUALS.md §4): what a click would do.
	aiming = true
	var line := "[Click] Cast the %s here" % _spell_name(readied)
	if closer:
		line = "[Click] Go closer and cast the %s here" % _spell_name(readied)
	elif can != 0:
		line = "%s: %s" % [_spell_name(readied), AIM_WHY.get(can, "it can't be cast now")]
	elif burns:
		line += ": it would burn you too"
	hud.show_prompts([[line, can == 0 or closer], [Prompts.keyed("[Right-click] let it go"), false]])


## A spell of yours landing on tile (x, y) (Event.CAST): its effect, by its kind and how far it
## reaches (DuskWorld.spell_info(): Spells.INFO numbers a spell, its kind first and its reach fifth).
func _spell_effect(spell: int, x: int, y: int) -> void:
	var info: PackedInt32Array = dusk.spell_info()
	var i := spell * Spells.INFO
	if i + 4 < info.size():
		effects.spell(info[i], x, y, info[i + 4], actors[me].position)


## The readied spell, cast on `tile`: the core says whether it can be, and this says why not.
func _cast_on(tile: Vector2i) -> void:
	var spell := readied
	match dusk.cast_check(spell, tile.x, tile.y):
		0:
			dusk.cast(spell, tile.x, tile.y)
			readied = -1
		-5, -6:
			# Too far, or out of sight: you go until it's in reach and in sight, then cast, as in
			# Diablo (the core keeps the cast and walks you: spell.rs). A key or a click stops it.
			dusk.cast(spell, tile.x, tile.y)
			readied = -1
			hud.say("You go closer to cast the %s." % _spell_name(spell))
		-1:
			hud.say("You don't know that spell.")
			readied = -1
		-2:
			hud.say(Prompts.keyed("Not with it holding you! Struggle {swing} or shove {shove}."))
		-3:
			hud.say("Not mana enough for the %s: it comes back by the second." % _spell_name(spell))
		-4:
			var info: PackedInt32Array = dusk.spell_info()
			var i := spell * Spells.INFO + 7
			var what: int = info[i] if i < info.size() else -1
			hud.say("You've no %s for it." % (_item_name(what) if what >= 0 else "component"))


func _item_name(def: int) -> String:
	return item_names[def] if def >= 0 and def < item_names.size() else "something"


## A skill's name, from an event's 100 × skill + level.
func _skill_name(packed: int) -> String:
	var skill := floori(packed / 100.0)
	return skill_names[skill] if skill >= 0 and skill < skill_names.size() else "A skill"


## "3 apples" or "the axe", from an event's 1000 × item + count.
func _items(packed: int) -> String:
	var count := packed % 1000
	var def := floori(packed / 1000.0)
	return ("the " + _item_name(def)) if count == 1 else "%d × %s" % [count, _item_name(def)]
