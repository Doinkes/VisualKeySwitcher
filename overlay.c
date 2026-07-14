#include <stdio.h>
#include "overlay.h"
#include "layout.h"

static HWND hwndOverlay = NULL;
static int timerRunning = 0;

LRESULT CALLBACK OverlayWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch(msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            
            RECT rect;
            GetClientRect(hwnd, &rect);
            
            int visualType = GetVisualType();
            HKL layout = GetActiveWindowLayout();
            WORD langID = LOWORD(layout);
            
            char displayText[10];
            if (langID == 0x0419) {
                sprintf(displayText, "EN");
            } else if (langID == 0x0409) {
                sprintf(displayText, "RU");
            } else {
                sprintf(displayText, "??");
            }
            
            // ============================================================
            // СТИЛЬ 0: CLASSIC (центр, чёрный фон, БЕЗ рамки, текст)
            // ============================================================
            if (visualType == 0) {
                COLORREF bgColor = RGB(0, 0, 0);
                COLORREF textColor = RGB(255, 255, 255);
                int fontSize = 120;
                
                HBRUSH bgBrush = CreateSolidBrush(bgColor);
                FillRect(hdc, &rect, bgBrush);
                DeleteObject(bgBrush);
                
                HFONT font = CreateFont(fontSize, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                    DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, "Arial");
                SelectObject(hdc, font);
                SetTextColor(hdc, textColor);
                SetBkMode(hdc, TRANSPARENT);
                
                RECT textRect;
                CopyRect(&textRect, &rect);
                DrawTextA(hdc, displayText, -1, &textRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                DeleteObject(font);
            }
            
            // ============================================================
            // СТИЛЬ 1: MODERN (внизу, цветной фон, БЕЗ рамки, текст)
            // ============================================================
            else if (visualType == 1) {
                COLORREF ruColor = RGB(255, 107, 53);
                COLORREF enColor = RGB(74, 144, 217);
                COLORREF textColor = RGB(255, 255, 255);
                int fontSize = 120;
                
                COLORREF bgColor = (langID == 0x0419) ? ruColor : 
                                   (langID == 0x0409) ? enColor : 
                                   RGB(255, 255, 255);
                
                HBRUSH bgBrush = CreateSolidBrush(bgColor);
                FillRect(hdc, &rect, bgBrush);
                DeleteObject(bgBrush);
                
                HFONT font = CreateFont(fontSize, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                    DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, "Arial");
                SelectObject(hdc, font);
                SetTextColor(hdc, textColor);
                SetBkMode(hdc, TRANSPARENT);
                
                RECT textRect;
                CopyRect(&textRect, &rect);
                DrawTextA(hdc, displayText, -1, &textRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                DeleteObject(font);
            }
            
            // ============================================================
            // СТИЛЬ 2: SQUARES (центр, цветной ПРЯМОУГОЛЬНИК, БЕЗ рамки, БЕЗ текста)
            // ============================================================
            else if (visualType == 2) {
                // ---- НАСТРОЙКИ ЦВЕТОВ ФОНА ----
                COLORREF ruColor = RGB(255, 107, 53);   // Оранжевый для RU
                COLORREF enColor = RGB(74, 144, 217);    // Голубой для EN
                // ------------------------------------
                
                COLORREF bgColor = (langID == 0x0419) ? ruColor : 
                                   (langID == 0x0409) ? enColor : 
                                   RGB(255, 255, 255);
                
                HBRUSH bgBrush = CreateSolidBrush(bgColor);
                FillRect(hdc, &rect, bgBrush);
                DeleteObject(bgBrush);
            }
            
            EndPaint(hwnd, &ps);
            return 0;
        }
        
        case WM_TIMER: {
            if (wParam == 1) {
                KillTimer(hwnd, 1);
                timerRunning = 0;
                ShowWindow(hwnd, SW_HIDE);
            }
            return 0;
        }
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

void ShowOverlay(HINSTANCE hInst) {
    if (!IsVisualMarkerEnabled()) return;
    
    int visualType = GetVisualType();
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    
    int width, height, x, y;
    int bottomOffset = 80;
    
    // ============================================================
    // НАСТРОЙКИ ТОЛЬКО ДЛЯ 3-ГО СТИЛЯ (SQUARES)
    // ============================================================
    int margin = 40;           // Отступ 40px от левого и правого края
    int squareHeight = 80;     // Высота полоски
    // ============================================================
    
    if (visualType == 0) {
        // CLASSIC: 200x200 по центру
        width = 200;
        height = 200;
        x = (screenWidth - width) / 2;
        y = (screenHeight - height) / 2;
    } else if (visualType == 1) {
        // MODERN: 200x200 внизу (только Y поменялся)
        width = 200;
        height = 200;
        x = (screenWidth - width) / 2;
        y = (screenHeight / 2) + (screenHeight / 4) - (height / 2);  // НОВАЯ ПОЗИЦИЯ
    } else {
        // SQUARES: полоска на всю ширину с отступами, в нижней половине экрана
        width = screenWidth - (margin * 2);
        height = squareHeight;
        x = margin;
        y = (screenHeight / 2) + (screenHeight / 4) - (height / 2);
    }
    
    if (!hwndOverlay) {
        WNDCLASSA wcOverlay = {0};
        wcOverlay.lpfnWndProc = OverlayWndProc;
        wcOverlay.hInstance = hInst;
        wcOverlay.hbrBackground = (HBRUSH)GetStockObject(NULL_BRUSH);
        wcOverlay.lpszClassName = "OverlayClass";
        wcOverlay.style = CS_HREDRAW | CS_VREDRAW;
        
        if (!RegisterClassA(&wcOverlay)) {
            return;
        }
        
        hwndOverlay = CreateWindowA("OverlayClass", "",
            WS_POPUP | WS_VISIBLE,
            x, y, width, height,
            NULL, NULL, hInst, NULL);
        
        if (!hwndOverlay) {
            return;
        }
        
        SetWindowPos(hwndOverlay, HWND_TOPMOST, x, y, width, height, SWP_SHOWWINDOW);
        SetWindowLong(hwndOverlay, GWL_EXSTYLE,
            GetWindowLong(hwndOverlay, GWL_EXSTYLE) | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE);
        SetLayeredWindowAttributes(hwndOverlay, 0, 255, LWA_ALPHA);
    } else {
        SetWindowPos(hwndOverlay, HWND_TOPMOST, x, y, width, height, SWP_SHOWWINDOW);
    }
    
    if (timerRunning) {
        KillTimer(hwndOverlay, 1);
        timerRunning = 0;
    }
    
    ShowWindow(hwndOverlay, SW_SHOW);
    SetWindowPos(hwndOverlay, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    InvalidateRect(hwndOverlay, NULL, TRUE);
    UpdateWindow(hwndOverlay);
    
    SetTimer(hwndOverlay, 1, 500, NULL);
    timerRunning = 1;
}

void HideOverlay() {
    if (hwndOverlay) {
        ShowWindow(hwndOverlay, SW_HIDE);
        if (timerRunning) {
            KillTimer(hwndOverlay, 1);
            timerRunning = 0;
        }
    }
}