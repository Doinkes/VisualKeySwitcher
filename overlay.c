#include <stdio.h>
#include "overlay.h"
#include "layout.h"

static HWND hwndOverlay = NULL;
static int timerRunning = 0;

// Функция для получения цвета в зависимости от раскладки
COLORREF GetLayoutColor(HKL layout) {
    WORD langID = LOWORD(layout);
    if (langID == 0x0419) {
        return RGB(74, 144, 217);  // Голубой
    } else if (langID == 0x0409) {
        return RGB(255, 107, 53);  // Оранжевый
    }
    return RGB(255, 255, 255);
}

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
            
            // Определяем текст (инвертированный)
            char displayText[10];
            if (langID == 0x0419) {
                sprintf(displayText, "EN");
            } else if (langID == 0x0409) {
                sprintf(displayText, "RU");
            } else {
                sprintf(displayText, "??");
            }
            
            if (visualType == 0) {
                // CLASSIC: чёрный фон, белая рамка, белый текст
                HBRUSH bgBrush = CreateSolidBrush(RGB(0, 0, 0));
                FillRect(hdc, &rect, bgBrush);
                DeleteObject(bgBrush);
                
                HPEN whitePen = CreatePen(PS_SOLID, 3, RGB(255, 255, 255));
                SelectObject(hdc, whitePen);
                SelectObject(hdc, GetStockObject(NULL_BRUSH));
                Rectangle(hdc, 0, 0, rect.right, rect.bottom);
                DeleteObject(whitePen);
                
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
            } else {
                // MODERN: цветной фон, цветной текст
                COLORREF bgColor;
                if (langID == 0x0419) {
                    bgColor = RGB(255, 107, 53);  // Русская -> оранжевый
                } else if (langID == 0x0409) {
                    bgColor = RGB(74, 144, 217);  // Английская -> голубой
                } else {
                    bgColor = RGB(255, 255, 255);
                }
                
                HBRUSH bgBrush = CreateSolidBrush(bgColor);
                FillRect(hdc, &rect, bgBrush);
                DeleteObject(bgBrush);
                
                HPEN whitePen = CreatePen(PS_SOLID, 3, RGB(255, 255, 255));
                SelectObject(hdc, whitePen);
                SelectObject(hdc, GetStockObject(NULL_BRUSH));
                Rectangle(hdc, 0, 0, rect.right, rect.bottom);
                DeleteObject(whitePen);
                
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
    int width = 200;
    int height = 200;
    int x, y;
    
    if (visualType == 0) {
        // CLASSIC: по центру экрана
        x = (screenWidth - width) / 2;
        y = (screenHeight - height) / 2;
    } else {
        // MODERN: внизу, 80px от низа
        x = (screenWidth - width) / 2;
        y = screenHeight - height - 80;
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
        // Обновляем позицию
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