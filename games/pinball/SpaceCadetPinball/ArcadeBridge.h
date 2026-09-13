#pragma once
struct SDL_Renderer;
namespace ArcadeBridge {
bool Enabled();
void Init();
void Pump();
bool BeginFrame(SDL_Renderer* renderer);
void Shutdown();
void Present(SDL_Renderer* renderer);
}
