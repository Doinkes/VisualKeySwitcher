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

void InitLayout(HINSTANCE hInst) {
    GetModuleFileNameA(NULL, exePath, MAX_PATH);
    strcpy(exeDir, exePath);
    char* lastSlash = strrchr(exeDir, '\\');
    if (lastSlash) {
        *lastSlash = '\0';
    }
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
}

void ToggleSound() {
    soundEnabled = !soundEnabled;
}

int GetHotkeyModifiers() {
    return hotkeyModifiers;
}

void SetHotkey(int modifiers, int keyCode) {
    hotkeyModifiers = modifiers;
    hotkeyKeyCode = keyCode;
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