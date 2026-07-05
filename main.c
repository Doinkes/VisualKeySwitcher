#include <windows.h>
#include <stdio.h>
#include "tray.h"
#include "layout.h"
#include "overlay.h"

#define WM_TRAYICON (WM_USER + 1)

static HWND hwndMain;
static HHOOK keyboardHook = NULL;
static HINSTANCE hInstGlobal;

static int ctrlPressed = 0;
static int shiftPressed = 0;
static int altPressed = 0;
static int hotkeyTriggered = 0;

void SwitchLayout() {
    Sleep(20);
    
    HKL currentLayout = GetActiveWindowLayout();
    
    int count = GetKeyboardLayoutList(0, NULL);
    HKL* layouts = (HKL*)malloc(count * sizeof(HKL));
    GetKeyboardLayoutList(count, layouts);
    
    HKL newLayout = NULL;
    for (int i = 0; i < count; i++) {
        if (layouts[i] == currentLayout) {
            if (i + 1 < count) {
                newLayout = layouts[i + 1];
            } else {
                newLayout = layouts[0];
            }
            break;
        }
    }
    
    free(layouts);
    
    if (newLayout && newLayout != currentLayout) {
        HWND foreground = GetForegroundWindow();
        if (foreground) {
            DWORD threadId = GetWindowThreadProcessId(foreground, NULL);
            if (threadId) {
                PostMessage(foreground, WM_INPUTLANGCHANGEREQUEST, 0, (LPARAM)newLayout);
                ActivateKeyboardLayout(newLayout, 0);
            }
        }
        
        Sleep(30);
        HKL actualLayout = GetActiveWindowLayout();
        PlayLayoutSound(actualLayout);
        ShowOverlay(hInstGlobal);
    }
}

LRESULT CALLBACK HookProc(int code, WPARAM wParam, LPARAM lParam) {
    if (code >= 0) {
        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 1;
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 1;
            }
            if (k->vkCode == VK_MENU || k->vkCode == 164 || k->vkCode == 165) {
                altPressed = 1;
            }
            
            int currentMods = (ctrlPressed ? 1 : 0) | (shiftPressed ? 2 : 0) | (altPressed ? 4 : 0);
            int hotkeyMods = GetHotkeyModifiers();
            
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
                            SwitchLayout();
                            if (hotkeyMods == (2 | 4)) {
                                PostMessage(hwndMain, WM_USER + 2, 0, 0);
                            }
                            return 1;
                        }
                    }
                }
            }
            
            int isSystemHotkey = 0;
            if (ctrlPressed && shiftPressed && !altPressed) isSystemHotkey = 1;
            if (shiftPressed && altPressed && !ctrlPressed) isSystemHotkey = 1;
            
            if (isSystemHotkey && currentMods != hotkeyMods) {
                return 1;
            }
        }
        
        if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 0;
                hotkeyTriggered = 0;
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 0;
                hotkeyTriggered = 0;
            }
            if (k->vkCode == VK_MENU || k->vkCode == 164 || k->vkCode == 165) {
                altPressed = 0;
                hotkeyTriggered = 0;
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
            
            keyboardHook = SetWindowsHookExA(WH_KEYBOARD_LL, HookProc, hInstGlobal, 0);
            if (!keyboardHook) {
                MessageBoxA(hwnd, "Run as Administrator!", "Error", MB_OK | MB_ICONERROR);
                PostQuitMessage(0);
            }
            break;
        }
        
        case WM_USER + 1:
            HandleTrayMessage(wParam, lParam);
            break;
        
        case WM_USER + 2: {
            Sleep(10);
            keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, 0);
            altPressed = 0;
            hotkeyTriggered = 0;
            break;
        }
        
        case WM_COMMAND:
            HandleTrayCommand(wParam);
            break;
        
        case WM_DESTROY: {
            if (keyboardHook) {
                UnhookWindowsHookEx(keyboardHook);
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