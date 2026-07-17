#include "tray.h"
#include "layout.h"
#include <stdio.h>

#define WM_TRAYICON (WM_USER + 1)

static NOTIFYICONDATA nid;
static HINSTANCE hInstGlobal;
static HWND hwndMain;

void HandleTrayMessage(WPARAM wParam, LPARAM lParam) {
    if (lParam == WM_RBUTTONDOWN) {
        HMENU hMenu = CreatePopupMenu();
        HMENU hHotkeyMenu = CreatePopupMenu();
        HMENU hVisualTypeMenu = CreatePopupMenu();
        HMENU hSoundPackMenu = CreatePopupMenu();
        
        // Подменю для выбора хоткея
        AppendMenuA(hHotkeyMenu, MF_STRING, 10, "Ctrl + Shift");
        AppendMenuA(hHotkeyMenu, MF_STRING, 11, "Shift + Alt");
        
        int mods = GetHotkeyModifiers();
        if (mods == (1 | 2)) {
            CheckMenuItem(hHotkeyMenu, 10, MF_BYCOMMAND | MF_CHECKED);
        } else if (mods == (2 | 4)) {
            CheckMenuItem(hHotkeyMenu, 11, MF_BYCOMMAND | MF_CHECKED);
        }
        
        // Подменю для выбора типа визуала
        AppendMenuA(hVisualTypeMenu, MF_STRING, 20, "Classic");
        AppendMenuA(hVisualTypeMenu, MF_STRING, 21, "Modern");
        AppendMenuA(hVisualTypeMenu, MF_STRING, 22, "Squares");
        
        int visualType = GetVisualType();
        if (visualType == 0) {
            CheckMenuItem(hVisualTypeMenu, 20, MF_BYCOMMAND | MF_CHECKED);
        } else if (visualType == 1) {
            CheckMenuItem(hVisualTypeMenu, 21, MF_BYCOMMAND | MF_CHECKED);
        } else {
            CheckMenuItem(hVisualTypeMenu, 22, MF_BYCOMMAND | MF_CHECKED);
        }
        
        // Подменю для выбора звукового пакета
        char* currentPack = GetCurrentSoundPack();
        for (int i = 0; i < soundPacksCount; i++) {
            char menuText[MAX_PATH];
            sprintf(menuText, "%s", soundPacksList[i]);
            AppendMenuA(hSoundPackMenu, MF_STRING, 30 + i, menuText);
            if (strcmp(currentPack, soundPacksList[i]) == 0) {
                CheckMenuItem(hSoundPackMenu, 30 + i, MF_BYCOMMAND | MF_CHECKED);
            }
        }
        
        // Основное меню
        AppendMenuA(hMenu, MF_STRING, 1, "Visual Marker");
        AppendMenuA(hMenu, MF_STRING, 2, "Sound");
        AppendMenuA(hMenu, MF_STRING, 3, "AutoStart");
        
        AppendMenuA(hMenu, MF_SEPARATOR, 0, NULL);
        AppendMenuA(hMenu, MF_POPUP, (UINT_PTR)hVisualTypeMenu, "Visual Type");
        AppendMenuA(hMenu, MF_POPUP, (UINT_PTR)hSoundPackMenu, "Sound Pack");
        
        AppendMenuA(hMenu, MF_SEPARATOR, 0, NULL);
        AppendMenuA(hMenu, MF_POPUP, (UINT_PTR)hHotkeyMenu, "Set Hotkey");
        
        AppendMenuA(hMenu, MF_SEPARATOR, 0, NULL);
        AppendMenuA(hMenu, MF_STRING, 5, "Exit");
        
        // Галочки
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
        
        DestroyMenu(hHotkeyMenu);
        DestroyMenu(hVisualTypeMenu);
        DestroyMenu(hSoundPackMenu);
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
        case 10:
            SetHotkey(1 | 2, 0);
            UpdateTrayMenu(hwndMain);
            break;
        case 11:
            SetHotkey(2 | 4, 0);
            UpdateTrayMenu(hwndMain);
            break;
        case 20:
            SetVisualType(0);  // Classic
            UpdateTrayMenu(hwndMain);
            break;
        case 21:
            SetVisualType(1);  // Modern
            UpdateTrayMenu(hwndMain);
            break;
        case 22:
            SetVisualType(2);  // Squares
            UpdateTrayMenu(hwndMain);
            break;
        case 5:
            DestroyTray();
            DestroyWindow(hwndMain);
            PostQuitMessage(0);
            break;
    }
    
    // Handle sound pack selection (30-129)
    if (LOWORD(wParam) >= 30 && LOWORD(wParam) < 30 + soundPacksCount) {
        int index = LOWORD(wParam) - 30;
        SetSoundPack(soundPacksList[index]);
        UpdateTrayMenu(hwndMain);
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