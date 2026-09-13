# Omarchy Arcade — player package

Eleven games in one native window, including a bundled Stockfish engine for
computer Chess. All games are available offline. No account is needed.

Open Omarchy Arcade from the app launcher. Select a game and click Play, or
select with the arrow keys and press Enter. Each game's Help explains its rules
and controls. Ctrl+H returns to the collection, F11 toggles fullscreen and Ctrl+Q
quits. Rust games save before leaving. If saving fails, Retry save (Enter)
tries again, Stay (Escape) keeps the game open, and Leave anyway (Alt+L) discards
unsaved session progress. Repair/recover rejected files before reopening to enable
saving; Retry never overwrites a rejected original. Pinball keeps records/settings
but not unfinished tables; its worker does not report save failures to this dialog. Existing per-game save locations are retained through package upgrades.

## Install

Download `arch-package` from a successful build and extract the ZIP. Run:

```sh
sudo pacman -U ./omarchy-retro-arcade-[0-9]*.pkg.tar.zst
```

This selects the player package and excludes `omarchy-retro-arcade-debug`.
Debug symbols are a separate, optional `arch-debug-symbols` download for debugging.
Stockfish and its evaluation network remain included in the player package.

## Included files

- Arcade executable, private Pinball worker and runtime table artwork.
- Stockfish executable with its embedded evaluation network (Arch package).
- Desktop entry, icon, this guide, licences and textual artwork attribution.

Other gameplay artwork is embedded in the executables. Development screenshots,
design images, tests, build tools and source archives are not installed.
Installed licences are under `/usr/share/licenses/omarchy-retro-arcade`; asset
provenance is under `/usr/share/doc/omarchy-retro-arcade`.

## Source, support and verification

Source, detailed rules, migration history and verification evidence:
https://github.com/tcballard/omarchy-retro-arcade

Report problems and playtest feedback:
https://github.com/tcballard/omarchy-retro-arcade/issues

Each CI build provides a separate `corresponding-source-and-engine` artifact;
retain the matching source when redistributing a build. The repository contains
locked dependency and build instructions. These are development builds, not a
claim of completed hands-on Omarchy/Wayland acceptance or an official package.
