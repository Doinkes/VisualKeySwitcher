#include <windows.h>
#include "tray.h"
#include "layout.h"
#include "overlay.h"

#define WM_TRAYICON (WM_USER + 1)

static HWND hwndMain;
static HHOOK keyboardHook = NULL;
static HINSTANCE hInstGlobal;

LRESULT CALLBACK HookProc(int code, WPARAM wParam, LPARAM lParam) {
    static int ctrlPressed = 0;
    static int shiftPressed = 0;
    static int notificationShown = 0;
    
    if (code >= 0) {
        if (wParam == WM_KEYDOWN) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 1;
            }
            
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 1;
            }
            
            if (ctrlPressed && shiftPressed && !notificationShown) {
                HKL layout = GetActiveWindowLayout();
                PlayLayoutSound(layout);
                ShowOverlay(hInstGlobal);
                notificationShown = 1;
            }
        }
        
        if (wParam == WM_KEYUP) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 0;
                notificationShown = 0;
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 0;
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