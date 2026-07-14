#include <stdio.h>
#include "layout.h"
#include <mmsystem.h>

#pragma comment(lib, "winmm.lib")

static int visualMarkerEnabled = 1;
static int soundEnabled = 1;
static char exeDir[MAX_PATH];
static char exePath[MAX_PATH];

static int hotkeyModifiers = 1 | 2; // Ctrl+Shift
static int hotkeyKeyCode = 0;

static void GetFullPath(const char* filename, char* outPath) {
    strcpy(outPath, exeDir);
    strcat(outPath, "\\");
    strcat(outPath, filename);
}

static void SaveSettingsToRegistry() {
    HKEY hKey;
    if (RegCreateKeyExA(HKEY_CURRENT_USER, 
        "Software\\VisualKeySwitcher", 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS) {
        
        RegSetValueExA(hKey, "VisualMarker", 0, REG_DWORD, (const BYTE*)&visualMarkerEnabled, sizeof(visualMarkerEnabled));
        RegSetValueExA(hKey, "Sound", 0, REG_DWORD, (const BYTE*)&soundEnabled, sizeof(soundEnabled));
        RegSetValueExA(hKey, "Hotkey", 0, REG_DWORD, (const BYTE*)&hotkeyModifiers, sizeof(hotkeyModifiers));
        
        RegCloseKey(hKey);
    }
}

static void LoadSettingsFromRegistry() {
    HKEY hKey;
    DWORD dwType = REG_DWORD;
    DWORD size = sizeof(DWORD);
    DWORD value = 0;
    
    if (RegOpenKeyExA(HKEY_CURRENT_USER, 
        "Software\\VisualKeySwitcher", 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        
        if (RegQueryValueExA(hKey, "VisualMarker", NULL, &dwType, (LPBYTE)&value, &size) == ERROR_SUCCESS) {
            visualMarkerEnabled = value;
        }
        size = sizeof(DWORD);
        if (RegQueryValueExA(hKey, "Sound", NULL, &dwType, (LPBYTE)&value, &size) == ERROR_SUCCESS) {
            soundEnabled = value;
        }
        size = sizeof(DWORD);
        if (RegQueryValueExA(hKey, "Hotkey", NULL, &dwType, (LPBYTE)&value, &size) == ERROR_SUCCESS) {
            hotkeyModifiers = value;
        }
        
        RegCloseKey(hKey);
    }
}

void InitLayout(HINSTANCE hInst) {
    GetModuleFileNameA(NULL, exePath, MAX_PATH);
    strcpy(exeDir, exePath);
    char* lastSlash = strrchr(exeDir, '\\');
    if (lastSlash) {
        *lastSlash = '\0';
    }
    LoadSettingsFromRegistry();
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
    return "??";
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

void ToggleVisualMarker() {
    visualMarkerEnabled = !visualMarkerEnabled;
    SaveSettingsToRegistry();
}

void ToggleSound() {
    soundEnabled = !soundEnabled;
    SaveSettingsToRegistry();
}

int GetHotkeyModifiers() {
    return hotkeyModifiers;
}

void SetHotkey(int modifiers, int keyCode) {
    hotkeyModifiers = modifiers;
    hotkeyKeyCode = keyCode;
    SaveSettingsToRegistry();
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