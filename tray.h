#ifndef TRAY_H
#define TRAY_H

#include <windows.h>

// Sound pack list from layout.c
extern char soundPacksList[100][MAX_PATH];
extern int soundPacksCount;

void InitTray(HWND hwnd, HINSTANCE hInst);
void UpdateTrayMenu(HWND hwnd);
void DestroyTray();
void HandleTrayMessage(WPARAM wParam, LPARAM lParam);
void HandleTrayCommand(WPARAM wParam);

#endif