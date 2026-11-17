@echo off
chcp 65001 >nul
setlocal
title Building VisualKeySwitcher

rem Build scripts live in scripts\; the project root is one level up.
cd /d "%~dp0.."

echo ========================================
echo  Building VisualKeySwitcher (Rust)
echo ========================================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Rust/cargo not found. Install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo [INFO] cargo build --release
cargo build --release
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed.
    pause
    exit /b 1
)

copy /Y "target\release\VisualKeySwitcher.exe" "VisualKeySwitcher.exe" >nul
if errorlevel 1 (
    echo [ERROR] Could not copy the executable.
    pause
    exit /b 1
)
echo [OK] %CD%\VisualKeySwitcher.exe

rem Keep the local Release folder up to date when it exists.
if exist "Release\." (
    copy /Y "target\release\VisualKeySwitcher.exe" "Release\VisualKeySwitcher.exe" >nul
    if errorlevel 1 (
        echo [WARN] Could not update Release\VisualKeySwitcher.exe - close the app first.
    ) else (
        echo [OK] %CD%\Release\VisualKeySwitcher.exe
    )
)

echo.
echo Tests: cargo test
echo.
pause
