@echo off
chcp 65001 >nul
title Компиляция VisualKeySwitcher

echo ========================================
echo  Компиляция программы VisualKeySwitcher
echo ========================================
echo.

if not exist "main.c" (
    echo [ОШИБКА] Файл main.c не найден!
    pause
    exit /b 1
)

where gcc >nul 2>nul
if %errorlevel% neq 0 (
    echo [ОШИБКА] Компилятор gcc не найден!
    pause
    exit /b 1
)

echo [ИНФО] Компилятор найден:
gcc --version | findstr "gcc"
echo.

if exist "resource.rc" (
    echo [ИНФО] Компилирую ресурсы...
    windres resource.rc -o resource.o
    if %errorlevel% neq 0 (
        echo [ОШИБКА] Ошибка компиляции ресурсов!
        pause
        exit /b 1
    )
    set RESOURCE=resource.o
) else (
    echo [ИНФО] Ресурсы не найдены, компилирую без иконки
    set RESOURCE=
)

echo [ИНФО] Компилирую программу...
gcc -o VisualKeySwitcher.exe main.c %RESOURCE% -luser32 -lgdi32 -lcomctl32 -mwindows -lwinmm

if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo [УСПЕХ] Компиляция завершена!
    echo Файл: VisualKeySwitcher.exe
    echo ========================================
    echo.
) else (
    echo.
    echo ========================================
    echo [ОШИБКА] Компиляция не удалась!
    echo ========================================
    pause
    exit /b 1
)

pause