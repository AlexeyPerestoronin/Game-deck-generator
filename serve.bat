@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "CARGO_BIN=%USERPROFILE%\.cargo\bin"
set "PATH=%CARGO_BIN%;%PATH%"

for /d %%D in ("%LOCALAPPDATA%\Microsoft\WinGet\Packages\BrechtSanders.WinLibs*") do (
    if exist "%%D\mingw64\bin\gcc.exe" (
        set "PATH=%%D\mingw64\bin;%PATH%"
    )
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo [fail] cargo not on PATH. Run start.bat first.
    exit /b 1
)
where trunk >nul 2>&1
if errorlevel 1 (
    echo [fail] trunk not on PATH. Run start.bat first.
    exit /b 1
)

echo Serving deck_gen_wasm at http://127.0.0.1:8080
trunk serve
exit /b %ERRORLEVEL%
