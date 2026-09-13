#include "pch.h"
#include "OmarchyTable.h"
#include "CircuitGeometry.h"
#include "../tests/CircuitRouteFixtures.h"
#include "GroupData.h"
#include "gdrv.h"
#include "zdrv.h"
#include "pb.h"
#include "nudge.h"
#include "TPinballTable.h"
#include "TPlunger.h"
#include "TDrain.h"
#include "TBumper.h"
#include "TTextBox.h"
#include "options.h"
#include "TTripwire.h"
#include "TRamp.h"
#include "TBall.h"
#include "TLine.h"
#include "TEdgeManager.h"
#include "TTableLayer.h"
#include <map>
#include <algorithm>

// Authored table data. All distances below are engine world units.
// No original DAT, bitmap, sound or table parameters are embedded here.
namespace OmarchyTable {
bool Enabled=false;
namespace {
DatFile* data;
std::vector<int16_t> objects;
unsigned hits=0;
unsigned targetMask=0, orbitCount=0, rampCount=0, circuits=0;
std::map<std::string,float> flashes, debounce;
std::string notice="HOLD SPACE TO LAUNCH";
float noticeUntil=0;
bool over=false;
void announce(const char* text){notice=text;noticeUntil=pb::time_now+3;}
std::vector<int16_t> tone;
Mix_Chunk* effect=nullptr;
void sound(){
 if(!options::Options.Sounds)return;
 int rate,channels;Uint16 format;if(!Mix_QuerySpec(&rate,&format,&channels)||format!=AUDIO_S16SYS)return;
 if(!effect){tone.resize(rate*channels/12);for(size_t i=0;i<tone.size();i++){double t=double(i/channels)/rate;tone[i]=(int16_t)(2500*sin(t*2*3.141592653589793*660)*exp(-t*40));}effect=Mix_QuickLoad_RAW((Uint8*)tone.data(),(Uint32)(tone.size()*2));}
 if(effect){Mix_VolumeChunk(effect,options::Options.SoundVolume);Mix_PlayChannel(-1,effect,0);}
}
GroupData* group(const char* name,int type=200){
 auto g=new GroupData((int)data->Groups.size());data->Groups.push_back(g);
 auto add=[&](FieldTypes t,const void* src,int n){auto e=new EntryData();e->EntryType=t;e->FieldSize=n;e->Buffer=new char[n];memcpy(e->Buffer,src,n);g->AddEntry(e);};
 int16_t v=type;add(FieldTypes::ShortValue,&v,2);
 if(name)add(FieldTypes::GroupName,name,(int)strlen(name)+1);
 return g;
}
template<class T> void values(GroupData* g,FieldTypes type,std::initializer_list<T> a){
 auto e=new EntryData();e->EntryType=type;e->FieldSize=(int)(a.size()*sizeof(T));e->Buffer=new char[e->FieldSize];memcpy(e->Buffer,a.begin(),e->FieldSize);g->AddEntry(e);
}
void floats(GroupData* g,std::initializer_list<float> a){values(g,FieldTypes::FloatArray,a);}
void shorts(GroupData* g,std::initializer_list<int16_t> a){values(g,FieldTypes::ShortArray,a);}
void object(GroupData* g,int type){objects.push_back(type);objects.push_back(g->GroupId);}
gdrv_bitmap8* bitmap(GroupData* g,int w,int h,int x,int y,bool depth=true){
 auto b=new gdrv_bitmap8(w,h,true);b->XPosition=x;b->YPosition=y;
 memset(b->IndexedBmpPtr,0,b->IndexedStride*h);
 auto e=new EntryData(FieldTypes::Bitmap8bit,(char*)b);g->AddEntry(e);
 if(depth){auto z=new zmap_header_type(w,h,w);std::fill(z->ZPtr1,z->ZPtr1+w*h,5000);g->AddEntry(new EntryData(FieldTypes::Bitmap16bit,(char*)z));}
 return b;
}
void pixel(gdrv_bitmap8* b,int x,int y,int c){if(x>=0&&y>=0&&x<b->Width&&y<b->Height)b->IndexedBmpPtr[(b->Height-1-y)*b->IndexedStride+x]=(char)c;}
void line(gdrv_bitmap8* b,float x,float y,float xx,float yy,int c,int width=2){
 int n=std::max(1,(int)(std::hypot(xx-x,yy-y)*2));
 for(int i=0;i<=n;++i)for(int dy=-width;dy<=width;++dy)for(int dx=-width;dx<=width;++dx)pixel(b,(int)(x+(xx-x)*i/n)+dx,(int)(y+(yy-y)*i/n)+dy,c);
}
void circle(gdrv_bitmap8* b,int cx,int cy,int r,int c){for(int y=-r;y<=r;y++)for(int x=-r;x<=r;x++)if(x*x+y*y<=r*r)pixel(b,cx+x,cy+y,c);}
float wx(float x){return (x-540)/25;} float wy(float y){return (y-500)/25;}

}
DatFile* Build(){
 hits=targetMask=orbitCount=rampCount=circuits=0;over=false;flashes.clear();debounce.clear();data=new DatFile();data->AppName="Omarchy Arcade";data->Description="Circuit table / authored data / upstream physics";objects.clear();
 auto background=group("background");auto bg=bitmap(background,600,416,0,0,false);
 memset(bg->IndexedBmpPtr,10,bg->IndexedStride*bg->Height);
 auto palette=reinterpret_cast<ColorRgba*>(new char[1024]{});palette[10]=ColorRgba(23,27,37,255);palette[11]=ColorRgba(158,206,106,255);palette[12]=ColorRgba(230,237,243,255);palette[13]=ColorRgba(88,111,139,255);palette[14]=ColorRgba(244,184,96,255);
 auto pe=new EntryData(FieldTypes::Palette,(char*)palette);pe->FieldSize=1024;background->AddEntry(pe);
 // Perspective matrix with a flat playfield and exactly 10 pixels per world unit.
 auto camera=group("camera_info");
 floats(camera,{1,0,0,0, 0,1,0,0, 0,0,-1,100, 1000,0,40});
 auto table=group("table");auto board=bitmap(table,390,416,0,0);
 memset(board->IndexedBmpPtr,10,board->IndexedStride*board->Height);
 floats(table,{600,5,-22,-21,20,-21,20,23,-22,23,-22,-21});
 floats(table,{700,200,208});floats(table,{701,.08f});floats(table,{305,25,.32f,1.5707963f});
 auto ball=group("ball");floats(ball,{500,.45f});floats(ball,{501,0,0,.45f});auto ballbmp=bitmap(ball,11,11,0,0,false);circle(ballbmp,5,5,5,12);circle(ballbmp,3,3,1,14);
 auto digits=(int)data->Groups.size();
 const int masks[]={63,6,91,79,102,109,125,7,127,111};
 for(int d=0;d<10;d++){auto g=group(nullptr);auto b=bitmap(g,12,20,0,0,false);int m=masks[d];
  if(m&1)line(b,3,2,8,2,12,1);if(m&2)line(b,9,3,9,8,12,1);if(m&4)line(b,9,11,9,16,12,1);if(m&8)line(b,3,18,8,18,12,1);if(m&16)line(b,2,11,2,16,12,1);if(m&32)line(b,2,3,2,8,12,1);if(m&64)line(b,3,10,8,10,12,1);
 }
 auto score=group("score1");shorts(score,{(int16_t)digits,415,70,165,20});
 auto count=group("ballcount1");shorts(count,{(int16_t)digits,415,110,30,20});
 auto player=group("player_number1");shorts(player,{(int16_t)digits,550,110,25,20});
 auto info=group("info_text_box");shorts(info,{1500,405,155,185,100,0,0,0,0});object(info,1033);
 auto mission=group("mission_text_box");shorts(mission,{1500,405,270,185,130,0,0,0,0});object(mission,1033);
 auto mat=group(nullptr,300);floats(mat,{301,.95f,302,.8f});
 auto kick=group(nullptr,400);floats(kick,{401,1,402,28});
 auto targetContact=group(nullptr,400);floats(targetContact,{401,1,402,0});
 // The collision coordinates below are measured against assets/circuit/table.png.
 // The custom view uses the same 25 pixels/world-unit mapping.
 auto rail=[&](const char* name,float x,float y,float xx,float yy,int mask=0,int kicker=-1){
  auto g=group(name);floats(g,{600,2,wx(xx),wy(yy),wx(x),wy(y)});
  if(kicker>=0)shorts(g,{300,(int16_t)mat->GroupId,400,(int16_t)kicker,602,(int16_t)mask});
  else shorts(g,{300,(int16_t)mat->GroupId,602,(int16_t)mask});object(g,1000);return g;
 };
 // A capsule chain has two collision faces and a round cap at every
 // endpoint/joint. This is shared by cabinet, lane, obstacle and ramp walls.
 auto cap=[&](const char* name,vector2 point,int layer){
  auto g=group(name);floats(g,{600,1,wx(point.X),wy(point.Y),0});
  shorts(g,{300,(int16_t)mat->GroupId,602,(int16_t)layer});object(g,1000);
 };
 auto boundary=[&](const char* name,const std::vector<vector2>& pts,int layer){
  for(size_t i=1;i<pts.size();++i){
   rail(name,pts[i-1].X,pts[i-1].Y,pts[i].X,pts[i].Y,layer);
   rail(name,pts[i].X,pts[i].Y,pts[i-1].X,pts[i-1].Y,layer);
  }
  for(auto point:pts)cap(name,point,layer);
 };
 for(const auto& wall:CircuitGeometry::Walls())for(int layer=0;layer<2;layer++)if(wall.layers&(1<<layer))boundary(wall.name,wall.points,layer);
 // An intentional one-way gate at the orbit EXIT, not across its floor:
 // launches pass left into play; field balls cannot re-enter the shooter.
 rail("shooter_gate",655,38,660,106);
 auto slingKick=group(nullptr,400);floats(slingKick,{401,1,402,20});
 rail("sling_left",359,759,298,574,0,slingKick->GroupId);
 rail("sling_right",767,583,701,764,0,slingKick->GroupId);
 // Stand-up targets rebound passively through the upstream wall response.
 for(int i=0;i<4;i++){
  std::string n="target"+std::to_string(i);
  rail(n.c_str(),500+i*28,144,525+i*28,144,0,targetContact->GroupId);
 }
 for(int i=0;i<4;i++){
  std::string n="module"+std::to_string(i);
  rail(n.c_str(),744-i*6,304+i*25,738-i*6,324+i*25,0,targetContact->GroupId);
 }
 auto drain=group("drain");floats(drain,{600,2,wx(60),wy(1030),wx(1000),wy(1030)});floats(drain,{407,.8f});shorts(drain,{602,0,602,1});object(drain,1007);
 const float launcherFloor=CircuitGeometry::LauncherY,launcherLeft=CircuitGeometry::LauncherLeft,launcherRight=CircuitGeometry::LauncherRight;
 for(int layer=0;layer<2;layer++){cap("launcher_corner",{launcherLeft,launcherFloor},layer);cap("launcher_corner",{launcherRight,launcherFloor},layer);}
 auto plunger=group("plunger");floats(plunger,{600,2,wx(launcherLeft),wy(launcherFloor),wx(launcherRight),wy(launcherFloor)});floats(plunger,{601,wx(CircuitGeometry::LauncherX),wy(launcherFloor-32)});object(plunger,1001);
 bitmap(plunger,45,12,0,0);
 for(int side=0;side<2;side++){
  float origin=side?720:340,tip=side?598:462;
  auto g=group(side?"flipper_r":"flipper_l");shorts(g,{100,9,300,(int16_t)mat->GroupId});
  floats(g,{800,wx(origin),wy(888),.7f});floats(g,{801,wx(tip),wy(943),.42f});floats(g,{802,wx(tip),wy(820),.42f});
  floats(g,{803,1});floats(g,{804,.055f});floats(g,{805,.095f});object(g,side?1004:1003);
  for(int f=0;f<9;f++){auto state=f?group(nullptr,201):g;bitmap(state,1,1,0,0);}
 }
 for(int i=0;i<4;i++){const auto& bumper=CircuitGeometry::Bumpers()[i];std::string name="bumper"+std::to_string(i);auto g=group(name.c_str());shorts(g,{100,2,300,(int16_t)mat->GroupId,400,(int16_t)kick->GroupId});floats(g,{600,1,wx(bumper.x),wy(bumper.y),bumper.radius/25});floats(g,{407,.12f});object(g,1005);
  for(int f=0;f<2;f++){auto state=f?group(nullptr,201):g;bitmap(state,1,1,0,0);}
 }
 auto sensor=[&](const char* name,float x,float y,float xx,float yy,int mask=0){
  auto g=group(name);floats(g,{600,2,wx(x),wy(y),wx(xx),wy(yy)});shorts(g,{602,(int16_t)mask});object(g,1024);
 };
 sensor("orbit",770,110,840,150);sensor("orbit_back",840,150,770,110);
 sensor("return",220,735,265,765);sensor("return_back",265,765,220,735);
 // Elevated ramp: a triangulated upstream TRamp surface follows the artwork.
 // Ground and ramp rails use separate collision masks; TRamp switches them at its portals.
 std::vector<vector2> left,right;CircuitGeometry::RampSides(left,right);
 for(const auto& wall:CircuitGeometry::RampWalls())for(int layer=0;layer<2;layer++)if(wall.layers&(1<<layer))boundary(wall.name,wall.points,layer);
 // Raised balls may leave the upper portal; ground balls cannot enter it.
 rail("ramp_exit_gate",left.back().X,left.back().Y,right.back().X,right.back().Y);
 auto ramp=group("ramp");shorts(ramp,{602,1});floats(ramp,{701,.04f});floats(ramp,{1305,1});
 std::vector<float> planes={1300,float((left.size()-1)*2)};
 auto triangle=[&](vector2 a,vector2 b,vector2 c){
  // Rising deck z = (405 - image_y) * .004: the upper bridge
  // clears a ground ball by y=120, and is continuous at the entry.
  planes.insert(planes.end(),{0,-.10f,-.38f,wx(a.X),wy(a.Y),wx(b.X),wy(b.Y),wx(c.X),wy(c.Y),.12f,1.5707963f,0,0});
 };
 for(size_t i=0;i+1<left.size();++i){triangle(left[i],right[i],right[i+1]);triangle(left[i],right[i+1],left[i+1]);}
 auto pe2=new EntryData();pe2->EntryType=FieldTypes::FloatArray;pe2->FieldSize=planes.size()*sizeof(float);pe2->Buffer=new char[pe2->FieldSize];memcpy(pe2->Buffer,planes.data(),pe2->FieldSize);ramp->AddEntry(pe2);
 floats(ramp,{1301,0,1,0,wx(left.front().X),wy(left.front().Y),wx(right.front().X),wy(right.front().Y),0});
 floats(ramp,{1302,0,0,0,wx(right.back().X),wy(right.back().Y),wx(left.back().X),wy(left.back().Y),0});
 floats(ramp,{1303,1,0,wx(left[5].X),wy(left[5].Y),wx(right[5].X),wy(right[5].Y)});object(ramp,1021);
 sensor("ramp_score",310,30,310,100,1);
 sensor("ramp_score_back",310,100,310,30,1);
 sensor("ramp_exit",left.back().X,left.back().Y,right.back().X,right.back().Y,1);
 auto tableObjects=group("table_objects");
 auto e=new EntryData();e->EntryType=FieldTypes::ShortArray;e->FieldSize=(objects.size()+1)*2;e->Buffer=new char[e->FieldSize];((int16_t*)e->Buffer)[0]=1025;memcpy(e->Buffer+2,objects.data(),objects.size()*2);tableObjects->AddEntry(e);
 // Finalize only authored groups; do not import the embedded original bitmap font.
 for(auto g:data->Groups){
  g->FinalizeGroup();
  auto b=g->GetBitmap(0);auto z=g->GetZMap(0);
  if(b&&z)for(int y=0;y<b->Height;y++)for(int x=0;x<b->Width;x++)if(!b->IndexedBmpPtr[(b->Height-1-y)*b->IndexedStride+x])z->ZPtr1[y*z->Stride+x]=65535;
 }
 return data;
}
namespace {
bool inside(const std::vector<vector2>& polygon,float x,float y){
 bool result=false;
 for(size_t i=0,j=polygon.size()-1;i<polygon.size();j=i++){
  const auto a=polygon[i],b=polygon[j];
  if((a.Y>y)!=(b.Y>y) && x<(b.X-a.X)*(y-a.Y)/(b.Y-a.Y)+a.X)result=!result;
 }
 return result;
}
}
float pathDistance(const std::vector<vector2>& path,float x,float y){
 float result=1e9f;
 for(size_t i=1;i<path.size();++i){
  auto a=path[i-1],b=path[i];float dx=b.X-a.X,dy=b.Y-a.Y;
  float t=std::max(0.f,std::min(1.f,((x-a.X)*dx+(y-a.Y)*dy)/(dx*dx+dy*dy)));
  result=std::min(result,std::hypot(x-a.X-t*dx,y-a.Y-t*dy));
 }
 return result;
}
const char* InvalidRegion(const TBall* ball){
 if(!ball->ActiveFlag)return nullptr;
 const float x=540+25*ball->Position.X,y=500+25*ball->Position.Y;
 if(!inside(CircuitGeometry::Walls().front().points,x,y))return "cabinet";
 if(ball->CollisionMask==2){
  std::vector<vector2> a,b;CircuitGeometry::RampSides(a,b);
  a.insert(a.end(),b.rbegin(),b.rend());
  if(!inside(a,x,y))return "ramp footprint";
 }
 for(const auto& wall:CircuitGeometry::Walls())if(wall.layers&ball->CollisionMask){
  if(wall.solid && inside(wall.points,x,y))return wall.name;
  if(pathDistance(wall.points,x,y)<ball->Radius*25-.3f)return wall.name;
 }
 for(const auto& wall:CircuitGeometry::RampWalls())if(wall.layers&ball->CollisionMask)
  if(pathDistance(wall.points,x,y)<ball->Radius*25-.3f)return "inside tube rail";
 if(ball->CollisionMask&1){
  static const auto exitPortal=[] {
   std::vector<vector2> left,right;CircuitGeometry::RampSides(left,right);
   return std::vector<vector2>{left.back(),right.back()};
  }();
  // TRamp changes to ground exactly on the exit edge; allow only the same
  // subpixel contact tolerance used by the rail checks, not a ball-wide hole.
  if(pathDistance(exitPortal,x,y)>.3f)
   for(const auto& body:CircuitGeometry::GroundRampBodies())
    if(inside(body,x,y))return "inside ground tube body";
  for(const auto& bumper:CircuitGeometry::Bumpers())
   if(std::hypot(x-bumper.x,y-bumper.y)<bumper.radius+ball->Radius*25-.3f)return "inside bumper";
 }
 // The launch floor is not a drain: its divider runs all the way to the apron.
 if(y>CircuitGeometry::LauncherY+1 && x>895+(y-667)*15/378)return "below launcher floor";
 return nullptr;
}
bool AuditGeometry(){
 auto table=pb::MainTable;auto ball=table->BallList.front();
 unsigned checked=0;
 std::vector<CircuitGeometry::Wall> walls=CircuitGeometry::Walls();
 std::vector<vector2> a,b;CircuitGeometry::RampSides(a,b);
 for(const auto& wall:CircuitGeometry::RampWalls())walls.push_back(wall);
 for(const auto& wall:walls)for(size_t i=1;i<wall.points.size();++i){
  auto a=wall.points[i-1],b=wall.points[i];
  float dx=b.X-a.X,dy=b.Y-a.Y,len=std::hypot(dx,dy);
  for(float t:{0.f,.02f,.25f,.5f,.75f,.98f,1.f})for(int sign:{-1,1})for(int layer:{1,2}){
   if(!(wall.layers&layer))continue;
   vector2 n={-dy/len*sign,dx/len*sign};
   ray_type ray{};ray.Origin={wx(a.X+(b.X-a.X)*t+n.X*20),wy(a.Y+(b.Y-a.Y)*t+n.Y*20)};
   ray.Direction={-n.X,-n.Y};ray.MaxDistance=20.f/25;ray.CollisionMask=layer;
   float nearest=1e9f;
   // Inspect installed upstream colliders, not a second geometry algorithm.
   for(auto component:table->ComponentList)if(component->GroupName && std::string(component->GroupName)==wall.name){
    auto collision=dynamic_cast<TCollisionComponent*>(component);if(!collision)continue;
    for(auto edge:collision->EdgeList)if(edge->CollisionGroup&layer)nearest=std::min(nearest,edge->FindCollisionDistance(ray));
   }
   if(nearest>ray.MaxDistance){fprintf(stderr,"WALL_GAP %s segment=%zu t=%.2f side=%d layer=%d\n",wall.name,i,t,sign,layer);return false;}
   // Also require the upstream spatial grid to expose a collision on this ray.
   ball->EdgeCollisionCount=0;TEdgeSegment* edge=nullptr;
   if(TTableLayer::edge_manager->FindCollisionDistance(&ray,ball,&edge)>ray.MaxDistance){fprintf(stderr,"GRID_GAP %s segment=%zu\n",wall.name,i);return false;}
   ++checked;
  }
 }
 for(size_t i=1;i<a.size();++i){
  vector2 from={(a[i-1].X+b[i-1].X)/2,(a[i-1].Y+b[i-1].Y)/2};
  vector2 to={(a[i].X+b[i].X)/2,(a[i].Y+b[i].Y)/2};
  float dx=to.X-from.X,dy=to.Y-from.Y,len=std::hypot(dx,dy);
  ray_type ray{};ray.Origin={wx(from.X),wy(from.Y)};ray.Direction={dx/len,dy/len};ray.MaxDistance=len/25;ray.CollisionMask=2;
  for(auto component:table->ComponentList)if(component->GroupName && std::string(component->GroupName).find("ramp_rail_")==0){
   auto collision=dynamic_cast<TCollisionComponent*>(component);if(!collision)continue;
   for(auto edge:collision->EdgeList)if((edge->CollisionGroup&2) && edge->FindCollisionDistance(ray)<=ray.MaxDistance){
    fprintf(stderr,"RAMP_ROUTE_PINCH segment=%zu x=%.2f y=%.2f\n",i,from.X,from.Y);return false;
   }
  }
 }
 // Independent artwork landmarks catch missing backs, wrongly oriented
 // scoring faces, a closed drain, and a gate that blocks outgoing launches.
 struct Landmark {float x,y,xx,yy;const char* expected;};
 const Landmark landmarks[]={
  {250,680,310,680,"sling_body_left"},{815,680,740,680,"sling_body_right"},
  {840,340,765,340,"module_bank"},{504,180,504,150,"target0"},
  {700,315,780,315,"module0"},{928,825,928,890,"plunger"},{948,825,948,890,"plunger"},
  {630,80,685,80,"shooter_gate"},{685,80,630,80,nullptr},
  {540,980,540,1040,"drain"},{374,435,374,390,"ramp"},
  {480,160,438,124,"target_bank"}
 };
 for(const auto& test:landmarks){
  float dx=test.xx-test.x,dy=test.yy-test.y,len=std::hypot(dx,dy);
  ray_type ray{};ray.Origin={wx(test.x),wy(test.y)};ray.Direction={dx/len,dy/len};ray.MaxDistance=len/25;ray.CollisionMask=1;
  ball->EdgeCollisionCount=0;TEdgeSegment* edge=nullptr;
  float distance=TTableLayer::edge_manager->FindCollisionDistance(&ray,ball,&edge);
  const char* found=distance<=ray.MaxDistance && edge ? edge->CollisionComponent->GroupName : nullptr;
  if((test.expected==nullptr)!=(found==nullptr) || (test.expected && found && strcmp(test.expected,found))){
   fprintf(stderr,"LANDMARK_FAIL %.0f,%.0f -> %.0f,%.0f expected=%s found=%s\n",test.x,test.y,test.xx,test.yy,test.expected?test.expected:"open",found?found:"open");return false;
  }
 }
 // These routes describe playable passages independently of wall construction.
 // Sweep the ball against every installed component, including endpoint caps,
 // rather than checking only a wall's own centreline or global map bounds.
 unsigned routeSegments=0;
 const auto savedPosition=ball->Position;const auto savedActive=ball->ActiveFlag;const auto savedMask=ball->CollisionMask;
 ball->ActiveFlag=1;ball->CollisionMask=1;
 for(const auto& route:CircuitRouteFixtures::Routes())for(size_t i=1;i<route.points.size();++i){
  auto a=route.points[i-1],b=route.points[i];
  float dx=b.X-a.X,dy=b.Y-a.Y,len=std::hypot(dx,dy);
  for(float t=0;t<=len;t+=1){
   ball->Position.X=wx(a.X+dx*t/len);ball->Position.Y=wy(a.Y+dy*t/len);
   if(const char* region=InvalidRegion(ball)){fprintf(stderr,"ROUTE_REGION %s %s\n",route.name,region);return false;}
  }
  ray_type ray{};ray.Origin={wx(a.X),wy(a.Y)};ray.Direction={dx/len,dy/len};ray.MaxDistance=len/25;ray.CollisionMask=1;
  for(auto component:table->ComponentList){
   if(dynamic_cast<TTripwire*>(component)||dynamic_cast<TDrain*>(component))continue;
   auto collision=dynamic_cast<TCollisionComponent*>(component);if(!collision)continue;
   for(auto edge:collision->EdgeList)if((edge->CollisionGroup&1) && edge->FindCollisionDistance(ray)<=ray.MaxDistance){
    fprintf(stderr,"ROUTE_BLOCKED %s segment=%zu at=%.0f,%.0f obstacle=%s\n",route.name,i,a.X,a.Y,component->GroupName?component->GroupName:"unnamed");return false;
   }
  }
  ++routeSegments;
 }
 ball->Position=savedPosition;ball->ActiveFlag=savedActive;ball->CollisionMask=savedMask;
 printf("PLAYABLE_ROUTES %zu routes, %u installed-collider sweeps passed\n",CircuitRouteFixtures::Routes().size(),routeSegments);
 // Audit every bumper around its full circumference, not only one hit face.
 unsigned bumperProbes=0;
 for(size_t i=0;i<CircuitGeometry::Bumpers().size();++i){
  const auto& b=CircuitGeometry::Bumpers()[i];std::string name="bumper"+std::to_string(i);
  std::vector<vector2> outline;
  for(int step=0;step<64;++step){
   float angle=step*6.2831853f/64,dx=std::cos(angle),dy=std::sin(angle);
   outline.push_back({b.x+dx*b.radius,b.y+dy*b.radius});
   ray_type ray{};ray.Origin={wx(b.x+dx*(b.radius+20)),wy(b.y+dy*(b.radius+20))};ray.Direction={-dx,-dy};ray.MaxDistance=20.f/25;ray.CollisionMask=1;
   float nearest=1e9f;
   for(auto component:table->ComponentList)if(component->GroupName && name==component->GroupName){
    auto collision=dynamic_cast<TCollisionComponent*>(component);
    for(auto edge:collision->EdgeList)if(edge->CollisionGroup&1)nearest=std::min(nearest,edge->FindCollisionDistance(ray));
   }
   if(nearest>ray.MaxDistance){fprintf(stderr,"BUMPER_GAP %s angle=%d\n",name.c_str(),step);return false;}
   ++bumperProbes;
  }
  outline.push_back(outline.front());
  static const char* names[]={"bumper0","bumper1","bumper2","bumper3"};
  walls.push_back({names[i],outline,true,1});
 }
 printf("BUMPER_BOUNDARIES %u circumference probes passed\n",bumperProbes);
 if(const char* path=getenv("OMARCHY_TEST_MAP")){
  FILE* out=fopen(path,"w");if(!out)return false;
  fprintf(out,"[\n");
  for(size_t i=0;i<walls.size();++i){const auto& w=walls[i];
   fprintf(out,"%s{\"name\":\"%s\",\"solid\":%s,\"layers\":%d,\"points\":[",i?",\n":"",w.name,w.solid?"true":"false",w.layers);
   for(size_t j=0;j<w.points.size();++j)fprintf(out,"%s[%.3f,%.3f]",j?",":"",w.points[j].X,w.points[j].Y);
   fprintf(out,"]}");
  }
  fprintf(out,"\n]\n");fclose(out);
 }
 printf("GEOMETRY_AUDIT %u directional wall/grid probes, %zu landmarks and ramp route passed\n",checked,sizeof(landmarks)/sizeof(landmarks[0]));return true;
}
unsigned Progress(){return hits%12;}
unsigned Targets(){return targetMask;}
unsigned Orbits(){return orbitCount;}
unsigned Ramps(){return rampCount;}
unsigned Circuits(){return circuits;}
bool GameOver(){return over;}
const char* Status(){
 if(over)return notice.c_str();
 if(pb::MainTable->TiltLockFlag)return "TILT  FLIPPERS LOCKED";
 if(nudge::nudge_count>.5f)return "DANGER  NUDGE LESS";
 return pb::time_now<noticeUntil?notice.c_str():"LIGHT THE CIRCUIT";
}
float Flash(const char* name){auto i=flashes.find(name);return i==flashes.end()?0:std::max(0.f,1-(pb::time_now-i->second)/.25f);}
void ComponentEvent(MessageCode code,TPinballComponent* c){
 auto t=c->PinballTable;if(!t)return;
 if(code==MessageCode::ControlCollision&&!t->TiltLockFlag&&c->GroupName){
  std::string name=c->GroupName;
  if(name=="drain"){announce("BALL DRAINED");return;}
  if(debounce.count(name)&&pb::time_now-debounce[name]<.15f)return;
  debounce[name]=pb::time_now;flashes[name]=pb::time_now;
  if(dynamic_cast<TBumper*>(c)){sound();t->AddScore(100);if(++hits%12==0){++circuits;t->AddScore(2500);announce("CIRCUIT +2500");}}
  else if(name.size()==7 && name.back()>='0' && name.back()<='3' && (name.compare(0,6,"target")==0||name.compare(0,6,"module")==0)){unsigned bit=unsigned(name.back()-'0')+(name[0]=='m'?4:0);targetMask|=1u<<bit;t->AddScore(250);sound();announce("MODULE +250");if(targetMask==255){t->AddScore(5000);targetMask=0;announce("SYSTEM ONLINE +5000");}}
  else if(name=="orbit"||name=="orbit_back"){if(!debounce.count("orbit_award")||pb::time_now-debounce["orbit_award"]>2){debounce["orbit_award"]=pb::time_now;++orbitCount;t->AddScore(1000);announce("ORBIT +1000");sound();}}
  else if(name=="ramp_score"||name=="ramp_score_back"){if(!debounce.count("ramp_award")||pb::time_now-debounce["ramp_award"]>3){debounce["ramp_award"]=pb::time_now;++rampCount;t->AddScore(1500);announce("RAMP +1500");sound();}}
  else if(name=="sling_left"||name=="sling_right"){t->AddScore(25);sound();}
 }
 if(dynamic_cast<TDrain*>(c)&&code==MessageCode::ControlTimerExpired){
  t->ChangeBallCount(t->BallCount-1);
  if(t->BallCount>0){t->Message(MessageCode::ClearTiltLock,0);t->Plunger->Message(MessageCode::PlungerFeedBall,0);announce("HOLD SPACE TO LAUNCH");}
  else {over=true;notice="GAME OVER  F2 NEW";t->Message(MessageCode::GameOver,0);}
 }
}
void Shutdown(){if(effect){Mix_HaltChannel(-1);Mix_FreeChunk(effect);effect=nullptr;}tone.clear();}
void TableEvent(MessageCode code){
 if(code==MessageCode::StartGamePlayer1)announce("HOLD SPACE TO LAUNCH");
 if(code==MessageCode::NewGame){pb::MainTable->Plunger->PullbackDelay=.10f;pb::MainTable->Plunger->MinimumReleaseDelay=.75f;hits=targetMask=orbitCount=rampCount=circuits=0;over=false;debounce.clear();flashes.clear();announce("HOLD SPACE TO LAUNCH");}
}
}
