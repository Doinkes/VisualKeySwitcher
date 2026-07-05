#include <windows.h>
#include <stdio.h>
#include "config.h"
#include "layout.h"

static char configPath[MAX_PATH];

// Получить путь к config.ini
static void GetConfigPath() {
    if (configPath[0] == '\0') {
        GetModuleFileNameA(NULL, configPath, MAX_PATH);
        char* lastSlash = strrchr(configPath, '\\');
        if (lastSlash) {
            *(lastSlash + 1) = '\0';
        }
        strcat(configPath, "config.ini");
    }
}

// Чтение числа из INI-файла
static int ReadIniInt(const char* section, const char* key, int defaultValue) {
    GetConfigPath();
    return GetPrivateProfileIntA(section, key, defaultValue, configPath);
}

// Запись в INI-файл
static void WriteIniString(const char* section, const char* key, const char* value) {
    GetConfigPath();
    WritePrivateProfileStringA(section, key, value, configPath);
}

// Загрузить настройки
void LoadConfig() {
    int marker = ReadIniInt("Settings", "VisualMarker", 1);
    if (marker != IsVisualMarkerEnabled()) {
        ToggleVisualMarker();
    }
    
    int sound = ReadIniInt("Settings", "Sound", 1);
    if (sound != IsSoundEnabled()) {
        ToggleSound();
    }
    
    int hotkey = ReadIniInt("Settings", "Hotkey", 3);
    if (hotkey != GetHotkeyModifiers()) {
        SetHotkey(hotkey, 0);
    }
}

// Сохранить настройки
void SaveConfig() {
    char value[10];
    
    sprintf(value, "%d", IsVisualMarkerEnabled() ? 1 : 0);
    WriteIniString("Settings", "VisualMarker", value);
    
    sprintf(value, "%d", IsSoundEnabled() ? 1 : 0);
    WriteIniString("Settings", "Sound", value);
    
    sprintf(value, "%d", GetHotkeyModifiers());
    WriteIniString("Settings", "Hotkey", value);
}