@echo off
echo ==========================================
echo CodeChat Universal - Build Installer Script
echo ==========================================

REM 1. Configurar entorno de Visual Studio (Ajustar ruta si es necesario)
set "VS_DEV_CMD=C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"

if exist "%VS_DEV_CMD%" (
    echo Found VsDevCmd.bat at: %VS_DEV_CMD%
    echo Setting up environment for x64...
    call "%VS_DEV_CMD%" -arch=x64 -host_arch=x64
) else (
    echo ERROR: Could not find VsDevCmd.bat.
    echo Please make sure Visual Studio 2022 Community is installed.
    pause
    exit /b 1
)

REM 2. Verificar variables clave
echo.
echo Checking environment...
where link.exe
if %errorlevel% neq 0 (
    echo ERROR: link.exe not found in PATH.
    pause
    exit /b 1
)

REM 3. Ejecutar Build
echo.
echo Starting Tauri Build...
call npm run tauri build

if %errorlevel% neq 0 (
    echo.
    echo BUILD FAILED!
    echo Check the error messages above.
    pause
    exit /b 1
)

echo.
echo ==========================================
echo BUILD SUCCESSFUL!
echo Installer is located in src-tauri/target/release/bundle/msi
echo ==========================================
pause
