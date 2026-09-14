#include "pch.h"
#include "ArcadeTextInput.h"

namespace ArcadeTextInput {
static ImGuiID target = 0;
static int pasteFrame = -1;
static std::string pending;

void Paste(const char* text) {
    auto& g = *ImGui::GetCurrentContext();
    if (!g.IO.WantTextInput || !ImGui::GetInputTextState(g.ActiveId)) return;
    if (target != g.ActiveId || pasteFrame != g.FrameCount + 1) pending.clear();
    target = g.ActiveId;
    pasteFrame = g.FrameCount + 1;
    // The bridge already bounds individual payloads. Bound accumulation too.
    if (pending.size() + strlen(text) <= 4096) pending += text;
}

static int paste(ImGuiInputTextCallbackData* data) {
    if (pasteFrame != ImGui::GetFrameCount() || target != *static_cast<ImGuiID*>(data->UserData)
        || pending.empty()) return 0;
    const int start = ImMin(data->SelectionStart, data->SelectionEnd);
    const int end = ImMax(data->SelectionStart, data->SelectionEnd);
    const int capacity = data->BufSize - 1 - (data->BufTextLen - (end - start));
    // Fit whole UTF-8 code points in the existing fixed-size field. Match ImGui's
    // supported character range and ignore single-line control characters.
    std::string text;
    for (const char* p = pending.c_str(); *p;) {
        unsigned int c;
        int bytes = ImTextCharFromUtf8(&c, p, nullptr);
        if (bytes <= 0 || c == 0) break;
        if (c >= 32 && c != 127 && c <= IM_UNICODE_CODEPOINT_MAX) {
            if (static_cast<int>(text.size()) + bytes > capacity) break;
            text.append(p, bytes);
        }
        p += bytes;
    }
    pending.clear();
    if (text.empty()) return 0;
    data->DeleteChars(start, end - start);
    data->CursorPos = start;
    data->InsertChars(start, text.c_str());
    return 0;
}

bool InputText(const char* label, char* buffer, size_t size, ImGuiInputTextFlags flags) {
    ImGuiID id = ImGui::GetID(label);
    return ImGui::InputText(label, buffer, size, flags | ImGuiInputTextFlags_CallbackAlways, paste, &id);
}
}
