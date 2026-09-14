cmake_minimum_required(VERSION 3.16)
project(SpaceCadetPinball)

set(CMAKE_CXX_STANDARD 11)


set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/bin)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/CMakeModules")

# On Windows, set paths to SDL-devel packages here
if(WIN32)
   list(APPEND SDL2_PATH "${CMAKE_CURRENT_LIST_DIR}/Libs/SDL2")
   list(APPEND SDL2_MIXER_PATH "${CMAKE_CURRENT_LIST_DIR}/Libs/SDL2_mixer")
endif()

# Link mingw libs static
if(MINGW)
	set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -static")
endif()

if(APPLE)
    set(MACOSX_RPATH)
    set(CMAKE_BUILD_WITH_INSTALL_RPATH true)
    set(CMAKE_INSTALL_RPATH "@executable_path/../Frameworks")
    set(CMAKE_OSX_DEPLOYMENT_TARGET "10.11")
    set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64")
    list(APPEND SDL2_PATH "${CMAKE_CURRENT_LIST_DIR}/Libs")
    list(APPEND SDL2_MIXER_PATH "${CMAKE_CURRENT_LIST_DIR}/Libs")
endif()

# SDL2main is not needed
set(SDL2_BUILDING_LIBRARY ON)

find_package(SDL2 REQUIRED)
FIND_PACKAGE(SDL2_mixer REQUIRED)
find_path(SDL2_IMAGE_INCLUDE_DIR SDL_image.h PATH_SUFFIXES SDL2 REQUIRED)
find_library(SDL2_IMAGE_LIBRARY NAMES SDL2_image REQUIRED)

include_directories(${SDL2_INCLUDE_DIR} ${SDL2_MIXER_INCLUDE_DIR})
get_property(dirs DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR} PROPERTY INCLUDE_DIRECTORIES)
foreach(dir ${dirs})
  message(STATUS "Include dir='${dir}'")
endforeach()



set(SOURCE_FILES
        SpaceCadetPinball/ArcadeBridge.cpp
        SpaceCadetPinball/ArcadeTextInput.cpp
        SpaceCadetPinball/CircuitView.cpp
        SpaceCadetPinball/OmarchyTable.cpp
        SpaceCadetPinball/OmarchyTheme.cpp
        SpaceCadetPinball/ThemePalette.cpp
        SpaceCadetPinball/control.cpp
        SpaceCadetPinball/control.h
        SpaceCadetPinball/EmbeddedData.cpp
        SpaceCadetPinball/EmbeddedData.h
        SpaceCadetPinball/font_selection.cpp
        SpaceCadetPinball/font_selection.h
        SpaceCadetPinball/fullscrn.cpp
        SpaceCadetPinball/fullscrn.h
        SpaceCadetPinball/gdrv.cpp
        SpaceCadetPinball/gdrv.h
        SpaceCadetPinball/GroupData.cpp
        SpaceCadetPinball/GroupData.h
        SpaceCadetPinball/high_score.cpp
        SpaceCadetPinball/high_score.h
        SpaceCadetPinball/loader.cpp
        SpaceCadetPinball/loader.h
        SpaceCadetPinball/maths.cpp
        SpaceCadetPinball/maths.h
        SpaceCadetPinball/midi.cpp
        SpaceCadetPinball/midi.h
        SpaceCadetPinball/nudge.cpp
        SpaceCadetPinball/nudge.h
        SpaceCadetPinball/options.cpp
        SpaceCadetPinball/options.h
        SpaceCadetPinball/partman.cpp
        SpaceCadetPinball/partman.h
        SpaceCadetPinball/pb.cpp
        SpaceCadetPinball/pb.h
        SpaceCadetPinball/pch.h
        SpaceCadetPinball/proj.cpp
        SpaceCadetPinball/proj.h
        SpaceCadetPinball/render.cpp
        SpaceCadetPinball/render.h
        SpaceCadetPinball/score.cpp
        SpaceCadetPinball/score.h
        SpaceCadetPinball/Sound.cpp
        SpaceCadetPinball/Sound.h
        SpaceCadetPinball/SpaceCadetPinball.cpp
        SpaceCadetPinball/TBall.cpp
        SpaceCadetPinball/TBall.h
        SpaceCadetPinball/TBlocker.cpp
        SpaceCadetPinball/TBlocker.h
        SpaceCadetPinball/TBumper.cpp
        SpaceCadetPinball/TBumper.h
        SpaceCadetPinball/TCircle.cpp
        SpaceCadetPinball/TCircle.h
        SpaceCadetPinball/TCollisionComponent.cpp
        SpaceCadetPinball/TCollisionComponent.h
        SpaceCadetPinball/TComponentGroup.cpp
        SpaceCadetPinball/TComponentGroup.h
        SpaceCadetPinball/TDemo.cpp
        SpaceCadetPinball/TDemo.h
        SpaceCadetPinball/TDrain.cpp
        SpaceCadetPinball/TDrain.h
        SpaceCadetPinball/TEdgeBox.h
        SpaceCadetPinball/TEdgeManager.cpp
        SpaceCadetPinball/TEdgeManager.h
        SpaceCadetPinball/TEdgeSegment.cpp
        SpaceCadetPinball/TEdgeSegment.h
        SpaceCadetPinball/TFlagSpinner.cpp
        SpaceCadetPinball/TFlagSpinner.h
        SpaceCadetPinball/TFlipper.cpp
        SpaceCadetPinball/TFlipper.h
        SpaceCadetPinball/TFlipperEdge.cpp
        SpaceCadetPinball/TFlipperEdge.h
        SpaceCadetPinball/TGate.cpp
        SpaceCadetPinball/TGate.h
        SpaceCadetPinball/THole.cpp
        SpaceCadetPinball/THole.h
        SpaceCadetPinball/timer.cpp
        SpaceCadetPinball/timer.h
        SpaceCadetPinball/TKickback.cpp
        SpaceCadetPinball/TKickback.h
        SpaceCadetPinball/TKickout.cpp
        SpaceCadetPinball/TKickout.h
        SpaceCadetPinball/TLight.cpp
        SpaceCadetPinball/TLight.h
        SpaceCadetPinball/TLightBargraph.cpp
        SpaceCadetPinball/TLightBargraph.h
        SpaceCadetPinball/TLightGroup.cpp
        SpaceCadetPinball/TLightGroup.h
        SpaceCadetPinball/TLightRollover.cpp
        SpaceCadetPinball/TLightRollover.h
        SpaceCadetPinball/TLine.cpp
        SpaceCadetPinball/TLine.h
        SpaceCadetPinball/TOneway.cpp
        SpaceCadetPinball/TOneway.h
        SpaceCadetPinball/TPinballComponent.cpp
        SpaceCadetPinball/TPinballComponent.h
        SpaceCadetPinball/TPinballTable.cpp
        SpaceCadetPinball/TPinballTable.h
        SpaceCadetPinball/TPlunger.cpp
        SpaceCadetPinball/TPlunger.h
        SpaceCadetPinball/TPopupTarget.cpp
        SpaceCadetPinball/TPopupTarget.h
        SpaceCadetPinball/TRamp.cpp
        SpaceCadetPinball/TRamp.h
        SpaceCadetPinball/translations.cpp
        SpaceCadetPinball/translations.h
        SpaceCadetPinball/TRollover.cpp
        SpaceCadetPinball/TRollover.h
        SpaceCadetPinball/TSink.cpp
        SpaceCadetPinball/TSink.h
        SpaceCadetPinball/TSoloTarget.cpp
        SpaceCadetPinball/TSoloTarget.h
        SpaceCadetPinball/TSound.cpp
        SpaceCadetPinball/TSound.h
        SpaceCadetPinball/TTableLayer.cpp
        SpaceCadetPinball/TTableLayer.h
        SpaceCadetPinball/TTextBox.cpp
        SpaceCadetPinball/TTextBox.h
        SpaceCadetPinball/TTextBoxMessage.cpp
        SpaceCadetPinball/TTextBoxMessage.h
        SpaceCadetPinball/TTimer.cpp
        SpaceCadetPinball/TTimer.h
        SpaceCadetPinball/TTripwire.cpp
        SpaceCadetPinball/TTripwire.h
        SpaceCadetPinball/TWall.cpp
        SpaceCadetPinball/TWall.h
        SpaceCadetPinball/winmain.cpp
        SpaceCadetPinball/winmain.h
        SpaceCadetPinball/zdrv.cpp
        SpaceCadetPinball/zdrv.h
        SpaceCadetPinball/imconfig.h
        SpaceCadetPinball/imgui_internal.h
        SpaceCadetPinball/imgui.cpp
        SpaceCadetPinball/imgui.h
        SpaceCadetPinball/imgui_sdl.cpp
        SpaceCadetPinball/imgui_sdl.h
        SpaceCadetPinball/imgui_draw.cpp
        SpaceCadetPinball/imgui_widgets.cpp
        SpaceCadetPinball/imgui_tables.cpp
        SpaceCadetPinball/imgui_demo.cpp
        SpaceCadetPinball/imgui_impl_sdl.cpp
        SpaceCadetPinball/imgui_impl_sdl.h
        SpaceCadetPinball/imgui_impl_sdlrenderer.h
        SpaceCadetPinball/imgui_impl_sdlrenderer.cpp
        SpaceCadetPinball/imstb_textedit.h
        SpaceCadetPinball/imstb_rectpack.h
        SpaceCadetPinball/imstb_truetype.h
        SpaceCadetPinball/DebugOverlay.cpp
        SpaceCadetPinball/DebugOverlay.h
        )

# On Windows, include resource file with the icon
if(WIN32)
    set_source_files_properties(SpaceCadetPinball/SpaceCadetPinball.rc LANGUAGE RC)
    list(APPEND SOURCE_FILES SpaceCadetPinball/SpaceCadetPinball.rc)
endif(WIN32)

add_executable(SpaceCadetPinball ${SOURCE_FILES})
if(UNIX AND NOT APPLE)
    set_target_properties(SpaceCadetPinball PROPERTIES OUTPUT_NAME "omarchy-spacecadet-game")
endif()

# Skip pch on foreign code
set_source_files_properties(
        SpaceCadetPinball/imgui.cpp
        SpaceCadetPinball/imgui_sdl.cpp
        SpaceCadetPinball/imgui_draw.cpp
        SpaceCadetPinball/imgui_widgets.cpp
        SpaceCadetPinball/imgui_tables.cpp
        SpaceCadetPinball/imgui_demo.cpp
        SpaceCadetPinball/imgui_impl_sdl.cpp
        SpaceCadetPinball/imgui_impl_sdlrenderer.cpp
        PROPERTIES SKIP_PRECOMPILE_HEADERS 1
)

if(${CMAKE_VERSION} VERSION_GREATER "3.16.0" OR ${CMAKE_VERSION} VERSION_EQUAL "3.16.0")
    target_precompile_headers(SpaceCadetPinball
            PUBLIC
            SpaceCadetPinball/pch.h
            )
endif()

target_link_libraries(SpaceCadetPinball ${SDL2_LIBRARY} ${SDL2_MIXER_LIBRARY})

# On Windows, copy DLL to output
if(WIN32)
    list(GET SDL2_LIBRARY -1 SDL2_DLL_PATH)
    list(GET SDL2_MIXER_LIBRARY -1 SDL2_MIXER_DLL_PATH)
    get_filename_component(SDL2_DLL_PATH ${SDL2_DLL_PATH} DIRECTORY)
    get_filename_component(SDL2_MIXER_DLL_PATH ${SDL2_MIXER_DLL_PATH} DIRECTORY)
    if(MINGW)
        string(REGEX REPLACE "lib$" "bin" SDL2_DLL_PATH ${SDL2_DLL_PATH})
        string(REGEX REPLACE "lib$" "bin" SDL2_MIXER_DLL_PATH ${SDL2_MIXER_DLL_PATH})
    endif()
    message(STATUS "copy paths='${SDL2_DLL_PATH}' '${SDL2_MIXER_DLL_PATH}'") 
    add_custom_command(TARGET SpaceCadetPinball POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different "${SDL2_DLL_PATH}/SDL2.dll" $<TARGET_FILE_DIR:SpaceCadetPinball>
        COMMAND ${CMAKE_COMMAND} -E copy_if_different "${SDL2_MIXER_DLL_PATH}/SDL2_mixer.dll" $<TARGET_FILE_DIR:SpaceCadetPinball>
        )
endif()

if(UNIX AND NOT APPLE)
    include(GNUInstallDirs)
    install(TARGETS SpaceCadetPinball RUNTIME DESTINATION "${CMAKE_INSTALL_LIBEXECDIR}/omarchy-spacecadet")
    configure_file(Platform/Linux/omarchy-spacecadet.in omarchy-spacecadet @ONLY)
    install(PROGRAMS "${CMAKE_CURRENT_BINARY_DIR}/omarchy-spacecadet" DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES Platform/Linux/omarchy-spacecadet.desktop DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
    install(FILES LICENSE DESTINATION "${CMAKE_INSTALL_DATADIR}/licenses/omarchy-spacecadet")
endif()
include(CTest)
if(BUILD_TESTING)
    add_executable(theme-tests tests/theme_test.cpp SpaceCadetPinball/ThemePalette.cpp)
    target_include_directories(theme-tests PRIVATE SpaceCadetPinball)
    add_test(NAME theme-palette COMMAND theme-tests)
endif()

# Experimental standalone prototype; default play uses SpaceCadetPinball.
add_executable(OmarchyPinball
    native/NativeApp.cpp native/PinballModel.cpp
    SpaceCadetPinball/ThemePalette.cpp
    SpaceCadetPinball/imgui.cpp SpaceCadetPinball/imgui_draw.cpp
    SpaceCadetPinball/imgui_widgets.cpp SpaceCadetPinball/imgui_tables.cpp
    SpaceCadetPinball/imgui_impl_sdl.cpp SpaceCadetPinball/imgui_impl_sdlrenderer.cpp)
set_target_properties(OmarchyPinball PROPERTIES OUTPUT_NAME "omarchy-pinball" CXX_STANDARD 14)
target_include_directories(OmarchyPinball PRIVATE native SpaceCadetPinball)
target_link_libraries(OmarchyPinball ${SDL2_LIBRARY})
if(UNIX AND NOT APPLE)
    install(TARGETS OmarchyPinball RUNTIME DESTINATION "${CMAKE_INSTALL_LIBEXECDIR}/omarchy-spacecadet")
    install(FILES assets/brand/arcade-pinball.svg DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps" RENAME "omarchy-spacecadet.svg")
    install(FILES assets/brand/README.md DESTINATION "${CMAKE_INSTALL_DATADIR}/licenses/omarchy-spacecadet" RENAME "BRAND.md")
endif()
if(BUILD_TESTING)
    add_executable(pinball-tests tests/pinball_test.cpp native/PinballModel.cpp)
    target_include_directories(pinball-tests PRIVATE native)
    set_target_properties(pinball-tests PROPERTIES CXX_STANDARD 14)
    add_test(NAME original-table COMMAND pinball-tests)
endif()
if(BUILD_TESTING)
    add_executable(high-score-tests tests/high_score_test.cpp
        SpaceCadetPinball/ArcadeTextInput.cpp
        SpaceCadetPinball/high_score.cpp SpaceCadetPinball/imgui.cpp
        SpaceCadetPinball/imgui_draw.cpp SpaceCadetPinball/imgui_tables.cpp
        SpaceCadetPinball/imgui_widgets.cpp)
    target_include_directories(high-score-tests PRIVATE SpaceCadetPinball)
    add_test(NAME high-score-dialog COMMAND high-score-tests)
endif()
if(BUILD_TESTING)
    add_test(NAME theme-path COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/theme_path_test.py" $<TARGET_FILE:theme-tests>)
endif()

if(BUILD_TESTING)
    add_test(NAME authored-table-depth COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/table_depth_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME authored-table-boundaries COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/table_boundaries_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME authored-upstream-table COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/upstream_table_test.py" $<TARGET_FILE:SpaceCadetPinball>)
endif()

target_include_directories(SpaceCadetPinball PRIVATE ${SDL2_IMAGE_INCLUDE_DIR})
target_link_libraries(SpaceCadetPinball ${SDL2_IMAGE_LIBRARY})
target_compile_definitions(SpaceCadetPinball PRIVATE
 CIRCUIT_SOURCE_DIR="${CMAKE_CURRENT_SOURCE_DIR}/assets/circuit"
 CIRCUIT_INSTALL_DIR="${CMAKE_INSTALL_PREFIX}/share/omarchy-retro-arcade/circuit")
install(FILES assets/circuit/table.png DESTINATION "${CMAKE_INSTALL_DATADIR}/omarchy-spacecadet/circuit")

if(BUILD_TESTING)
    add_test(NAME authored-geometry-sweep COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/geometry_sweep_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    set_tests_properties(authored-geometry-sweep PROPERTIES TIMEOUT 480)
    add_test(NAME authored-rear-routes COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/rear_routes_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME authored-ramp-entry COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/ramp_entry_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME authored-bridge-layers COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/bridge_layers_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME authored-launcher-contact COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/launcher_contact_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME arcade-launcher-render COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/launcher_render_test.py" $<TARGET_FILE:SpaceCadetPinball> --software)
    add_test(NAME authored-passive-targets COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/passive_targets_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME arcade-bridge-render COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/bridge_render_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    add_test(NAME arcade-bridge-software COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/bridge_render_test.py" $<TARGET_FILE:SpaceCadetPinball> --software)
    set_tests_properties(arcade-bridge-render arcade-bridge-software PROPERTIES TIMEOUT 45)
    add_test(NAME arcade-bridge-resize COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/bridge_resize_test.py" $<TARGET_FILE:SpaceCadetPinball> --software)
    add_executable(circuit-timing-tests tests/circuit_timing_test.cpp)
    target_include_directories(circuit-timing-tests PRIVATE SpaceCadetPinball)
    target_compile_features(circuit-timing-tests PRIVATE cxx_std_11)
    add_test(NAME circuit-timing COMMAND circuit-timing-tests)
    add_test(NAME arcade-bridge-timing COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/bridge_timing_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    set_tests_properties(arcade-bridge-timing PROPERTIES TIMEOUT 30)
    add_test(NAME authored-nudge COMMAND python3 "${CMAKE_CURRENT_SOURCE_DIR}/tests/nudge_test.py" $<TARGET_FILE:SpaceCadetPinball>)
    set_tests_properties(authored-nudge PROPERTIES TIMEOUT 90)
endif()
