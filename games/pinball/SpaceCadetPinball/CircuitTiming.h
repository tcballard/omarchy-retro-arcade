#pragma once
#include <algorithm>

namespace CircuitTiming {
// Preserve elapsed wall time during normal rendering stalls, with small physics
// steps so forces, low-speed motion and contacts do not depend on render rate.
// A long suspend/debugger stall advances at most 100 ms, not a hidden full turn.
template<class Step> void Advance(float elapsedMs, Step step) {
    float remaining = std::max(0.f, std::min(elapsedMs, 100.f));
    while (remaining > 0.f) {
        const float dt = std::min(remaining, 1000.f / 120.f);
        step(dt);
        remaining -= dt;
    }
}
}
