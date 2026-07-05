#include <windows.h>
#include <stdio.h>
#include <time.h>
#include <mmsystem.h>

#pragma comment(lib, "winmm.lib")

// Ресурс иконки
#define IDI_MAIN_ICON 101

// Глобальные переменные
HWND hwndMain = NULL;
HWND hwndOverlay = NULL;
NOTIFYICONDATA nid;
int timerRunning = 0;
HINSTANCE hInst;
int ctrlPressed = 0;
int shiftPressed = 0;
int notificationShown = 0;
int soundEnabled = 1;

// Функция записи в лог
void WriteLog(const char* message) {
    FILE* f = fopen("layout_log.txt", "a");
    if (f) {
        time_t now = time(NULL);
        struct tm* tm_info = localtime(&now);
        char time_str[20];
        strftime(time_str, 20, "%H:%M:%S", tm_info);
        fprintf(f, "[%s] %s\n", time_str, message);
        fclose(f);
    }
}

// Функция для получения названия языка (реальная раскладка)
char* GetLanguageName(HKL hkl) {
    WORD langID = LOWORD(hkl);
    switch(langID) {
        case 0x0419: return "RU";
        case 0x0409: return "EN";
        case 0x041A: return "HR";
        case 0x040C: return "FR";
        case 0x0407: return "DE";
        case 0x0411: return "JP";
        default: return "??";
    }
}

// Функция для получения ИНВЕРТИРОВАННОГО названия языка
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

// Получение раскладки через активное окно
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

// Воспроизведение звука (инвертированное)
void PlayLayoutSound(HKL layout) {
    if (!soundEnabled) return;
    
    char* lang = GetInvertedLanguageName(layout);
    char wavPath[256];
    
    GetCurrentDirectoryA(sizeof(wavPath), wavPath);
    strcat(wavPath, "\\");
    
    if (strcmp(lang, "RU") == 0) {
        strcat(wavPath, "ru.wav");
    } else if (strcmp(lang, "EN") == 0) {
        strcat(wavPath, "en.wav");
    } else {
        WriteLog("Unknown layout, no sound");
        return;
    }
    
    WriteLog("Playing sound: ");
    WriteLog(wavPath);
    
    if (GetFileAttributesA(wavPath) == INVALID_FILE_ATTRIBUTES) {
        char log_msg[200];
        sprintf(log_msg, "Sound file not found: %s", wavPath);
        WriteLog(log_msg);
        return;
    }
    
    PlaySoundA(wavPath, NULL, SND_ASYNC | SND_FILENAME);
}

// Оконная процедура для оверлея
LRESULT CALLBACK OverlayWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch(msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            
            RECT rect;
            GetClientRect(hwnd, &rect);
            
            HBRUSH blackBrush = CreateSolidBrush(RGB(0, 0, 0));
            FillRect(hdc, &rect, blackBrush);
            DeleteObject(blackBrush);
            
            HPEN whitePen = CreatePen(PS_SOLID, 3, RGB(255, 255, 255));
            SelectObject(hdc, whitePen);
            SelectObject(hdc, GetStockObject(NULL_BRUSH));
            Rectangle(hdc, 0, 0, rect.right, rect.bottom);
            DeleteObject(whitePen);
            
            HKL layout = GetActiveWindowLayout();
            char* lang = GetInvertedLanguageName(layout);
            char displayText[10];
            sprintf(displayText, "%s", lang);
            
            HFONT font = CreateFont(120, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, "Arial");
            SelectObject(hdc, font);
            SetTextColor(hdc, RGB(255, 255, 255));
            SetBkMode(hdc, TRANSPARENT);
            
            RECT textRect;
            CopyRect(&textRect, &rect);
            DrawTextA(hdc, displayText, -1, &textRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            
            DeleteObject(font);
            EndPaint(hwnd, &ps);
            return 0;
        }
        
        case WM_TIMER: {
            if (wParam == 1) {
                KillTimer(hwnd, 1);
                timerRunning = 0;
                ShowWindow(hwnd, SW_HIDE);
                WriteLog("Notification hidden");
            }
            return 0;
        }
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

// Показать уведомление
void ShowNotification(HKL layout) {
    if (notificationShown) return;
    
    WriteLog("Showing notification");
    notificationShown = 1;
    
    if (!hwndOverlay) {
        WNDCLASSA wcOverlay = {0};
        wcOverlay.lpfnWndProc = OverlayWndProc;
        wcOverlay.hInstance = hInst;
        wcOverlay.hbrBackground = (HBRUSH)GetStockObject(BLACK_BRUSH);
        wcOverlay.lpszClassName = "OverlayClass";
        wcOverlay.style = CS_HREDRAW | CS_VREDRAW;
        
        if (!RegisterClassA(&wcOverlay)) {
            WriteLog("ERROR: Failed to register overlay class");
            return;
        }
        
        int screenWidth = GetSystemMetrics(SM_CXSCREEN);
        int screenHeight = GetSystemMetrics(SM_CYSCREEN);
        int width = 200;
        int height = 200;
        int x = (screenWidth - width) / 2;
        int y = (screenHeight - height) / 2;
        
        hwndOverlay = CreateWindowA("OverlayClass", "",
            WS_POPUP | WS_VISIBLE,
            x, y, width, height,
            NULL, NULL, hInst, NULL);
        
        if (!hwndOverlay) {
            WriteLog("ERROR: Failed to create overlay window");
            return;
        }
        
        WriteLog("Overlay window created");
        SetWindowPos(hwndOverlay, HWND_TOPMOST, x, y, width, height, SWP_SHOWWINDOW);
        SetWindowLong(hwndOverlay, GWL_EXSTYLE,
            GetWindowLong(hwndOverlay, GWL_EXSTYLE) | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE);
        SetLayeredWindowAttributes(hwndOverlay, 0, 255, LWA_ALPHA);
    }
    
    ShowWindow(hwndOverlay, SW_SHOW);
    SetWindowPos(hwndOverlay, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    InvalidateRect(hwndOverlay, NULL, TRUE);
    UpdateWindow(hwndOverlay);
    
    if (timerRunning) {
        KillTimer(hwndOverlay, 1);
        timerRunning = 0;
    }
    SetTimer(hwndOverlay, 1, 500, NULL);
    timerRunning = 1;
    WriteLog("Timer started (500ms)");
}

// Хук клавиатуры
LRESULT CALLBACK HookProc(int code, WPARAM wParam, LPARAM lParam) {
    if (code >= 0) {
        if (wParam == WM_KEYDOWN) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 1;
            }
            
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 1;
            }
            
            if (ctrlPressed && shiftPressed) {
                WriteLog("Ctrl+Shift detected!");
                
                HKL layout = GetActiveWindowLayout();
                char* lang = GetInvertedLanguageName(layout);
                WriteLog("Layout: ");
                WriteLog(lang);
                
                PlayLayoutSound(layout);
                ShowNotification(layout);
            }
        }
        
        if (wParam == WM_KEYUP) {
            KBDLLHOOKSTRUCT *k = (KBDLLHOOKSTRUCT*)lParam;
            if (k->vkCode == VK_CONTROL || k->vkCode == 162 || k->vkCode == 163) {
                ctrlPressed = 0;
                notificationShown = 0;
                WriteLog("Ctrl released");
            }
            if (k->vkCode == VK_SHIFT || k->vkCode == 160 || k->vkCode == 161) {
                shiftPressed = 0;
                WriteLog("Shift released");
            }
        }
    }
    
    return CallNextHookEx(NULL, code, wParam, lParam);
}

// Обработчик главного окна
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch(msg) {
        case WM_CREATE: {
            WriteLog("Window created");
            
            nid.cbSize = sizeof(NOTIFYICONDATA);
            nid.hWnd = hwnd;
            nid.uID = 1001;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_USER + 1;
            
            // Загружаем иконку из ресурсов
            nid.hIcon = LoadIcon(hInst, MAKEINTRESOURCE(IDI_MAIN_ICON));
            
            // Если не загрузилась — пробуем через LoadImage
            if (!nid.hIcon) {
                WriteLog("LoadIcon failed, trying LoadImage...");
                nid.hIcon = (HICON)LoadImageA(hInst, MAKEINTRESOURCEA(IDI_MAIN_ICON), 
                                              IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);
            }
            
            // Если всё равно не загрузилась — пробуем загрузить из файла
            if (!nid.hIcon) {
                WriteLog("LoadImage from resource failed, trying from file...");
                nid.hIcon = (HICON)LoadImageA(NULL, "app.ico", IMAGE_ICON, 16, 16, LR_LOADFROMFILE);
            }
            
            // Если всё равно не загрузилась — стандартная
            if (!nid.hIcon) {
                WriteLog("ERROR: Failed to load icon, using default");
                nid.hIcon = LoadIcon(NULL, IDI_INFORMATION);
            } else {
                WriteLog("Icon loaded successfully");
            }
            
            strcpy(nid.szTip, "VisualKeySwitcher");
            
            if (!Shell_NotifyIcon(NIM_ADD, &nid)) {
                WriteLog("ERROR: Shell_NotifyIcon failed!");
            } else {
                WriteLog("Tray icon added");
            }
            
            HHOOK hook = SetWindowsHookExA(WH_KEYBOARD_LL, HookProc, hInst, 0);
            if (hook) {
                WriteLog("Keyboard hook installed successfully");
            } else {
                WriteLog("ERROR: Keyboard hook failed! Run as Administrator");
            }
            break;
        }
        
        case WM_USER + 1: {
            if (lParam == WM_RBUTTONDOWN) {
                WriteLog("Right click on tray icon");
                HMENU hMenu = CreatePopupMenu();
                
                // Меню: Sound сверху, Exit снизу
                AppendMenuA(hMenu, MF_STRING, 2, "Sound");
                if (soundEnabled) {
                    CheckMenuItem(hMenu, 2, MF_BYCOMMAND | MF_CHECKED);
                }
                AppendMenuA(hMenu, MF_SEPARATOR, 0, NULL);
                AppendMenuA(hMenu, MF_STRING, 1, "Exit");
                
                POINT pt;
                GetCursorPos(&pt);
                SetForegroundWindow(hwnd);
                TrackPopupMenu(hMenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, NULL);
                PostMessage(hwnd, WM_NULL, 0, 0);
                DestroyMenu(hMenu);
            }
            break;
        }
        
        case WM_COMMAND: {
            if (LOWORD(wParam) == 1) {
                WriteLog("Exit selected");
                Shell_NotifyIcon(NIM_DELETE, &nid);
                if (hwndOverlay) DestroyWindow(hwndOverlay);
                PostQuitMessage(0);
            }
            if (LOWORD(wParam) == 2) {
                soundEnabled = !soundEnabled;
                WriteLog(soundEnabled ? "Sound enabled" : "Sound disabled");
            }
            break;
        }
        
        case WM_DESTROY: {
            WriteLog("Window destroyed");
            Shell_NotifyIcon(NIM_DELETE, &nid);
            PostQuitMessage(0);
            break;
        }
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    hInst = hInstance;
    
    FILE* f = fopen("layout_log.txt", "w");
    if (f) {
        time_t now = time(NULL);
        struct tm* tm_info = localtime(&now);
        char time_str[20];
        strftime(time_str, 20, "%Y-%m-%d %H:%M:%S", tm_info);
        fprintf(f, "=== VisualKeySwitcher started at %s ===\n", time_str);
        fprintf(f, "Press Ctrl+Shift to see INVERTED layout\n\n");
        fclose(f);
    }
    
    WriteLog("Program starting...");
    
    WNDCLASSA wc = {0};
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = "VisualKeySwitcherClass";
    
    if (!RegisterClassA(&wc)) {
        WriteLog("ERROR: Failed to register window class");
        return 1;
    }
    WriteLog("Window class registered");
    
    hwndMain = CreateWindowA("VisualKeySwitcherClass", "VisualKeySwitcher",
        WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
        300, 200, NULL, NULL, hInstance, NULL);
    
    if (!hwndMain) {
        WriteLog("ERROR: Failed to create window");
        return 1;
    }
    WriteLog("Window created");
    
    ShowWindow(hwndMain, SW_HIDE);
    WriteLog("Window hidden");
    WriteLog("Program ready! Press Ctrl+Shift");
    
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    
    WriteLog("Program exiting...");
    return 0;
}