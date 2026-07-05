#ifndef TRAY_H
#define TRAY_H

#include <windows.h>

void InitTray(HWND hwnd, HINSTANCE hInst);
void UpdateTrayMenu(HWND hwnd);
void DestroyTray();
void HandleTrayMessage(WPARAM wParam, LPARAM lParam);
void HandleTrayCommand(WPARAM wParam);

#endif