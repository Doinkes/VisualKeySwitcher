#include <stdio.h>
#include "layout.h"
#include <mmsystem.h>

#pragma comment(lib, "winmm.lib")

static int visualMarkerEnabled = 1;
static int soundEnabled = 1;
static int autoStartEnabled = 0;
static char exeDir[MAX_PATH];
static char exePath[MAX_PATH];

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

void InitLayout(HINSTANCE hInst) {
    GetModuleFileNameA(NULL, exePath, MAX_PATH);
    strcpy(exeDir, exePath);
    char* lastSlash = strrchr(exeDir, '\\');
    if (lastSlash) {
        *lastSlash = '\0';
    }
    autoStartEnabled = IsAutoStartEnabledInternal();
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