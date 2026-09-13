#include "CircuitTiming.h"
#include <cmath>
#include <cstdio>
#include <cstdlib>
void require(bool ok) { if (!ok) std::abort(); }
int main() {
    // Two seconds must remain two seconds at 120, 60, 30 and 20 rendered FPS.
    for (int fps : {120,60,30,20}) {
        double total=0; int count=0;
        for (int i=0;i<fps*2;i++) CircuitTiming::Advance(1000.f/fps,[&](float dt){
            require(dt>0 && dt<=1000.f/120.f); total+=dt; count++;
        });
        require(std::abs(total-2000)<.001 && count>=240);
    }
    float total=0;
    CircuitTiming::Advance(10000.f,[&](float dt){total+=dt;});
    require(std::abs(total-100)<.001);
    CircuitTiming::Advance(-1.f,[](float){ std::abort(); });
    CircuitTiming::Advance(0.f,[](float){ std::abort(); });
    std::puts("Circuit elapsed time and bounded physics steps passed");
}
