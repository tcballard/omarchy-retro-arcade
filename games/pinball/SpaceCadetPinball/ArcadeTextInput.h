#pragma once

#include "imgui.h"

namespace ArcadeTextInput {
// Committed clipboard text replaces the active selection without changing the
// physical modifier state. InputText's callback reconciliation preserves undo.
void Paste(const char* text);
bool InputText(const char* label, char* buffer, size_t size, ImGuiInputTextFlags flags = 0);
}
