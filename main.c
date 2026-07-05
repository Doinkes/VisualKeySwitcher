#include <windows.h>
#include <stdio.h>
#include "tray.h"
#include "layout.h"
#include "overlay.h"

#define WM_TRAYICON (WM_USER + 1)

static HWND hwndMain;
static HHOOK keyboardHook = NULL;
static HINSTANCE hInstGlobal;

// Состояние модификаторов
static int ctrlPressed = 0;
static int shiftPressed = 0;
static int altPressed = 0;
static int hotkeyTriggered = 0;

void WriteLog(const char* message);

void SwitchLayout() {
    WriteLog("=== SwitchLayout called ===");
    
    Sleep(20);
    
    HKL currentLayout = GetActiveWindowLayout();
    char msg[256];
    sprintf(msg, "Current layout: %d", (int)(long long)currentLayout);
    WriteLog(msg);
    
    int count = GetKeyboardLayoutList(0, NULL);
    WriteLog("Getting keyboard layout list");
    HKL* layouts = (HKL*)malloc(count * sizeof(HKL));
    GetKeyboardLayoutList(count, layouts);
    
    sprintf(msg, "Found %d layouts", count);
    WriteLog(msg);
    
    HKL newLayout = NULL;
    for (int i = 0; i < count; i++) {
        sprintf(msg, "Layout %d: %d", i, (int)(long long)layouts[i]);
        WriteLog(msg);
        if (layouts[i] == currentLayout) {
            if (i + 1 < count) {
                newLayout = layouts[i + 1];
                WriteLog("Next layout found");
            } else {
                newLayout = layouts[0];
                WriteLog("Wrap to first layout");
            }
            break;
        }
    }
    
    free(layouts);
    
    if (newLayout && newLayout != currentLayout) {
        sprintf(msg, "Switching to layout: %d", (int)(long long)newLayout);
        WriteLog(msg);
        
        HWND foreground = GetForegroundWindow();
        if (foreground) {
            DWORD threadId = GetWindowThreadProcessId(foreground, NULL);
            sprintf(msg, "Foreground window thread: %d", threadId);
            WriteLog(msg);
            
            if (threadId) {
                PostMessage(foreground, WM_INPUTLANGCHANGEREQUEST, 0, (LPARAM)newLayout);
                ActivateKeyboardLayout(newLayout, 0);
                WriteLog("Layout switch requested");
            }
        }
        
        Sleep(30);
        
        HKL actualLayout = GetActiveWindowLayout();
        PlayLayoutSound(actualLayout);
        ShowOverlay(hInstGlobal);
        WriteLog("Notification and sound shown");
    } else {
        WriteLog("No layout change needed");
    }
}

LRESULT CALLBACK HookProc(int code, WPARAM wParam, LPARAM lParam) {
    if (code >= 0) {
        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            
            char msg[256];
            sprintf(msg, "Key pressed: vkCode=%d, wParam=%d", k->vkCode, (int)wParam);
            WriteLog(msg);
            
            // Отслеживаем состояние модификаторов
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 1;
                WriteLog("Ctrl pressed");
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 1;
                WriteLog("Shift pressed");
            }
            if (k->vkCode == VK_MENU || k->vkCode == 164 || k->vkCode == 165) {
                altPressed = 1;
                WriteLog("Alt pressed (SYSKEY)");
            }
            
            int currentMods = (ctrlPressed ? 1 : 0) | (shiftPressed ? 2 : 0) | (altPressed ? 4 : 0);
            int hotkeyMods = GetHotkeyModifiers();
            
            sprintf(msg, "Current mods: ctrl=%d, shift=%d, alt=%d, mods=%d, hotkey=%d", 
                    ctrlPressed, shiftPressed, altPressed, currentMods, hotkeyMods);
            WriteLog(msg);
            
            // ============================================================
            // 1. Сначала проверяем НАШ хоткей
            // ============================================================
            if (currentMods == hotkeyMods) {
                int isModifier = (k->vkCode == VK_CONTROL || k->vkCode == VK_SHIFT || k->vkCode == VK_MENU ||
                                  k->vkCode == 162 || k->vkCode == 163 || k->vkCode == 160 || 
                                  k->vkCode == 161 || k->vkCode == 164 || k->vkCode == 165);
                
                if (isModifier) {
                    int allModsPressed = 1;
                    if ((hotkeyMods & 1) && !ctrlPressed) allModsPressed = 0;
                    if ((hotkeyMods & 2) && !shiftPressed) allModsPressed = 0;
                    if ((hotkeyMods & 4) && !altPressed) allModsPressed = 0;
                    
                    if (allModsPressed) {
                        int totalMods = (ctrlPressed ? 1 : 0) + (shiftPressed ? 1 : 0) + (altPressed ? 1 : 0);
                        if (totalMods >= 2 && !hotkeyTriggered) {
                            hotkeyTriggered = 1;
                            WriteLog("HOTKEY DETECTED! Switching layout...");
                            SwitchLayout();
                            WriteLog("Returning 1 to block system");
                            return 1;  // Блокируем, чтобы Windows не переключила
                        }
                    }
                }
            }
            
            // ============================================================
            // 2. Если это системная комбинация (Ctrl+Shift или Shift+Alt) -
            //    блокируем, НО только если она НЕ совпадает с нашим хоткеем
            // ============================================================
            int isSystemHotkey = 0;
            if (ctrlPressed && shiftPressed && !altPressed) isSystemHotkey = 1;  // Ctrl+Shift
            if (shiftPressed && altPressed && !ctrlPressed) isSystemHotkey = 1;   // Shift+Alt
            
            // Если это системная комбинация И она НЕ совпадает с нашим хоткеем
            if (isSystemHotkey && currentMods != hotkeyMods) {
                WriteLog("BLOCKING system hotkey (not our hotkey)");
                return 1;
            }
        }
        
        if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 0;
                hotkeyTriggered = 0;
                WriteLog("Ctrl released");
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 0;
                hotkeyTriggered = 0;
                WriteLog("Shift released");
            }
            if (k->vkCode == VK_MENU || k->vkCode == 164 || k->vkCode == 165) {
                altPressed = 0;
                hotkeyTriggered = 0;
                WriteLog("Alt released");
            }
        }
    }
    
    return CallNextHookEx(keyboardHook, code, wParam, lParam);
}

LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch(msg) {
        case WM_CREATE: {
            hwndMain = hwnd;
            InitLayout(hInstGlobal);
            InitTray(hwnd, hInstGlobal);
            
            WriteLog("Setting keyboard hook...");
            keyboardHook = SetWindowsHookExA(WH_KEYBOARD_LL, HookProc, hInstGlobal, 0);
            if (!keyboardHook) {
                WriteLog("ERROR: Hook failed!");
                MessageBoxA(hwnd, "Run as Administrator!", "Error", MB_OK | MB_ICONERROR);
                PostQuitMessage(0);
            } else {
                WriteLog("Hook installed successfully");
            }
            break;
        }
        
        case WM_USER + 1:
            HandleTrayMessage(wParam, lParam);
            break;
        
        case WM_COMMAND:
            HandleTrayCommand(wParam);
            break;
        
        case WM_DESTROY: {
            WriteLog("Destroying window...");
            if (keyboardHook) {
                UnhookWindowsHookEx(keyboardHook);
                WriteLog("Hook removed");
            }
            DestroyTray();
            PostQuitMessage(0);
            break;
        }
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    hInstGlobal = hInstance;
    
    WNDCLASSA wc = {0};
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = "VisualKeySwitcherClass";
    
    if (!RegisterClassA(&wc)) {
        return 1;
    }
    
    hwndMain = CreateWindowA("VisualKeySwitcherClass", "VisualKeySwitcher",
        WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
        300, 200, NULL, NULL, hInstance, NULL);
    
    if (!hwndMain) {
        return 1;
    }
    
    ShowWindow(hwndMain, SW_HIDE);
    
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    
    return 0;
}