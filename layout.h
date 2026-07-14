#ifndef LAYOUT_H
#define LAYOUT_H

#include <windows.h>

void InitLayout(HINSTANCE hInst);
HKL GetActiveWindowLayout();
char* GetInvertedLanguageName(HKL hkl);
void PlayLayoutSound(HKL layout);

int IsVisualMarkerEnabled();
int IsSoundEnabled();
int IsAutoStartEnabled();
void ToggleVisualMarker();
void ToggleSound();
void ToggleAutoStart();

int GetHotkeyModifiers();
void SetHotkey(int modifiers, int keyCode);
int IsHotkeyPressed(int vkCode);

// Visual Type
int GetVisualType();
void SetVisualType(int type);

#endif