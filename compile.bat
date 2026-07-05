@echo off
chcp 65001 >nul
title Compiling VisualKeySwitcher

echo ========================================
echo  Compiling VisualKeySwitcher
echo ========================================
echo.

if not exist "main.c" (
    echo [ERROR] Source files not found!
    pause
    exit /b 1
)

where gcc >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] GCC compiler not found!
    pause
    exit /b 1
)

echo [INFO] Compiler found:
gcc --version | findstr "gcc"
echo.

if exist "resource.rc" (
    echo [INFO] Compiling resources...
    windres resource.rc -o resource.o
    if %errorlevel% neq 0 (
        echo [ERROR] Resource compilation failed!
        pause
        exit /b 1
    )
    set RESOURCE=resource.o
)

echo [INFO] Compiling program...
gcc -o VisualKeySwitcher.exe main.c tray.c overlay.c layout.c config.c %RESOURCE% -luser32 -lgdi32 -lcomctl32 -mwindows -lwinmm

if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo [SUCCESS] Compilation complete!
    echo File: VisualKeySwitcher.exe
    echo ========================================
    echo.
    if exist "resource.o" (
        del resource.o
    )
) else (
    echo.
    echo ========================================
    echo [ERROR] Compilation failed!
    echo ========================================
    pause
    exit /b 1
)

pause