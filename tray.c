#include "tray.h"
#include "layout.h"

#define WM_TRAYICON (WM_USER + 1)

static NOTIFYICONDATA nid;
static HINSTANCE hInstGlobal;
static HWND hwndMain;

void HandleTrayMessage(WPARAM wParam, LPARAM lParam) {
    if (lParam == WM_RBUTTONDOWN) {
        HMENU hMenu = CreatePopupMenu();
        
        AppendMenuA(hMenu, MF_STRING, 1, "Visual Marker");
        AppendMenuA(hMenu, MF_STRING, 2, "Sound");
        AppendMenuA(hMenu, MF_STRING, 3, "AutoStart");
        AppendMenuA(hMenu, MF_SEPARATOR, 0, NULL);
        AppendMenuA(hMenu, MF_STRING, 4, "Exit");
        
        if (IsVisualMarkerEnabled()) {
            CheckMenuItem(hMenu, 1, MF_BYCOMMAND | MF_CHECKED);
        }
        if (IsSoundEnabled()) {
            CheckMenuItem(hMenu, 2, MF_BYCOMMAND | MF_CHECKED);
        }
        if (IsAutoStartEnabled()) {
            CheckMenuItem(hMenu, 3, MF_BYCOMMAND | MF_CHECKED);
        }
        
        POINT pt;
        GetCursorPos(&pt);
        SetForegroundWindow(hwndMain);
        TrackPopupMenu(hMenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwndMain, NULL);
        PostMessage(hwndMain, WM_NULL, 0, 0);
        DestroyMenu(hMenu);
    }
}

void HandleTrayCommand(WPARAM wParam) {
    switch(LOWORD(wParam)) {
        case 1:
            ToggleVisualMarker();
            UpdateTrayMenu(hwndMain);
            break;
        case 2:
            ToggleSound();
            UpdateTrayMenu(hwndMain);
            break;
        case 3:
            ToggleAutoStart();
            UpdateTrayMenu(hwndMain);
            break;
        case 4:
            DestroyTray();
            DestroyWindow(hwndMain);
            PostQuitMessage(0);
            break;
    }
}

void InitTray(HWND hwnd, HINSTANCE hInst) {
    hwndMain = hwnd;
    hInstGlobal = hInst;
    
    nid.cbSize = sizeof(NOTIFYICONDATA);
    nid.hWnd = hwnd;
    nid.uID = 1001;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    
    nid.hIcon = LoadIcon(hInst, MAKEINTRESOURCE(101));
    if (!nid.hIcon) {
        nid.hIcon = (HICON)LoadImageA(NULL, "app.ico", IMAGE_ICON, 16, 16, LR_LOADFROMFILE);
    }
    if (!nid.hIcon) {
        nid.hIcon = LoadIcon(NULL, IDI_INFORMATION);
    }
    
    strcpy(nid.szTip, "VisualKeySwitcher");
    Shell_NotifyIcon(NIM_ADD, &nid);
}

void UpdateTrayMenu(HWND hwnd) {
    Shell_NotifyIcon(NIM_MODIFY, &nid);
}

void DestroyTray() {
    Shell_NotifyIcon(NIM_DELETE, &nid);
}