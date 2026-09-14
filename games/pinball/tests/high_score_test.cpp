// Exercise the real score dialog and ImGui input; isolate the settings store.
#include "pch.h"
#include "high_score.h"
#include "ArcadeTextInput.h"
#include "options.h"
#include "pb.h"
#include "score.h"
#include "translations.h"
#include <stdexcept>

static std::map<std::string, std::string> testSettings;
const std::string& options::GetSetting(const std::string& key, const std::string& fallback) {
    auto found=testSettings.find(key);return found==testSettings.end()?fallback:found->second;
}
void options::SetSetting(const std::string& key,const std::string& value){testSettings[key]=value;}
int options::get_int(LPCSTR key,int fallback){return std::stoi(GetSetting(key,std::to_string(fallback)));}
void options::set_int(LPCSTR key,int value){SetSetting(key,std::to_string(value));}
LPCSTR pb::get_rc_string(Msg message){
    switch(message){
    case Msg::HIGHSCORES_Caption:return "High Scores";
    case Msg::HIGHSCORES_Rank:return "Rank";
    case Msg::HIGHSCORES_Name:return "Name";
    case Msg::HIGHSCORES_Score:return "Score";
    case Msg::GenericOk:return "OK";
    case Msg::GenericCancel:return "Cancel";
    case Msg::HIGHSCORES_Clear:return "Clear";
    default:return "Confirm clear";
    }
}
void score::string_format(int value,char* buffer){snprintf(buffer,36,"%d",value);}
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
static void frame(){ImGui::NewFrame();high_score::RenderHighScoreDialog();ImGui::Render();}
static void settle(){for(int i=0;i<3;++i)frame();}
static void key(ImGuiKey value){auto& io=ImGui::GetIO();io.AddKeyEvent(value,true);frame();io.AddKeyEvent(value,false);settle();}
static void replace(const char* text){
    auto& io=ImGui::GetIO();require(io.WantTextInput,"Name field did not receive keyboard focus");
    io.AddKeyEvent(ImGuiMod_Ctrl,true);key(ImGuiKey_A);io.AddKeyEvent(ImGuiMod_Ctrl,false);frame();
    io.AddInputCharactersUTF8(text);settle();
}
static void pasteHeld(const char* text){
    auto& io=ImGui::GetIO();require(io.WantTextInput,"Paste field did not receive keyboard focus");
    io.AddKeyEvent(ImGuiMod_Ctrl,true);key(ImGuiKey_A);
    ArcadeTextInput::Paste(text);settle();
    require(io.KeyCtrl,"Paste changed the held Ctrl modifier");
}
static void releaseCtrl(){ImGui::GetIO().AddKeyEvent(ImGuiMod_Ctrl,false);settle();}
static void cancel(){
    auto window=ImGui::FindWindowByName("High Scores");require(window&&window->Active,"Dialog missing");
    auto& style=ImGui::GetStyle();auto& io=ImGui::GetIO();
    float x=window->Pos.x+style.WindowPadding.x+ImGui::CalcTextSize("OK").x+2*style.FramePadding.x
        +style.ItemSpacing.x+ImGui::CalcTextSize("Cancel").x/2+style.FramePadding.x;
    float y=window->Pos.y+window->Size.y-style.WindowPadding.y-ImGui::GetFontSize()/2-style.FramePadding.y;
    io.AddMousePosEvent(x,y);frame();io.AddMouseButtonEvent(0,true);frame();io.AddMouseButtonEvent(0,false);settle();
    require(GImGui->OpenPopupStack.empty(),"Cancel did not close the dialog");
}
int main(){
    ImGui::CreateContext();auto& io=ImGui::GetIO();io.IniFilename=nullptr;io.DisplaySize=ImVec2(1152,790);
    io.DeltaTime=1.f/60;unsigned char* pixels;int width,height;io.Fonts->GetTexDataAsRGBA32(&pixels,&width,&height);
    high_score::read();strcpy(high_score::highscore_table[0].Name,"Previous");high_score::highscore_table[0].Score=1000;
    high_score::write();
    high_score_entry entry{};strcpy(entry.Entry.Name,"Player 1");entry.Entry.Score=2000;entry.Position=-1;
    high_score::show_and_set_high_score_dialog(entry);settle();pasteHeld("Riél O'Neil");releaseCtrl();key(ImGuiKey_Enter);
    require(testSettings["0.Name"]=="Riél O'Neil","New name was not saved");high_score::read();
    require(high_score::highscore_table[0].Score==2000&&high_score::highscore_table[1].Score==1000,"Scores changed");
    require(std::string(high_score::highscore_table[0].Name)=="Riél O'Neil","Checksum reload lost the new name");
    high_score::show_high_score_dialog();settle();replace("Updated 9!");key(ImGuiKey_Backspace);key(ImGuiKey_Enter);
    require(testSettings["0.Name"]=="Updated 9","Editing or Backspace failed");high_score::read();
    require(high_score::highscore_table[0].Score==2000&&high_score::highscore_table[1].Score==1000,"Rename changed scores");
    high_score::show_high_score_dialog();settle();pasteHeld("Zoë O'Neil 7!");
    auto state=ImGui::GetInputTextState(ImGui::GetActiveID());
    require(state&&std::string(state->TextA.Data)=="Zoë O'Neil 7!","Ctrl-held paste lost the selection replacement");
    key(ImGuiKey_Z);
    require(std::string(state->TextA.Data)=="Updated 9","Paste was not one undoable edit");
    key(ImGuiKey_Y);
    require(std::string(state->TextA.Data)=="Zoë O'Neil 7!","Paste redo failed");
    releaseCtrl();key(ImGuiKey_Enter);high_score::read();
    require(std::string(high_score::highscore_table[0].Name)=="Zoë O'Neil 7!","Pasted UTF-8 name failed checksum reload");
    high_score::show_high_score_dialog();settle();pasteHeld("Discard this");releaseCtrl();cancel();
    require(testSettings["0.Name"]=="Zoë O'Neil 7!"&&std::string(high_score::highscore_table[0].Name)=="Zoë O'Neil 7!","Cancel committed pasted edits");
    high_score::show_high_score_dialog();settle();pasteHeld("123456789012345678901234567890é!");releaseCtrl();key(ImGuiKey_Enter);
    require(testSettings["0.Name"]=="123456789012345678901234567890","Paste split a UTF-8 character at the name limit");
    high_score::show_high_score_dialog();settle();replace("1234567890123456789012345678901234567890");key(ImGuiKey_Enter);
    require(testSettings["0.Name"].size()==31,"Name exceeded the saved format");high_score::read();
    require(high_score::highscore_table[0].Score==2000,"Long name corrupted checksum");
    ImGui::DestroyContext();puts("PASS new score names, Ctrl-held paste, UTF-8 bounds, undo/redo, rename, Backspace, Cancel and checksum reload");
}
