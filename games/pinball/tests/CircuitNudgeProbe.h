#pragma once
#include "../SpaceCadetPinball/nudge.h"
#include "../SpaceCadetPinball/TFlipper.h"
#include "../SpaceCadetPinball/TFlipperEdge.h"

// Runs only in the opt-in, fixed-step authored-table test runner.
inline bool CheckCircuitNudge(int tick) {
 static int tiltTick=-1;
 auto t=pb::MainTable;auto b=t->BallList.front();
 auto released=[](){return !nudge::held_inputs&&!nudge::nudged_left&&!nudge::nudged_right&&!nudge::nudged_up;};
 if(tick==180){
  tiltTick=-1;
  b->Position={0,6,b->Radius};b->Direction={0,-1,0};b->Speed=10;
  struct Input {SDL_Keycode key;float dx;};
  for(auto input:{Input{SDLK_x,1},Input{SDLK_PERIOD,-1},Input{SDLK_UP,0}}){
   pb::InputDown({InputTypes::Keyboard,input.key});
   if(std::abs(b->Direction.X*b->Speed-input.dx)>.001f || std::abs(b->Direction.Y*b->Speed+9.5f)>.001f)return false;
   pb::InputUp({InputTypes::Keyboard,input.key});
   if(!released() || std::abs(b->Direction.X*b->Speed)>.001f || std::abs(b->Direction.Y*b->Speed+10)>.001f)return false;
  }
  pb::InputDown({InputTypes::Keyboard,SDLK_x});pb::InputDown({InputTypes::Keyboard,SDLK_PERIOD});
  pb::InputUp({InputTypes::Keyboard,SDLK_x});
  if(nudge::nudged_right || !nudge::nudged_left)return false;
  pb::InputUp({InputTypes::Keyboard,SDLK_PERIOD});if(!released())return false;
  pb::InputDown({InputTypes::Keyboard,SDLK_x});pb::pause_continue();
  if(!released())return false;
  pb::pause_continue();
  pb::InputDown({InputTypes::Keyboard,SDLK_PERIOD});pb::loose_focus();
  if(!released())return false;
  printf("NUDGE impulses/release/pause/focus PASS\n");
  nudge::nudge_count=0;pb::InputDown({InputTypes::Keyboard,SDLK_x});
 }
 if(tick>180 && t->TiltLockFlag && tiltTick<0){
  tiltTick=tick;if(tick<388 || tick>393)return false;
  printf("NUDGE tilt after %.3f seconds\n",(tick-180)/120.f);
 }
 if(tick>180 && tick<=384 && t->TiltLockFlag)return false;
 if(getenv("OMARCHY_NUDGE_RELEASE_EARLY") && tick>=360){
  if(tick==360)pb::InputUp({InputTypes::Keyboard,SDLK_x});
  if(t->TiltLockFlag)return false;
  if(tick==610)printf("NUDGE early release prevents delayed tilt PASS\n");
  return true;
 }
 if(tick==360){
  if(!nudge::held_inputs || nudge::nudged_right)return false;
  printf("NUDGE no premature tilt at 1.5 seconds PASS\n");
 }
 if(tick==300){
  if(t->TiltLockFlag || std::string(OmarchyTable::Status()).find("DANGER")==std::string::npos)return false;
  printf("NUDGE warning PASS\n");
 }
 if(tick==400){
  if(!t->TiltLockFlag || std::string(OmarchyTable::Status()).find("TILT")==std::string::npos)return false;
  auto previous=t->FlipperL->FlipperEdge->FlipperFlag;
  pb::InputDown({InputTypes::Keyboard,SDLK_a});
  if(t->FlipperL->FlipperEdge->FlipperFlag!=previous)return false;
  const int score=t->CurScore;bool found=false;
  for(auto c:t->ComponentList)if(c->GroupName && std::string(c->GroupName)=="bumper0"){
   OmarchyTable::ComponentEvent(MessageCode::ControlCollision,c);found=true;break;
  }
  if(!found || t->CurScore!=score)return false;
  pb::InputUp({InputTypes::Keyboard,SDLK_x});
  printf("NUDGE tilt disables flippers/scoring PASS\n");
  b->Position={0,(985.f-500)/25,b->Radius};b->Direction={0,1,0};b->Speed=15;
 }
 if(tick==610){
  if(t->TiltLockFlag || t->BallCount!=2 || !released())return false;
  printf("NUDGE drain restores next ball PASS\n");
 }
 return true;
}
