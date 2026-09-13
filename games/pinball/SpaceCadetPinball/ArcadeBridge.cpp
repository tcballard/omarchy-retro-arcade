// Local, bounded pixel/input transport for the single-window Arcade host.
#include "pch.h"
#include "ArcadeBridge.h"
#include "winmain.h"
#include "pb.h"
#include "options.h"
#include <unistd.h>
#include <fcntl.h>
#include <cerrno>
#include <csignal>
#include <vector>
#include <string>
namespace ArcadeBridge {
static int output=-1;
static std::string pending;
static Uint32 last=0;
static int width=1152,height=790;
static SDL_Texture* surface=nullptr;
static bool resizePending=false;
static bool validSize(int w,int h){return w>=64&&h>=64&&w<=1600&&h<=1600&&w*h<=1000000;}
bool Enabled(){return output>=0;}
void Init(){
    fflush(stdout);output=dup(STDOUT_FILENO);dup2(STDERR_FILENO,STDOUT_FILENO);
    fcntl(STDIN_FILENO,F_SETFL,fcntl(STDIN_FILENO,F_GETFL)|O_NONBLOCK);
    signal(SIGPIPE,SIG_IGN);
    // Keep the worker windowless while allowing GPU rendering. SDL tries dummy
    // when offscreen is unavailable; winmain retains its software renderer fallback.
    SDL_setenv("SDL_VIDEODRIVER","offscreen,dummy",1);
}
static void quit(){SDL_Event e{SDL_QUIT};winmain::event_handler(&e);}
static void command(const std::string& line){
    int a=0,b=0,c=0;
    SDL_Event e{};
    if(line=="quit"){quit();return;}
    if(sscanf(line.c_str(),"resize %d %d",&a,&b)==2){
        if(validSize(a,b)&&(a!=width||b!=height)){width=a;height=b;resizePending=true;}
        return;
    }
    if(line=="blur"){
        pb::loose_focus();winmain::pause(false);return;
    }
    if(sscanf(line.c_str(),"key %d %d %d",&a,&b,&c)==3){
        e.type=b?SDL_KEYDOWN:SDL_KEYUP;e.key.state=b?SDL_PRESSED:SDL_RELEASED;
        e.key.keysym.sym=a;e.key.keysym.scancode=SDL_GetScancodeFromKey(a);e.key.keysym.mod=c;
        SDL_SetModState(static_cast<SDL_Keymod>(c));
        // Fullscreen belongs to the Arcade host, never to the hidden SDL window.
        if(a==SDLK_F11)return;
        winmain::event_handler(&e);
    }else if(sscanf(line.c_str(),"mouse %d %d %d",&a,&b,&c)==3){
        auto& io=ImGui::GetIO();io.AddMousePosEvent(static_cast<float>(a),static_cast<float>(b));
        if(c>=0&&c<=1)io.AddMouseButtonEvent(0,c!=0);
    }else if(sscanf(line.c_str(),"wheel %d",&a)==1){ImGui::GetIO().MouseWheel+=static_cast<float>(a);}
}
void Pump(){
    if(!Enabled())return;
    char buffer[512];
    for(int i=0;i<16;++i){
        ssize_t n=read(STDIN_FILENO,buffer,sizeof(buffer));
        if(n==0){quit();return;}
        if(n<0){if(errno!=EAGAIN&&errno!=EWOULDBLOCK&&errno!=EINTR)quit();break;}
        pending.append(buffer,static_cast<size_t>(n));
        if(pending.size()>8192){quit();return;}
        size_t pos;while((pos=pending.find('\n'))!=std::string::npos){command(pending.substr(0,pos));pending.erase(0,pos+1);}
    }
}
static bool send(const void* bytes,size_t size){
    const char* p=static_cast<const char*>(bytes);
    while(size){ssize_t n=write(output,p,size);if(n<0&&errno==EINTR)continue;if(n<=0)return false;p+=n;size-=n;}
    return true;
}
// Render into a bounded texture rather than resizing an offscreen drawable:
// some drivers retain the original drawable extent after SDL_SetWindowSize.
bool BeginFrame(SDL_Renderer* renderer){
    if(!Enabled())return true;
    if(!surface||resizePending){
        SDL_SetRenderTarget(renderer,nullptr);
        if(surface)SDL_DestroyTexture(surface);
        surface=SDL_CreateTexture(renderer,SDL_PIXELFORMAT_RGBA8888,SDL_TEXTUREACCESS_TARGET,width,height);
        resizePending=false;
    }
    if(!surface||SDL_SetRenderTarget(renderer,surface)!=0){quit();return false;}
    auto& io=ImGui::GetIO();io.DisplaySize=ImVec2(width,height);io.DisplayFramebufferScale=ImVec2(1,1);io.DeltaTime=1.f/60;
    return true;
}
void Shutdown(){if(surface)SDL_DestroyTexture(surface);surface=nullptr;}
void Present(SDL_Renderer* renderer){
    if(!Enabled()||SDL_GetTicks()-last<16)return;
    last=SDL_GetTicks();
    static std::vector<unsigned char> pixels;
    pixels.resize(static_cast<size_t>(width)*height*4);
    if(SDL_RenderReadPixels(renderer,nullptr,SDL_PIXELFORMAT_RGBA32,pixels.data(),width*4)!=0){quit();return;}
    unsigned char header[]={ 'O','A','R','1',0,0,0,0,0,0,0,0 };
    for(int i=0;i<4;i++){header[4+i]=(width>>(8*i))&255;header[8+i]=(height>>(8*i))&255;}
    if(!send(header,sizeof(header))||!send(pixels.data(),pixels.size()))quit();
}
}
