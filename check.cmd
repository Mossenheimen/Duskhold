@echo off
rem Duskhold: every check before a commit.
rem   check.cmd         all of it: the Rust, then the game, headless
rem   check.cmd rust    the Rust only (formatting, lints, tests, the data)
rem The Rust: formatting, lints (none allowed), every test (client\codes.gd matching codes.rs
rem among them) and the data. Then the game, with no window, through scripted runs that walk
rem most of its paths (the prompts and E's list, the classes' powers, the panels, the settings, a
rem fight); any script error, or any error from Godot, fails it. The game's log is kept in
rem %TEMP%\duskhold-check.log.
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
rem The prompts, E's list and E tapped, at seed 1's chamomile by the pond.
call :run 4000 "--seed=1 --walk=57,119 --do=12:here,13:esc,14:use"
rem A Cleric praying by E at the abbey's altar, then its powers.
call :run 6000 "--seed=7 --class=cleric --walk=74,114 --do=9:use,11:power2,22:power1"
rem A Wizard's spells by number, then every panel and the settings, opened and closed.
call :run 1500 "--seed=7 --class=wizard --do=1:power1,2:power4,3:esc,4:spells,5:skills,6:craft,7:keys,7.5:keys,8:settings,9:esc"
rem Every setting on, through the near farm and its fight.
call :run 6000 "--seed=7 --hour=12 --speed=4 --walk=118,84 --look=colour_blind,high_contrast,reduce_motion,show_numbers --do=20:here,21:esc"
findstr /c:"SCRIPT ERROR" /c:"ERROR:" /c:"Parse Error" "%LOG%" >nul && goto failed_game
rem And each run got where it was going: no news of these, and it didn't play at all.
for %%m in ("You pick it" "You pray" "Fireball ready" "Pick the bramble") do (
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
