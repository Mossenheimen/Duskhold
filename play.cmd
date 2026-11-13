@echo off
rem Duskhold: builds the game's Rust core as a Godot extension, then starts the game.
rem   play.cmd                the game
rem   play.cmd -e             the Godot editor, on the client project
rem   play.cmd -- --seed=7    anything after "--" goes to the game (see client\main.gd)
setlocal
cd /d "%~dp0"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=%LOCALAPPDATA%\cargo-target"
set "GODOT=%LOCALAPPDATA%\Programs\Godot\Godot_v4.7.2-stable_win64_console.exe"

cargo build -p dusk-godot
if errorlevel 1 exit /b 1

rem Godot finds the extension through client\dusk.gdextension. It's written here, not committed,
rem because it names this PC's build folder (outside OneDrive). Delete it to have it written again.
set "LIB=%CARGO_TARGET_DIR:\=/%"
if not exist client\dusk.gdextension (
  > client\dusk.gdextension (
    echo [configuration]
    echo entry_symbol = "gdext_rust_init"
    echo compatibility_minimum = 4.6
    echo reloadable = true
    echo.
    echo [libraries]
    echo windows.debug.x86_64 = "%LIB%/debug/dusk_godot.dll"
    echo windows.release.x86_64 = "%LIB%/release/dusk_godot.dll"
  )
)

rem The first time, Godot imports the project, which lists the extension in client\.godot. It
rem imports again when there's a font or an icon it hasn't yet (client\fonts, client\icons: a new
rem one, or a fresh clone).
set "IMPORT="
if not exist client\.godot\extension_list.cfg set "IMPORT=1"
for %%f in (client\fonts\*.ttf) do if not exist "client\.godot\imported\%%~nxf-*" set "IMPORT=1"
for /r client\icons %%f in (*.svg) do if not exist "client\.godot\imported\%%~nxf-*" set "IMPORT=1"
if defined IMPORT "%GODOT%" --headless --path client --import

"%GODOT%" --path client %*
