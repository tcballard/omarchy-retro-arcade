# Ridgeline provenance

All Ridgeline content is original Omarchy Arcade work, created in September 2026
under GPL-3.0-or-later.

| Source | Contents | Creation method |
| --- | --- | --- |
| `src/data.rs` | Ten maps, terrain, roads, flight lanes, wave tables, tower and enemy statistics | Original authored integer data |
| `src/art.rs` | Mesa plateaus with cliff faces, cracks and scrub, boulders, curbed roads with ruts, flight lanes, cave spawns, fortified gates, rotating turrets, oriented animated enemies, projectiles, particles, map thumbnails and vignettes | Original hand-authored egui vector geometry |
| `src/ui.rs` | Interface kit: cards, chips, keycaps, buttons, segmented controls, toggles, stat bars, health pips, coin, shield, lock and medal emblems | Original hand-authored egui vector geometry |
| `src/audio.rs` | Build, upgrade, sell, wave, breach, clear, victory and defeat cues | Original mono PCM synthesis from oscillators; no samples |
| `docs/shelf.png` | Last Pass mid-wave (dark theme) | Unretouched native app capture from a production-engine fixture, cropped to the board and resized to 800 × 486 |
| `docs/screenshots/` | Review captures (inspector, light placement, map select, result, compact campaign, 200%) | Native app captures under Xvfb, colour-quantized for size |
| `../../shared/presentation/assets/README.md` | Arcade cabinet backdrop and bezel materials | Existing approved shared asset and provenance |

Canyon Defense and other tower-defence games were genre references only. No
names, maps, wave data, artwork, sprites or sounds from them are included.
Playback uses the shared Arcade audio worker (`shared/platform/src/audio.rs`)
and the optional `paplay` command. Theme colours come from the existing Omarchy
theme loader.
