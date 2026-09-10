@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "CARGO_BIN=%USERPROFILE%\.cargo\bin"
set "PATH=%CARGO_BIN%;%PATH%"
set "TRUNK_VERSION=0.21.14"
set "TRUNK_ZIP_URL=https://github.com/trunk-rs/trunk/releases/download/v%TRUNK_VERSION%/trunk-x86_64-pc-windows-msvc.zip"
set "RUSTUP_INIT_URL=https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"

call :ensure_cargo
if errorlevel 1 goto :fail

call :ensure_wasm_target
call :ensure_mingw_path

call :ensure_trunk
if errorlevel 1 goto :fail

echo.
echo Building deck_gen (native CLI with PDF)...
cargo build --release -p deck_gen --features cli
if errorlevel 1 goto :fail

if exist target\release\deck_gen.exe copy /Y target\release\deck_gen.exe deck_gen.exe >nul

echo.
echo deck_gen commands:
deck_gen.exe --help
echo.
echo Examples:
echo   deck_gen.exe list
echo   deck_gen.exe html --name monopoly.events.positives.lucky_day
echo   deck_gen.exe pdf --name monopoly.events.positives.lucky_day
echo   deck_gen.exe html --name events.positives.lucky_day
echo.
echo Web editor (from this repo root):
echo   trunk serve
echo   then open http://127.0.0.1:8080
goto :success

:fail
call :export_path
exit /b 1

:success
call :export_path
exit /b 0

:export_path
call :persist_user_path
echo.
echo [ok] PATH in this terminal now includes:
echo      %CARGO_BIN%
echo      cargo and trunk should work here without restarting.
echo      Already open terminals / VS Code windows need a restart to see them.
REM Expand PATH while setlocal is still active, then push it to the caller.
endlocal & set "PATH=%PATH%"
goto :eof

:persist_user_path
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$bin = Join-Path $env:USERPROFILE '.cargo\bin';" ^
  "if (-not (Test-Path -LiteralPath $bin)) { exit 0 };" ^
  "$user = [Environment]::GetEnvironmentVariable('Path', 'User');" ^
  "if ($null -eq $user) { $user = '' };" ^
  "$parts = @($user -split ';' | Where-Object { $_ -ne '' });" ^
  "if ($parts -contains $bin) { exit 0 };" ^
  "[Environment]::SetEnvironmentVariable('Path', ($bin + ';' + $user).TrimEnd(';'), 'User');" ^
  "Write-Host '[ok] added cargo bin to the user PATH (new terminals will see it)'"
exit /b 0

:ensure_cargo
call :tool_exists cargo
if not errorlevel 1 (
    echo [ok] cargo
    cargo --version
    exit /b 0
)

echo [..] cargo not found, installing Rust via rustup...
set "RUSTUP_INIT=%TEMP%\rustup-init.exe"
call :download "%RUSTUP_INIT_URL%" "%RUSTUP_INIT%"
if errorlevel 1 (
    echo [fail] Could not download rustup-init.exe
    echo        Install manually from https://rustup.rs
    exit /b 1
)

"%RUSTUP_INIT%" -y --default-host x86_64-pc-windows-gnu --default-toolchain stable
if errorlevel 1 (
    echo [fail] rustup-init failed
    exit /b 1
)

set "PATH=%CARGO_BIN%;%PATH%"
call :tool_exists cargo
if errorlevel 1 (
    echo [fail] cargo still missing after rustup.
    exit /b 1
)
echo [ok] cargo
cargo --version
exit /b 0

:ensure_wasm_target
where rustup >nul 2>&1
if errorlevel 1 exit /b 0
echo [..] ensuring wasm32-unknown-unknown target...
rustup target add wasm32-unknown-unknown
if errorlevel 1 (
    echo [warn] could not add wasm32-unknown-unknown; trunk serve may fail
    exit /b 0
)
echo [ok] wasm32-unknown-unknown
exit /b 0

:ensure_mingw_path
for /d %%D in ("%LOCALAPPDATA%\Microsoft\WinGet\Packages\BrechtSanders.WinLibs*") do (
    if exist "%%D\mingw64\bin\gcc.exe" (
        set "PATH=%%D\mingw64\bin;%PATH%"
        echo [ok] WinLibs gcc
        exit /b 0
    )
)
exit /b 0

:ensure_trunk
call :tool_exists trunk
if not errorlevel 1 (
    echo [ok] trunk
    trunk --version
    exit /b 0
)

echo [..] trunk not found, downloading prebuilt %TRUNK_VERSION%...
if not exist "%CARGO_BIN%" mkdir "%CARGO_BIN%"
set "TRUNK_ZIP=%TEMP%\trunk-%TRUNK_VERSION%.zip"
call :download "%TRUNK_ZIP_URL%" "%TRUNK_ZIP%"
if not errorlevel 1 (
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -LiteralPath '%TRUNK_ZIP%' -DestinationPath '%CARGO_BIN%' -Force"
    if exist "%CARGO_BIN%\trunk.exe" (
        echo [ok] trunk
        trunk --version
        exit /b 0
    )
)

echo [..] prebuilt trunk failed, trying cargo install trunk...
cargo install trunk
set "PATH=%CARGO_BIN%;%PATH%"
call :tool_exists trunk
if errorlevel 1 (
    echo [fail] Could not install trunk.
    echo        Download trunk.exe from:
    echo        %TRUNK_ZIP_URL%
    echo        and put it into %CARGO_BIN%
    exit /b 1
)
echo [ok] trunk
trunk --version
exit /b 0

:tool_exists
where %1 >nul 2>&1
exit /b %ERRORLEVEL%

:download
powershell -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -UseBasicParsing -Uri '%~1' -OutFile '%~2'"
if exist "%~2" exit /b 0
echo [fail] download failed: %~1
exit /b 1
