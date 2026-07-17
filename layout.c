#include <stdio.h>
#include "layout.h"
#include <mmsystem.h>

#pragma comment(lib, "winmm.lib")

static int visualMarkerEnabled = 1;
static int soundEnabled = 1;
static int autoStartEnabled = 0;
static int visualType = 1; // 0 = Classic (центр, чЄрный), 1 = Modern (внизу, цветной)
static char exeDir[MAX_PATH];
static char exePath[MAX_PATH];

static int hotkeyModifiers = 1 | 2; // Ctrl+Shift
static int hotkeyKeyCode = 0;

// Sound Pack
static char currentSoundPack[MAX_PATH] = "default";
char soundPacksList[100][MAX_PATH];
int soundPacksCount = 0;

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
    char regValue[MAX_PATH + 20];
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

static void SaveSettingsToRegistry() {
    HKEY hKey;
    if (RegCreateKeyExA(HKEY_CURRENT_USER, 
        "Software\\VisualKeySwitcher", 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS) {
        
        RegSetValueExA(hKey, "VisualMarker", 0, REG_DWORD, (const BYTE*)&visualMarkerEnabled, sizeof(visualMarkerEnabled));
        RegSetValueExA(hKey, "Sound", 0, REG_DWORD, (const BYTE*)&soundEnabled, sizeof(soundEnabled));
        RegSetValueExA(hKey, "Hotkey", 0, REG_DWORD, (const BYTE*)&hotkeyModifiers, sizeof(hotkeyModifiers));
        RegSetValueExA(hKey, "VisualType", 0, REG_DWORD, (const BYTE*)&visualType, sizeof(visualType));
        RegSetValueExA(hKey, "SoundPack", 0, REG_SZ, (const BYTE*)currentSoundPack, strlen(currentSoundPack) + 1);
        
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
        size = sizeof(DWORD);
        if (RegQueryValueExA(hKey, "VisualType", NULL, &dwType, (LPBYTE)&value, &size) == ERROR_SUCCESS) {
            visualType = value;
        }
        
        // Load SoundPack
        DWORD dwTypeSZ = REG_SZ;
        char packName[MAX_PATH];
        DWORD sizeSZ = sizeof(packName);
        if (RegQueryValueExA(hKey, "SoundPack", NULL, &dwTypeSZ, (LPBYTE)packName, &sizeSZ) == ERROR_SUCCESS) {
            strcpy(currentSoundPack, packName);
        }
        
        RegCloseKey(hKey);
    }
}

void LoadSoundPacks() {
    soundPacksCount = 0;
    char soundPackPath[MAX_PATH];
    char searchPath[MAX_PATH];
    GetFullPath("sound_pack", soundPackPath);
    
    sprintf(searchPath, "%s\\*", soundPackPath);
    
    WIN32_FIND_DATAA findData;
    HANDLE hFind = FindFirstFileA(searchPath, &findData);
    
    if (hFind != INVALID_HANDLE_VALUE) {
        do {
            if (findData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
                if (strcmp(findData.cFileName, ".") != 0 && strcmp(findData.cFileName, "..") != 0) {
                    // Check if this directory contains en.wav and ru.wav
                    char enPath[MAX_PATH], ruPath[MAX_PATH];
                    sprintf(enPath, "%s\\%s\\en.wav", soundPackPath, findData.cFileName);
                    sprintf(ruPath, "%s\\%s\\ru.wav", soundPackPath, findData.cFileName);
                    
                    if (GetFileAttributesA(enPath) != INVALID_FILE_ATTRIBUTES &&
                        GetFileAttributesA(ruPath) != INVALID_FILE_ATTRIBUTES) {
                        strcpy(soundPacksList[soundPacksCount], findData.cFileName);
                        soundPacksCount++;
                    }
                }
            }
        } while (FindNextFileA(hFind, &findData) && soundPacksCount < 100);
        FindClose(hFind);
    }
    
    // ≈сли есть звуковые пакеты и текущий пакет не установлен или равен "default"
    if (soundPacksCount > 0) {
        // ѕровер€ем, существует ли текущий пакет в списке
        int packExists = 0;
        for (int i = 0; i < soundPacksCount; i++) {
            if (strcmp(currentSoundPack, soundPacksList[i]) == 0) {
                packExists = 1;
                break;
            }
        }
        
        // ≈сли пакет не существует или это "default", выбираем первый пакет
        if (!packExists || strcmp(currentSoundPack, "default") == 0) {
            strcpy(currentSoundPack, soundPacksList[0]);
            SaveSettingsToRegistry(); // —охран€ем выбранный пакет
        }
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
    LoadSettingsFromRegistry();
    LoadSoundPacks();
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
    char soundPackDir[MAX_PATH];
    
    GetFullPath("sound_pack", soundPackDir);
    
    // Use current sound pack
    sprintf(wavPath, "%s\\%s\\%s.wav", soundPackDir, currentSoundPack, 
            strcmp(lang, "RU") == 0 ? "ru" : 
            strcmp(lang, "EN") == 0 ? "en" : "en");
    
    if (GetFileAttributesA(wavPath) == INVALID_FILE_ATTRIBUTES) {
        // Fallback to default sound pack
        char defaultPath[MAX_PATH];
        sprintf(defaultPath, "%s\\default\\%s.wav", soundPackDir,
                strcmp(lang, "RU") == 0 ? "ru" : 
                strcmp(lang, "EN") == 0 ? "en" : "en");
                
        if (GetFileAttributesA(defaultPath) != INVALID_FILE_ATTRIBUTES) {
            strcpy(wavPath, defaultPath);
        } else {
            // Fallback to root directory (old behavior)
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
        }
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
    SaveSettingsToRegistry();
}

void ToggleSound() {
    soundEnabled = !soundEnabled;
    SaveSettingsToRegistry();
}

void ToggleAutoStart() {
    if (autoStartEnabled) {
        DisableAutoStartInternal();
    } else {
        EnableAutoStartInternal();
    }
    autoStartEnabled = IsAutoStartEnabledInternal();
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

// Visual Type
int GetVisualType() {
    return visualType;
}

void SetVisualType(int type) {
    visualType = type;
    SaveSettingsToRegistry();
}

// Sound Pack
char* GetCurrentSoundPack() {
    return currentSoundPack;
}

void SetSoundPack(const char* packName) {
    strcpy(currentSoundPack, packName);
    SaveSettingsToRegistry();
}