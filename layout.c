#include <stdio.h>
#include <time.h>
#include "layout.h"
#include <mmsystem.h>

#pragma comment(lib, "winmm.lib")

static int visualMarkerEnabled = 1;
static int soundEnabled = 1;
static int autoStartEnabled = 0;
static char exeDir[MAX_PATH];
static char exePath[MAX_PATH];

// Настройки хоткея (по умолчанию Ctrl+Shift)
static int hotkeyModifiers = 1 | 2; // Ctrl+Shift
static int hotkeyKeyCode = 0;

// Функция записи в лог
void WriteLog(const char* message) {
    char logPath[MAX_PATH];
    GetCurrentDirectoryA(MAX_PATH, logPath);
    strcat(logPath, "\\debug_log.txt");
    
    FILE* f = fopen(logPath, "a");
    if (f) {
        time_t now = time(NULL);
        struct tm* tm_info = localtime(&now);
        char time_str[20];
        strftime(time_str, 20, "%H:%M:%S", tm_info);
        fprintf(f, "[%s] %s\n", time_str, message);
        fclose(f);
    }
}

static void GetFullPath(const char* filename, char* outPath) {
    strcpy(outPath, exeDir);
    strcat(outPath, "\\");
    strcat(outPath, filename);
}

static int IsAutoStartEnabledInternal() {
    HKEY hKey;
    DWORD dwType = REG_SZ;
    char value[1024];
    DWORD size = sizeof(value);
    
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run", 
        0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        
        if (RegQueryValueExA(hKey, "VisualKeySwitcher", NULL, &dwType, (LPBYTE)value, &size) == ERROR_SUCCESS) {
            RegCloseKey(hKey);
            return 1;
        }
        RegCloseKey(hKey);
    }
    return 0;
}

static void EnableAutoStartInternal() {
    char regValue[MAX_PATH + 3];
    sprintf(regValue, "\"%s\"", exePath);
    
    HKEY hKey;
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run", 
        0, KEY_SET_VALUE, &hKey) == ERROR_SUCCESS) {
        
        RegSetValueExA(hKey, "VisualKeySwitcher", 0, REG_SZ, (const BYTE*)regValue, strlen(regValue) + 1);
        RegCloseKey(hKey);
        autoStartEnabled = 1;
    }
}

static void DisableAutoStartInternal() {
    HKEY hKey;
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run", 
        0, KEY_SET_VALUE, &hKey) == ERROR_SUCCESS) {
        
        RegDeleteValueA(hKey, "VisualKeySwitcher");
        RegCloseKey(hKey);
        autoStartEnabled = 0;
    }
}

// === Хоткей: сохранение и загрузка ===

static void SaveHotkeyInternal() {
    HKEY hKey;
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\VisualKeySwitcher", 0, KEY_SET_VALUE, &hKey) == ERROR_SUCCESS) {
        RegSetValueExA(hKey, "HotkeyModifiers", 0, REG_DWORD, (const BYTE*)&hotkeyModifiers, sizeof(hotkeyModifiers));
        RegSetValueExA(hKey, "HotkeyKeyCode", 0, REG_DWORD, (const BYTE*)&hotkeyKeyCode, sizeof(hotkeyKeyCode));
        RegCloseKey(hKey);
    }
}

static void LoadHotkeyInternal() {
    HKEY hKey;
    DWORD dwType = REG_DWORD;
    DWORD size = sizeof(DWORD);
    DWORD value = 0;
    
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\VisualKeySwitcher", 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        
        if (RegQueryValueExA(hKey, "HotkeyModifiers", NULL, &dwType, (LPBYTE)&value, &size) == ERROR_SUCCESS) {
            if (value != 0) hotkeyModifiers = value;
        }
        RegCloseKey(hKey);
    }
}

// === Публичные функции ===

void InitLayout(HINSTANCE hInst) {
    GetModuleFileNameA(NULL, exePath, MAX_PATH);
    strcpy(exeDir, exePath);
    char* lastSlash = strrchr(exeDir, '\\');
    if (lastSlash) {
        *lastSlash = '\0';
    }
    autoStartEnabled = IsAutoStartEnabledInternal();
    LoadHotkeyInternal();
    
    WriteLog("=== Program initialized ===");
    char msg[256];
    sprintf(msg, "Hotkey modifiers: %d", hotkeyModifiers);
    WriteLog(msg);
    sprintf(msg, "EXE path: %s", exePath);
    WriteLog(msg);
    sprintf(msg, "EXE dir: %s", exeDir);
    WriteLog(msg);
}

HKL GetActiveWindowLayout() {
    HWND hwnd = GetForegroundWindow();
    if (hwnd) {
        DWORD threadId = GetWindowThreadProcessId(hwnd, NULL);
        if (threadId) {
            return GetKeyboardLayout(threadId);
        }
    }
    return GetKeyboardLayout(0);
}

char* GetInvertedLanguageName(HKL hkl) {
    WORD langID = LOWORD(hkl);
    if (langID == 0x0419) return "EN";
    if (langID == 0x0409) return "RU";
    switch(langID) {
        case 0x041A: return "HR";
        case 0x040C: return "FR";
        case 0x0407: return "DE";
        case 0x0411: return "JP";
        default: return "??";
    }
}

void PlayLayoutSound(HKL layout) {
    if (!soundEnabled) return;
    
    char* lang = GetInvertedLanguageName(layout);
    char wavPath[MAX_PATH];
    
    if (strcmp(lang, "RU") == 0) {
        GetFullPath("ru.wav", wavPath);
    } else if (strcmp(lang, "EN") == 0) {
        GetFullPath("en.wav", wavPath);
    } else {
        return;
    }
    
    if (GetFileAttributesA(wavPath) == INVALID_FILE_ATTRIBUTES) {
        return;
    }
    
    PlaySoundA(wavPath, NULL, SND_ASYNC | SND_FILENAME);
}

int IsVisualMarkerEnabled() {
    return visualMarkerEnabled;
}

int IsSoundEnabled() {
    return soundEnabled;
}

int IsAutoStartEnabled() {
    return autoStartEnabled;
}

void ToggleVisualMarker() {
    visualMarkerEnabled = !visualMarkerEnabled;
}

void ToggleSound() {
    soundEnabled = !soundEnabled;
}

void ToggleAutoStart() {
    if (autoStartEnabled) {
        DisableAutoStartInternal();
    } else {
        EnableAutoStartInternal();
    }
    autoStartEnabled = IsAutoStartEnabledInternal();
}

// === Функции для хоткея ===

int GetHotkeyModifiers() {
    return hotkeyModifiers;
}

void SetHotkey(int modifiers, int keyCode) {
    hotkeyModifiers = modifiers;
    hotkeyKeyCode = keyCode;
    SaveHotkeyInternal();
    char msg[256];
    sprintf(msg, "Hotkey set to: %d", modifiers);
    WriteLog(msg);
}

int IsHotkeyPressed(int vkCode) {
    int ctrl = (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0;
    int shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
    int alt = (GetAsyncKeyState(VK_MENU) & 0x8000) != 0;
    
    int currentMods = (ctrl ? 1 : 0) | (shift ? 2 : 0) | (alt ? 4 : 0);
    
    if (vkCode == VK_CONTROL || vkCode == VK_SHIFT || vkCode == VK_MENU) {
        return 0;
    }
    
    return currentMods == hotkeyModifiers;
}