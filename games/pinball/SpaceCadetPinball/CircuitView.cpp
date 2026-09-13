#include "pch.h"
#include "CircuitView.h"
#include "OmarchyTable.h"
#include "OmarchyTheme.h"
#include "pb.h"
#include "CircuitGeometry.h"
#include "nudge.h"
#include "TPinballTable.h"
#include "TFlipper.h"
#include "TFlipperEdge.h"
#include "TBall.h"
#include "TPlunger.h"
#include "winmain.h"
#include "options.h"
#include "SDL_image.h"
#include "../native/BrandWordmark.h"
#include <algorithm>
#include <string>
#ifndef CIRCUIT_SOURCE_DIR
#define CIRCUIT_SOURCE_DIR "assets/circuit"
#endif
#ifndef CIRCUIT_INSTALL_DIR
#define CIRCUIT_INSTALL_DIR "/usr/share/omarchy-spacecadet/circuit"
#endif
namespace CircuitView {
namespace {
SDL_Texture *board=nullptr,*wordmark=nullptr,*bridge=nullptr;
constexpr SDL_Rect bridgeBounds={200,0,272,120};
struct CapSprite { SDL_Texture* texture; SDL_Rect bounds; float groundY; };
std::vector<CapSprite> bumperCaps;
SDL_Surface* original=nullptr;
SDL_Renderer* renderer=nullptr;
uint32_t lastAccent=0;
float scale,ox,oy;
bool portrait=false;
ImDrawList* draw;
ImVec2 p(float x,float y){return {ox+x*scale,oy+y*scale};}
ImU32 rgba(int r,int g,int b,int a=255){return IM_COL32(r,g,b,a);}
ImU32 accent(int a=255){auto c=OmarchyTheme::Accent();return rgba((c>>16)&255,(c>>8)&255,c&255,a);}
void disc(float x,float y,float r,ImU32 c){draw->AddCircleFilled(p(x,y),r*scale,c,24);}
void lamp(float x,float y,bool on,float radius=9){
 if(on){disc(x,y,radius+8,accent(15));disc(x,y,radius+4,accent(35));disc(x,y,radius,accent());disc(x-2,y-2,radius*.5f,rgba(238,249,213));}
}
void label(float x,float y,const char* text,float size=18,ImU32 c=IM_COL32(220,221,202,255)){
 draw->AddText(ImGui::GetFont(),size*scale,p(x,y),c,text);
}
// A small legible 5x7 dot-matrix alphabet; rendering stays sharp at every window size.
const char* glyph(char c){
 switch(c){
#define G(c,s) case c:return s
 G('0',"0E11131519110E");G('1',"040C040404040E");G('2',"0E11010204081F");G('3',"1E01010601011E");G('4',"02060A121F0202");G('5',"1F10101E01011E");G('6',"0610101E11110E");G('7',"1F010204080808");G('8',"0E11110E11110E");G('9',"0E11110F01010C");
 G('A',"0E11111F111111");G('B',"1E11111E11111E");G('C',"0E11101010110E");G('D',"1E11111111111E");G('E',"1F10101E10101F");G('F',"1F10101E101010");G('G',"0E11101711110F");G('H',"1111111F111111");G('I',"0E04040404040E");G('J',"0702020212120C");G('K',"11121418141211");G('L',"1010101010101F");G('M',"111B1515111111");G('N',"11191513111111");G('O',"0E11111111110E");G('P',"1E11111E101010");G('Q',"0E11111115120D");G('R',"1E11111E141211");G('S',"0F10100E01011E");G('T',"1F040404040404");G('U',"1111111111110E");G('V',"11111111110A04");G('W',"11111115151B11");G('X',"11110A040A1111");G('Y',"11110A04040404");G('Z',"1F01020408101F");G('+',"0004041F040400");G('-',"0000001F000000");G('/',"01010204081010");G(':',"00040000040000");
#undef G
 default:return "00000000000000";
 }
}
void matrix(float x,float y,float w,float h,const std::string& text,float dot){
 auto hex=[](char c){return c<='9'?c-'0':c-'A'+10;};
 float width=(text.size()*6-1)*dot;float start=x+(w-width)/2,top=y+(h-7*dot)/2;
 for(size_t i=0;i<text.size();i++){auto g=glyph(text[i]);for(int row=0;row<7;row++){int bits=hex(g[row*2])*16+hex(g[row*2+1]);for(int col=0;col<5;col++)if(bits&(1<<(4-col))){float xx=start+(i*6+col)*dot,yy=top+row*dot;disc(xx,yy,dot*.61f,rgba(255,120,15,22));disc(xx,yy,dot*.36f,rgba(255,185,72));}}}
}
void capsule(float x,float y,float xx,float yy,float r,ImU32 c){float len=std::hypot(xx-x,yy-y),nx=-(yy-y)/len,ny=(xx-x)/len,tip=r*.65f;
 ImVec2 q[]={p(x+nx*r,y+ny*r),p(xx+nx*tip,yy+ny*tip),p(xx-nx*tip,yy-ny*tip),p(x-nx*r,y-ny*r)};draw->AddConvexPolyFilled(q,4,c);disc(x,y,r,c);disc(xx,yy,tip,c);}
void drawBridge(const ImDrawList*,const ImDrawCmd*){
 SDL_FRect destination={ox+bridgeBounds.x*scale,oy,bridgeBounds.w*scale,bridgeBounds.h*scale};
 SDL_RenderCopyF(renderer,bridge,nullptr,&destination);
}
void drawCap(const ImDrawList*,const ImDrawCmd* command){
 auto cap=static_cast<const CapSprite*>(command->UserCallbackData);
 SDL_FRect destination={ox+cap->bounds.x*scale,oy+cap->bounds.y*scale,cap->bounds.w*scale,cap->bounds.h*scale};
 SDL_RenderCopyF(renderer,cap->texture,nullptr,&destination);
}
void drawBoard(const ImDrawList*, const ImDrawCmd*){
 SDL_Rect source={0,0,portrait?1024:1536,1024};
 SDL_FRect destination={ox,oy,source.w*scale,1024*scale};
 SDL_RenderCopyF(renderer,board,&source,&destination);
}
void updateTexture(){
 uint32_t c=OmarchyTheme::Accent();if(c==lastAccent&&board)return;lastAccent=c;
 auto surface=SDL_ConvertSurfaceFormat(original,SDL_PIXELFORMAT_ARGB8888,0);auto pixels=(uint32_t*)surface->pixels;
 // Recolour green glass and lamps only. Ivory, chrome, black and amber keep their materials.
 float ar=((c>>16)&255)/255.f,ag=((c>>8)&255)/255.f,ab=(c&255)/255.f;
 for(int y=0;y<surface->h;y++)for(int x=0;x<surface->w;x++){
  auto& q=pixels[y*surface->pitch/4+x];float r=((q>>16)&255)/255.f,g=((q>>8)&255)/255.f,b=(q&255)/255.f;
  float amount=std::max(0.f,std::min(1.f,(g-std::max(r,b))/.15f));
  float bright=std::max(r,std::max(g,b));
  if(amount>.02f){int rr=(int)(255*(r*(1-amount)+ar*bright*amount));int gg=(int)(255*(g*(1-amount)+ag*bright*amount));int bb=(int)(255*(b*(1-amount)+ab*bright*amount));q=0xff000000|(rr<<16)|(gg<<8)|bb;}
 }
 if(board)SDL_DestroyTexture(board);board=SDL_CreateTextureFromSurface(renderer,surface);
 // A small alpha-masked rectangular sprite preserves exact source sampling.
 // Textured triangle patches produce seams in SDL's software rasterizer.
 std::vector<vector2> left,right;CircuitGeometry::RampSides(left,right);
 left.insert(left.end(),right.rbegin(),right.rend());
 auto foreground=SDL_CreateRGBSurfaceWithFormat(0,bridgeBounds.w,bridgeBounds.h,32,SDL_PIXELFORMAT_ARGB8888);
 if(foreground){
  auto dest=static_cast<uint32_t*>(foreground->pixels);
  for(int y=0;y<bridgeBounds.h;++y)for(int x=0;x<bridgeBounds.w;++x){
   float px=x+bridgeBounds.x+.5f,py=y+.5f;bool inside=false;
   for(size_t i=0,j=left.size()-1;i<left.size();j=i++){
    auto a=left[i],b=left[j];
    if((a.Y>py)!=(b.Y>py) && px<(b.X-a.X)*(py-a.Y)/(b.Y-a.Y)+a.X)inside=!inside;
   }
   dest[y*foreground->pitch/4+x]=inside?pixels[y*surface->pitch/4+x+bridgeBounds.x]:0;
  }
  if(bridge)SDL_DestroyTexture(bridge);
  bridge=SDL_CreateTextureFromSurface(renderer,foreground);
  SDL_SetTextureBlendMode(bridge,SDL_BLENDMODE_BLEND);
  SDL_FreeSurface(foreground);
 }
 // Preserve #19's cap occlusion with seam-free sprites from the shared layout.
 for(auto& cap:bumperCaps)SDL_DestroyTexture(cap.texture);
 bumperCaps.clear();
 for(auto b:CircuitGeometry::Bumpers()){
  float rx=b.radius+3,ry=b.radius*.9f+9,cy=b.y-16;
  SDL_Rect bounds={int(std::floor(b.x-rx)),int(std::floor(cy-ry)),int(std::ceil(rx*2))+2,int(std::ceil(ry*2))+2};
  auto capSurface=SDL_CreateRGBSurfaceWithFormat(0,bounds.w,bounds.h,32,SDL_PIXELFORMAT_ARGB8888);
  if(!capSurface)continue;
  auto dest=static_cast<uint32_t*>(capSurface->pixels);
  for(int y=0;y<bounds.h;++y)for(int x=0;x<bounds.w;++x){
   float dx=(bounds.x+x+.5f-b.x)/rx,dy=(bounds.y+y+.5f-cy)/ry;
   dest[y*capSurface->pitch/4+x]=dx*dx+dy*dy<=1?pixels[(bounds.y+y)*surface->pitch/4+bounds.x+x]:0;
  }
  auto texture=SDL_CreateTextureFromSurface(renderer,capSurface);
  SDL_FreeSurface(capSurface);
  if(texture){SDL_SetTextureBlendMode(texture,SDL_BLENDMODE_BLEND);bumperCaps.push_back({texture,bounds,b.y});}
 }
 SDL_FreeSurface(surface);
}
}
bool Init(SDL_Renderer* r){
 renderer=r;char* base=SDL_GetBasePath();
 std::vector<std::string> dirs={CIRCUIT_INSTALL_DIR,std::string(base?base:"")+"../../share/omarchy-retro-arcade/circuit",CIRCUIT_SOURCE_DIR};SDL_free(base);
 if(const char* dir=getenv("OMARCHY_CIRCUIT_ASSETS"))dirs.insert(dirs.begin(),dir);
 for(const auto& dir:dirs){original=IMG_Load((dir+"/table.png").c_str());if(original)break;}
 if(!original){SDL_Log("Cannot load Circuit artwork: %s",IMG_GetError());return false;}
 SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY,"1");
 auto surface=SDL_CreateRGBSurfaceWithFormat(0,4131,950,32,SDL_PIXELFORMAT_RGBA32);
 if(!surface)return false;
 SDL_FillRect(surface,nullptr,SDL_MapRGBA(surface->format,0,0,0,0));
 for(const auto& r:BrandWordmark){SDL_Rect rect={r[0],r[1],r[2],r[3]};SDL_FillRect(surface,&rect,SDL_MapRGBA(surface->format,158,206,106,255));}
 wordmark=SDL_CreateTextureFromSurface(renderer,surface);SDL_FreeSurface(surface);
 updateTexture();return board&&wordmark;
}
void Draw(){
 if(!board||!pb::MainTable)return;updateTexture();draw=ImGui::GetBackgroundDrawList();
 auto size=ImGui::GetIO().DisplaySize;float menu=options::Options.ShowMenu?winmain::MainMenuHeight:0;
 portrait=size.x/(size.y-menu)<1.2f;
 float layoutWidth=portrait?1024.f:1536.f,layoutHeight=portrait?1230.f:1024.f;
 scale=std::min(size.x/layoutWidth,(size.y-menu)/layoutHeight);
 ox=(size.x-layoutWidth*scale)/2;oy=menu+(size.y-menu-layoutHeight*scale)/2;
 if(!winmain::single_step){
  ox+=6.f*scale*(nudge::nudged_right-nudge::nudged_left);
  oy-=3.f*scale*(nudge::nudged_right+nudge::nudged_left+nudge::nudged_up);
 }
 draw->AddRectFilled({0,menu},size,rgba(5,8,8));
 // Copy the rectangular plate once, retaining ImGui overlay ordering.
 draw->AddCallback(drawBoard,nullptr);
 // Exact official wordmark geometry, proportionally placed after material tinting.
 const float wordScale=144.f/4131.f,wordX=562-72,wordY=520-950*wordScale/2;
 draw->AddImage((ImTextureID)wordmark,p(wordX,wordY),p(wordX+144,wordY+950*wordScale));
 auto t=pb::MainTable;
 const float lamps[12][2]={{562,422},{626,438},{664,478},{674,525},{659,565},{618,599},{562,617},{507,599},{464,565},{450,526},{458,479},{498,439}};
 for(unsigned i=0;i<12;i++)lamp(lamps[i][0],lamps[i][1],i<OmarchyTable::Progress());
 for(int i=0;i<4;i++){const auto& bumper=CircuitGeometry::Bumpers()[i];std::string n="bumper"+std::to_string(i);float f=OmarchyTable::Flash(n.c_str());if(f>0){draw->AddCircle(p(bumper.x,bumper.y),(i==3?33:44)*scale,accent((int)(f*230)),32,5*scale);disc(bumper.x,bumper.y-15,16,rgba(255,241,179,(int)(f*100)));}}
 for(int i=0;i<8;i++){float x=i<4?504+i*31:747-(i-4)*8,y=i<4?117:319+(i-4)*25;lamp(x,y,(OmarchyTable::Targets()&(1u<<i))!=0,7);}
 for(int i=0;i<2;i++){std::string name=i?"sling_right":"sling_left";float f=OmarchyTable::Flash(name.c_str());if(f>0)draw->AddLine(p(i?755:295,635),p(i?690:360,778),accent((int)(f*220)),5*scale);}
 // Flipper endpoints come from the engine's current collision edge, not a UI animation.
 for(auto flipper:t->FlipperList){auto e=flipper->FlipperEdge;float x=540+e->RotOrigin.X*25,y=500+e->RotOrigin.Y*25;
  float dx=e->T1Src.X-e->RotOrigin.X,dy=e->T1Src.Y-e->RotOrigin.Y;
  float a=e->CurrentAngle,xx=x+(dx*cos(a)-dy*sin(a))*25,yy=y+(dx*sin(a)+dy*cos(a))*25;
  capsule(x+5,y+10,xx+5,yy+10,18,rgba(0,0,0,150));capsule(x,y,xx,yy,18,rgba(37,42,34));
  capsule(x,y-3,xx,yy-3,15,rgba(173,168,132));capsule(x,y-6,xx,yy-6,12,rgba(229,226,196));
  capsule(x+2,y+10,xx,yy+8,3,accent());disc(x,y-5,10,rgba(58,61,52));disc(x-2,y-7,7,rgba(210,212,193));disc(x-4,y-9,3,rgba(253,251,226));
 }
 // Compress the actual coil in its existing housing. Keep the artwork file
 // intact. The contact head sits at the coil, not atop the beige lane arrows.
 // Charge is engine state, so holding, releasing, pause and reset stay in sync.
 const float pull=t->Plunger->PullbackStartedFlag
     ? std::max(0.f,std::min(t->Plunger->Boost/t->Plunger->MaxPullback,1.f)) : 0.f;
 if(pull>0.f){
  ImVec2 housing[]={p(924,873),p(963,873),p(973,953),p(930,953)};
  draw->AddConvexPolyFilled(housing,4,rgba(14,15,12));
  // The shaft stays visible through the opening above the compressed coil.
  draw->AddLine(p(941,873),p(950,952),rgba(62,65,59),8*scale);
  draw->AddLine(p(939,873),p(948,952),rgba(176,181,163),2*scale);
  const float travel=48*pull;
  draw->AddImage((ImTextureID)board,p(925+5*pull,873+travel),p(969,953),
      {925.f/1536,873.f/1024},{969.f/1536,953.f/1024});
 }
 // A metal head and continuous shaft visibly connect ball contact to the coil.
 draw->AddRectFilled(p(CircuitGeometry::LauncherLeft,CircuitGeometry::LauncherY),
     p(CircuitGeometry::LauncherRight,CircuitGeometry::LauncherY+5),rgba(171,178,162),2*scale);
 draw->AddLine(p(CircuitGeometry::LauncherLeft+1,CircuitGeometry::LauncherY),
     p(CircuitGeometry::LauncherRight-1,CircuitGeometry::LauncherY),rgba(243,245,220),scale);
 auto drawBall=[&](TBall* ball){if(getenv("OMARCHY_TEST_HIDE_BALL"))return;float x=540+ball->Position.X*25,y=500+ball->Position.Y*25,r=ball->Radius*25;
  disc(x+5,y+10,r+2,rgba(0,0,0,155));disc(x,y,r+1,rgba(204,211,205));disc(x,y,r,rgba(29,36,38));
  disc(x-2,y-3,r*.82f,rgba(126,145,146));disc(x+2,y+3,r*.68f,rgba(33,46,47));disc(x-3,y-4,r*.55f,rgba(213,227,220));disc(x-4,y-5,r*.3f,rgba(255,255,241));
 };
 bool underBridge=false;
 for(auto ball:t->BallList)if(ball->ActiveFlag && !(ball->CollisionMask&2)){
  drawBall(ball);
  float ballY=500+ball->Position.Y*25;
  for(auto& cap:bumperCaps)if(ballY<cap.groundY)draw->AddCallback(drawCap,&cap);
  underBridge|=500+ball->Position.Y*25<132;
 }
 if(underBridge && bridge)draw->AddCallback(drawBridge,nullptr);
 for(auto ball:t->BallList)if(ball->ActiveFlag && (ball->CollisionMask&2))drawBall(ball);
 // Both layouts use the same priority: pause, game over, tilt/danger, charge, notice.
 std::string charge;
 if(t->Plunger->PullbackStartedFlag)charge=t->Plunger->Boost>=t->Plunger->MaxPullback?"RELEASE TO LAUNCH":"CHARGE "+std::to_string(int(100*t->Plunger->Boost/t->Plunger->MaxPullback))+"/100";
 const bool urgent=OmarchyTable::GameOver()||t->TiltLockFlag||nudge::nudge_count>.5f;
 const char* status=winmain::single_step?"PAUSED  P RESUME":urgent||charge.empty()?OmarchyTable::Status():charge.c_str();
 if(portrait){
  // Keep the approved playfield at its native proportions; replace the tall
  // decorative cabinet with a compact score strip below it.
  draw->AddRectFilled(p(8,1036),p(1016,1220),rgba(15,21,20),12*scale);
  draw->AddLine(p(24,1036),p(1000,1036),accent(150),2*scale);
  char score[32];snprintf(score,sizeof(score),"%07d",std::max(0,t->CurScore));
  matrix(20,1050,470,56,score,6);
  std::string ball=OmarchyTable::GameOver()?"GAME OVER":"BALL "+std::to_string(4-std::max(1,t->BallCount));
  matrix(510,1050,490,56,ball,4.5f);
  matrix(20,1110,980,42,status,3.25f);
  unsigned count=0;for(unsigned v=OmarchyTable::Targets();v;v>>=1)count+=v&1;
  std::string stats="CIRCUIT "+std::to_string(OmarchyTable::Progress())+"/12    TARGETS "+std::to_string(count)+"/8    ORBITS "+std::to_string(OmarchyTable::Orbits())+"    RAMPS "+std::to_string(OmarchyTable::Ramps());
  label(60,1160,stats.c_str(),20);
  label(60,1193,"A/D OR Z/SLASH   SPACE LAUNCH    X/./UP NUDGE",17);
  return;
 }
 char score[32];snprintf(score,sizeof(score),"%07d",std::max(0,t->CurScore));matrix(1067,554,414,67,score,7);
 std::string ball=OmarchyTable::GameOver()?"GAME OVER":"BALL "+std::to_string(4-std::max(1,t->BallCount));matrix(1067,638,414,56,ball,4.5f);
matrix(1067,714,414,66,status,3.25f);
 label(1090,820,"CIRCUIT",19);label(1300,820,"TARGET BANK",19);
 label(1090,852,(std::to_string(OmarchyTable::Progress())+" / 12").c_str(),23);unsigned count=0;for(unsigned v=OmarchyTable::Targets();v;v>>=1)count+=v&1;
 label(1300,852,(std::to_string(count)+" / 8").c_str(),23);
 label(1090,899,("ORBITS  "+std::to_string(OmarchyTable::Orbits())).c_str(),18);label(1300,899,("RAMPS  "+std::to_string(OmarchyTable::Ramps())).c_str(),18);
 label(1080,936,"A/D OR Z/SLASH   SPACE LAUNCH",17);
 label(1080,958,"X / . / UP   NUDGE",15);
 label(1110,48,"OMARCHY ARCADE  /  PINBALL",18);
}
void Shutdown(){for(auto& cap:bumperCaps)SDL_DestroyTexture(cap.texture);bumperCaps.clear();SDL_DestroyTexture(bridge);bridge=nullptr;SDL_DestroyTexture(board);SDL_DestroyTexture(wordmark);SDL_FreeSurface(original);board=wordmark=nullptr;original=nullptr;lastAccent=0;}
}
