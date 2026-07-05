#ifndef LAYOUT_H
#define LAYOUT_H

#include <windows.h>

void InitLayout(HINSTANCE hInst);
HKL GetActiveWindowLayout();
char* GetInvertedLanguageName(HKL hkl);
void PlayLayoutSound(HKL layout);

int IsVisualMarkerEnabled();
int IsSoundEnabled();
void ToggleVisualMarker();
void ToggleSound();

int GetHotkeyModifiers();
void SetHotkey(int modifiers, int keyCode);
int IsHotkeyPressed(int vkCode);

#endif