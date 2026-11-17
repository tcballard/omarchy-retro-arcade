#pragma once
class DatFile;
class TBall;
class TPinballComponent;
enum class MessageCode;
namespace OmarchyTable {
extern bool Enabled;
DatFile* Build();
void ComponentEvent(MessageCode code, TPinballComponent* component);
void TableEvent(MessageCode code);
void Shutdown();
// Read-only real-engine geometry checks used by the authored-table test runner.
bool AuditGeometry();
const char* InvalidRegion(const TBall* ball);
// Ball search for the authored table: after a ball has rested in open play
// for several seconds, give it a small kick, as a real machine pulses its coils.
void BallSearch(TBall& ball, int restedMs);
unsigned BallSearches();
unsigned Progress();
unsigned Targets();
unsigned Orbits();
unsigned Ramps();
unsigned Circuits();
bool GameOver();
const char* Status();
float Flash(const char* name);
}
