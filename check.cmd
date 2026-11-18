@echo off
rem Duskhold: every check before a commit.
rem   check.cmd         all of it: the Rust, then the game, headless
rem   check.cmd rust    the Rust only (formatting, lints, tests, the data)
rem The Rust: formatting, lints (none allowed), every test (client\codes.gd matching codes.rs
rem among them) and the data. Then the game, with no window, through scripted runs that walk
rem most of its paths (the prompts and E's list, the classes' powers, the panels, the settings, a
rem fight, the inventory); any script error, or any error from Godot, fails it. The game's log is
rem kept in %TEMP%\duskhold-check.log.
setlocal
cd /d "%~dp0"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=%LOCALAPPDATA%\cargo-target"

echo == formatting
cargo fmt --all -- --check || goto failed
echo == lints
cargo clippy --workspace --all-features --all-targets -q -- -D warnings || goto failed
echo == tests
cargo test --workspace --all-features -q || goto failed
echo == the data
cargo run -q -p dusk-cli -- check || goto failed
if /i "%~1"=="rust" goto passed

echo == the game, headless
set "LOG=%TEMP%\duskhold-check.log"
if exist "%LOG%" del "%LOG%"
rem The prompts, E's list and E tapped, at seed 1's chamomile by the pond; then a magic bow's
rem card, a magic buckler taken up on the arm, and a clatterbox wound and set down (a decoy).
call :run 4000 "--seed=1 --walk=57,119 --give=hunting_bow:masterwork:farflung+kindled,buckler:fine:steadfast+spiked,clatterbox --do=12:here,13:esc,14:use,20:card,21:shield,23:decoy"
rem By the same pond, a wall of planks put up on the side faced, with a hammer (build.rs).
call :run 4500 "--seed=1 --walk=57,119 --give=hammer,plank_wall --do=12:putup"
rem A Cleric praying by E at the abbey's altar, then its powers; then a unique shield's card,
rem and the shield taken up on the arm.
call :run 6000 "--seed=7 --class=cleric --walk=74,114 --give=the_wasp --do=9:use,11:power2,22:power1,23:card,24:shield"
rem A Wizard's spells by number, every panel and the settings, opened and closed; then two cast.
rem First a weapon's card: a unique given (Redhand, a hand axe: its class's edge, the bleeding, on
rem its card too), and a masterwork Rare sword, named, set against the staff;
rem then the sword taken in hand, and a magic helm put on, so the skills panel shows what they give,
rem and the spells panel the sword's +3 to Arcana: a fireball's harm and Silence's time, as yours;
rem then Light, a fireball, a firebolt and a fire nova cast (no reagent wanted).
call :run 2900 "--seed=7 --class=wizard --give=redhand,sword:masterwork:keen+swift+sorcerous,iron_helm:fine:warded+brawny --do=0.5:card,1:power1,2:power4,3:esc,3.2:wield,4.2:wear,4.5:spells,5:skills,6:craft,7:keys,7.5:keys,8:settings,9:esc,9.5:cast_light,10:cast_fireball,12:cast_firebolt,13.5:cast_fire_nova"
rem The bar and the belt (hotbar.gd, potion.rs): a Wizard's spells on the number keys, its kit's
rem potions on the belt; one from the kit drunk, and one given put on the belt (into the loop it
rem left empty) and drunk; then the bar's first two powers swapped, the first key readying what's
rem on it now, and the kit's potion of mana drunk from the second loop; on guard and a swing; and
rem after the nearest of the dead, as a click on it goes (Intent::Pursue).
call :run 3500 "--seed=7 --class=wizard --give=haste_potion --do=1:bar,1.5:belt1,3:tobelt,3.5:belt1,5:swap,5.5:bar,6:power1,6.5:belt2,7.5:esc,8:esc,8.5:esc,8.8:run,9.2:guard,9.6:swing,10:unguard,10.5:pursue"
rem Every setting on, through the near farm and its fight.
call :run 6000 "--seed=7 --hour=12 --speed=4 --walk=118,84 --look=colour_blind,high_contrast,reduce_motion,show_numbers --do=20:here,21:esc"
rem The inventory as Path of Exile 2's (inventory.gd): in the near farm's kitchen, a bag put on, the
rem inventory opened, the first thing in the stove or the cupboard beside you taken onto a cell of
rem the pockets, a stack moved within them at once, a card shown, and Esc letting it all go.
call :run 4000 "--seed=7 --hour=12 --speed=4 --walk=118,84 --give=travellers_pack --do=11:wear,12.5:inventory,13:takein,14:arrange,14.5:card,15:esc,15.5:esc"
rem A sword dropped at your feet in the abbey, and put straight back in hand from the ground through the
rem inventory (the third notes: "Allow equipping directly from looting").
call :run 1500 "--seed=7 --give=sword --do=1:dropweapon,2:inventory,3:equipin,5:esc"
rem Every effect made once and left to play out: what the runs above can't reach (a block, a
rem felling, a skeleton's bone). It must leave nothing behind (client\tests\check_effects.gd).
echo   every effect, once
"%LOCALAPPDATA%\Programs\Godot\Godot_v4.7.2-stable_win64_console.exe" --headless --path client --quit-after 5000 -s res://tests/check_effects.gd >> "%LOG%" 2>&1
findstr /c:"SCRIPT ERROR" /c:"ERROR:" /c:"Parse Error" "%LOG%" >nul && goto failed_game
rem And each run got where it was going: no news of these, and it didn't play at all.
for %%m in ("You took the arming sword in hand" "You cast Firebolt" "You cast Fire Nova" "Going after it." "You pick it" "You pray" "Fireball ready" "You cast Light" "Pick the bramble" "Against your quarterstaff" "Card: Redhand" "You put on the iron helm" "Card: Far-flung Kindled masterwork hunting bow" "You took up the wooden buckler" "Card: The Wasp" "28.7 to 40.2 damage" "lasts 39 s" "A blow bleeds 4 damage over 8 s" "You wind the decoy" "You put up the wall of planks" "You drink the potion of life: every wound closed" "You drink the potion of haste" "You drink the potion of mana: 25 mana back over 4 s" "Firebolt ready" "Menu opened" "Walking." "You put on the traveller's pack" "Inventory opened." "Taking the" "Arranged the" "On guard." "EFFECTS OK") do (
  findstr /c:%%m "%LOG%" >nul || (echo A run never got to %%m. & goto failed_game)
)
goto passed

rem :run FRAMES "FLAGS": the game for FRAMES frames with no window, with FLAGS after "--" (a
rem trial run: it never touches the player's saved world). The flags are one quoted argument:
rem cmd would split them at every = and , otherwise. Its output goes to the log.
:run
echo   %~2
call "%~dp0play.cmd" --headless --quit-after %~1 -- --news %~2 >> "%LOG%" 2>&1
exit /b 0

:failed_game
echo.
echo The game failed its runs (the log: %LOG%):
findstr /c:"SCRIPT ERROR" /c:"ERROR:" /c:"Parse Error" "%LOG%"
:failed
echo.
echo CHECK FAILED
exit /b 1

:passed
echo.
echo All checks passed.
exit /b 0
