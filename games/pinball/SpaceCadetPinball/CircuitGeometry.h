#pragma once
#include "maths.h"
#include <vector>
#include <algorithm>
#include <cmath>

// Authored pixel coordinates on the unchanged 1024x1024 playfield.
// Every physical rail is a connected, two-sided, round-ended path. Only
// the shooter return gate and ramp portals have directional semantics.
namespace CircuitGeometry {
// The arrow-marked beige strip is lane artwork; contact is at the coil head.
constexpr float LauncherY=873, LauncherX=939;
constexpr float LauncherLeft=895+(LauncherY-667)*15/378;
constexpr float LauncherRight=945+(LauncherY-667)*47/378;
struct Bumper { float x,y,radius; };
inline const std::vector<Bumper>& Bumpers() {
    // Ground collision footprints. The upper-right skirt ends at y=250;
    // its old centre at y=202 pinched the return below the rightmost target.
    static const std::vector<Bumper> bumpers={{470,226,41},{617,210,40},{563,287,40},{202,431,30}};
    return bumpers;
}
struct Wall { const char* name; std::vector<vector2> points; bool solid; int layers; };
inline const std::vector<Wall>& Walls() {
    static const std::vector<Wall> walls = {
        {"outer", {{55,1045},{97,800},{137,580},{177,380},{201,260},{214,180},{227,100},{242,37},{255,12},{410,12},{550,17},{682,44},{781,91},{848,166},{880,244},{907,387},{945,667},{992,1045}}, false,3},
        // The divider passes the launcher floor and continues below it. It
        // follows the right orbit to its open exit above the target bank.
        {"shooter_inner", {{910,1045},{895,667},{878,490},{855,355},{842,289},{807,210},{770,165},{720,130},{660,106}}, false,3},
        {"left_outlane", {{192,550},{203,710},{181,930},{170,1045}}, false,3},
        {"left_inlane", {{300,420},{302,535},{281,724},{291,759},{355,815}}, false,3},
        {"left_return", {{203,771},{330,868}}, false,3},
        {"right_inlane", {{774,432},{768,574},{788,731},{732,788}}, false,3},
        {"right_outlane", {{840,480},{842,648},{869,842},{875,1045}}, false,3},
        {"right_return", {{825,784},{731,863}}, false,3},
        {"left_apron", {{290,900},{400,990},{475,1030}}, false,3},
        {"right_apron", {{600,1030},{740,948},{765,895}}, false,3},
        // Closed bodies behind the scoring faces, not isolated front edges.
        {"sling_body_left", {{285,588},{298,574},{359,759},{354,776},{273,727},{285,588}}, true,1},
        {"sling_body_right", {{767,583},{777,580},{789,730},{717,778},{701,764},{767,583}}, true,1},
        // The target plinth ends at its rear edge, NOT at the cabinet roof.
        // Its shallow rear bevel guides slow balls left into the underpass.
        // Gaps between the four uprights are narrower than the ball diameter.
        {"target_bank", {{401,96},{480,87},{611,82},{611,142},{500,142},{481,153},{438,170},{403,174},{391,133},{401,96}}, true,1},
        {"module_bank", {{755,267},{810,278},{761,428},{721,414},{755,267}}, true,1},
        {"left_upper_lane", {{192,550},{184,514},{185,493},{204,469},{248,443},{264,418},{272,383},{263,347},{244,331},{216,331},{193,344}}, false,3},
        {"left_upper_return", {{273,472},{293,442},{308,400},{303,356},{288,326},{278,300},{248,291},{225,295},{211,308}}, false,3},
        {"right_bank", {{642,122},{661,116},{696,144},{728,201},{746,231},{746,262},{730,256},{710,209},{676,159},{642,142},{642,122}}, true,1},
    };
    return walls;
}
inline const std::vector<vector2>& RampCentres() {
    static const std::vector<vector2> points = {{374,405},{369,344},{350,292},{310,263},{265,248},{238,220},{233,181},{244,132},{270,91},{305,61},{353,42},{379,40},{395,44},{402,57},{407,76},{420,94},{427,126},{423,160},{414,195}};
    return points;
}
inline void RampSides(std::vector<vector2>& left, std::vector<vector2>& right) {
    const auto& knots=RampCentres();
    const float halfWidth[]={24,24,26,27,26,25,24,25,26,27,27,22,18,16,16,18,20,22,22};
    std::vector<vector2> centres;std::vector<float> widths;
    // Catmull-Rom samples author a smooth tube instead of a chain of sharp
    // deflecting corners. The upstream engine still owns all motion/collision.
    for(size_t i=0;i+1<knots.size();++i)for(int step=0;step<4;++step){
        float t=step/4.f,t2=t*t,t3=t2*t;
        auto a=knots[i?i-1:i],b=knots[i],c=knots[i+1],d=knots[std::min(i+2,knots.size()-1)];
        auto interpolate=[&](float a,float b,float c,float d){return .5f*((2*b)+(-a+c)*t+(2*a-5*b+4*c-d)*t2+(-a+3*b-3*c+d)*t3);};
        centres.push_back({interpolate(a.X,b.X,c.X,d.X),interpolate(a.Y,b.Y,c.Y,d.Y)});
        widths.push_back(halfWidth[i]*(1-t)+halfWidth[i+1]*t);
    }
    centres.push_back(knots.back());widths.push_back(halfWidth[knots.size()-1]);
    for(size_t i=0;i<centres.size();++i) {
        auto a=centres[i?i-1:i],b=centres[i+1<centres.size()?i+1:i];
        float dx=b.X-a.X,dy=b.Y-a.Y,len=std::hypot(dx,dy),w=widths[i];
        left.push_back({centres[i].X-dy/len*w,centres[i].Y+dx/len*w});
        right.push_back({centres[i].X+dy/len*w,centres[i].Y-dx/len*w});
    }
}
// Diagnostic footprint of each low tube body, clipped independently so the
// high-arch underpass is never filled by a polygon joining the two legs.
inline const std::vector<std::vector<vector2>>& GroundRampBodies() {
    static const auto bodies=[] {
        std::vector<vector2> left,right; RampSides(left,right);
        std::vector<std::vector<vector2>> result;
        for(size_t i=1;i<left.size();++i){
            std::vector<vector2> quad={left[i-1],right[i-1],right[i],left[i]},part;
            auto a=quad.back();
            for(auto b:quad){
                if((a.Y>=120)!=(b.Y>=120))part.push_back({a.X+(120-a.Y)*(b.X-a.X)/(b.Y-a.Y),120});
                if(b.Y>=120)part.push_back(b);
                a=b;
            }
            if(part.size()>=3)result.push_back(part);
        }
        return result;
    }();
    return bodies;
}
// The high arch is a bridge, not a wall across the ground-level rear orbit.
// Low rails remain solid on the ground; the entire tube is closed on layer 2.
inline std::vector<Wall> RampWalls() {
    std::vector<vector2> left,right; RampSides(left,right);
    std::vector<Wall> result={{"ramp_rail_l",left,false,2},{"ramp_rail_r",right,false,2}};
    for(int side=0;side<2;++side){
        const auto& rail=side?right:left;
        std::vector<vector2> part;
        for(size_t i=1;i<rail.size();++i){
            auto a=rail[i-1],b=rail[i];
            bool lowA=a.Y>=120,lowB=b.Y>=120;
            if(lowA && part.empty())part.push_back(a);
            if(lowA!=lowB){
                float t=(120-a.Y)/(b.Y-a.Y);
                vector2 crossing={a.X+t*(b.X-a.X),120};
                if(lowA){part.push_back(crossing);result.push_back({side?"ramp_support_r":"ramp_support_l",part,false,1});part.clear();}
                else part.push_back(crossing);
            }
            if(lowB)part.push_back(b);
        }
        if(part.size()>1)result.push_back({side?"ramp_support_r":"ramp_support_l",part,false,1});
    }
    // Close the low tube bodies at the bridge cut without spanning the open
    // ground underpass between them. Each rail crosses the cut twice.
    std::vector<float> cuts;
    for(const auto& rail:{left,right})for(size_t i=1;i<rail.size();++i){
        auto a=rail[i-1],b=rail[i];
        if((a.Y>=120)!=(b.Y>=120))cuts.push_back(a.X+(120-a.Y)*(b.X-a.X)/(b.Y-a.Y));
    }
    std::sort(cuts.begin(),cuts.end());
    for(size_t i=1;i<cuts.size();i+=2)
        result.push_back({"ramp_support_end",{{cuts[i-1],120},{cuts[i],120}},false,1});
    return result;
}
}
